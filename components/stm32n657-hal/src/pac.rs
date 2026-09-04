//! PAC re-export with the 4 NVIC priority bits the Cortex-M55 in the N6 implements.
pub use stm32n6::stm32n657::*;

pub const NVIC_PRIO_BITS: u8 = 4;
