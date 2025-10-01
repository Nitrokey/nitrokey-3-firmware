#![no_std]

use core::fmt::Debug;

#[macro_use]
extern crate delog;
generate_macros!();

use embedded_hal::{
    blocking::i2c::WriteRead,
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use embedded_time::duration::Microseconds;

pub struct Fm11nt082c<I2C, CSN, IRQ> {
    i2c: I2C,
    csn: CSN,
    irq: IRQ,
}

impl<I2C: WriteRead, CSN: OutputPin, IRQ: InputPin> Fm11nt082c<I2C, CSN, IRQ>
where
    I2C::Error: Debug,
    CSN::Error: Debug,
    IRQ::Error: Debug,
{
    pub fn new(i2c: I2C, csn: CSN, irq: IRQ) -> Self {
        Self { i2c, csn, irq }
    }

    pub fn init(&mut self, timer: &mut impl CountDown<Time = Microseconds>) {
        for address in 0x00..0xFF {
            debug_now!("IRQ: {:?}", self.irq.is_high());
            timer.start(Microseconds::new(100_000));
            nb::block!(timer.wait()).unwrap();
            self.csn.set_low().unwrap();
            timer.start(Microseconds::new(250));
            nb::block!(timer.wait()).unwrap();
            let mut buf = [0; 2];
            let res = self.i2c.write_read(address, &[0x00, 0x00], &mut buf);
            self.csn.set_high().unwrap();
            debug_now!("Address: {address:02X}, {res:?}");
        }
    }
}

impl<E: Debug, I2C: WriteRead<Error = E>, CSN: OutputPin, IRQ: InputPin>
    nfc_device::traits::nfc::Device for Fm11nt082c<I2C, CSN, IRQ>
{
    fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<nfc_device::traits::nfc::State, nfc_device::traits::nfc::Error> {
        todo!()
    }

    fn send(&mut self, buf: &[u8]) -> Result<(), nfc_device::traits::nfc::Error> {
        todo!()
    }

    fn frame_size(&self) -> usize {
        todo!()
    }
}
