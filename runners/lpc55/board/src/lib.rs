#![no_std]

#[cfg(not(any(feature = "soc-lpc55", feature = "soc-nrf52")))]
compile_error!{"No SoC selected! Did you choose a board feature?"}

#[macro_use]
extern crate delog;
generate_macros!();

#[cfg(feature = "soc-lpc55")]
pub use lpc55_hal as hal;
#[cfg(feature = "soc-lpc55")]
pub mod soc_lpc55;
#[cfg(feature = "soc-lpc55")]
pub use soc_lpc55 as soc;

#[cfg(feature = "soc-nrf52")]
pub use nrf52840_hal as hal;
#[cfg(feature = "soc-nrf52")]
pub mod soc_nrf52;
#[cfg(feature = "soc-nrf52")]
pub use soc_nrf52 as soc;

