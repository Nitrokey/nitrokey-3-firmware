use apps::InitStatus;
use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use trussed::store;

use crate::{
    soc::{self, types::Soc as SocT},
    types::{Soc, VolatileStorage},
};

static mut INTERNAL_STORAGE: Option<<SocT as Soc>::InternalFlashStorage> = None;

#[cfg(feature = "provisioner")]
pub unsafe fn steal_internal_storage() -> &'static mut <SocT as Soc>::InternalFlashStorage {
    INTERNAL_STORAGE.as_mut().unwrap()
}

store!(
    RunnerStore,
    Internal: <SocT as Soc>::InternalFlashStorage,
    External: <SocT as Soc>::ExternalFlashStorage,
    Volatile: VolatileStorage
);

pub fn init_store(
    int_flash: <SocT as Soc>::InternalFlashStorage,
    ext_flash: <SocT as Soc>::ExternalFlashStorage,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> RunnerStore {
    static mut INTERNAL_FS_ALLOC: Option<Allocation<<SocT as Soc>::InternalFlashStorage>> = None;
    static mut INTERNAL_FS: Option<Filesystem<<SocT as Soc>::InternalFlashStorage>> = None;
    static mut EXTERNAL_STORAGE: Option<<SocT as Soc>::ExternalFlashStorage> = None;
    static mut EXTERNAL_FS_ALLOC: Option<Allocation<<SocT as Soc>::ExternalFlashStorage>> = None;
    static mut EXTERNAL_FS: Option<Filesystem<<SocT as Soc>::ExternalFlashStorage>> = None;
    static mut VOLATILE_STORAGE: Option<VolatileStorage> = None;
    static mut VOLATILE_FS_ALLOC: Option<Allocation<VolatileStorage>> = None;
    static mut VOLATILE_FS: Option<Filesystem<VolatileStorage>> = None;

    unsafe {
        let ifs_storage = INTERNAL_STORAGE.insert(int_flash);
        let ifs_alloc = INTERNAL_FS_ALLOC.insert(Filesystem::allocate());
        let efs_storage = EXTERNAL_STORAGE.insert(ext_flash);
        let efs_alloc = EXTERNAL_FS_ALLOC.insert(Filesystem::allocate());
        let vfs_storage = VOLATILE_STORAGE.insert(VolatileStorage::new());
        let vfs_alloc = VOLATILE_FS_ALLOC.insert(Filesystem::allocate());

        let Ok(ifs) = init_ifs(ifs_storage, ifs_alloc, efs_storage, status) else {
            error!("IFS Mount Error {:?}", _e);
            panic!("IFS");
        };

        let Ok(efs) = init_efs(efs_storage, efs_alloc, simulated_efs, status) else {
            error!("EFS Mount Error {:?}", _e);
            panic!("EFS");
        };

        let Ok(vfs) = init_vfs(vfs_storage, vfs_alloc) else {
            error!("VFS Mount Error {:?}", _e);
            panic!("VFS");
        };

        let ifs = INTERNAL_FS.insert(ifs);
        let efs = EXTERNAL_FS.insert(efs);
        let vfs = VOLATILE_FS.insert(vfs);

        RunnerStore::init_raw(ifs, efs, vfs)
    }
}

#[inline(always)]
fn init_ifs(
    ifs_storage: &'static mut <SocT as Soc>::InternalFlashStorage,
    ifs_alloc: &'static mut Allocation<<SocT as Soc>::InternalFlashStorage>,
    efs_storage: &mut <SocT as Soc>::ExternalFlashStorage,
    status: &mut InitStatus,
) -> LfsResult<Filesystem<'static, <SocT as Soc>::InternalFlashStorage>> {
    if !Filesystem::is_mountable(ifs_storage) {
        // handle provisioner
        if cfg!(feature = "provisioner") {
            info_now!("IFS mount failed - provisioner => formatting");
            Filesystem::format(ifs_storage).ok();
        } else {
            status.insert(InitStatus::INTERNAL_FLASH_ERROR);
            error_now!("IFS mount-fail");
            soc::recover_ifs(ifs_storage, ifs_alloc, efs_storage).ok();
        }
    }

    soc::prepare_ifs(ifs_storage);

    Filesystem::mount(ifs_alloc, ifs_storage)
}

#[inline(always)]
fn init_efs(
    efs_storage: &'static mut <SocT as Soc>::ExternalFlashStorage,
    efs_alloc: &'static mut Allocation<<SocT as Soc>::ExternalFlashStorage>,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> LfsResult<Filesystem<'static, <SocT as Soc>::ExternalFlashStorage>> {
    if !Filesystem::is_mountable(efs_storage) {
        let fmt_ext = Filesystem::format(efs_storage);
        if simulated_efs && fmt_ext == Err(littlefs2::io::Error::NoSpace) {
            info_now!("Formatting simulated EFS failed as expected");
        } else {
            error_now!("EFS Mount Error, Reformat {:?}", fmt_ext);
            status.insert(InitStatus::EXTERNAL_FLASH_ERROR);
        }
    };
    Filesystem::mount(efs_alloc, efs_storage)
}

#[inline(always)]
fn init_vfs(
    vfs_storage: &'static mut VolatileStorage,
    vfs_alloc: &'static mut Allocation<VolatileStorage>,
) -> LfsResult<Filesystem<'static, VolatileStorage>> {
    if !Filesystem::is_mountable(vfs_storage) {
        Filesystem::format(vfs_storage).ok();
    }
    Filesystem::mount(vfs_alloc, vfs_storage)
}
