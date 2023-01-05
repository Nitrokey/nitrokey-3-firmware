#![no_std]

//! [CCID Specification for Integrated Circuit(s) Cards Interface Devices](https://www.usb.org/sites/default/files/DWG_Smart-Card_CCID_Rev110.pdf)
//!
//! [CCID SpecificationUSB Integrated Circuit(s) Card Devices](https://www.usb.org/sites/default/files/DWG_Smart-Card_USB-ICC_ICCD_rev10.pdf)

#[macro_use]
extern crate delog;
generate_macros!();

pub mod class;
pub mod constants;
pub mod pipe;
pub mod types;

// pub mod piv;

pub use class::Ccid;
