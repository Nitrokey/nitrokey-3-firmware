#![no_std]

//! LittleFS2 backup and restore mechanism to/from low-level NVM interfaces.
//!
//! This component realizes a backup/restore operation using a littlefs2
//! `Filesystem` as the source resp. target. The counterpart is denoted
//! a `BackupBackend`, which is a trait that needs to be implemented.
//!
//! Low-level flash devices (NVMs) mostly come with intrinsic restrictions
//! like read/write/erase sizes (i.e., blocks). The `BackupBackend`
//! implementation has to strictly enforce them and set `RW_SIZE` accordingly.
//!
//! # Backup Data Layout
//! A full backup binary blob consists of:
//! ```text
//! FS_BACKUP_START_DELIM | Entry1 | Entry2 | … | FS_BACKUP_END_DELIM
//! ```
//! With `EntryX` being:
//!
//! | bytes   | content                           |
//! |---------|-----------------------------------|
//! |  0 - 3  | Big-Endian length of the blob     |
//! |  4 - n  | postcard-serialized `FSEntryBlob` |
//!
//! # Important Implementation Details
//! * The `BackupBackend` implementation has to maintain an internal *cursor*
//!   pointing at the current position inside the backup blob
//! * `read` & `write` are explicitly *not* responsible to cache or manipulate
//!   the passed content in any way - this is implemented in `read_next` &
//!   `write_entry`
//! * The *cursor* should move forward by multiples of `RW_SIZE` for
//!   *every* `read` & `write` invocation as by definition the low-level
//!   interfaces for most flash memories will not allow multiple writes
//!   within the same block (i.e., within `RW_SIZE` bytes)

mod lfs_backup;

pub use crate::lfs_backup::*;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[macro_use]
extern crate std;
