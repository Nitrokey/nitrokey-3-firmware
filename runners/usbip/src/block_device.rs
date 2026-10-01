//! File- or memory-backed block device for the USB/IP runner.

use std::{
    fs::OpenOptions,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
    slice,
};

use usb_classes::scsi::{BlockDevice, BLOCK_SIZE};

enum Backing {
    File(std::fs::File),
    Memory(Vec<[u8; BLOCK_SIZE]>),
}

/// A fixed-size block device, optionally encrypted on the fly.
pub struct HostBlockDevice {
    backing: Backing,
    blocks: u32,
}

impl HostBlockDevice {
    /// Opens `path` as a file image, or uses memory when `None`.
    pub fn open(path: Option<&Path>, blocks: u32) -> io::Result<Self> {
        let backing = if let Some(path) = path {
            let len = blocks as u64 * BLOCK_SIZE as u64;
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
        } else {
            log::info!("storage in memory");
            Backing::Memory(vec![[0x00; BLOCK_SIZE]; blocks as usize])
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
        self.read_blocks(lba, slice::from_mut(buf))
    }

    fn read_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> io::Result<()> {
        match &mut self.backing {
            Backing::File(file) => {
                let buf = buf.as_flattened_mut();
                let offset = Self::offset(lba);
                file.seek(SeekFrom::Start(offset))?;
                file.read_exact(buf)?;
            }
            Backing::Memory(mem) => {
                let block = usize::try_from(lba).unwrap();
                buf.copy_from_slice(&mem[block..][..buf.len()]);
            }
        }
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> io::Result<()> {
        self.write_blocks(lba, slice::from_mut(buf))
    }

    fn write_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> Result<(), Self::Error> {
        match &mut self.backing {
            Backing::File(file) => {
                let buf = buf.as_flattened();
                let offset = Self::offset(lba);
                file.seek(SeekFrom::Start(offset))?;
                file.write_all(buf)?;
                file.flush()
            }
            Backing::Memory(mem) => {
                let block = usize::try_from(lba).unwrap();
                mem[block..][..buf.len()].copy_from_slice(buf);
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
