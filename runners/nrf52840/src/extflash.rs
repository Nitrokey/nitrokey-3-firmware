// use crate::spi_nor_flash::SpiNorFlash;
use embedded_hal::blocking::spi::Transfer;
use nrf52840_hal::{
	gpio::{Output, Pin, PushPull},
	prelude::OutputPin,
	// spim::TransferSplitRead,
};

struct FlashProperties {
	flash_size: usize,
	flash_jedec: [u8; 12],
}

#[cfg(feature = "board-proto1")]
const FLASH_PROPERTIES: FlashProperties = FlashProperties {
	/* GD25Q16C, 16 Mbit == 2 MB */
	flash_size: 0x20_0000,
	/* should really be [0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xc8, 0x40, 0x15, 0xc8],
	   but GigaDevice doesn't understand JEDEC216 */
	flash_jedec: [0x00, 0xc8, 0x40, 0x15, 0xc8, 0x40, 0x15, 0xc8, 0x40, 0x15, 0xc8, 0x40],
};

#[cfg(feature = "board-nk3mini")]
const FLASH_PROPERTIES: FlashProperties = FlashProperties {
	/* GD25Q16C, 16 Mbit == 2 MB */
	flash_size: 0x20_0000,
	/* should really be [0x00, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0xc8, 0x40, 0x15, 0xc8],
	   but GigaDevice doesn't understand JEDEC216 */
	flash_jedec: [0x00, 0xc8, 0x40, 0x15, 0xc8, 0x40, 0x15, 0xc8, 0x40, 0x15, 0xc8, 0x40],
};

#[cfg(feature = "board-nrfdk")]
const FLASH_PROPERTIES: FlashProperties = FlashProperties {
	/* MX25R6435F, 64 Mbit == 8 MB */
	flash_size: 0x80_0000,
	flash_jedec: [0x00, 0xc2, 0x28, 0x17, 0xc2, 0x28, 0x17, 0xc2, 0x28, 0x17, 0xc2, 0x28],
};

pub struct ExtFlashStorage<SPI> where SPI: Transfer<u8> /*+ TransferSplitRead<u8>*/ {
	// extflash: SpiNorFlash<SPI, Pin<Output<PushPull>>>,
	cs_pin: Pin<Output<PushPull>>,
	power_pin: Option<Pin<Output<PushPull>>>,
	_spi: core::marker::PhantomData<SPI>,
}

impl<SPI> littlefs2::driver::Storage for ExtFlashStorage<SPI> where SPI: Transfer<u8> /*+ TransferSplitRead<u8>*/ {

	const BLOCK_SIZE: usize = 4096;
	const READ_SIZE: usize = 4;
	const WRITE_SIZE: usize = 4;
	const BLOCK_COUNT: usize = FLASH_PROPERTIES.flash_size / Self::BLOCK_SIZE;
	type CACHE_SIZE = generic_array::typenum::U256;
	type LOOKAHEADWORDS_SIZE = generic_array::typenum::U1;

	fn read(&self, off: usize, buf: &mut [u8]) -> Result<usize, littlefs2::io::Error> {
		if off + buf.len() > FLASH_PROPERTIES.flash_size {
			return Err(littlefs2::io::Error::Unknown(0x6578_7046));
		}
		let _buf: [u8; 4] = [0x03, (off >> 16) as u8, (off >> 8) as u8, off as u8];
		// if spim.transfer_split_read(buft, bufr0, bufr1).is_err() {

		// trace!("F RD {:x} {:x}", off, buf.len());
		Err(littlefs2::io::Error::Unknown(0x6565_6565))
	}

	fn write(&mut self, off: usize, buf: &[u8]) -> Result<usize, littlefs2::io::Error> {
		// trace!("F WR {:x} {:x}", off, buf.len());
		Err(littlefs2::io::Error::Unknown(0x6565_6565))
	}

	fn erase(&mut self, off: usize, len: usize) -> Result<usize, littlefs2::io::Error> {
		// trace!("F ER {:x} {:x}", off, len);
		Err(littlefs2::io::Error::Unknown(0x6565_6565))
	}
}

impl<SPI> ExtFlashStorage<SPI> where SPI: Transfer<u8> /*+ TransferSplitRead<u8>*/ {

	pub fn new(_spim: &mut SPI, cs: Pin<Output<PushPull>>, power_pin: Option<Pin<Output<PushPull>>>) -> Self {
		Self { cs_pin: cs, power_pin, _spi: core::marker::PhantomData {} }
	}

	pub fn init(&mut self, spim: &mut SPI) {
		unsafe {
			let spim3_pac = nrf52840_hal::pac::Peripherals::steal().SPIM3;
			/* TODO: ensure stopped before writing CSN? */
			spim3_pac.psel.csn.write(|w| w.bits(self.cs_pin.psel_bits()));
		}
		self.power_on();

		let mut jedec = [0u8; 12];
		read_jedec(spim, &mut jedec);
		if jedec != FLASH_PROPERTIES.flash_jedec {
			error!("FLASH JEDEC Mismatch: {:?}", jedec);
			panic!("jedec");
		}

		let density = get_sfdp_attributes(spim);
		if density >> 3 != FLASH_PROPERTIES.flash_size {
			error!("FLASH SIZE Mismatch: {:x}", density);
			panic!("density");
		}
	}

	fn power_on(&mut self) {
		if let Some(pwr_pin) = self.power_pin.as_mut() {
			pwr_pin.set_high().ok();
			crate::board_delay(200u32);
		}
	}

	pub fn power_off(&mut self) {
		if let Some(pwr_pin) = self.power_pin.as_mut() {
			pwr_pin.set_low().ok();
		}
	}
}

fn read_jedec<SPI>(spim: &mut SPI, buf: &mut [u8; 12]) where SPI: Transfer<u8> {
	buf.fill(0);
	buf[0] = 0x9f;

	if spim.transfer(buf).is_err() {
		buf.fill(0);
	}
}

#[inline(never)]
fn get_sfdp_attributes<SPI>(spim: &mut SPI) -> usize where SPI: Transfer<u8> {
	let mut buf = [0u8; 384];
	buf[0] = 0x5a;

/* doesn't work (= flash stops transmission) on Proto1
	let (buft, bufr) = buf.split_at_mut(8);
	let (bufr0, bufr1) = bufr.split_at_mut(8);
	if spim.transfer_split_read(buft, bufr0, bufr1).is_err() {
		return 1;
	}
*/
	if spim.transfer(&mut buf).is_err() {
		return 1;
	}
	let (_, bufr1) = buf.split_at_mut(5);
	if u32::from_le_bytes([bufr1[0], bufr1[1], bufr1[2], bufr1[3]]) != 0x5044_4653 {
		return 2;
	}
	let nph = bufr1[6];
	for i in 0usize..(1+nph as usize) {
		let off = u32::from_le_bytes([bufr1[i*8+12], bufr1[i*8+13], bufr1[i*8+14], 0]) as usize;
		let len: usize = (bufr1[i*8+11] as usize) * 4;
		if (off+len) > bufr1.len() { return 3; }
		if bufr1[i*8+8] == 0x00u8 {	/* JEDEC parameter block found */
			let raw_density = u32::from_le_bytes([bufr1[off+4], bufr1[off+5], bufr1[off+6], bufr1[off+7]]);
			if raw_density & 0x8000_0000 != 0 {
				return 1usize << (raw_density & 0x7fff_ffffu32);
			} else {
				return (raw_density + 1) as usize;
			}
		}
	}

	return 0;
}
