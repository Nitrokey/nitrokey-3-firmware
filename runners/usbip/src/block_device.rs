//! File- or memory-backed block device for the USB/IP runner.

use std::{
    fs::OpenOptions,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

use log::info;
use usb_classes::storage::{BlockDevice, BLOCK_SIZE};

pub struct Storage;

impl storage_app::Storage for Storage {
    fn init(&mut self, key: &[u8; 32]) -> Result<(), storage_app::Error> {
        info!("Storage::init called with key = {key:?}");
        Ok(())
    }

    fn unlock(&mut self, key: &[u8; 32]) -> Result<(), storage_app::Error> {
        info!("Storage::unlock called with key = {key:?}");
        Ok(())
    }

    fn lock(&mut self) -> Result<(), storage_app::Error> {
        info!("Storage::lock called");
        Ok(())
    }
}

enum Backing {
    File(std::fs::File),
    Memory(Vec<u8>),
}

/// A fixed-size block device, optionally encrypted on the fly.
pub struct HostBlockDevice {
    backing: Backing,
    blocks: u32,
}

impl HostBlockDevice {
    /// Opens `path` as a file image, or uses memory when `None`.
    pub fn open(path: Option<&Path>, blocks: u32) -> io::Result<Self> {
        let len = blocks as u64 * BLOCK_SIZE as u64;

        let backing = match path {
            Some(path) => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(false)
                    .open(path)?;
                // Fixes the size on first use; an existing image keeps its data.
                file.set_len(len)?;
                log::info!("storage image: {}", path.display());
                Backing::File(file)
            }
            None => {
                log::info!("storage in memory");
                Backing::Memory(vec![0; len as usize])
            }
        };

        Ok(Self { backing, blocks })
    }

    fn offset(lba: u32) -> u64 {
        lba as u64 * BLOCK_SIZE as u64
    }
}

impl BlockDevice for HostBlockDevice {
    type Error = io::Error;

    fn blocks(&self) -> u32 {
        self.blocks
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> io::Result<()> {
        let offset = Self::offset(lba);
        match &mut self.backing {
            Backing::File(file) => {
                file.seek(SeekFrom::Start(offset))?;
                file.read_exact(buf)?;
            }
            Backing::Memory(mem) => {
                let offset = offset as usize;
                buf.copy_from_slice(&mem[offset..][..BLOCK_SIZE]);
            }
        }
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> io::Result<()> {
        let offset = Self::offset(lba);
        match &mut self.backing {
            Backing::File(file) => {
                file.seek(SeekFrom::Start(offset))?;
                file.write_all(buf)?;
                file.flush()
            }
            Backing::Memory(mem) => {
                let offset = offset as usize;
                mem[offset..offset + buf.len()].copy_from_slice(buf);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    const BLOCKS: u32 = 4;

    fn block(fill: u8) -> [u8; BLOCK_SIZE] {
        [fill; _]
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("nk3-storage-{}-{name}.img", std::process::id()))
    }

    #[test]
    fn memory_round_trip() {
        let mut device = HostBlockDevice::open(None, BLOCKS).unwrap();
        assert_eq!(device.blocks(), BLOCKS);

        device.write_block(2, &mut block(0xA5)).unwrap();

        let mut buf = block(0);
        device.read_block(2, &mut buf).unwrap();
        assert_eq!(buf, block(0xA5));

        device.read_block(1, &mut buf).unwrap();
        assert_eq!(buf, block(0));
    }

    #[test]
    fn file_is_sized_and_persists() {
        let path = temp_path("persist");
        let _ = fs::remove_file(&path);

        {
            let mut device = HostBlockDevice::open(Some(&path), BLOCKS).unwrap();
            device.write_block(3, &mut block(0x5A)).unwrap();
        }
        assert_eq!(
            fs::metadata(&path).unwrap().len(),
            BLOCKS as u64 * BLOCK_SIZE as u64
        );

        let mut device = HostBlockDevice::open(Some(&path), BLOCKS).unwrap();
        let mut buf = block(0);
        device.read_block(3, &mut buf).unwrap();
        assert_eq!(buf, block(0x5A));

        fs::remove_file(&path).unwrap();
    }
}
