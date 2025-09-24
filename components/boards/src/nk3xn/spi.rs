use embedded_hal::{blocking::spi, spi::{FullDuplex, MODE_0}};
use embedded_hal_1::spi as spi_1;
use lpc55_hal::{
    drivers::{
        pins::{self, Pin},
        SpiMaster,
    },
    peripherals::flexcomm::Spi0,
    time::{Hertz, RateExtensions},
    traits::wg::{
        blocking::spi::Transfer,
        spi::{Mode, Phase, Polarity},
    },
    typestates::pin::{
        self,
        flexcomm::{NoCs, NoPio},
    },
    Enabled, Iocon,
};

pub type SckPin = pins::Pio0_28;
pub type MosiPin = pins::Pio0_24;
pub type MisoPin = pins::Pio0_25;
pub type FlashCsPin = pins::Pio0_13;

pub type Sck = Pin<SckPin, pin::state::Special<pin::function::FC0_SCK>>;
pub type Mosi = Pin<MosiPin, pin::state::Special<pin::function::FC0_RXD_SDA_MOSI_DATA>>;
pub type Miso = Pin<MisoPin, pin::state::Special<pin::function::FC0_TXD_SCL_MISO_WS>>;
pub type FlashCs = Pin<FlashCsPin, pin::state::Gpio<pin::gpio::direction::Output>>;

pub struct Spi(SpiMaster<SckPin, MosiPin, MisoPin, NoPio, Spi0, (Sck, Mosi, Miso, NoCs)>);

impl FullDuplex<u8> for Spi {
    type Error = SpiError;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.0.read().map_err(|e| e.map(SpiError))
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.0.send(word).map_err(|e| e.map(SpiError))
    }
}

impl spi::transfer::Default<u8> for Spi {}

impl spi::write::Default<u8> for Spi {}

impl spi_1::ErrorType for Spi {
    type Error = SpiError;
}

impl spi_1::SpiBus for Spi {
    fn read(&mut self, _words: &mut [u8]) -> Result<(), Self::Error> {
        unimplemented!();
    }

    fn write(&mut self, _words: &[u8]) -> Result<(), Self::Error> {
        unimplemented!();
    }

    fn transfer(&mut self, _read: &mut [u8], _write: &[u8]) -> Result<(), Self::Error> {
        unimplemented!();
    }

    fn transfer_in_place(&mut self, words: &mut [u8]) -> Result<(), Self::Error> {
        Transfer::transfer(self, words)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct SpiError(lpc55_hal::drivers::spi::Error);

impl spi_1::Error for SpiError {
    fn kind(&self) -> spi_1::ErrorKind {
        use lpc55_hal::drivers::spi::Error;

        match self.0 {
            Error::Overrun => spi_1::ErrorKind::Overrun,
            Error::ModeFault => spi_1::ErrorKind::ModeFault,
            Error::Crc => spi_1::ErrorKind::FrameFormat,
            _ => spi_1::ErrorKind::Other,
        }
    }
}

pub enum SpiConfig {
    ExternalFlash,
    Nfc,
    Tropic01,
}

impl SpiConfig {
    pub fn speed(&self) -> Hertz {
        match self {
            Self::ExternalFlash => 1_000_000u32.Hz(),
            Self::Nfc => 2_000_000u32.Hz(),
            Self::Tropic01 => 1_000_000u32.Hz(),
        }
    }

    pub fn mode(&self) -> Mode {
        let (polarity, phase) = match self {
            Self::ExternalFlash => (Polarity::IdleLow, Phase::CaptureOnFirstTransition),
            Self::Nfc => (Polarity::IdleLow, Phase::CaptureOnSecondTransition),
            Self::Tropic01 => {
                return MODE_0;
            }
        };
        Mode { polarity, phase }
    }
}

pub fn init(spi: Spi0<Enabled>, iocon: &mut Iocon<Enabled>, config: SpiConfig) -> Spi {
    let sck = SckPin::take().unwrap().into_spi0_sck_pin(iocon);
    let mosi = MosiPin::take().unwrap().into_spi0_mosi_pin(iocon);
    let miso = MisoPin::take().unwrap().into_spi0_miso_pin(iocon);
    configure(spi, (sck, mosi, miso, NoCs), config)
}

pub fn configure(spi: Spi0<Enabled>, pins: (Sck, Mosi, Miso, NoCs), config: SpiConfig) -> Spi {
    Spi(SpiMaster::new(spi, pins, config.speed(), config.mode()))
}

pub struct SpiMut<'a, SPI: Transfer<u8>>(pub &'a mut SPI);

impl<SPI: Transfer<u8>> Transfer<u8> for SpiMut<'_, SPI> {
    type Error = SPI::Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.0.transfer(words)
    }
}
