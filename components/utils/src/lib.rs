#![no_std]

extern crate delog;
delog::generate_macros!();

mod storage;

pub use storage::{OptionalStorage, RamStorage};
