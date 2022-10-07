use crate::hal::{
    self,
    drivers::{
        pins::{self, Pin},
        SpiMaster,
    },
    Enabled,
    peripherals::flexcomm::Spi0,
    time::RateExtensions,
    typestates::{
        pin::{
            self,
            flexcomm::NoPio,
        },
    },
};

pub type SckPin = pins::Pio0_28;
pub type MosiPin = pins::Pio0_24;
pub type MisoPin = pins::Pio0_25;

pub type Spi = SpiMaster<
    SckPin,
    MosiPin,
    MisoPin,
    NoPio,
    Spi0,
    (
        Pin<SckPin, pin::state::Special<pin::function::FC0_SCK>>,
        Pin<MosiPin, pin::state::Special<pin::function::FC0_RXD_SDA_MOSI_DATA>>,
        Pin<MisoPin, pin::state::Special<pin::function::FC0_TXD_SCL_MISO_WS>>,
        pin::flexcomm::NoCs,
    ),
>;

pub fn init(
    spi: Spi0<Enabled>,
    iocon: &mut hal::Iocon<Enabled>,
) -> Spi {
    let sck = SckPin::take().unwrap().into_spi0_sck_pin(iocon);
    let mosi = MosiPin::take().unwrap().into_spi0_mosi_pin(iocon);
    let miso = MisoPin::take().unwrap().into_spi0_miso_pin(iocon);
    let spi_mode = hal::traits::wg::spi::Mode {
        polarity: hal::traits::wg::spi::Polarity::IdleLow,
        // phase: hal::traits::wg::spi::Phase::CaptureOnSecondTransition,
        phase: hal::traits::wg::spi::Phase::CaptureOnFirstTransition,
    };
    SpiMaster::new(
        spi,
        (sck, mosi, miso, hal::typestates::pin::flexcomm::NoCs),
        // 2_000_000u32.Hz(),
        1_000_000u32.Hz(),
        spi_mode,
    )
}

pub fn reconfigure(spi: Spi) -> Spi {
    let (spi, pins) = spi.release();
    let spi_mode = hal::traits::wg::spi::Mode {
        polarity: hal::traits::wg::spi::Polarity::IdleLow,
        phase: hal::traits::wg::spi::Phase::CaptureOnSecondTransition,
    };
    SpiMaster::new(
        spi,
        pins,
        2_000_000u32.Hz(),
        spi_mode,
    )
}
