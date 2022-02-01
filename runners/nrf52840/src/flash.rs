use core::convert::TryInto;
use embedded_storage::nor_flash::{NorFlash};

pub const FLASH_SIZE: usize = 0x2_0000;

pub struct FlashStorage {
	nvmc: nrf52840_hal::nvmc::Nvmc<nrf52840_pac::NVMC>,
}

impl littlefs2::driver::Storage for FlashStorage {
	const BLOCK_SIZE: usize = 4096;
	const READ_SIZE: usize = 4;
	const WRITE_SIZE: usize = 4;
	const BLOCK_COUNT: usize = FLASH_SIZE / Self::BLOCK_SIZE;
	type CACHE_SIZE = generic_array::typenum::U256;
	type LOOKAHEADWORDS_SIZE = generic_array::typenum::U1;

	// the ReadNorFlash trait exposes a try_read() which (stupidly) expects a mutable self
	// can't get those two to align - so clone the function and drop the mut there
	fn read(&self, off: usize, buf: &mut [u8]) -> Result<usize, littlefs2::io::Error> {
		// trace!("F RD {:x} {:x}", off, buf.len());
		let res = self.nvmc.try_read_nonmut(off as u32, buf);
		nvmc_to_lfs_return(res, buf.len())
	}

	fn write(&mut self, off: usize, buf: &[u8]) -> Result<usize, littlefs2::io::Error> {
		// trace!("F WR {:x} {:x}", off, buf.len());
		let res = self.nvmc.try_write(off as u32, buf);
		nvmc_to_lfs_return(res, buf.len())
	}

	fn erase(&mut self, off: usize, len: usize) -> Result<usize, littlefs2::io::Error> {
		// trace!("F ER {:x} {:x}", off, len);
		let res = self.nvmc.try_erase(off as u32, len as u32);
		nvmc_to_lfs_return(res, len)
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
	pub fn new(nvmc_pac: nrf52840_hal::pac::NVMC, mem: *mut u32, size: usize) -> Self {
		if ((mem as usize | size) & 0xfff) != 0 {
			panic!("Invalid NVMC base or size.");
		}
		if size != FLASH_SIZE {
			panic!("Invalid NVMC size.");
		}
		let buf = unsafe { core::slice::from_raw_parts_mut(mem, size).try_into().unwrap() };
		let nvmc = nrf52840_hal::nvmc::Nvmc::new(nvmc_pac, buf);

		Self { nvmc }
	}
}
