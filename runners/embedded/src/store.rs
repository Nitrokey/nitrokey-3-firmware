use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

use apps::InitStatus;
use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use trussed::store::{Fs, Store};

use crate::types::{Soc, VolatileStorage};

#[cfg(feature = "provisioner")]
pub unsafe fn steal_internal_storage<S: Soc>() -> &'static mut S::InternalFlashStorage {
    S::ifs_storage().as_mut().unwrap()
}

pub struct RunnerStore<S: Soc> {
    _marker: PhantomData<*mut S>,
}

impl<S: Soc> RunnerStore<S> {
    fn new(
        ifs: &'static Filesystem<'static, S::InternalFlashStorage>,
        efs: &'static Filesystem<'static, S::ExternalFlashStorage>,
        vfs: &'static Filesystem<'static, VolatileStorage>,
    ) -> Self {
        unsafe {
            S::ifs_ptr().write(Fs::new(ifs));
            S::efs_ptr().write(Fs::new(efs));
            Self::vfs_ptr().write(Fs::new(vfs));
        }

        Self {
            _marker: Default::default(),
        }
    }

    unsafe fn vfs_ptr() -> *mut Fs<VolatileStorage> {
        static mut VFS: MaybeUninit<Fs<VolatileStorage>> = MaybeUninit::uninit();
        VFS.as_mut_ptr()
    }
}

impl<S: Soc> Clone for RunnerStore<S> {
    fn clone(&self) -> Self {
        Self {
            _marker: self._marker,
        }
    }
}

impl<S: Soc> Copy for RunnerStore<S> {}

unsafe impl<S: Soc> Store for RunnerStore<S> {
    type I = S::InternalFlashStorage;
    type E = S::ExternalFlashStorage;
    type V = VolatileStorage;

    fn ifs(self) -> &'static Fs<Self::I> {
        unsafe { &*S::ifs_ptr() }
    }

    fn efs(self) -> &'static Fs<Self::E> {
        unsafe { &*S::efs_ptr() }
    }

    fn vfs(self) -> &'static Fs<Self::V> {
        unsafe { &*Self::vfs_ptr() }
    }
}

pub fn init_store<S: Soc>(
    int_flash: S::InternalFlashStorage,
    ext_flash: S::ExternalFlashStorage,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> RunnerStore<S> {
    static CLAIMED: AtomicBool = AtomicBool::new(false);
    CLAIMED
        .compare_exchange_weak(false, true, Ordering::AcqRel, Ordering::Acquire)
        .expect("multiple instances of RunnerStore are not allowed");

    static mut VOLATILE_STORAGE: Option<VolatileStorage> = None;
    static mut VOLATILE_FS_ALLOC: Option<Allocation<VolatileStorage>> = None;
    static mut VOLATILE_FS: Option<Filesystem<VolatileStorage>> = None;

    unsafe {
        let ifs_storage = S::ifs_storage().insert(int_flash);
        let ifs_alloc = S::ifs_alloc().insert(Filesystem::allocate());
        let efs_storage = S::efs_storage().insert(ext_flash);
        let efs_alloc = S::efs_alloc().insert(Filesystem::allocate());
        let vfs_storage = VOLATILE_STORAGE.insert(VolatileStorage::new());
        let vfs_alloc = VOLATILE_FS_ALLOC.insert(Filesystem::allocate());

        let ifs = match init_ifs::<S>(ifs_storage, ifs_alloc, efs_storage, status) {
            Ok(ifs) => S::ifs().insert(ifs),
            Err(_e) => {
                error!("IFS Mount Error {:?}", _e);
                panic!("IFS");
            }
        };

        let efs = match init_efs::<S>(efs_storage, efs_alloc, simulated_efs, status) {
            Ok(efs) => S::efs().insert(efs),
            Err(_e) => {
                error!("EFS Mount Error {:?}", _e);
                panic!("EFS");
            }
        };

        let vfs = match init_vfs(vfs_storage, vfs_alloc) {
            Ok(vfs) => VOLATILE_FS.insert(vfs),
            Err(_e) => {
                error!("VFS Mount Error {:?}", _e);
                panic!("VFS");
            }
        };

        RunnerStore::new(ifs, efs, vfs)
    }
}

#[inline(always)]
fn init_ifs<S: Soc>(
    ifs_storage: &'static mut S::InternalFlashStorage,
    ifs_alloc: &'static mut Allocation<S::InternalFlashStorage>,
    efs_storage: &mut S::ExternalFlashStorage,
    status: &mut InitStatus,
) -> LfsResult<Filesystem<'static, S::InternalFlashStorage>> {
    if !Filesystem::is_mountable(ifs_storage) {
        // handle provisioner
        if cfg!(feature = "provisioner") {
            info_now!("IFS mount failed - provisioner => formatting");
            Filesystem::format(ifs_storage).ok();
        } else {
            status.insert(InitStatus::INTERNAL_FLASH_ERROR);
            error_now!("IFS mount-fail");
            S::recover_ifs(ifs_storage, ifs_alloc, efs_storage).ok();
        }
    }

    S::prepare_ifs(ifs_storage);

    Filesystem::mount(ifs_alloc, ifs_storage)
}

#[inline(always)]
fn init_efs<S: Soc>(
    efs_storage: &'static mut S::ExternalFlashStorage,
    efs_alloc: &'static mut Allocation<S::ExternalFlashStorage>,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> LfsResult<Filesystem<'static, S::ExternalFlashStorage>> {
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
