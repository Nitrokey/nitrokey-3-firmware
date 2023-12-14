#![cfg_attr(not(test), no_std)]

extern crate delog;
delog::generate_macros!();

mod constants;
#[cfg(feature = "storage")]
mod storage;

pub use constants::{Version, VERSION, VERSION_STRING};
#[cfg(feature = "storage")]
pub use storage::{OptionalStorage, RamStorage};
