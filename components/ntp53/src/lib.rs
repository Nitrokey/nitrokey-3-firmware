#![no_std]

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use embedded_hal::digital::v2::InputPin;

#[macro_use]
extern crate delog;

generate_macros!();

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

    pub fn test(&mut self) {
        let command = [0x10, 0xA0, 0, 0, 0, 0];
        // RESYNC response
        let mut response = [0; 4];
        let addr = 0x54;
        for i in 1..=command.len() {
            debug_now!("Sending bytes: {i}");
            match self.i2c.write(addr, &command[..i]) {
                Ok(()) => debug_now!("Address {addr} OK"),
                Err(_err) => debug_now!("Address {addr} Err ({_err:?})"),
            };
        }

        debug_now!("{response:02x?}");
    }
}
