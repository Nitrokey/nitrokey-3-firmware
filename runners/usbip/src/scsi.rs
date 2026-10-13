//! File- or memory-backed block device for the USB/IP runner.

use std::{
    fs::OpenOptions,
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use aes::{cipher::KeyInit as _, Aes128};
use usb_classes::scsi::{BlockDevice, Ciphers, EncryptedBlockDevice, BLOCK_SIZE};

use super::usb::UsbClasses;

/// 16384 blocks of 512 bytes, i.e. 8 MiB.
const BLOCKS: u32 = 16_384;

pub struct Setup {
    /// Backing file for the block device; memory when `None`.
    pub block_device: Option<PathBuf>,
    pub block_device_key: Option<[u8; 32]>,
}

pub struct State {
    device: HostBlockDevice,
    xts: Option<Ciphers<Aes128, Aes128>>,
}

impl State {
    pub fn new(setup: &Setup) -> Self {
        let device = HostBlockDevice::open(setup.block_device.as_deref(), BLOCKS)
            .expect("failed to open block device");
        let xts = setup.block_device_key.map(|key| {
            log::info!("storage encryption: AES-128 XTS");
            let ([key1, key2], []) = key.as_chunks() else {
                unreachable!();
            };
            Ciphers {
                cipher1: Aes128::new(key1.into()),
                cipher2: Aes128::new(key2.into()),
            }
        });
        Self { device, xts }
    }

    pub fn poll(&mut self, classes: &mut UsbClasses<'_>) {
        if let Some(xts) = self.xts.as_mut() {
            let mut block_device = EncryptedBlockDevice::new(&mut self.device, xts);
            classes.poll_scsi(&mut block_device);
        } else {
            classes.poll_scsi(&mut self.device);
        }
    }
}

enum Backing {
    File(std::fs::File),
    Memory(Vec<u8>),
}

/// A fixed-size block device.
struct HostBlockDevice {
    backing: Backing,
    blocks: u32,
}

impl HostBlockDevice {
    /// Opens `path` as a file image, or uses memory when `None`.
    fn open(path: Option<&Path>, blocks: u32) -> io::Result<Self> {
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

    fn read_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> io::Result<()> {
        let offset = Self::offset(lba);
        let buf = buf.as_flattened_mut();
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

    fn write_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> io::Result<()> {
        let offset = Self::offset(lba);
        let buf = buf.as_flattened();
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
