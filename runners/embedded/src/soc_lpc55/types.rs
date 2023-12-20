use core::time::Duration;

use crate::{types::Uuid, ui::Clock};
use apps::Variant;
use embedded_time::duration::Milliseconds;
use lpc55_hal::{
    drivers::{
        pins::{Pio0_9, Pio1_14},
        timer::Timer,
    },
    peripherals::{ctimer, flash, flexcomm::I2c5, rtc::Rtc, syscon},
    raw::{Interrupt, SCB},
    traits::flash::WriteErase,
    typestates::{
        init_state::Enabled,
        pin::{
            function::{FC5_CTS_SDA_SSEL0, FC5_TXD_SCL_MISO_WS},
            state::Special,
        },
    },
    I2cMaster,
};
use memory_regions::MemoryRegions;

//////////////////////////////////////////////////////////////////////////////
// Upper Interface (definitions towards ERL Core)

pub static mut DEVICE_UUID: Uuid = [0u8; 16];

type UsbPeripheral = lpc55_hal::peripherals::usbhs::EnabledUsbhsDevice;

pub(super) type I2C = I2cMaster<
    Pio0_9,
    Pio1_14,
    I2c5,
    (
        lpc55_hal::Pin<Pio0_9, Special<FC5_TXD_SCL_MISO_WS>>,
        lpc55_hal::Pin<Pio1_14, Special<FC5_CTS_SDA_SSEL0>>,
    ),
>;

pub const MEMORY_REGIONS: &'static MemoryRegions = &MemoryRegions::LPC55;

pub struct Soc {}
impl crate::types::Soc for Soc {
    type UsbBus = lpc55_hal::drivers::UsbBus<UsbPeripheral>;
    type Clock = RtcClock;

    type Duration = Milliseconds;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::OS_EVENT;

    const SOC_NAME: &'static str = "LPC55";
    const VARIANT: Variant = Variant::Lpc55;

    fn device_uuid() -> &'static Uuid {
        unsafe { &DEVICE_UUID }
    }
}

impl apps::Reboot for Soc {
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

pub type DynamicClockController = super::clock_controller::DynamicClockController;
pub type NfcWaitExtender = Timer<ctimer::Ctimer0<Enabled>>;
pub type PerformanceTimer = Timer<ctimer::Ctimer4<Enabled>>;

pub type RtcClock = Rtc<Enabled>;

impl Clock for RtcClock {
    fn uptime(&mut self) -> Duration {
        Rtc::uptime(self)
    }
}
