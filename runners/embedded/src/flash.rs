use core::cell::RefCell;

use embedded_hal::{blocking::spi::Transfer, digital::v2::OutputPin};
use littlefs2::{driver::Storage, io::Error};
use spi_memory::{BlockDevice, Read};

struct FlashProperties {
    size: usize,
    jedec: [u8; 3],
    _cont: u8,
}

const FLASH_PROPERTIES: FlashProperties = FlashProperties {
    size: 0x20_0000,
    jedec: [0xc8, 0x40, 0x15],
    _cont: 0, /* should be 6, but device doesn't report those */
};

pub struct ExtFlashStorage<SPI, CS>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    s25flash: RefCell<spi_memory::series25::Flash<SPI, CS>>,
}

impl<SPI, CS> Storage for ExtFlashStorage<SPI, CS>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    const BLOCK_SIZE: usize = 4096;
    const READ_SIZE: usize = 4;
    const WRITE_SIZE: usize = 256;
    const BLOCK_COUNT: usize = FLASH_PROPERTIES.size / Self::BLOCK_SIZE;
    type CACHE_SIZE = generic_array::typenum::U256;
    type LOOKAHEADWORDS_SIZE = generic_array::typenum::U1;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, Error> {
        trace!("EFr {:x} {:x}", off, buf.len());
        if buf.len() == 0 {
            return Ok(0);
        }
        if buf.len() > FLASH_PROPERTIES.size || off > FLASH_PROPERTIES.size - buf.len() {
            return Err(Error::Unknown(0x6578_7046));
        }
        let mut flash = self.s25flash.borrow_mut();
        let r = flash.read(off as u32, buf);
        if r.is_ok() {
            trace!("r >>> {}", delog::hex_str!(&buf[0..4]));
        }
        map_result(r, buf.len())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, Error> {
        trace!("EFw {:x} {:x}", off, data.len());
        trace!("w >>> {}", delog::hex_str!(&data[0..4]));
        const CHUNK_SIZE: usize = 256;
        let mut buf = [0; CHUNK_SIZE];
        let mut off = off as u32;
        let mut flash = self.s25flash.borrow_mut();
        for chunk in data.chunks(CHUNK_SIZE) {
            let buf = &mut buf[..chunk.len()];
            buf.copy_from_slice(chunk);
            flash
                .write_bytes(off, buf)
                .map_err(|_| Error::Unknown(0x6565_6565))?;
            off += CHUNK_SIZE as u32;
        }
        Ok(data.len())
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize, Error> {
        trace!("EFe {:x} {:x}", off, len);
        if len > FLASH_PROPERTIES.size || off > FLASH_PROPERTIES.size - len {
            return Err(Error::Unknown(0x6578_7046));
        }
        let result = self
            .s25flash
            .borrow_mut()
            .erase_sectors(off as u32, len / 256);
        map_result(result, len)
    }
}

fn map_result<SPI, CS>(
    r: Result<(), spi_memory::Error<SPI, CS>>,
    len: usize,
) -> Result<usize, Error>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    match r {
        Ok(()) => Ok(len),
        Err(_) => Err(Error::Unknown(0x6565_6565)),
    }
}

impl<SPI, CS> ExtFlashStorage<SPI, CS>
where
    SPI: Transfer<u8>,
    CS: OutputPin,
{
    fn raw_command(spim: &mut SPI, cs: &mut CS, buf: &mut [u8]) {
        cs.set_low().ok().unwrap();
        spim.transfer(buf).ok().unwrap();
        cs.set_high().ok().unwrap();
    }

    pub fn new(mut spim: SPI, mut cs: CS) -> Self {
        Self::selftest(&mut spim, &mut cs);

        let mut flash = spi_memory::series25::Flash::init(spim, cs).ok().unwrap();
        let jedec_id = flash.read_jedec_id().ok().unwrap();
        info!("Ext. Flash: {:?}", jedec_id);
        if jedec_id.mfr_code() != FLASH_PROPERTIES.jedec[0]
            || jedec_id.device_id() != &FLASH_PROPERTIES.jedec[1..]
        {
            panic!("Unknown Ext. Flash!");
        }
        let s25flash = RefCell::new(flash);

        Self { s25flash }
    }

    pub fn selftest(spim: &mut SPI, cs: &mut CS) {
        macro_rules! doraw {
            ($buf:expr, $len:expr, $str:expr) => {
                let mut buf: [u8; $len] = $buf;
                Self::raw_command(spim, cs, &mut buf);
                trace!($str, delog::hex_str!(&buf[1..]));
            };
        }

        doraw!([0x9f, 0, 0, 0], 4, "JEDEC {}");
        doraw!([0x05, 0], 2, "RDSRl {}");
        doraw!([0x35, 0], 2, "RDSRh {}");
    }

    pub fn size(&self) -> usize {
        FLASH_PROPERTIES.size
    }

    pub fn erase_chip(&mut self) -> Result<usize, Error> {
        map_result(
            self.s25flash.borrow_mut().erase_all(),
            FLASH_PROPERTIES.size,
        )
    }
}
