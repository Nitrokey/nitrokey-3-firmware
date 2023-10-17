use littlefs2::{driver::Storage, io::{Error, Result}};
use lpc55_hal::{
    drivers::flash::{FlashGordon, littlefs_params::{self, BLOCK_SIZE}},
    peripherals::prince::{Prince, Region}, traits::flash::WriteErase, typestates::init_state::Enabled,
};

use crate::types::build_constants::CONFIG_FILESYSTEM_BOUNDARY;

const BASE_OFFSET: usize = CONFIG_FILESYSTEM_BOUNDARY;
const FLASH_SIZE: usize = 631 * 1024 + 512;
const BLOCK_COUNT: usize = (FLASH_SIZE - BASE_OFFSET) / BLOCK_SIZE;

const PRINCE_REGION2_START: usize = 0x80_000;
const PRINCE_REGION2_ENABLE: u32 = {
    let offset = BASE_OFFSET - PRINCE_REGION2_START;
    let subregion_count = offset / 8 / 1024;
    0xffffffff << subregion_count
};
const PRINCE_REGION2_DISABLE: u32 = 0;

const _: () = assert!(BASE_OFFSET < FLASH_SIZE);
const _: () = assert!(BASE_OFFSET > PRINCE_REGION2_START);
// Compile time assertion that offset and size are 512 byte aligned.
const _: () = assert!(BASE_OFFSET % 512 == 0);
const _: () = assert!(FLASH_SIZE % 512 == 0);
// Compile time assertion that flash region does NOT spill over the flash boundary.
const _: () = assert!(BASE_OFFSET + BLOCK_COUNT * BLOCK_SIZE <= FLASH_SIZE);

pub fn enable(prince: &mut Prince<Enabled>) {
    prince.set_region_enable(Region::Region2, PRINCE_REGION2_ENABLE);
}

pub fn disable(prince: &mut Prince<Enabled>) {
    prince.set_region_enable(Region::Region2, PRINCE_REGION2_DISABLE);
}

pub fn with_enabled<T>(prince: &mut Prince<Enabled>, mut f: impl FnMut() -> T) -> T{
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
        Self { flash_gordon, prince }
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
            let flash: *const u8 = (BASE_OFFSET + off) as *const u8;
            for i in 0 .. buf.len() {
                buf[i] = unsafe{ *flash.offset(i as isize) };
            }
        });
        Ok(buf.len())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize> {
        let prince = &mut self.prince;
        let flash_gordon = &mut self.flash_gordon;
        let ret = prince.write_encrypted(|prince| {
            with_enabled(prince, || {
                flash_gordon.write(BASE_OFFSET + off, data)
            })
        });
        ret
            .map(|_| data.len())
            .map_err(|_| Error::Io)
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize> {
        let first_page = (BASE_OFFSET + off) / 512;
        let pages = len / 512;
        for i in 0..pages {
            self.flash_gordon
                .erase_page(first_page + i)
                .map_err(|_| Error::Io)?;
        }
        Ok(512 * len)
    }

}
