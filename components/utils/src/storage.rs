use core::marker::PhantomData;

use littlefs2::{
    driver::Storage,
    fs::{Attribute, FileOpenFlags, Metadata},
    io::{Error, OpenSeekFrom},
    object_safe::{DirEntriesCallback, DynFilesystem, FileCallback, Predicate},
    path::Path,
};

pub struct EmptyFilesystem;

impl DynFilesystem for EmptyFilesystem {
    fn total_blocks(&self) -> usize {
        0
    }

    fn total_space(&self) -> usize {
        0
    }
    fn available_blocks(&self) -> Result<usize, Error> {
        Ok(0)
    }
    fn available_space(&self) -> Result<usize, Error> {
        Ok(0)
    }

    fn remove(&self, _: &Path) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn remove_dir(&self, _: &Path) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn remove_dir_all(&self, _: &Path) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn remove_dir_all_where(&self, _: &Path, _: Predicate) -> Result<usize, Error> {
        Err(Error::NO_SPACE)
    }
    fn rename(&self, _: &Path, _: &Path) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn exists(&self, _: &Path) -> bool {
        false
    }
    fn metadata(&self, _: &Path) -> Result<Metadata, Error> {
        Err(Error::NO_SPACE)
    }

    fn create_file_and_then_unit(&self, _: &Path, _: FileCallback) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn open_file_and_then_unit(&self, _: &Path, _: FileCallback) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn open_file_with_flags_and_then_unit(
        &self,
        _: FileOpenFlags,
        _: &Path,
        _: FileCallback,
    ) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn attribute<'a>(
        &self,
        _: &Path,
        _: u8,
        _: &'a mut [u8],
    ) -> Result<Option<Attribute<'a>>, Error> {
        Err(Error::NO_SPACE)
    }
    fn remove_attribute<'a>(&self, _: &Path, _: u8) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn set_attribute(&self, _: &Path, _: u8, _: &[u8]) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }

    fn create_dir(&self, _: &Path) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }

    fn create_dir_all(&self, _: &Path) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn write(&self, _: &Path, _: &[u8]) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn write_chunk(&self, _: &Path, _: &[u8], _: OpenSeekFrom) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
    fn read_dir_and_then_unit(&self, _: &Path, _: DirEntriesCallback<'_>) -> Result<(), Error> {
        Err(Error::NO_SPACE)
    }
}

pub trait MaybeStorage: 'static {
    type Storage: Storage + 'static;

    fn as_storage(&mut self) -> Option<&mut Self::Storage>;
}

/// Exists only to avoid conflicting impls
pub struct OptionalStorage<S>(pub Option<S>);

impl<T> From<Option<T>> for OptionalStorage<T> {
    fn from(value: Option<T>) -> Self {
        Self(value)
    }
}

/// Exists only to avoid conflicting impls
pub struct PhantomStorage<S>(pub PhantomData<S>);

impl<S> Default for PhantomStorage<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<S: Storage + 'static> MaybeStorage for OptionalStorage<S> {
    type Storage = S;
    fn as_storage(&mut self) -> Option<&mut S> {
        self.0.as_mut()
    }
}

impl<S: Storage + 'static> MaybeStorage for S {
    type Storage = S;
    fn as_storage(&mut self) -> Option<&mut S> {
        Some(self)
    }
}

impl<S: Storage + 'static> MaybeStorage for PhantomStorage<S> {
    type Storage = S;
    fn as_storage(&mut self) -> Option<&mut S> {
        None
    }
}
