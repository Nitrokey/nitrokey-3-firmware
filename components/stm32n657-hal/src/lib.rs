#![no_std]

//! Hardware abstraction layer for the STM32N657 chip.
//!
//! This implementation is mostly based on the [Reference Manual for STM32N6x5 (RM0486)][rm0486].
//!
//! [rm0486]: https://www.st.com/resource/en/reference_manual/rm0468-stm32h723733-stm32h725735-and-stm32h730-value-line-advanced-armbased-32bit-mcus-stmicroelectronics.pdf

pub mod bsec;
pub mod gpio;
#[cfg(feature = "otg")]
pub mod otg;
pub mod rcc;
pub mod timer;

pub type Rate = fugit::HertzU32;
