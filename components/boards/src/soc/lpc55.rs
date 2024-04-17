use core::time::Duration;

use super::{Soc, Uuid};
use crate::ui::Clock;
use apps::Variant;
use embedded_time::duration::Milliseconds;
use lpc55_hal::{
    drivers::timer::Timer,
    peripherals::{ctimer, flash, rtc::Rtc, syscon},
    raw::{Interrupt, SCB},
    traits::flash::WriteErase,
    typestates::init_state::Enabled,
};

pub mod clock_controller;
pub mod monotonic;

type UsbPeripheral = lpc55_hal::peripherals::usbhs::EnabledUsbhsDevice;

pub struct Lpc55 {
    uuid: Uuid,
}

impl Lpc55 {
    pub fn new() -> Self {
        Self {
            uuid: lpc55_hal::uuid(),
        }
    }
}

impl Soc for Lpc55 {
    type UsbBus = lpc55_hal::drivers::UsbBus<UsbPeripheral>;
    type Clock = RtcClock;

    type Duration = Milliseconds;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::OS_EVENT;

    const SOC_NAME: &'static str = "lpc55";
    const VARIANT: Variant = Variant::Lpc55;

    fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

impl apps::Reboot for Lpc55 {
    fn reboot() -> ! {
        SCB::sys_reset()
    }
    fn reboot_to_firmware_update() -> ! {
        lpc55_hal::boot_to_bootrom()
    }
    fn reboot_to_firmware_update_destructive() -> ! {
        // Erasing the first flash page & rebooting will keep processor in bootrom persistently.
        // This is however destructive, as a valid firmware will need to be flashed.
        let flash =
            unsafe { flash::Flash::steal() }.enabled(&mut unsafe { syscon::Syscon::steal() });
        lpc55_hal::drivers::flash::FlashGordon::new(flash)
            .erase_page(0)
            .ok();
        SCB::sys_reset()
    }
    fn locked() -> bool {
        let seal = &unsafe { lpc55_hal::raw::Peripherals::steal() }
            .FLASH_CMPA
            .sha256_digest;
        seal.iter().any(|word| word.read().bits() != 0)
    }
}

pub type DynamicClockController = clock_controller::DynamicClockController;
pub type NfcWaitExtender = Timer<ctimer::Ctimer0<Enabled>>;
pub type PerformanceTimer = Timer<ctimer::Ctimer4<Enabled>>;

pub type RtcClock = Rtc<Enabled>;

impl Clock for RtcClock {
    fn uptime(&mut self) -> Duration {
        Rtc::uptime(self)
    }
}
