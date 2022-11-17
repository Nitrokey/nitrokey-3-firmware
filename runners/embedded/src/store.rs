// This is mostly copied from store.rs in Trussed, written by Conor Patrick and Nicolas Stalder.
// https://github.com/trussed-dev/trussed/blob/369d32509b6049bac1974966f66e0cffee805b1b/src/store.rs
// Ideally, Trussed would give us a better way to do this without copying code.

use crate::types::{Soc, VolatileStorage, VOLATILE_STORAGE};
use core::{
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering},
};
use littlefs2::fs::Filesystem;
use trussed::store::{Fs, Store};

pub struct RunnerStore<S: Soc> {
    _marker: PhantomData<*mut S>,
}

impl<S: Soc> Clone for RunnerStore<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: Soc> Copy for RunnerStore<S> {}

impl<S: Soc> RunnerStore<S> {
    pub fn init_raw(
        ifs: &'static Filesystem<S::InternalFlashStorage>,
        efs: &'static Filesystem<S::ExternalFlashStorage>,
        vfs: &'static Filesystem<VolatileStorage>,
    ) -> Self {
        let store_ifs = Fs::new(ifs);
        let store_efs = Fs::new(efs);
        let store_vfs = Fs::new(vfs);
        unsafe {
            Self::ifs_ptr().write(store_ifs);
            Self::efs_ptr().write(store_efs);
            Self::vfs_ptr().write(store_vfs);
        }
        Self::claim().unwrap()
    }

    fn claim() -> Option<Self> {
        static CLAIMED: AtomicBool = AtomicBool::new(false);

        if CLAIMED
            .compare_exchange_weak(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            Some(Self {
                _marker: PhantomData,
            })
        } else {
            None
        }
    }

    fn ifs_ptr() -> *mut Fs<S::InternalFlashStorage> {
        unsafe { S::internal_storage().ptr.as_mut_ptr() }
    }

    fn efs_ptr() -> *mut Fs<S::ExternalFlashStorage> {
        unsafe { S::external_storage().ptr.as_mut_ptr() }
    }

    fn vfs_ptr() -> *mut Fs<VolatileStorage> {
        unsafe { VOLATILE_STORAGE.ptr.as_mut_ptr() }
    }
}

unsafe impl<S: Soc> Store for RunnerStore<S> {
    type I = S::InternalFlashStorage;
    type E = S::ExternalFlashStorage;
    type V = VolatileStorage;

    fn ifs(self) -> &'static Fs<Self::I> {
        unsafe { &*Self::ifs_ptr() }
    }

    fn efs(self) -> &'static Fs<Self::E> {
        unsafe { &*Self::efs_ptr() }
    }

    fn vfs(self) -> &'static Fs<Self::V> {
        unsafe { &*Self::vfs_ptr() }
    }
}
