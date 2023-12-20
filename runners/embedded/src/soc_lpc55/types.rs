use core::{mem::MaybeUninit, time::Duration};

use super::board::{button::ThreeButtons, led::RgbLed};
use super::prince;
use super::spi::{FlashCs, Spi};
use crate::{flash::ExtFlashStorage, types::Uuid, ui::Clock};
use apps::Variant;
#[cfg(feature = "se050")]
use embedded_hal::{blocking::delay::DelayUs, timer::CountDown};
#[cfg(feature = "se050")]
use embedded_time::duration::Microseconds;
use embedded_time::duration::Milliseconds;
use littlefs2::fs::{Allocation, Filesystem};
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
use trussed::store::Fs;

use memory_regions::MemoryRegions;
use utils::OptionalStorage;

//////////////////////////////////////////////////////////////////////////////
// Upper Interface (definitions towards ERL Core)

pub static mut DEVICE_UUID: Uuid = [0u8; 16];

#[cfg(feature = "no-encrypted-storage")]
use lpc55_hal::littlefs2_filesystem;
#[cfg(feature = "no-encrypted-storage")]
use trussed::types::LfsResult;

#[cfg(feature = "no-encrypted-storage")]
littlefs2_filesystem!(InternalFilesystem: (prince::FS_START, prince::BLOCK_COUNT));
#[cfg(not(feature = "no-encrypted-storage"))]
pub use prince::InternalFilesystem;

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

#[cfg(feature = "se050")]
pub struct TimerDelay<T>(pub T);

#[cfg(feature = "se050")]
impl<T> DelayUs<u32> for TimerDelay<T>
where
    T: CountDown<Time = Microseconds<u32>>,
{
    fn delay_us(&mut self, delay: u32) {
        self.0.start(Microseconds::new(delay));
        nb::block!(self.0.wait()).unwrap();
    }
}

pub const MEMORY_REGIONS: &'static MemoryRegions = &MemoryRegions::LPC55;

pub type InternalFlashStorage = InternalFilesystem;
pub type ExternalFlashStorage = OptionalStorage<ExtFlashStorage<Spi, FlashCs>>;

pub struct Soc {}
impl crate::types::Soc for Soc {
    type UsbBus = lpc55_hal::drivers::UsbBus<UsbPeripheral>;
    type NfcDevice = super::nfc::NfcChip;
    type Clock = RtcClock;
    type Buttons = ThreeButtons;
    type Led = RgbLed;
    #[cfg(feature = "se050")]
    type Se050Timer = TimerDelay<Timer<ctimer::Ctimer2<Enabled>>>;
    #[cfg(feature = "se050")]
    type Twi = I2C;
    #[cfg(not(feature = "se050"))]
    type Twi = ();
    #[cfg(not(feature = "se050"))]
    type Se050Timer = ();

    type Duration = Milliseconds;

    type Interrupt = Interrupt;
    const SYSCALL_IRQ: Interrupt = Interrupt::OS_EVENT;

    const SOC_NAME: &'static str = "LPC55";
    const BOARD_NAME: &'static str = super::board::BOARD_NAME;
    const VARIANT: Variant = Variant::Lpc55;

    fn device_uuid() -> &'static Uuid {
        unsafe { &DEVICE_UUID }
    }
}

impl_storage_pointers!(
    Soc,
    Internal = InternalFlashStorage,
    External = ExternalFlashStorage,
);

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
