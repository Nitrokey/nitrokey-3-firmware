#![no_std]

mod lfs_backup;

pub use crate::lfs_backup::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[macro_use]
extern crate std;
