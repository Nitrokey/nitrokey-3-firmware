// Based on:
//     https://github.com/lpc55/lpc55-hal/blob/25881c90026b3c7f175ccb5863399c1f18f9836c/src/drivers/flash.rs
// Authors: Conor Patrick, Nicolas Stalder
// License: Apache-2.0 or MIT

use littlefs2::{
    driver::Storage,
    io::{Error, Result},
};
use lpc55_hal::{
    drivers::flash::{
        littlefs_params::{self, BLOCK_SIZE},
        FlashGordon,
    },
    peripherals::prince::{Prince, Region},
    traits::flash::WriteErase,
    typestates::init_state::Enabled,
};

use super::MEMORY_REGIONS;

pub const FS_START: usize = MEMORY_REGIONS.filesystem.start;
pub const FS_END: usize = MEMORY_REGIONS.filesystem.end;
pub const BLOCK_COUNT: usize = (FS_END - FS_START) / BLOCK_SIZE;
const _FLASH_SIZE: usize = 631 * 1024 + 512;

const PRINCE_REGION2_START: usize = 0x80_000;
const PRINCE_SUBREGION_SIZE: usize = 8 * 1024;
const PRINCE_REGION2_ENABLE: u32 = {
    let offset = FS_START - PRINCE_REGION2_START;
    let subregion_count = offset / PRINCE_SUBREGION_SIZE;
    0xffffffff << subregion_count
};
const PRINCE_REGION2_DISABLE: u32 = 0;

// Check that the FS is placed in PRINCE Region 2
const _: () = assert!(FS_START >= PRINCE_REGION2_START);
const _: () = assert!(FS_START < FS_END);
const _: () = assert!(FS_END <= _FLASH_SIZE);
// Check that the firmware does not overlap with the PRINCE subregions used for the FS
const _: () = assert!(
    MEMORY_REGIONS.firmware.end
        <= PRINCE_REGION2_START
            + (FS_START - PRINCE_REGION2_START) / PRINCE_SUBREGION_SIZE * PRINCE_SUBREGION_SIZE
);
// Check that offset and size are multiples of the block size
const _: () = assert!(FS_START % BLOCK_SIZE == 0);
const _: () = assert!(FS_END % BLOCK_SIZE == 0);
// Check that flash region does NOT spill over the flash boundary
const _: () = assert!(FS_START + BLOCK_COUNT * BLOCK_SIZE <= _FLASH_SIZE);

pub fn enable(prince: &mut Prince<Enabled>) {
    prince.set_region_enable(Region::Region2, PRINCE_REGION2_ENABLE);
}

pub fn disable(prince: &mut Prince<Enabled>) {
    prince.set_region_enable(Region::Region2, PRINCE_REGION2_DISABLE);
}

pub fn with_enabled<T>(prince: &mut Prince<Enabled>, mut f: impl FnMut() -> T) -> T {
    enable(prince);
    let result = f();
    disable(prince);
    result
}

pub struct InternalFilesystem {
    flash_gordon: FlashGordon,
    prince: Prince<Enabled>,
}

impl InternalFilesystem {
    pub fn new(flash_gordon: FlashGordon, prince: Prince<Enabled>) -> Self {
        Self {
            flash_gordon,
            prince,
        }
    }
}

impl Storage for InternalFilesystem {
    const READ_SIZE: usize = littlefs_params::READ_SIZE;
    const WRITE_SIZE: usize = littlefs_params::WRITE_SIZE;
    const BLOCK_SIZE: usize = BLOCK_SIZE;

    const BLOCK_COUNT: usize = BLOCK_COUNT;
    const BLOCK_CYCLES: isize = littlefs_params::BLOCK_CYCLES;

    type CACHE_SIZE = littlefs_params::CACHE_SIZE;
    type LOOKAHEAD_SIZE = littlefs_params::LOOKAHEAD_SIZE;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize> {
        with_enabled(&mut self.prince, || {
            let flash: *const u8 = (FS_START + off) as *const u8;
            for i in 0..buf.len() {
                buf[i] = unsafe { *flash.offset(i as isize) };
            }
        });
        Ok(buf.len())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize> {
        let ret = self.prince.write_encrypted(|prince| {
            with_enabled(prince, || self.flash_gordon.write(FS_START + off, data))
        });
        ret.map(|_| data.len()).map_err(|_| Error::Io)
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize> {
        assert_eq!(len % BLOCK_SIZE, 0);
        let first_page = (FS_START + off) / BLOCK_SIZE;
        let pages = len / BLOCK_SIZE;
        for i in 0..pages {
            self.flash_gordon
                .erase_page(first_page + i)
                .map_err(|_| Error::Io)?;
        }
        Ok(BLOCK_SIZE * pages)
    }
}
