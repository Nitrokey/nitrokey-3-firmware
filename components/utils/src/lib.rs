#![cfg_attr(not(feature = "std"), no_std)]

extern crate delog;
delog::generate_macros!();

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "storage")]
mod storage;
mod version;

#[cfg(feature = "build")]
pub use build::version_string;
#[cfg(feature = "storage")]
pub use storage::{OptionalStorage, RamStorage};
pub use version::Version;
