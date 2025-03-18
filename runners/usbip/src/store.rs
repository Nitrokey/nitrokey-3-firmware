use std::{
    fs::{File, OpenOptions},
    io::{Read as _, Seek as _, SeekFrom, Write as _},
    marker::PhantomData,
    path::PathBuf,
};

use littlefs2::{
    const_ram_storage,
    driver::Storage,
    fs::{Allocation, Filesystem},
};
use littlefs2_core::{DynFilesystem, Error, Result};
use trussed_usbip::Store;

const IFS_STORAGE_SIZE: usize = 512 * 128;
const FILESYSTEM_BLOCK_SIZE: usize = 512;

const_ram_storage!(InternalRamStorage, IFS_STORAGE_SIZE);
// Modelled after the actual external RAM, see src/flash.rs in the embedded runner
const_ram_storage!(
    name = ExternalRamStorage,
    erase_value = 0xff,
    read_size = 4,
    write_size = 256,
    cache_size = 512,
    block_size = 4096,
    block_count = 0x2_0000 / 4096,
    lookahead_size = 1,
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
    let len = u64::try_from(IFS_STORAGE_SIZE).unwrap();
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
    let alloc = Box::leak(Box::new(Allocation::new(&storage)));
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
    fn read_size(&self) -> usize {
        256
    }

    fn write_size(&self) -> usize {
        256
    }

    fn block_size(&self) -> usize {
        FILESYSTEM_BLOCK_SIZE
    }

    fn cache_size(&self) -> usize {
        256
    }

    fn lookahead_size(&self) -> usize {
        1
    }

    fn block_count(&self) -> usize {
        self.block_size() / IFS_STORAGE_SIZE
    }
    
    type CACHE_BUFFER = S::CACHE_BUFFER;
    type LOOKAHEAD_BUFFER = S::LOOKAHEAD_BUFFER;

    fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<usize> {
        let mut file = File::open(&self.path).unwrap();
        file.seek(SeekFrom::Start(offset as _)).unwrap();
        let bytes_read = file.read(buffer).unwrap();
        assert!(bytes_read <= buffer.len());
        Ok(bytes_read as _)
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<usize> {
        if offset + data.len() > self.block_count() * FILESYSTEM_BLOCK_SIZE {
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
        if offset + len > self.block_count() * FILESYSTEM_BLOCK_SIZE {
            return Err(Error::NO_SPACE);
        }
        let mut file = OpenOptions::new().write(true).open(&self.path).unwrap();
        file.seek(SeekFrom::Start(offset as _)).unwrap();
        let zero_block = vec![0xFFu8; FILESYSTEM_BLOCK_SIZE];
        for _ in 0..(len / FILESYSTEM_BLOCK_SIZE) {
            let bytes_written = file.write(&zero_block).unwrap();
            assert_eq!(bytes_written, FILESYSTEM_BLOCK_SIZE);
        }
        file.flush().unwrap();
        Ok(len)
    }
}
