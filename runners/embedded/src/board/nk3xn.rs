#[cfg(feature = "se050")]
use embedded_hal::{blocking::delay::DelayUs, timer::CountDown};
#[cfg(feature = "se050")]
use embedded_time::duration::Microseconds;
#[cfg(feature = "se050")]
use lpc55_hal::drivers::Timer;
use lpc55_hal::{
    drivers::pins::{Pio0_9, Pio1_14},
    peripherals::{ctimer, flexcomm::I2c5},
    typestates::{
        init_state::Unknown,
        pin::{
            function::{FC5_CTS_SDA_SSEL0, FC5_TXD_SCL_MISO_WS},
            state::Special,
        },
    },
    I2cMaster, Pin,
};

use memory_regions::MemoryRegions;
use utils::OptionalStorage;

use crate::{
    board::Board, flash::ExtFlashStorage, soc::lpc55::Lpc55, store::impl_storage_pointers,
};

pub mod button;
pub mod init;
pub mod led;
pub mod nfc;
pub mod prince;
pub mod spi;

#[cfg(feature = "no-encrypted-storage")]
use trussed::types::LfsResult;

#[cfg(feature = "no-encrypted-storage")]
lpc55_hal::littlefs2_filesystem!(InternalFilesystem: (prince::FS_START, prince::BLOCK_COUNT));
#[cfg(not(feature = "no-encrypted-storage"))]
use prince::InternalFilesystem;

use nfc::NfcChip;
use spi::{FlashCs, Spi};

const MEMORY_REGIONS: &'static MemoryRegions = &MemoryRegions::LPC55;

pub type PwmTimer = ctimer::Ctimer3<Unknown>;
pub type ButtonsTimer = ctimer::Ctimer1<Unknown>;

type I2C = I2cMaster<
    Pio0_9,
    Pio1_14,
    I2c5,
    (
        Pin<Pio0_9, Special<FC5_TXD_SCL_MISO_WS>>,
        Pin<Pio1_14, Special<FC5_CTS_SDA_SSEL0>>,
    ),
>;

pub struct NK3xN;

impl Board for NK3xN {
    type Soc = Lpc55;

    type NfcDevice = NfcChip;
    type Buttons = button::ThreeButtons;
    type Led = led::RgbLed;

    #[cfg(feature = "se050")]
    type Se050Timer = TimerDelay<Timer<ctimer::Ctimer2<lpc55_hal::Enabled>>>;
    #[cfg(feature = "se050")]
    type Twi = I2C;
    #[cfg(not(feature = "se050"))]
    type Twi = ();
    #[cfg(not(feature = "se050"))]
    type Se050Timer = ();

    const BOARD_NAME: &'static str = "nk3xn";
}

pub type InternalFlashStorage = InternalFilesystem;
pub type ExternalFlashStorage = OptionalStorage<ExtFlashStorage<Spi, FlashCs>>;

impl_storage_pointers!(
    NK3xN,
    Internal = InternalFlashStorage,
    External = ExternalFlashStorage,
);

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

pub fn init(
    device_peripherals: lpc55_hal::raw::Peripherals,
    core_peripherals: rtic::export::Peripherals,
) -> init::All {
    const SECURE_FIRMWARE_VERSION: u32 = utils::VERSION.encode();

    crate::init_logger::<NK3xN>();

    let hal = lpc55_hal::Peripherals::from((device_peripherals, core_peripherals));

    let require_prince = cfg!(not(feature = "no-encrypted-storage"));
    let secure_firmware_version = Some(SECURE_FIRMWARE_VERSION);
    let nfc_enabled = true;
    let boot_to_bootrom = true;

    init::start(hal.syscon, hal.pmc, hal.anactrl)
        .next(hal.iocon, hal.gpio)
        .next(
            hal.adc,
            hal.ctimer.0,
            hal.ctimer.1,
            hal.ctimer.2,
            hal.ctimer.3,
            hal.ctimer.4,
            hal.pfr,
            secure_firmware_version,
            require_prince,
            boot_to_bootrom,
        )
        .next(
            hal.flexcomm.0,
            hal.flexcomm.5,
            hal.inputmux,
            hal.pint,
            nfc_enabled,
        )
        .next(hal.rng, hal.prince, hal.flash)
        .next()
        .next(hal.rtc)
        .next(hal.usbhs)
}
