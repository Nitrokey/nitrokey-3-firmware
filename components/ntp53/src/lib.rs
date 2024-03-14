#![no_std]

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use embedded_hal::digital::v2::InputPin;

pub mod registers;

use registers::*;

#[macro_use]
extern crate delog;

generate_macros!();

pub const NFC_ADDR: u8 = 0x54;

pub struct Ntp53<I2C, ED> {
    i2c: I2C,
    ed: ED,
}

impl<I2C, ED, E> Ntp53<I2C, ED>
where
    I2C: WriteRead<Error = E> + Write<Error = E> + Read<Error = E>,
    E: core::fmt::Debug,
    ED: InputPin,
{
    pub fn new(i2c: I2C, ed: ED) -> Self {
        Self { i2c, ed }
    }

    pub fn read_status(&mut self) -> Result<StatusRegister, E> {
        let status0 = self.read_register(Register::Status, 0)?.into();
        let status1 = self.read_register(Register::Status, 1)?.into();
        Ok(StatusRegister { status0, status1 })
    }

    pub fn read_synch_block_address(&mut self) -> Result<u16, E> {
        let lsb = self.read_register(Register::SyncDataBlock, 0)?;
        let msb = self.read_register(Register::SyncDataBlock, 1)?;
        Ok(u16::from_be_bytes([msb, lsb]))
    }

    pub fn write_synch_block_address(&mut self, addr: u16) -> Result<(), E> {
        let [msb, lsb] = addr.to_be_bytes();
        self.write_register(Register::SyncDataBlock, 0, 0xFF, lsb)?;
        self.write_register(Register::SyncDataBlock, 0, 0xFF, msb)?;
        Ok(())
    }

    pub fn write_block(&mut self, addr: u16, data: [u8; 4]) -> Result<(), E> {
        let [addr_msb, addr_lsb] = addr.to_be_bytes();
        let [b1, b2, b3, b4] = data;
        let buf = [addr_msb, addr_lsb, b1, b2, b3, b4];
        self.i2c.write(NFC_ADDR, &buf)
    }

    pub fn read_block(&mut self, addr: u16) -> Result<[u8; 4], E> {
        let mut data = [0; 4];
        self.i2c
            .write_read(NFC_ADDR, &addr.to_be_bytes(), &mut data)?;
        Ok(data)
    }

    pub fn read_register(&mut self, register: Register, register_offset: u8) -> Result<u8, E> {
        let addr = register as u16;
        let [addr_msb, addr_lsb] = addr.to_be_bytes();
        let mut buffer = [0; 1];
        let data = &[addr_msb, addr_lsb, register_offset];
        self.i2c.write_read(NFC_ADDR, data, &mut buffer)?;
        let [register] = buffer;
        Ok(register)
    }

    pub fn read_register_block(&mut self, register: Register) -> Result<[u8; 4], E> {
        Ok([
            self.read_register(register, 0)?,
            self.read_register(register, 1)?,
            self.read_register(register, 2)?,
            self.read_register(register, 3)?,
        ])
    }

    /// Write `data` to the register
    /// Only the bits set to 1 in `mask` are written
    ///
    /// Registers are 4 bytes, only the byte `offset` is written
    pub fn write_register(
        &mut self,
        register: Register,
        register_offset: u8,
        mask: u8,
        data: u8,
    ) -> Result<(), E> {
        let addr = register as u16;
        let [addr_msb, addr_lsb] = addr.to_be_bytes();
        let data = &[addr_msb, addr_lsb, register_offset, mask, data];
        self.i2c.write(NFC_ADDR, data)
    }

    pub fn test(&mut self) {
        match self.read_register_block(Register::Config) {
            Ok(b) => debug_now!("Config register: {:032b}", u32::from_be_bytes(b)),
            Err(_err) => debug_now!("Could not read config block: {_err:?}"),
        };
        let Ok(_config) = self.read_block(0x1037) else {
            error_now!("Could not read config block: ");
            return;
        };
        debug_now!("Config block: {:032b}", u32::from_be_bytes(_config));

        match self.write_register(Register::Config, 0, 0b0000_0010, 0b0000_0010) {
            Ok(()) => debug_now!("Wrote register"),
            Err(_err) => debug_now!("Failed to write register: {_err:?}"),
        };
        // match self.read_register_block(Register::Config) {
        //     Ok(b) => debug_now!("Config register: {:032b}", u32::from_be_bytes(b)),
        //     Err(_err) => debug_now!("Could not read config register: {_err:?}"),
        // };

        debug_now!("{:?}", self.read_status());
    }
}
