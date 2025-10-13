#![no_std]

#[macro_use]
extern crate delog;
generate_macros!();

pub mod ndef;
pub use ndef::*;
