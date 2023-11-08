use super::board::{button::ThreeButtons, led::RgbLed};
use super::prince;
use super::spi::{FlashCs, Spi};
use super::trussed::UserInterface;
use crate::flash::ExtFlashStorage;
use apps::Variant;
#[cfg(feature = "se050")]
use embedded_hal::{blocking::delay::DelayUs, timer::CountDown};
#[cfg(feature = "se050")]
use embedded_time::duration::Microseconds;
use embedded_time::duration::Milliseconds;
#[cfg(feature = "se050")]
use lpc55_hal::drivers::Timer;
use lpc55_hal::{
    drivers::{
        pins::{Pio0_9, Pio1_14},
        timer,
    },
    peripherals::{ctimer, flash, flexcomm::I2c5, syscon},
    raw,
    traits::flash::WriteErase,
    typestates::pin::{
        function::{FC5_CTS_SDA_SSEL0, FC5_TXD_SCL_MISO_WS},
        state::Special,
    },
    I2cMaster,
};

use utils::OptionalStorage;

//////////////////////////////////////////////////////////////////////////////
// Upper Interface (definitions towards ERL Core)

pub static mut DEVICE_UUID: [u8; 16] = [0u8; 16];

#[cfg(feature = "no-encrypted-storage")]
use lpc55_hal::littlefs2_filesystem;
#[cfg(feature = "no-encrypted-storage")]
use trussed::types::LfsResult;

#[cfg(feature = "no-encrypted-storage")]
littlefs2_filesystem!(InternalFilesystem: (prince::FS_START, prince::BLOCK_COUNT));
#[cfg(not(feature = "no-encrypted-storage"))]
pub use prince::InternalFilesystem;

type UsbPeripheral = lpc55_hal::peripherals::usbhs::EnabledUsbhsDevice;

const INTERFACE_CONFIG: crate::types::Config = crate::types::Config {
    card_issuer: &crate::types::build_constants::CCID_ISSUER,
    usb_product: crate::types::build_constants::USB_PRODUCT,
    usb_manufacturer: crate::types::build_constants::USB_MANUFACTURER,
    usb_serial: "00000000-0000-0000-00000000",
    usb_id_vendor: crate::types::build_constants::USB_ID_VENDOR,
    usb_id_product: crate::types::build_constants::USB_ID_PRODUCT,
};

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

pub struct Soc {}
impl crate::types::Soc for Soc {
    type InternalFlashStorage = InternalFilesystem;
    type ExternalFlashStorage = OptionalStorage<ExtFlashStorage<Spi, FlashCs>>;
    type UsbBus = lpc55_hal::drivers::UsbBus<UsbPeripheral>;
    type NfcDevice = super::nfc::NfcChip;
    type Rng = chacha20::ChaCha8Rng;
    type TrussedUI = UserInterface<ThreeButtons, RgbLed>;
    type Reboot = Lpc55Reboot;
    type UUID = [u8; 16];
    #[cfg(feature = "se050")]
    type Se050Timer = TimerDelay<Timer<ctimer::Ctimer2<lpc55_hal::Enabled>>>;
    #[cfg(feature = "se050")]
    type Twi = I2C;
    #[cfg(not(feature = "se050"))]
    type Twi = ();
    #[cfg(not(feature = "se050"))]
    type Se050Timer = ();

    type Instant = ();
    type Duration = Milliseconds;

    const SYSCALL_IRQ: crate::types::IrqNr = crate::types::IrqNr {
        i: raw::Interrupt::OS_EVENT as u16,
    };

    const SOC_NAME: &'static str = "LPC55";
    const BOARD_NAME: &'static str = super::board::BOARD_NAME;
    const INTERFACE_CONFIG: &'static crate::types::Config = &INTERFACE_CONFIG;
    const VARIANT: Variant = Variant::Lpc55;

    fn device_uuid() -> &'static [u8; 16] {
        unsafe { &DEVICE_UUID }
    }
}

pub struct Lpc55Reboot {}
impl apps::Reboot for Lpc55Reboot {
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
