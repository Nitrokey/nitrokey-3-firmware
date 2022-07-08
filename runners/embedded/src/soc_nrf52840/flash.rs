use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};

use crate::types::build_constants::{
       CONFIG_FILESYSTEM_BOUNDARY as FS_BASE,
       CONFIG_FILESYSTEM_END as FS_CEIL
};

pub const FLASH_BASE: *mut u8 = FS_BASE as *mut u8;
pub const FLASH_SIZE: usize = FS_CEIL - FS_BASE;

pub struct FlashStorage {
	nvmc: nrf52840_hal::nvmc::Nvmc<nrf52840_pac::NVMC>,
}

impl littlefs2::driver::Storage for FlashStorage {
	const BLOCK_SIZE: usize = 256;
	const READ_SIZE: usize = 4;
	const WRITE_SIZE: usize = 4;
	const BLOCK_COUNT: usize = FLASH_SIZE / Self::BLOCK_SIZE;
	type CACHE_SIZE = generic_array::typenum::U256;
	type LOOKAHEADWORDS_SIZE = generic_array::typenum::U1;

	// the ReadNorFlash trait exposes a try_read() which (stupidly) expects a mutable self
	// can't get those two to align - so clone the function and drop the mut there
	fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, littlefs2::io::Error> {
		trace!("IFr {:x} {:x}", off, buf.len());
		let res = self.nvmc.read(off as u32, buf);
		nvmc_to_lfs_return(res, buf.len())
	}

	fn write(&mut self, off: usize, buf: &[u8]) -> Result<usize, littlefs2::io::Error> {
		trace!("IFw {:x} {:x}", off, buf.len());
		let res = self.nvmc.write(off as u32, buf);
		nvmc_to_lfs_return(res, buf.len())
	}

	fn erase(&mut self, off: usize, len: usize) -> Result<usize, littlefs2::io::Error> {
		const REAL_BLOCK_SIZE: u32 = 4096;
		trace!("IFe {:x} {:x}", off, len);

		let real_off_remainer: u32 = (off as u32) % REAL_BLOCK_SIZE;
		let real_off: u32 = (off as u32) - real_off_remainer;
		let mut buf: [u8; REAL_BLOCK_SIZE as usize] = [0x00; REAL_BLOCK_SIZE as usize];

		// 1) read the physical block
		let res = self.nvmc.read(real_off as u32, &mut buf);
		if res.is_err() {
			// "best-case" error, lfs can likely handle this
			return nvmc_to_lfs_return(res, len)
		}

		// 2) now erase the full block
		let erase_res = self.nvmc.erase(real_off as u32, (real_off + REAL_BLOCK_SIZE) as u32);
		if erase_res.is_err() {
			// 50:50 depending on whether something was erased or not...
			return nvmc_to_lfs_return(erase_res, len);
		}

		trace!("IFex {:x} {:x} {:x}", real_off, (off - (real_off as usize)), ((real_off_remainer as usize) + len) );

		// 3) write left part back again
		let write_res1 = self.nvmc.write(real_off as u32, &buf[0..(off - (real_off as usize))]);
		// 4) write right part back again
		let write_res2 = self.nvmc.write((off + len) as u32, &buf[((real_off_remainer as usize) + len)..]);

		// even if the 1st write fails - better try both!
		if write_res1.is_err() || write_res2.is_err() {
			// no way lfs can do something useful, thus just pass the 1st error found
			let res = if write_res1.is_err() { write_res1 } else { write_res2 };
			return nvmc_to_lfs_return(res, len);
		}
		// return erase result, as this is what lfs might expect
		nvmc_to_lfs_return(erase_res, len)

		// original implementation, with 4k blocks
		/* nrf52840_hal has nvmc.erase(from, to) */
		//let res = self.nvmc.erase(off as u32, (off+len) as u32);
	}
}

/**
 * Source Result type does not provide a useful Ok value, and Destination Result type
 * does not contain a meaningful low-level error code we could return; so here goes
 * the most stupid result conversion routine ever
 */
fn nvmc_to_lfs_return(r: Result<(), nrf52840_hal::nvmc::NvmcError>, len: usize) -> Result<usize, littlefs2::io::Error> {
	r.map(|_| len)
	.map_err(|_| littlefs2::io::Error::Unknown(0x4e56_4d43))	// 'NVMC'
}

impl FlashStorage {
	pub fn new(nvmc_pac: nrf52840_hal::pac::NVMC) -> Self {
		let buf = unsafe { core::slice::from_raw_parts_mut(FLASH_BASE, FLASH_SIZE) };
		let nvmc = nrf52840_hal::nvmc::Nvmc::new(nvmc_pac, buf);
		Self { nvmc }
	}
}
