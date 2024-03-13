#![no_std]

use bitflags::bitflags;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use embedded_hal::digital::v2::InputPin;

#[macro_use]
extern crate delog;

generate_macros!();

pub const NFC_ADDR: u8 = 0x54;

pub struct Ntp53<I2C, ED> {
    i2c: I2C,
    ed: ED,
}

// bitflags! {
//     #[derive(Debug)]
//     pub struct Configuration: u32 {
//         /// 1: SRAM copy on POR disabled
//         const SRAM_COPY_EN               = 0b1000_0000_0000_0000_0000_0000_00000000;
//         const RFU1               = 0b0100_0000_0000_0000_0000_0000_00000000;
//         const RFU2               = 0b0010_0000_0000_0000_0000_0000_00000000;
//         const RFU3               = 0b0001_0000_0000_0000_0000_0000_00000000;
//         const RFU4               = 0b0000_1000_0000_0000_0000_0000_00000000;
//         /// 1: energy harversting for high field strength
//         /// 0: energy harversting for low field strength
//         const EH_MODE            = 0b0000_0100_0000_0000_0000_0000_00000000;
//         const GPIO               = 0b0000_0010_0000_0000_0000_0000_00000000;
//         /// 1: master (NTP5332 only)
//         /// 0: slave
//         const I2C_STATUS               = 0b0000_0001_0000_0000_0000_0000_00000000;
//         const TODO               = 0b0000_0000_1000_0000_0000_0000_00000000;
//         const TODO               = 0b0000_0000_0100_0000_0000_0000_00000000;
//         const TODO               = 0b0000_0000_0010_0000_0000_0000_00000000;
//         /// 1: GPIO_1 input is HIGH
//         const TODO               = 0b0000_0000_0001_0000_0000_0000_00000000;
//         /// 1: GPIO_0 input is HIGH
//         const TODO               = 0b0000_0000_0000_1000_0000_0000_00000000;
//         const TODO               = 0b0000_0000_0000_0100_0000_0000_00000000;
//         /// 1: arbitrer locked to I2C
//         const TODO               = 0b0000_0000_0000_0010_0000_0000_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0001_0000_0000_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_1000_0000_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_0100_0000_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_0010_0000_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_0001_0000_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_0000_1000_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_0000_0100_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_0000_0010_00000000;
//         /// 1: arbitrer locked to NFC
//         const TODO               = 0b0000_0000_0000_0000_0000_0000_00000000;

//     }
// }

bitflags! {
    #[derive(Debug)]
    pub struct StatusRegister: u32 {
        /// 1: EEPROM is busy
        const WR_BUSY           = 0b1000_0000_0000_0000_00000000_00000000;
        /// 1: EEPROM write error happened
        const WR_ERROR          = 0b0100_0000_0000_0000_00000000_00000000;
        /// 1: data is ready, used in pass-through mode
        const SRAM_DATA_READ    = 0b0010_0000_0000_0000_00000000_00000000;
        /// 1: data has been written to SYNCH_BLOCK
        const SYNCH_BLOCK_WRITE = 0b0001_0000_0000_0000_00000000_00000000;
        /// 1: data has been read from SYNCH_BLOCK
        const SYNCH_BLOCK_READ  = 0b0000_1000_0000_0000_00000000_00000000;
        /// 1: I2C to NFC passthrough direction
        /// 0: NFC to I2C passthrough direction
        const PT_TRANSFER_DIR   = 0b0000_0100_0000_0000_00000000_00000000;
        const VCC_SUPPLY_OK     = 0b0000_0010_0000_0000_00000000_00000000;
        const NFC_FIELD_OK      = 0b0000_0001_0000_0000_00000000_00000000;
        const VCC_BOOT_OK       = 0b0000_0000_1000_0000_00000000_00000000;
        const NFC_BOOT_OK       = 0b0000_0000_0100_0000_00000000_00000000;
        const RFU1              = 0b0000_0000_0010_0000_00000000_00000000;
        /// 1: GPIO_1 input is HIGH
        const GPIO1             = 0b0000_0000_0001_0000_00000000_00000000;
        /// 1: GPIO_0 input is HIGH
        const GPIO0             = 0b0000_0000_0000_1000_00000000_00000000;
        const RFU2              = 0b0000_0000_0000_0100_00000000_00000000;
        /// 1: arbitrer locked to I2C
        const I2C_IF_LOCKED     = 0b0000_0000_0000_0010_00000000_00000000;
        /// 1: arbitrer locked to NFC
        const NFC_IF_LOCKED     = 0b0000_0000_0000_0001_00000000_00000000;
    }
}

#[repr(u16)]
pub enum Register {
    Status = 0x10A0,
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
        let block = self.read_register_block(Register::Status as _)?;
        let bits = u32::from_be_bytes(block);
        debug_now!("Status bits: {bits:0b}");
        Ok(StatusRegister::from_bits_retain(bits))
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

    pub fn read_register(&mut self, addr: u16, register_offset: u8) -> Result<u8, E> {
        let [addr_msb, addr_lsb] = addr.to_be_bytes();
        let mut buffer = [0; 1];
        let data = &[addr_msb, addr_lsb, register_offset];
        self.i2c.write_read(NFC_ADDR, data, &mut buffer)?;
        let [register] = buffer;
        Ok(register)
    }

    pub fn read_register_block(&mut self, addr: u16) -> Result<[u8; 4], E> {
        Ok([
            self.read_register(addr, 0)?,
            self.read_register(addr, 1)?,
            self.read_register(addr, 2)?,
            self.read_register(addr, 3)?,
        ])
    }

    /// Write `data` to the register
    /// Only the bits set to 1 in `mask` are written
    ///
    /// Registers are 4 bytes, only the byte `offset` is written
    pub fn write_register(
        &mut self,
        addr: u16,
        register_offset: u8,
        mask: u8,
        data: u8,
    ) -> Result<(), E> {
        let [addr_msb, addr_lsb] = addr.to_be_bytes();
        let data = &[addr_msb, addr_lsb, register_offset, mask, data];
        self.i2c.write(NFC_ADDR, data)
    }

    pub fn test(&mut self) {
        match self.read_register_block(0x10A1) {
            Ok(b) => debug_now!("Config register: {:032b}", u32::from_be_bytes(b)),
            Err(_err) => debug_now!("Could not read config block: {_err:?}"),
        };
        let Ok(_config) = self.read_block(0x1037) else {
            error_now!("Could not read config block: ");
            return;
        };
        debug_now!("Config block: {:032b}", u32::from_be_bytes(_config));

        match self.write_register(0x10A1, 0, 0b0000_0010, 0b0000_0010) {
            Ok(()) => debug_now!("Wrote register"),
            Err(_err) => debug_now!("Failed to write register: {_err:?}"),
        };
        match self.read_register_block(0x10A1) {
            Ok(b) => debug_now!("Config register: {:032b}", u32::from_be_bytes(b)),
            Err(_err) => debug_now!("Could not read config register: {_err:?}"),
        };

        debug_now!("{:?}", self.read_status());
    }
}
