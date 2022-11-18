use super::board::{button::ThreeButtons, led::RgbLed};
use super::trussed::UserInterface;
use crate::types::{build_constants, Storage};
use embedded_time::duration::Milliseconds;
use littlefs2::const_ram_storage;
use lpc55_hal::{
    drivers::timer,
    peripherals::{ctimer, flash, rng, syscon},
    raw,
    traits::flash::WriteErase,
};
use trussed::types::{LfsResult, LfsStorage};

//////////////////////////////////////////////////////////////////////////////
// Upper Interface (definitions towards ERL Core)

const_ram_storage!(ExternalRAMStorage, 1024);

#[cfg(feature = "no-encrypted-storage")]
use lpc55_hal::littlefs2_filesystem;
#[cfg(not(feature = "no-encrypted-storage"))]
use lpc55_hal::littlefs2_prince_filesystem;

#[cfg(feature = "no-encrypted-storage")]
littlefs2_filesystem!(InternalFilesystem: (build_constants::CONFIG_FILESYSTEM_BOUNDARY));
#[cfg(not(feature = "no-encrypted-storage"))]
littlefs2_prince_filesystem!(InternalFilesystem: (build_constants::CONFIG_FILESYSTEM_BOUNDARY));

static mut INTERNAL_STORAGE: Storage<InternalFilesystem> = Storage::new();
static mut EXTERNAL_STORAGE: Storage<ExternalRAMStorage> = Storage::new();

type UsbPeripheral = lpc55_hal::peripherals::usbhs::EnabledUsbhsDevice;

const INTERFACE_CONFIG: crate::types::Config = crate::types::Config {
    card_issuer: &crate::types::build_constants::CCID_ISSUER,
    usb_product: crate::types::build_constants::USB_PRODUCT,
    usb_manufacturer: crate::types::build_constants::USB_MANUFACTURER,
    usb_serial: "00000000-0000-0000-00000000",
    usb_id_vendor: crate::types::build_constants::USB_ID_VENDOR,
    usb_id_product: crate::types::build_constants::USB_ID_PRODUCT,
};

pub struct Soc {}
impl crate::types::Soc for Soc {
    type InternalFlashStorage = InternalFilesystem;
    type ExternalFlashStorage = ExternalRAMStorage;
    type UsbBus = lpc55_hal::drivers::UsbBus<UsbPeripheral>;
    type NfcDevice = super::nfc::NfcChip;
    type Rng = rng::Rng<lpc55_hal::Enabled>;
    type TrussedUI = UserInterface<ThreeButtons, RgbLed>;
    type Reboot = Lpc55Reboot;

    type Interrupt = raw::Interrupt;
    type Duration = Milliseconds;

    const SYSCALL_IRQ: Self::Interrupt = Self::Interrupt::OS_EVENT;

    const SOC_NAME: &'static str = "LPC55";
    const BOARD_NAME: &'static str = super::board::BOARD_NAME;
    const INTERFACE_CONFIG: &'static crate::types::Config = &INTERFACE_CONFIG;

    fn device_uuid() -> [u8; 16] {
        lpc55_hal::uuid()
    }

    unsafe fn internal_storage() -> &'static mut Storage<'static, Self::InternalFlashStorage> {
        &mut INTERNAL_STORAGE
    }

    unsafe fn external_storage() -> &'static mut Storage<'static, Self::ExternalFlashStorage> {
        &mut EXTERNAL_STORAGE
    }
}

pub struct Lpc55Reboot {}
impl admin_app::Reboot for Lpc55Reboot {
    fn reboot() -> ! {
        raw::SCB::sys_reset()
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
        raw::SCB::sys_reset()
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
