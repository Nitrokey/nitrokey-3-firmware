use core::{mem::MaybeUninit, time::Duration};

use super::board::{button::ThreeButtons, led::RgbLed};
use super::prince;
use super::spi::{FlashCs, Spi};
use crate::{flash::ExtFlashStorage, traits::Clock, types::Uuid, ui::UserInterface};
use apps::Variant;
#[cfg(feature = "se050")]
use embedded_hal::{blocking::delay::DelayUs, timer::CountDown};
#[cfg(feature = "se050")]
use embedded_time::duration::Microseconds;
use embedded_time::duration::Milliseconds;
use littlefs2::fs::{Allocation, Filesystem};
#[cfg(feature = "se050")]
use lpc55_hal::drivers::Timer;
use lpc55_hal::{
    drivers::{
        pins::{Pio0_9, Pio1_14},
        timer,
    },
    peripherals::{ctimer, flash, flexcomm::I2c5, rtc::Rtc, syscon},
    raw::{Interrupt, SCB},
    traits::flash::WriteErase,
    typestates::{
        init_state,
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
    type InternalFlashStorage = InternalFlashStorage;
    type ExternalFlashStorage = ExternalFlashStorage;
    type UsbBus = lpc55_hal::drivers::UsbBus<UsbPeripheral>;
    type NfcDevice = super::nfc::NfcChip;
    type TrussedUI = UserInterface<RtcClock, ThreeButtons, RgbLed>;
    #[cfg(feature = "se050")]
    type Se050Timer = TimerDelay<Timer<ctimer::Ctimer2<lpc55_hal::Enabled>>>;
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

    unsafe fn ifs_ptr() -> *mut Fs<Self::InternalFlashStorage> {
        static mut IFS: MaybeUninit<Fs<InternalFlashStorage>> = MaybeUninit::uninit();
        IFS.as_mut_ptr()
    }

    unsafe fn efs_ptr() -> *mut Fs<Self::ExternalFlashStorage> {
        static mut EFS: MaybeUninit<Fs<ExternalFlashStorage>> = MaybeUninit::uninit();
        EFS.as_mut_ptr()
    }

    unsafe fn ifs_storage() -> &'static mut Option<Self::InternalFlashStorage> {
        static mut IFS_STORAGE: Option<InternalFlashStorage> = None;
        &mut IFS_STORAGE
    }

    unsafe fn ifs_alloc() -> &'static mut Option<Allocation<Self::InternalFlashStorage>> {
        static mut IFS_ALLOC: Option<Allocation<InternalFlashStorage>> = None;
        &mut IFS_ALLOC
    }

    unsafe fn ifs() -> &'static mut Option<Filesystem<'static, Self::InternalFlashStorage>> {
        static mut IFS: Option<Filesystem<InternalFlashStorage>> = None;
        &mut IFS
    }

    unsafe fn efs_storage() -> &'static mut Option<Self::ExternalFlashStorage> {
        static mut EFS_STORAGE: Option<ExternalFlashStorage> = None;
        &mut EFS_STORAGE
    }

    unsafe fn efs_alloc() -> &'static mut Option<Allocation<Self::ExternalFlashStorage>> {
        static mut EFS_ALLOC: Option<Allocation<ExternalFlashStorage>> = None;
        &mut EFS_ALLOC
    }

    unsafe fn efs() -> &'static mut Option<Filesystem<'static, Self::ExternalFlashStorage>> {
        static mut EFS: Option<Filesystem<ExternalFlashStorage>> = None;
        &mut EFS
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
pub type NfcWaitExtender =
    timer::Timer<ctimer::Ctimer0<lpc55_hal::typestates::init_state::Enabled>>;
pub type PerformanceTimer =
    timer::Timer<ctimer::Ctimer4<lpc55_hal::typestates::init_state::Enabled>>;

pub type RtcClock = Rtc<init_state::Enabled>;

impl Clock for RtcClock {
    fn uptime(&mut self) -> Duration {
        Rtc::uptime(self)
    }
}
