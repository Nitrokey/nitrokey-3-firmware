use stm32n6::stm32n657::SDMMC2_S;
use stm32n657_hal::gpio::{
    Alternate, PinC0, PinC2, PinC3, PinC4, PinC5, PinE4, PullUp, ALTERNATE_FUNCTION_11,
};
use stm32n657_hal::mmc::MmcMaster;
use stm32n657_hal::sdmmc::Enabled;

pub type CmdPin = PinC3<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>;
pub type CkPin = PinC2<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>;
pub type D0Pin = PinC4<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>;
#[allow(unused)]
pub type D1Pin = PinC5<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>;
#[allow(unused)]
pub type D2Pin = PinC0<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>;
#[allow(unused)]
pub type D3Pin = PinE4<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>;

pub type Pins = (
    CmdPin,
    CkPin,
    D0Pin,
    // PinC5<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
    // PinC0<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
    // PinE4<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
);

pub type Mmc = MmcMaster<SDMMC2_S, Pins, Enabled>;
