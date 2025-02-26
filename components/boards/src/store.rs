use core::{marker::PhantomData, mem::MaybeUninit};

use apps::InitStatus;
use littlefs2::{
    const_ram_storage,
    driver::Storage,
    driver::Storage as LfsStorage,
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use trussed::store::{Fs, Store};

use crate::Board;

// 8KB of RAM
const_ram_storage!(
    name = VolatileStorage,
    trait = LfsStorage,
    erase_value = 0xff,
    read_size = 16,
    write_size = 256,
    cache_size_ty = littlefs2::consts::U256,
    // We use 256 instead of the default 512 to avoid loosing too much space to nearly empty blocks containing only folder metadata.
    block_size = 256,
    block_count = 8192/256,
    lookahead_size_ty = littlefs2::consts::U1,
    filename_max_plus_one_ty = littlefs2::consts::U256,
    path_max_plus_one_ty = littlefs2::consts::U256,
    result = LfsResult,
);

pub struct StoreResources<B: Board> {
    ifs: MaybeUninit<Filesystem<'static, B::InternalStorage>>,
    ifs_alloc: MaybeUninit<Allocation<B::InternalStorage>>,
    efs: MaybeUninit<Filesystem<'static, B::ExternalStorage>>,
    efs_alloc: MaybeUninit<Allocation<B::ExternalStorage>>,
    efs_storage: MaybeUninit<B::ExternalStorage>,
    vfs: MaybeUninit<Filesystem<'static, VolatileStorage>>,
    vfs_alloc: MaybeUninit<Allocation<VolatileStorage>>,
    vfs_storage: MaybeUninit<VolatileStorage>,
}

impl<B: Board> StoreResources<B> {
    pub const fn new() -> Self {
        Self {
            ifs: MaybeUninit::uninit(),
            ifs_alloc: MaybeUninit::uninit(),
            efs: MaybeUninit::uninit(),
            efs_alloc: MaybeUninit::uninit(),
            efs_storage: MaybeUninit::uninit(),
            vfs: MaybeUninit::uninit(),
            vfs_alloc: MaybeUninit::uninit(),
            vfs_storage: MaybeUninit::uninit(),
        }
    }
}

// FIXME: document safety
#[allow(clippy::missing_safety_doc)]
#[cfg(feature = "provisioner")]
pub unsafe fn steal_internal_storage<S: StoragePointers>() -> &'static mut S::InternalStorage {
    S::ifs_storage().assume_init_mut()
}

// FIXME: document safety
#[allow(clippy::missing_safety_doc)]
pub trait StoragePointers: 'static {
    type InternalStorage: Storage;
    type ExternalStorage: Storage;

    unsafe fn ifs_storage() -> &'static mut MaybeUninit<Self::InternalStorage>;
    unsafe fn ifs_ptr() -> *mut Fs<Self::InternalStorage>;

    unsafe fn efs_ptr() -> *mut Fs<Self::ExternalStorage>;
}

#[cfg_attr(
    not(any(feature = "board-nk3am", feature = "board-nk3xn")),
    allow(unused)
)]
macro_rules! impl_storage_pointers {
    ($name:ident, Internal = $I:ty, External = $E:ty,) => {
        impl $crate::store::StoragePointers for $name {
            type InternalStorage = $I;
            type ExternalStorage = $E;

            unsafe fn ifs_storage() -> &'static mut ::core::mem::MaybeUninit<Self::InternalStorage>
            {
                static mut IFS_STORAGE: ::core::mem::MaybeUninit<$I> =
                    ::core::mem::MaybeUninit::uninit();
                (&mut *&raw mut IFS_STORAGE)
            }

            unsafe fn ifs_ptr() -> *mut ::trussed::store::Fs<Self::InternalStorage> {
                static mut IFS: ::core::mem::MaybeUninit<::trussed::store::Fs<$I>> =
                    ::core::mem::MaybeUninit::uninit();
                let ifs_ptr: *mut ::core::mem::MaybeUninit<::trussed::store::Fs<$I>> = &raw mut IFS;
                ifs_ptr as _
            }

            unsafe fn efs_ptr() -> *mut ::trussed::store::Fs<Self::ExternalStorage> {
                static mut EFS: ::core::mem::MaybeUninit<::trussed::store::Fs<$E>> =
                    ::core::mem::MaybeUninit::uninit();
                let efs_ptr: *mut ::core::mem::MaybeUninit<::trussed::store::Fs<$E>> = &raw mut EFS;
                efs_ptr as _
            }
        }
    };
}

#[cfg_attr(
    not(any(feature = "board-nk3am", feature = "board-nk3xn")),
    allow(unused)
)]
pub(crate) use impl_storage_pointers;

pub struct RunnerStore<S> {
    _marker: PhantomData<*mut S>,
}

impl<S: StoragePointers> RunnerStore<S> {
    fn new(
        ifs: &'static Filesystem<'static, S::InternalStorage>,
        efs: &'static Filesystem<'static, S::ExternalStorage>,
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
        let vfs_ptr: *mut MaybeUninit<Fs<VolatileStorage>> = &raw mut VFS;
        vfs_ptr as _
    }
}

impl<S> Clone for RunnerStore<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for RunnerStore<S> {}

unsafe impl<S: StoragePointers> Store for RunnerStore<S> {
    type I = S::InternalStorage;
    type E = S::ExternalStorage;
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

pub fn init_store<B: Board>(
    resources: &'static mut StoreResources<B>,
    int_flash: B::InternalStorage,
    ext_flash: B::ExternalStorage,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> RunnerStore<B> {
    let ifs_alloc = resources.ifs_alloc.write(Filesystem::allocate());
    let efs_storage = resources.efs_storage.write(ext_flash);
    let efs_alloc = resources.efs_alloc.write(Filesystem::allocate());
    let vfs_storage = resources.vfs_storage.write(VolatileStorage::new());
    let vfs_alloc = resources.vfs_alloc.write(Filesystem::allocate());

    let ifs_storage = unsafe { B::ifs_storage().write(int_flash) };

    let ifs = match init_ifs::<B>(ifs_storage, ifs_alloc, efs_storage, status) {
        Ok(ifs) => resources.ifs.write(ifs),
        Err(_e) => {
            error!("IFS Mount Error {:?}", _e);
            panic!("IFS");
        }
    };

    let efs = match init_efs::<B>(efs_storage, efs_alloc, simulated_efs, status) {
        Ok(efs) => resources.efs.write(efs),
        Err(_e) => {
            error!("EFS Mount Error {:?}", _e);
            panic!("EFS");
        }
    };

    let vfs = match init_vfs(vfs_storage, vfs_alloc) {
        Ok(vfs) => resources.vfs.write(vfs),
        Err(_e) => {
            error!("VFS Mount Error {:?}", _e);
            panic!("VFS");
        }
    };

    RunnerStore::new(ifs, efs, vfs)
}

#[inline(always)]
fn init_ifs<B: Board>(
    ifs_storage: &'static mut B::InternalStorage,
    ifs_alloc: &'static mut Allocation<B::InternalStorage>,
    efs_storage: &mut B::ExternalStorage,
    status: &mut InitStatus,
) -> LfsResult<Filesystem<'static, B::InternalStorage>> {
    if !Filesystem::is_mountable(ifs_storage) {
        // handle provisioner
        if cfg!(feature = "provisioner") {
            info_now!("IFS mount failed - provisioner => formatting");
            Filesystem::format(ifs_storage).ok();
        } else {
            status.insert(InitStatus::INTERNAL_FLASH_ERROR);
            error_now!("IFS mount-fail");
            B::recover_ifs(ifs_storage, ifs_alloc, efs_storage).ok();
        }
    }

    B::prepare_ifs(ifs_storage);

    Filesystem::mount(ifs_alloc, ifs_storage)
}

#[inline(always)]
fn init_efs<B: Board>(
    efs_storage: &'static mut B::ExternalStorage,
    efs_alloc: &'static mut Allocation<B::ExternalStorage>,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> LfsResult<Filesystem<'static, B::ExternalStorage>> {
    Filesystem::mount_or_else(efs_alloc, efs_storage, |_err, storage| {
        error_now!("EFS Mount Error {:?}", _err);
        let fmt_ext = Filesystem::format(storage);
        if simulated_efs && fmt_ext == Err(littlefs2::io::Error::NO_SPACE) {
            info_now!("Formatting simulated EFS failed as expected");
        } else {
            error_now!("EFS Reformat {:?}", fmt_ext);
            status.insert(InitStatus::EXTERNAL_FLASH_ERROR);
        }
        Ok(())
    })
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
