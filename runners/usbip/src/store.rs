// for store!
#![allow(clippy::too_many_arguments)]

use std::marker::PhantomData;

use littlefs2::{const_ram_storage, fs::Allocation};
use trussed::{
    store,
    types::{LfsResult, LfsStorage},
    virt::StoreProvider,
};
use utils::OptionalStorage;

const STORAGE_SIZE: usize = 512 * 128;

static mut INTERNAL_RAM_STORAGE: Option<InternalStorage> = None;
static mut INTERNAL_RAM_FS_ALLOC: Option<Allocation<InternalStorage>> = None;

static mut EXTERNAL_STORAGE: Option<ExternalStorage> = None;
static mut EXTERNAL_FS_ALLOC: Option<Allocation<ExternalStorage>> = None;

static mut VOLATILE_STORAGE: Option<VolatileStorage> = None;
static mut VOLATILE_FS_ALLOC: Option<Allocation<VolatileStorage>> = None;

const_ram_storage!(InternalStorage, STORAGE_SIZE);
// Modelled after the actual external RAM, see src/flash.rs in the embedded runner
const_ram_storage!(
    name=ExternalRamStorage,
    trait=LfsStorage,
    erase_value=0xff,
    read_size=4,
    write_size=256,
    cache_size_ty=littlefs2::consts::U512,
    block_size=4096,
    block_count=0x2_0000 / 4096,
    lookaheadwords_size_ty=littlefs2::consts::U2,
    filename_max_plus_one_ty=littlefs2::consts::U256,
    path_max_plus_one_ty=littlefs2::consts::U256,
    result=LfsResult,
);
const_ram_storage!(VolatileStorage, STORAGE_SIZE);

// TODO: use 256 -- would cause a panic because formatting fails
type ExternalStorage = OptionalStorage<ExternalRamStorage, 4356>;

store!(
    RamStore,
    Internal: InternalStorage,
    External: ExternalStorage,
    Volatile: VolatileStorage
);

#[derive(Copy, Clone, Debug, Default)]
pub struct Ram;

impl StoreProvider for Ram {
    type Store = RamStore;

    unsafe fn ifs() -> &'static mut InternalStorage {
        INTERNAL_RAM_STORAGE.as_mut().expect("ifs not initialized")
    }

    unsafe fn store() -> Self::Store {
        Self::Store { __: PhantomData }
    }

    unsafe fn reset(&self) {
        INTERNAL_RAM_STORAGE.replace(InternalStorage::new());
        INTERNAL_RAM_FS_ALLOC.replace(littlefs2::fs::Filesystem::allocate());
        reset_external();
        reset_volatile();

        Self::store()
            .mount(
                INTERNAL_RAM_FS_ALLOC.as_mut().unwrap(),
                INTERNAL_RAM_STORAGE.as_mut().unwrap(),
                EXTERNAL_FS_ALLOC.as_mut().unwrap(),
                EXTERNAL_STORAGE.as_mut().unwrap(),
                VOLATILE_FS_ALLOC.as_mut().unwrap(),
                VOLATILE_STORAGE.as_mut().unwrap(),
                true,
            )
            .expect("failed to mount filesystem");
    }
}

unsafe fn reset_external() {
    EXTERNAL_STORAGE.replace(ExternalStorage::default());
    EXTERNAL_FS_ALLOC.replace(littlefs2::fs::Filesystem::allocate());
}

unsafe fn reset_volatile() {
    VOLATILE_STORAGE.replace(VolatileStorage::new());
    VOLATILE_FS_ALLOC.replace(littlefs2::fs::Filesystem::allocate());
}
