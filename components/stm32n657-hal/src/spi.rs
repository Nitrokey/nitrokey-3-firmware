//! SPI controller on XSPI1 (single-line indirect mode) -> Section 28

use stm32n6::stm32n657::{XSPI1_S, XSPIM_S};

use crate::{
    Rate,
    gpio::{ALTERNATE_FUNCTION_9, Alternate, Floating, PinO0, PinO4, PinP0, PinP1, PullUp},
    rcc::{Peripheral, Rcc},
};

pub type NcsPin = PinO0<Alternate<PullUp, { ALTERNATE_FUNCTION_9 }>>;
pub type ClkPin = PinO4<Alternate<Floating, { ALTERNATE_FUNCTION_9 }>>;
pub type Io0Pin = PinP0<Alternate<Floating, { ALTERNATE_FUNCTION_9 }>>;
pub type Io1Pin = PinP1<Alternate<PullUp, { ALTERNATE_FUNCTION_9 }>>;

const POLL_LIMIT: u32 = 1_000_000;

// IMODE/ADMODE/DMODE: 001 = single line, 000 = phase skipped
const SINGLE_LINE: u8 = 0b001;
// ADSIZE: 10 = 24-bit address
const ADDRESS_24_BIT: u8 = 0b10;
// DEVSIZE: 2^(23 + 1) bytes, everything a 24-bit address reaches
const DEVICE_SIZE: u8 = 23;

const FMODE_WRITE: u8 = 0b00;
const FMODE_READ: u8 = 0b01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Transfer,
    Timeout,
}

pub struct Xspi1 {
    xspi: XSPI1_S,
    _pins: (NcsPin, ClkPin, Io0Pin, Io1Pin),
}

impl Xspi1 {
    pub fn new(
        xspi: XSPI1_S,
        xspim: XSPIM_S,
        pins: (NcsPin, ClkPin, Io0Pin, Io1Pin),
        rcc: &Rcc,
        max_clock: Rate,
    ) -> Self {
        let _ = xspim;
        rcc.enable(Peripheral::Xspim);
        rcc.enable(Peripheral::Xspi1);
        rcc.reset(Peripheral::Xspi1);

        let prescaler = rcc.xspi1_kernel_clock().to_Hz().div_ceil(max_clock.to_Hz()) - 1;
        assert!(prescaler <= u8::MAX as u32);
        // MTYP: 010 = standard mode
        xspi.dcr1()
            .write(|w| unsafe { w.mtyp().bits(0b010).devsize().bits(DEVICE_SIZE) });
        xspi.dcr2()
            .write(|w| unsafe { w.prescaler().bits(prescaler as u8) });
        xspi.cr().write(|w| w.en().set_bit());
        Self { xspi, _pins: pins }
    }

    /// Instruction with optional 24-bit address and no data phase
    pub fn command(&mut self, instruction: u8, address: Option<u32>) -> Result<(), Error> {
        self.start(FMODE_WRITE, instruction, address, 0)?;
        self.finish()
    }

    pub fn read(
        &mut self,
        instruction: u8,
        address: Option<u32>,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        if buffer.is_empty() {
            return self.command(instruction, address);
        }
        self.start(FMODE_READ, instruction, address, buffer.len())?;
        for byte in buffer {
            self.wait(|sr| sr.flevel().bits() > 0)?;
            *byte = unsafe { self.xspi.dr().as_ptr().cast::<u8>().read_volatile() };
        }
        self.finish()
    }

    pub fn write(
        &mut self,
        instruction: u8,
        address: Option<u32>,
        data: &[u8],
    ) -> Result<(), Error> {
        if data.is_empty() {
            return self.command(instruction, address);
        }
        self.start(FMODE_WRITE, instruction, address, data.len())?;
        for byte in data {
            self.wait(|sr| sr.ftf().bit())?;
            unsafe { self.xspi.dr().as_ptr().cast::<u8>().write_volatile(*byte) };
        }
        self.finish()
    }

    /// The frame starts with the last write of IR, AR or DR that it needs
    fn start(
        &mut self,
        fmode: u8,
        instruction: u8,
        address: Option<u32>,
        len: usize,
    ) -> Result<(), Error> {
        // registers are write-protected while BUSY is set
        self.wait(|sr| !sr.busy().bit())?;
        // FMODE: 00 = indirect write, 01 = indirect read
        self.xspi
            .cr()
            .modify(|_, w| unsafe { w.fmode().bits(fmode) });
        if len > 0 {
            // DL: number of data bytes - 1
            self.xspi
                .dlr()
                .write(|w| unsafe { w.dl().bits(len as u32 - 1) });
        }
        // frame format: 8-bit instruction, optional 24-bit address, optional data
        self.xspi.ccr().write(|w| unsafe {
            w.imode()
                .bits(SINGLE_LINE)
                .admode()
                .bits(if address.is_some() { SINGLE_LINE } else { 0 })
                .adsize()
                .bits(ADDRESS_24_BIT)
                .dmode()
                .bits(if len > 0 { SINGLE_LINE } else { 0 })
        });
        // starts the frame if it has neither address nor written data
        self.xspi
            .ir()
            .write(|w| unsafe { w.instruction().bits(instruction as u32) });
        if let Some(address) = address {
            // starts the frame unless data is written, then the first DR write does
            self.xspi
                .ar()
                .write(|w| unsafe { w.address().bits(address) });
        }
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        self.wait(|sr| sr.tcf().bit())?;
        self.xspi.fcr().write(|w| w.ctcf().set_bit());
        Ok(())
    }

    fn wait(
        &mut self,
        flag: impl Fn(&stm32n6::stm32n657::xspi1::sr::R) -> bool,
    ) -> Result<(), Error> {
        for _ in 0..POLL_LIMIT {
            let sr = self.xspi.sr().read();
            if sr.tef().bit() {
                self.xspi.fcr().write(|w| w.ctef().set_bit());
                return Err(Error::Transfer);
            }
            if flag(&sr) {
                return Ok(());
            }
        }
        // ABORT releases NCS and flushes the FIFO
        self.xspi.cr().modify(|_, w| w.abort().set_bit());
        Err(Error::Timeout)
    }
}
