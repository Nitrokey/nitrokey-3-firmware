//! I2C controller

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use stm32n6::stm32n657::I2C1_S;

use crate::{
    gpio::{ALTERNATE_FUNCTION_4, Alternate, PinE5, PinE6, PullUp},
    rcc::{Peripheral, Rcc},
};

pub type SclPin = PinE5<Alternate<PullUp, { ALTERNATE_FUNCTION_4 }>>;
pub type SdaPin = PinE6<Alternate<PullUp, { ALTERNATE_FUNCTION_4 }>>;

const POLL_LIMIT: u32 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Nack,
    Bus,
    Arbitration,
    Timeout,
}

pub struct I2c1 {
    i2c: I2C1_S,
    _pins: (SclPin, SdaPin),
}

impl I2c1 {
    /// 100 kHz from the 64 MHz hsi_div_ck (RM0486 Table 611 scaled by PRESC).
    pub fn new(i2c: I2C1_S, pins: (SclPin, SdaPin), rcc: &Rcc) -> Self {
        rcc.select_i2c1_hsi();
        rcc.enable(Peripheral::I2c1);
        i2c.cr1().modify(|_, w| w.pe().clear_bit());
        i2c.timingr().write(|w| unsafe {
            w.presc()
                .bits(0xF)
                .scldel()
                .bits(0x4)
                .sdadel()
                .bits(0x2)
                .sclh()
                .bits(0xF)
                .scll()
                .bits(0x13)
        });
        i2c.cr1().modify(|_, w| w.pe().set_bit());
        Self { i2c, _pins: pins }
    }

    fn start(&mut self, address: u8, len: usize, read: bool, autoend: bool) {
        self.i2c.cr2().write(|w| unsafe {
            w.sadd()
                .bits((address as u16) << 1)
                .rd_wrn()
                .bit(read)
                .nbytes()
                .bits(len as u8)
                .autoend()
                .bit(autoend)
                .start()
                .set_bit()
        });
    }

    fn check_errors(&mut self) -> Result<(), Error> {
        let isr = self.i2c.isr().read();
        if isr.nackf().bit() {
            self.i2c
                .icr()
                .write(|w| w.nackcf().set_bit().stopcf().set_bit());
            return Err(Error::Nack);
        }
        if isr.arlo().bit() {
            self.i2c.icr().write(|w| w.arlocf().set_bit());
            return Err(Error::Arbitration);
        }
        if isr.berr().bit() {
            self.i2c.icr().write(|w| w.berrcf().set_bit());
            return Err(Error::Bus);
        }
        Ok(())
    }

    fn wait(
        &mut self,
        flag: impl Fn(&stm32n6::stm32n657::i2c1::isr::R) -> bool,
    ) -> Result<(), Error> {
        for _ in 0..POLL_LIMIT {
            if flag(&self.i2c.isr().read()) {
                return Ok(());
            }
            self.check_errors()?;
        }
        Err(Error::Timeout)
    }

    fn wait_stop(&mut self) -> Result<(), Error> {
        self.wait(|isr| isr.stopf().bit())?;
        self.i2c.icr().write(|w| w.stopcf().set_bit());
        Ok(())
    }

    fn send(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for byte in bytes {
            self.wait(|isr| isr.txis().bit())?;
            self.i2c.txdr().write(|w| unsafe { w.txdata().bits(*byte) });
        }
        Ok(())
    }

    fn receive(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        for byte in buffer {
            self.wait(|isr| isr.rxne().bit())?;
            *byte = self.i2c.rxdr().read().rxdata().bits();
        }
        Ok(())
    }
}

impl Write for I2c1 {
    type Error = Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Error> {
        assert!(bytes.len() <= u8::MAX as usize);
        self.wait(|isr| !isr.busy().bit())?;
        self.start(address, bytes.len(), false, true);
        self.send(bytes)?;
        self.wait_stop()
    }
}

impl Read for I2c1 {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        assert!(buffer.len() <= u8::MAX as usize);
        self.wait(|isr| !isr.busy().bit())?;
        self.start(address, buffer.len(), true, true);
        self.receive(buffer)?;
        self.wait_stop()
    }
}

impl WriteRead for I2c1 {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        assert!(bytes.len() <= u8::MAX as usize && buffer.len() <= u8::MAX as usize);
        self.wait(|isr| !isr.busy().bit())?;
        self.start(address, bytes.len(), false, false);
        self.send(bytes)?;
        self.wait(|isr| isr.tc().bit())?;
        self.start(address, buffer.len(), true, true);
        self.receive(buffer)?;
        self.wait_stop()
    }
}
