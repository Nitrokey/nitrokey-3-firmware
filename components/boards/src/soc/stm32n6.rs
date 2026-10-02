use core::time::Duration;

use apps::Variant;
use cortex_m::peripheral::SCB;
use embedded_time::duration::Milliseconds;
use stm32n6::stm32n657::{Interrupt, BSEC, OTG1_S};
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

pub mod mmc {
    use stm32n6::stm32n657::SDMMC1_S;
    use stm32n657_hal::gpio::{
        Alternate, PinC1, PinC10, PinC11, PinC12, PinC6, PinC7, PinC9, PinD11, PinH2, PinH9,
        PullUp, ALTERNATE_FUNCTION_10,
    };
    use stm32n657_hal::mmc::MmcMaster;
    use stm32n657_hal::sdmmc::Enabled;

    // the eMMC drops its DAT1-7 pull-ups in wide bus mode, the MCU ones (30-50k) stay
    pub type CmdPin = PinH2<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type CkPin = PinC12<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D0Pin = PinD11<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D1Pin = PinC9<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D2Pin = PinC10<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D3Pin = PinC11<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D4Pin = PinH9<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D5Pin = PinC1<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D6Pin = PinC6<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;
    pub type D7Pin = PinC7<Alternate<PullUp, { ALTERNATE_FUNCTION_10 }>>;

    pub type Pins = (
        CmdPin,
        CkPin,
        D0Pin,
        D1Pin,
        D2Pin,
        D3Pin,
        D4Pin,
        D5Pin,
        D6Pin,
        D7Pin,
    );
    pub type Peripheral = SDMMC1_S;

    pub type Mmc<S = Enabled> = MmcMaster<Peripheral, Pins, S>;
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
    pwr: &Pwr,
    rcc: &Rcc,
    clock_config: ClockConfig,
) -> UsbBusAllocator<UsbBus1> {
    let ep_memory = ep_memory.insert([0; 1024]);
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
