use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

use crate::nk3am::MEMORY_REGIONS;

const FS_BASE: usize = MEMORY_REGIONS.filesystem.start;
const FS_CEIL: usize = MEMORY_REGIONS.filesystem.end;

pub const FLASH_BASE: *mut u8 = FS_BASE as *mut u8;
pub const FLASH_SIZE: usize = FS_CEIL - FS_BASE;

pub struct FlashStorage {
    nvmc: nrf52840_hal::nvmc::Nvmc<nrf52840_pac::NVMC>,
}

impl littlefs2::driver::Storage for FlashStorage {
    fn read_size(&self) -> usize {
        4
    }

    fn write_size(&self) -> usize {
        4
    }

    fn block_size(&self) -> usize {
        256
    }

    fn cache_size(&self) -> usize {
        256
    }

    fn lookahead_size(&self) -> usize {
        1
    }

    fn block_count(&self) -> usize {
        FLASH_SIZE / self.block_size()
    }

    type CACHE_BUFFER = [u8; 256];
    type LOOKAHEAD_BUFFER = [u8; 8];

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, littlefs2::io::Error> {
        // w/o this too much spam is generated, thus writes/deletes traces get lost
        if buf.len() > 4 {
            trace!("IFr {:x} {:x}", off, buf.len());
        }
        let res = self.nvmc.read(off as u32, buf);
        nvmc_to_lfs_return(res, buf.len())
    }

    fn write(&mut self, off: usize, buf: &[u8]) -> Result<usize, littlefs2::io::Error> {
        trace!("IFw {:x} {:x}", off, buf.len());
        let res = self.nvmc.write(off as u32, buf);
        nvmc_to_lfs_return(res, buf.len())
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize, littlefs2::io::Error> {
        trace!("EE {:x} {:x}", off, len);

        const REAL_BLOCK_SIZE: usize = 4 * 1024;

        let block_off: usize = off - (off % REAL_BLOCK_SIZE);

        let mut buf: [u8; REAL_BLOCK_SIZE] = [0x00; REAL_BLOCK_SIZE];
        self.nvmc
            .read(block_off as u32, &mut buf)
            .expect("EE - failed read");
        let erase_res = self
            .nvmc
            .erase(block_off as u32, (block_off + REAL_BLOCK_SIZE) as u32);

        let left_end: usize = off - block_off;
        if left_end > 0 {
            self.nvmc
                .write(block_off as u32, &buf[..left_end])
                .expect("EE - failed write 1");
        }

        let right_off: usize = left_end + len;
        if REAL_BLOCK_SIZE - right_off > 0 {
            self.nvmc
                .write((off + len) as u32, &buf[right_off..])
                .expect("EE - failed write 2");
        }

        nvmc_to_lfs_return(erase_res, len)
    }
}

/**
 * Source Result type does not provide a useful Ok value, and Destination Result type
 * does not contain a meaningful low-level error code we could return; so here goes
 * the most stupid result conversion routine ever
 */
fn nvmc_to_lfs_return(
    r: Result<(), nrf52840_hal::nvmc::NvmcError>,
    len: usize,
) -> Result<usize, littlefs2::io::Error> {
    r.map(|_| len).map_err(|_| littlefs2::io::Error::IO) // 'NVMC'
}

impl FlashStorage {
    pub fn new(nvmc_pac: nrf52840_hal::pac::NVMC) -> Self {
        let buf = unsafe { core::slice::from_raw_parts_mut(FLASH_BASE, FLASH_SIZE) };
        let nvmc = nrf52840_hal::nvmc::Nvmc::new(nvmc_pac, buf);
        Self { nvmc }
    }
}
