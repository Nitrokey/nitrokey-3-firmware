//! File- or memory-backed block device for host runners.

use std::{
    fs::OpenOptions,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

use aes::{
    cipher::{generic_array::GenericArray, KeyInit},
    Aes128,
};
use xts_mode::{get_tweak_default, Xts128};

use super::{BlockDevice, BLOCK_SIZE};

enum Backing {
    File(std::fs::File),
    Memory(Vec<u8>),
}

/// A fixed-size block device, optionally encrypted on the fly.
pub struct HostBlockDevice {
    backing: Backing,
    blocks: u32,
    /// AES-128 XTS, keyed when the device is encrypted.
    cipher: Option<Xts128<Aes128>>,
}

impl HostBlockDevice {
    /// Opens `path` as a file image, or uses memory when `None`.
    ///
    /// With a `key`, data is transparently encrypted with AES-128 XTS, each
    /// block tweaked by its own index.
    pub fn open(path: Option<&Path>, blocks: u32, key: Option<[u8; 32]>) -> io::Result<Self> {
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

        let cipher = key.map(|key| {
            log::info!("storage encryption: AES-128 XTS");
            let data_key = Aes128::new(GenericArray::from_slice(&key[..16]));
            let tweak_key = Aes128::new(GenericArray::from_slice(&key[16..]));
            Xts128::new(data_key, tweak_key)
        });

        Ok(Self {
            backing,
            blocks,
            cipher,
        })
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

    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> io::Result<()> {
        let offset = Self::offset(lba);
        match &mut self.backing {
            Backing::File(file) => {
                file.seek(SeekFrom::Start(offset))?;
                file.read_exact(buf)?;
            }
            Backing::Memory(mem) => {
                let offset = offset as usize;
                buf.copy_from_slice(&mem[offset..offset + buf.len()]);
            }
        }

        if let Some(cipher) = &self.cipher {
            cipher.decrypt_area(buf, BLOCK_SIZE as usize, lba as u128, get_tweak_default);
        }
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &[u8]) -> io::Result<()> {
        let mut staging;
        let buf = match &self.cipher {
            Some(cipher) => {
                staging = buf.to_vec();
                cipher.encrypt_area(
                    &mut staging,
                    BLOCK_SIZE as usize,
                    lba as u128,
                    get_tweak_default,
                );
                &staging[..]
            }
            None => buf,
        };

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
    const KEY: [u8; 32] = *b"0123456789abcdef0123456789abcdef";

    fn block(fill: u8) -> Vec<u8> {
        vec![fill; BLOCK_SIZE as usize]
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("nk3-storage-{}-{name}.img", std::process::id()))
    }

    fn raw_block(path: &Path, lba: usize) -> Vec<u8> {
        let raw = fs::read(path).unwrap();
        let offset = lba * BLOCK_SIZE as usize;
        raw[offset..offset + BLOCK_SIZE as usize].to_vec()
    }

    #[test]
    fn memory_round_trip() {
        let mut device = HostBlockDevice::open(None, BLOCKS, None).unwrap();
        assert_eq!(device.blocks(), BLOCKS);

        device.write_block(2, &block(0xA5)).unwrap();

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
            let mut device = HostBlockDevice::open(Some(&path), BLOCKS, None).unwrap();
            device.write_block(3, &block(0x5A)).unwrap();
        }
        assert_eq!(
            fs::metadata(&path).unwrap().len(),
            BLOCKS as u64 * BLOCK_SIZE as u64
        );

        let mut device = HostBlockDevice::open(Some(&path), BLOCKS, None).unwrap();
        let mut buf = block(0);
        device.read_block(3, &mut buf).unwrap();
        assert_eq!(buf, block(0x5A));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn encrypted_round_trip() {
        let path = temp_path("encrypted");
        let _ = fs::remove_file(&path);

        {
            let mut device = HostBlockDevice::open(Some(&path), BLOCKS, Some(KEY)).unwrap();
            device.write_block(1, &block(0xC3)).unwrap();
        }
        assert_ne!(raw_block(&path, 1), block(0xC3));

        let mut device = HostBlockDevice::open(Some(&path), BLOCKS, Some(KEY)).unwrap();
        let mut buf = block(0);
        device.read_block(1, &mut buf).unwrap();
        assert_eq!(buf, block(0xC3));

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn tweak_differs_per_block() {
        let path = temp_path("tweak");
        let _ = fs::remove_file(&path);

        let mut device = HostBlockDevice::open(Some(&path), BLOCKS, Some(KEY)).unwrap();
        device.write_block(0, &block(0xFF)).unwrap();
        device.write_block(1, &block(0xFF)).unwrap();

        assert_ne!(raw_block(&path, 0), raw_block(&path, 1));

        fs::remove_file(&path).unwrap();
    }
}
