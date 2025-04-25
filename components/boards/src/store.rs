use core::{marker::PhantomData, mem::MaybeUninit};

use apps::InitStatus;
use littlefs2::{
    const_ram_storage,
    driver::Storage,
    fs::{Allocation, Filesystem},
    io::Result,
    object_safe::DynFilesystem,
    path,
};
use trussed::store::Store;

use crate::Board;

// 8KB of RAM
const_ram_storage!(
    name = VolatileStorage,
    erase_value = 0xff,
    read_size = 16,
    write_size = 256,
    cache_size_ty = littlefs2::consts::U256,
    // We use 256 instead of the default 512 to avoid loosing too much space to nearly empty blocks containing only folder metadata.
    block_size = 256,
    block_count = 8192 / 256,
    lookahead_size_ty = littlefs2::consts::U1,
    filename_max_plus_one_ty = littlefs2::consts::U256,
    path_max_plus_one_ty = littlefs2::consts::U256,
);

pub struct StoreResources<B: Board> {
    internal: StorageResources<B::InternalStorage>,
    external: StorageResources<B::ExternalStorage>,
    volatile: StorageResources<VolatileStorage>,
}

impl<B: Board> StoreResources<B> {
    pub const fn new() -> Self {
        Self {
            internal: StorageResources::new(),
            external: StorageResources::new(),
            volatile: StorageResources::new(),
        }
    }
}

pub struct StorageResources<S: Storage + 'static> {
    fs: MaybeUninit<Filesystem<'static, S>>,
    alloc: MaybeUninit<Allocation<S>>,
    storage: MaybeUninit<S>,
}

impl<S: Storage + 'static> StorageResources<S> {
    pub const fn new() -> Self {
        Self {
            fs: MaybeUninit::uninit(),
            alloc: MaybeUninit::uninit(),
            storage: MaybeUninit::uninit(),
        }
    }
}

struct StorePointers {
    ifs: MaybeUninit<&'static dyn DynFilesystem>,
    efs: MaybeUninit<&'static dyn DynFilesystem>,
    vfs: MaybeUninit<&'static dyn DynFilesystem>,
}

impl StorePointers {
    const fn new() -> Self {
        Self {
            ifs: MaybeUninit::uninit(),
            efs: MaybeUninit::uninit(),
            vfs: MaybeUninit::uninit(),
        }
    }
}

pub struct RunnerStore<B> {
    _marker: PhantomData<*mut B>,
}

impl<B: Board> RunnerStore<B> {
    fn new(
        ifs: &'static dyn DynFilesystem,
        efs: &'static dyn DynFilesystem,
        vfs: &'static dyn DynFilesystem,
    ) -> Self {
        unsafe {
            let pointers = Self::pointers();
            pointers.ifs.write(ifs);
            pointers.efs.write(efs);
            pointers.vfs.write(vfs);
        }

        Self {
            _marker: Default::default(),
        }
    }

    unsafe fn pointers() -> &'static mut StorePointers {
        static mut POINTERS: StorePointers = StorePointers::new();
        (&raw mut POINTERS).as_mut().unwrap()
    }
}

impl<B> Clone for RunnerStore<B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B> Copy for RunnerStore<B> {}

impl<B: Board> Store for RunnerStore<B> {
    fn ifs(&self) -> &dyn DynFilesystem {
        unsafe { Self::pointers().ifs.assume_init() }
    }

    fn efs(&self) -> &dyn DynFilesystem {
        unsafe { Self::pointers().efs.assume_init() }
    }

    fn vfs(&self) -> &dyn DynFilesystem {
        unsafe { Self::pointers().vfs.assume_init() }
    }
}

pub fn init_store<B: Board>(
    resources: &'static mut StoreResources<B>,
    int_flash: B::InternalStorage,
    ext_flash: B::ExternalStorage,
    simulated_efs: bool,
    status: &mut InitStatus,
) -> RunnerStore<B> {
    let ifs_storage = resources.internal.storage.write(int_flash);
    let ifs_alloc = resources.internal.alloc.write(Filesystem::allocate());
    let efs_storage = resources.external.storage.write(ext_flash);
    let efs_alloc = resources.external.alloc.write(Filesystem::allocate());
    let vfs_storage = resources.volatile.storage.write(VolatileStorage::new());
    let vfs_alloc = resources.volatile.alloc.write(Filesystem::allocate());

    let ifs = match init_ifs::<B>(ifs_storage, ifs_alloc, efs_storage, status) {
        Ok(ifs) => resources.internal.fs.write(ifs),
        Err(_e) => {
            error!("IFS Mount Error {:?}", _e);
            panic!("IFS");
        }
    };

    let efs = match init_efs::<B>(efs_storage, efs_alloc, simulated_efs, status) {
        Ok(efs) => resources.external.fs.write(efs),
        Err(_e) => {
            error!("EFS Mount Error {:?}", _e);
            panic!("EFS");
        }
    };

    let vfs = match init_vfs(vfs_storage, vfs_alloc) {
        Ok(vfs) => resources.volatile.fs.write(vfs),
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
) -> Result<Filesystem<'static, B::InternalStorage>> {
    if cfg!(feature = "format-filesystem") {
        Filesystem::format(ifs_storage).ok();
    }

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
) -> Result<Filesystem<'static, B::ExternalStorage>> {
    if cfg!(feature = "format-filesystem") {
        Filesystem::format(efs_storage).ok();
    }

    let fs = Filesystem::mount_or_else(efs_alloc, efs_storage, |_err, storage| {
        error_now!("EFS Mount Error {:?}", _err);
        let fmt_ext = Filesystem::format(storage);
        if simulated_efs && fmt_ext == Err(littlefs2::io::Error::NO_SPACE) {
            info_now!("Formatting simulated EFS failed as expected");
        } else {
            error_now!("EFS Reformat {:?}", fmt_ext);
            status.insert(InitStatus::EXTERNAL_FLASH_ERROR);
        }
        Ok(())
    })?;

    if fs.exists(path!("/factory-reset-must-reformat")) {
        debug_now!("Reformatting EFS filesystem");
        let (efs_alloc, efs_storage) = fs.into_inner();
        Filesystem::format(efs_storage).ok();
        Filesystem::mount(efs_alloc, efs_storage)
    } else {
        Ok(fs)
    }
}

#[inline(always)]
fn init_vfs(
    vfs_storage: &'static mut VolatileStorage,
    vfs_alloc: &'static mut Allocation<VolatileStorage>,
) -> Result<Filesystem<'static, VolatileStorage>> {
    if !Filesystem::is_mountable(vfs_storage) {
        Filesystem::format(vfs_storage).ok();
    }
    Filesystem::mount(vfs_alloc, vfs_storage)
}
