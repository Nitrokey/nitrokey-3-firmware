use apps::InitStatus;
use littlefs2::fs::{Allocation, Filesystem};
use trussed::store;

use crate::{
    soc::{self, types::Soc as SocT},
    types::{Soc, VolatileStorage},
};

pub static mut INTERNAL_STORAGE: Option<<SocT as Soc>::InternalFlashStorage> = None;
pub static mut INTERNAL_FS_ALLOC: Option<Allocation<<SocT as Soc>::InternalFlashStorage>> = None;
pub static mut INTERNAL_FS: Option<Filesystem<<SocT as Soc>::InternalFlashStorage>> = None;
pub static mut EXTERNAL_STORAGE: Option<<SocT as Soc>::ExternalFlashStorage> = None;
pub static mut EXTERNAL_FS_ALLOC: Option<Allocation<<SocT as Soc>::ExternalFlashStorage>> = None;
pub static mut EXTERNAL_FS: Option<Filesystem<<SocT as Soc>::ExternalFlashStorage>> = None;
pub static mut VOLATILE_STORAGE: Option<VolatileStorage> = None;
pub static mut VOLATILE_FS_ALLOC: Option<Allocation<VolatileStorage>> = None;
pub static mut VOLATILE_FS: Option<Filesystem<VolatileStorage>> = None;

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
    let volatile_storage = VolatileStorage::new();

    /* Step 1: let our stack-based filesystem objects transcend into higher
    beings by blessing them with static lifetime
    */
    macro_rules! transcend {
        ($global:expr, $content:expr) => {
            unsafe {
                $global.replace($content);
                $global.as_mut().unwrap()
            }
        };
    }

    let ifs_storage = transcend!(INTERNAL_STORAGE, int_flash);
    let ifs_alloc = transcend!(INTERNAL_FS_ALLOC, Filesystem::allocate());
    let efs_storage = transcend!(EXTERNAL_STORAGE, ext_flash);
    let efs_alloc = transcend!(EXTERNAL_FS_ALLOC, Filesystem::allocate());
    let vfs_storage = transcend!(VOLATILE_STORAGE, volatile_storage);
    let vfs_alloc = transcend!(VOLATILE_FS_ALLOC, Filesystem::allocate());

    /* Step 2: try mounting each FS in turn */
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

    let ifs_ = Filesystem::mount(ifs_alloc, ifs_storage).expect("Could not bring up IFS!");
    let ifs = transcend!(INTERNAL_FS, ifs_);

    if !littlefs2::fs::Filesystem::is_mountable(efs_storage) {
        let fmt_ext = littlefs2::fs::Filesystem::format(efs_storage);
        if simulated_efs && fmt_ext == Err(littlefs2::io::Error::NoSpace) {
            info_now!("Formatting simulated EFS failed as expected");
        } else {
            error_now!("EFS Mount Error, Reformat {:?}", fmt_ext);
            status.insert(InitStatus::EXTERNAL_FLASH_ERROR);
        }
    };
    let efs = match littlefs2::fs::Filesystem::mount(efs_alloc, efs_storage) {
        Ok(efs_) => {
            transcend!(EXTERNAL_FS, efs_)
        }
        Err(_e) => {
            error!("EFS Mount Error {:?}", _e);
            panic!("store");
        }
    };

    if !littlefs2::fs::Filesystem::is_mountable(vfs_storage) {
        littlefs2::fs::Filesystem::format(vfs_storage).ok();
    }
    let vfs = match littlefs2::fs::Filesystem::mount(vfs_alloc, vfs_storage) {
        Ok(vfs_) => {
            transcend!(VOLATILE_FS, vfs_)
        }
        Err(_e) => {
            error!("VFS Mount Error {:?}", _e);
            panic!("store");
        }
    };

    RunnerStore::init_raw(ifs, efs, vfs)
}
