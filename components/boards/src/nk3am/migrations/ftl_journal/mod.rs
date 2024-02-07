pub mod backends;
pub mod ifs_flash_old;

use littlefs2::fs::{Allocation, Filesystem};

use backends::EFSBackupBackend;
use ifs_flash_old::FlashStorage as OldFlashStorage;
use lfs_backup::{BackupBackend, FSBackupError, Result};

use crate::nk3am::{ExternalFlashStorage, InternalFlashStorage};

pub fn migrate(
    old_ifs_storage: &mut OldFlashStorage,
    old_ifs_alloc: &mut Allocation<OldFlashStorage>,
    ifs_alloc: &mut Allocation<InternalFlashStorage>,
    ifs_storage: &mut InternalFlashStorage,
    efs_storage: &mut ExternalFlashStorage,
) -> Result<()> {
    let old_mounted = Filesystem::mount(old_ifs_alloc, old_ifs_storage)
        .map_err(|_| FSBackupError::LittleFs2Err)?;

    trace!("old IFS mount success - migrating");

    // ext.flash = 2MB, spare for e.g., backup operations = 128kb (at end)
    let spare_len = 4096 * 32;
    let spare_offset = (2 * 1024 * 1024) - spare_len;
    let mut backend = EFSBackupBackend::new(efs_storage, spare_offset, spare_len);

    backend.erase()?;

    trace!("backing: old IFS -> backend");
    backend.backup(&old_mounted)?;

    // only format IFS on failed backup...
    trace!("backup done, format new IFS");
    let _fmt_ifs = Filesystem::format(ifs_storage);
    ifs_storage.format_journal_blocks();

    let new_mounted =
        Filesystem::mount(ifs_alloc, ifs_storage).map_err(|_| FSBackupError::LittleFs2Err)?;

    trace!("restore: backend -> new IFS");
    backend.reset();
    let _res = backend.restore(&new_mounted)?;

    // any outcome should erase the external flash contents
    backend.erase()?;
    Ok(())
}
