use stm32n6::stm32n657::SDMMC2;
use stm32n657_hal::gpio::{
    Alternate, PinC0, PinC2, PinC3, PinC4, PinC5, PinE4, PullUp, ALTERNATE_FUNCTION_11,
};
use stm32n657_hal::mmc::MmcMaster;
use stm32n657_hal::sdmmc::Enabled;

type Pins = (
    PinC3<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
    PinC2<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
    PinC4<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
    PinC5<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
    PinC0<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
    PinE4<Alternate<PullUp, { ALTERNATE_FUNCTION_11 }>>,
);

pub type Mmc = MmcMaster<SDMMC2, Pins, Enabled>;
