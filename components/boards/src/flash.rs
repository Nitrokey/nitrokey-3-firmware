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

// defines how much space we leave untouched at the end
pub const SPARE_LEN: usize = 4096 * 32; // 128kb

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
    fn read_size(&self) -> usize {
        4
    }

    fn write_size(&self) -> usize {
        256
    }

    fn block_size(&self) -> usize {
        4096
    }

    fn cache_size(&self) -> usize {
        256
    }

    fn lookahead_size(&self) -> usize {
        1
    }

    fn block_count(&self) -> usize {
        (FLASH_PROPERTIES.size / self.block_size()) - (SPARE_LEN / self.block_size())
    }

    type CACHE_BUFFER = [u8; 256];
    type LOOKAHEAD_BUFFER = [u8; 8];

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, Error> {
        /*trace!("EFr {:x} {:x}", off, buf.len());
        if buf.len() == 0 {
            return Ok(0);
        }*/
        if buf.len() > FLASH_PROPERTIES.size || off > FLASH_PROPERTIES.size - buf.len() {
            return Err(Error::IO);
        }
        let mut flash = self.s25flash.borrow_mut();
        let r = flash.read(off as u32, buf);
        map_result(r, buf.len())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, Error> {
        trace!("EFw {:x} {:x}", off, data.len());
        const CHUNK_SIZE: usize = 256;
        let mut buf = [0; CHUNK_SIZE];
        let mut off = off as u32;
        let mut flash = self.s25flash.borrow_mut();
        for chunk in data.chunks(CHUNK_SIZE) {
            let buf = &mut buf[..chunk.len()];
            buf.copy_from_slice(chunk);
            flash.write_bytes(off, buf).map_err(|_| Error::IO)?;
            off += CHUNK_SIZE as u32;
        }
        Ok(data.len())
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize, Error> {
        trace!("EFe {:x} {:x}", off, len);
        if len > FLASH_PROPERTIES.size || off > FLASH_PROPERTIES.size - len {
            return Err(Error::IO);
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
        Err(_) => Err(Error::IO),
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

    pub fn try_new(mut spim: SPI, mut cs: CS) -> Option<Self> {
        Self::selftest(&mut spim, &mut cs);

        let mut flash = spi_memory::series25::Flash::init(spim, cs).ok()?;
        let jedec_id = flash.read_jedec_id().ok()?;
        info!("Ext. Flash: {:?}", jedec_id);
        if jedec_id.mfr_code() != FLASH_PROPERTIES.jedec[0]
            || jedec_id.device_id() != &FLASH_PROPERTIES.jedec[1..]
        {
            error_now!("Unknown Ext. Flash: {:?}", jedec_id);
            None
        } else {
            let s25flash = RefCell::new(flash);
            Some(Self { s25flash })
        }
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
