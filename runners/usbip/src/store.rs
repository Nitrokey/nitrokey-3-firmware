use std::{
    fs::{File, OpenOptions},
    io::{Read as _, Seek as _, SeekFrom, Write as _},
    marker::PhantomData,
    path::PathBuf,
};

use littlefs2::{
    const_ram_storage,
    consts::{U1, U512, U8},
    driver::Storage,
    fs::{Allocation, Filesystem},
};
use littlefs2_core::{DynFilesystem, Error, Result};
use trussed_usbip::Store;

const IFS_STORAGE_SIZE: usize = 512 * 128;

const_ram_storage!(InternalRamStorage, IFS_STORAGE_SIZE);
// Modelled after the actual external RAM, see src/flash.rs in the embedded runner
const_ram_storage!(
    name = ExternalRamStorage,
    erase_value = 0xff,
    read_size = 4,
    write_size = 256,
    cache_size_ty = U512,
    block_size = 4096,
    block_count = 0x2_0000 / 4096,
    lookahead_size_ty = U1,
    filename_max_plus_one_ty = U256,
    path_max_plus_one_ty = U256,
);
const_ram_storage!(VolatileStorage, IFS_STORAGE_SIZE);

pub fn init(ifs: Option<PathBuf>, efs: Option<PathBuf>) -> Store {
    let ifs = if let Some(ifs) = ifs {
        mount_fs::<InternalRamStorage>(ifs)
    } else {
        mount(InternalRamStorage::new(), true)
    };
    let efs = if let Some(efs) = efs {
        mount_fs::<ExternalRamStorage>(efs)
    } else {
        mount(ExternalRamStorage::new(), true)
    };
    let vfs = mount(VolatileStorage::new(), true);
    Store {
        ifs: ifs.expect("failed to mount IFS"),
        efs: efs.expect("failed to mount EFS"),
        vfs: vfs.expect("failed to mount VFS"),
    }
}

fn mount_fs<S: Storage + 'static>(path: PathBuf) -> Result<&'static dyn DynFilesystem> {
    let len = u64::try_from(S::BLOCK_SIZE * S::BLOCK_COUNT).unwrap();
    let format = if let Ok(file) = File::open(&path) {
        assert_eq!(file.metadata().unwrap().len(), len);
        false
    } else {
        let file = File::create(&path).expect("failed to create storage file");
        file.set_len(len).expect("failed to set storage file len");
        true
    };
    let storage = FilesystemStorage::<S>::new(path);
    mount(storage, format)
}

fn mount<S: Storage + 'static>(storage: S, format: bool) -> Result<&'static dyn DynFilesystem> {
    let alloc = Box::leak(Box::new(Allocation::new()));
    let storage = Box::leak(Box::new(storage));
    if format {
        Filesystem::format(storage)?;
    }
    let fs = Filesystem::mount(alloc, storage)?;
    Ok(Box::leak(Box::new(fs)))
}

pub struct FilesystemStorage<S: Storage> {
    path: PathBuf,
    _storage: PhantomData<S>,
}

impl<S: Storage> FilesystemStorage<S> {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            _storage: Default::default(),
        }
    }
}

impl<S: Storage> Storage for FilesystemStorage<S> {
    const READ_SIZE: usize = S::READ_SIZE;
    const WRITE_SIZE: usize = S::WRITE_SIZE;
    const BLOCK_SIZE: usize = S::BLOCK_SIZE;

    const BLOCK_COUNT: usize = S::BLOCK_COUNT;
    const BLOCK_CYCLES: isize = S::BLOCK_CYCLES;

    type CACHE_SIZE = U512;
    type LOOKAHEAD_SIZE = U8;

    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize> {
        let mut file = File::open(&self.path).unwrap();
        file.seek(SeekFrom::Start(offset as _)).unwrap();
        let bytes_read = file.read(buffer).unwrap();
        assert!(bytes_read <= buffer.len());
        Ok(bytes_read as _)
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<usize> {
        if offset + data.len() > Self::BLOCK_COUNT * Self::BLOCK_SIZE {
            return Err(Error::NO_SPACE);
        }
        let mut file = OpenOptions::new().write(true).open(&self.path).unwrap();
        file.seek(SeekFrom::Start(offset as _)).unwrap();
        let bytes_written = file.write(data).unwrap();
        assert_eq!(bytes_written, data.len());
        file.flush().unwrap();
        Ok(bytes_written)
    }

    fn erase(&mut self, offset: usize, len: usize) -> Result<usize> {
        if offset + len > Self::BLOCK_COUNT * Self::BLOCK_SIZE {
            return Err(Error::NO_SPACE);
        }
        let mut file = OpenOptions::new().write(true).open(&self.path).unwrap();
        file.seek(SeekFrom::Start(offset as _)).unwrap();
        let zero_block = vec![0xFFu8; S::BLOCK_SIZE];
        for _ in 0..(len / Self::BLOCK_SIZE) {
            let bytes_written = file.write(&zero_block).unwrap();
            assert_eq!(bytes_written, Self::BLOCK_SIZE);
        }
        file.flush().unwrap();
        Ok(len)
    }
}
