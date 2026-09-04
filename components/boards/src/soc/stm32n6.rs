use core::time::Duration;

use apps::Variant;
use cortex_m::peripheral::SCB;
use embedded_time::duration::Milliseconds;
use stm32n6::stm32n657::{Interrupt, BSEC, OTG1_S, PWR_S};
use stm32n657_hal::{
    bsec::Bsec,
    otg::{Otg1, UsbBus1},
    pwr::Pwr,
    rcc::{ClockConfig, Rcc},
    timer::{MillisecondsCounter, Tim7},
};
use usb_device::bus::UsbBusAllocator;

use super::{Soc, Uuid};
use crate::ui::Clock;

pub struct Stm32n6 {
    uuid: Uuid,
}

impl Soc for Stm32n6 {
    type UsbBus = UsbBus1;
    type Clock = TimerClock;

    type Duration = Milliseconds;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::LPTIM4;

    const SOC_NAME: &'static str = "stm32n6";
    const VARIANT: Variant = Variant::Stm32n6;

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl apps::Reboot for Stm32n6 {
    fn reboot() -> ! {
        SCB::sys_reset()
    }
    fn reboot_to_firmware_update() -> ! {
        // No bootloader yet, the firmware is loaded over the debug port.
        SCB::sys_reset()
    }
    fn reboot_to_firmware_update_destructive() -> ! {
        SCB::sys_reset()
    }
    fn locked() -> bool {
        false
    }
}

pub fn init_bootup(bsec: BSEC) -> Stm32n6 {
    let uid = Bsec::new(bsec).uid();

    let mut uuid = Uuid::default();
    for (chunk, word) in uuid.chunks_exact_mut(4).zip(uid) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    info!("BSEC UID {}", delog::hex_str!(&uuid));

    Stm32n6 { uuid }
}

/// Buffer memory for the OUT endpoints of the USB driver.
pub type EpMemory = [u32; 1024];

pub fn setup_usb_bus(
    ep_memory: &'static mut Option<EpMemory>,
    otg1: OTG1_S,
    pwr: PWR_S,
    rcc: &Rcc,
    clock_config: ClockConfig,
) -> UsbBusAllocator<UsbBus1> {
    let ep_memory = ep_memory.insert([0; 1024]);
    let pwr = Pwr::new(pwr);
    let otg1 = Otg1::new(otg1, rcc, &pwr, clock_config);
    UsbBus1::new(otg1, ep_memory)
}

/// Millisecond uptime from TIM7, must be polled at least once per 65 s.
pub struct TimerClock(MillisecondsCounter<Tim7>);

impl TimerClock {
    pub fn new(tim7: Tim7, clock_config: ClockConfig) -> Self {
        Self(MillisecondsCounter::new(tim7, clock_config))
    }
}

impl Clock for TimerClock {
    fn uptime(&mut self) -> Duration {
        Duration::from_millis(self.0.now().ticks().into())
    }
}
