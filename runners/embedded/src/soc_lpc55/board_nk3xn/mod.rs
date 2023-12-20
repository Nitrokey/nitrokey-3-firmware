#[cfg(feature = "se050")]
use embedded_hal::{blocking::delay::DelayUs, timer::CountDown};
#[cfg(feature = "se050")]
use embedded_time::duration::Microseconds;
use lpc55_hal::{peripherals::ctimer, typestates::init_state::Unknown};

use utils::OptionalStorage;

use super::{
    nfc::NfcChip,
    spi::{FlashCs, Spi},
    types::Soc,
};
use crate::{flash::ExtFlashStorage, types::Board};

pub mod button;
pub mod led;

#[cfg(feature = "no-encrypted-storage")]
use lpc55_hal::littlefs2_filesystem;
#[cfg(feature = "no-encrypted-storage")]
use trussed::types::LfsResult;

#[cfg(feature = "no-encrypted-storage")]
littlefs2_filesystem!(InternalFilesystem: (super::prince::FS_START, super::prince::BLOCK_COUNT));
#[cfg(not(feature = "no-encrypted-storage"))]
use super::prince::InternalFilesystem;

pub use led::set_panic_led;

pub type PwmTimer = ctimer::Ctimer3<Unknown>;
pub type ButtonsTimer = ctimer::Ctimer1<Unknown>;

pub struct NK3xN;

impl Board for NK3xN {
    type Soc = Soc;

    type NfcDevice = NfcChip;
    type Buttons = button::ThreeButtons;
    type Led = led::RgbLed;

    #[cfg(feature = "se050")]
    type Se050Timer = TimerDelay<lpc55_hal::drivers::Timer<ctimer::Ctimer2<lpc55_hal::Enabled>>>;
    #[cfg(feature = "se050")]
    type Twi = super::types::I2C;
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
