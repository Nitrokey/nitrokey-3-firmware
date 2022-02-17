#![no_std]

#[macro_use]
extern crate delog;
delog::generate_macros!();

pub mod types;

#[cfg(feature = "soc-nrf52840")]
pub mod soc_nrf52840;
#[cfg(feature = "soc-nrf52840")]
pub use soc_nrf52840 as soc;

#[cfg(feature = "soc-lpc55")]
pub mod soc_lpc55;
#[cfg(feature = "soc-lpc55")]
pub use soc_lpc55 as soc;

#[cfg(not(any(feature = "soc-lpc55", feature = "soc-nrf52840")))]
compile_error!("No SoC chosen!");
