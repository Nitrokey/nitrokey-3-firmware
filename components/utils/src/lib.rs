#![cfg_attr(not(test), no_std)]

extern crate delog;
delog::generate_macros!();

mod constants;
mod storage;

pub use constants::{Version, VERSION, VERSION_STRING};
pub use storage::{OptionalStorage, RamStorage};
