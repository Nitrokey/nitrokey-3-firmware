#![cfg_attr(not(test), no_std)]

use core::fmt::Debug;

use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use embedded_hal::digital::v2::InputPin;
use embedded_hal::timer::CountDown;
use embedded_time::duration::{
    // Extensions,
    // Duration,
    Microseconds,
};

pub mod blocks_1k;
pub mod registers;

use blocks_1k::Block;

#[macro_use]
extern crate delog;
generate_macros!();

use registers::*;

pub struct Nt3h2111<I2C, FD> {
    i2c: I2C,
    fd: FD,
    address: u8,
}

pub const NDEF: [u8; 20] = [
    0x00, 0x12, /* two-byte length */
    0xd1, /* TNF: well-known + flags */
    0x01, /* payload type length */
    0x0e, /* payload data length */
    0x55, /* payload type: U = URL */
    0x02, /* https://www. */
    0x6e, 0x69, 0x74, 0x72, 0x6f, 0x6b, 0x65, 0x79, 0x2e, 0x63, 0x6f, 0x6d,
    0x2f, /* nitrokey.com/ */
];

impl<I2C, E, FD> Nt3h2111<I2C, FD>
where
    I2C: WriteRead<Error = E> + Write<Error = E> + Read<Error = E>,
    E: Debug,
    FD: InputPin,
{
    pub fn new(i2c: I2C, fd: FD) -> Self {
        Self {
            i2c,
            fd,
            address: 0x55,
        }
    }

    pub fn read_block_raw(&mut self, address: u8) -> Result<[u8; 16], E> {
        let mut buffer = [0; 16];
        debug_now!("Read block raw");
        self.i2c.write_read(self.address, &[address], &mut buffer)?;
        Ok(buffer)
    }

    pub fn read_block<B: Block>(&mut self) -> Result<B, E> {
        self.read_block_raw(B::ADDRESS).map(B::parse)
    }

    pub fn write_block_raw(&mut self, address: u8, data: [u8; 16]) -> Result<(), E> {
        let mut buffer = [0; 17];
        buffer[0] = address;
        buffer[1..].copy_from_slice(&data);
        self.i2c.write(self.address, &buffer)?;
        Ok(())
    }

    pub fn write_block<B: Block>(&mut self, block: B) -> Result<(), E> {
        self.write_block_raw(B::ADDRESS, block.serialize())
    }

    pub fn init(&mut self) {
        debug_now!("Initializing nfc chip!");
    }

    pub fn slow_wdt(&mut self, countdown: &mut impl CountDown<Time = Microseconds>) {
        debug_now!("Testing nfc chip");

        self.write_session_register(0xFF, WdtMs(0xFF)).unwrap();
        self.write_session_register(0xFF, WdtLs(0xFF)).unwrap();

        let mut ns_reg = NsReg(0);
        ns_reg.set_i2c_locked(true);
        debug_now!("NSReg: {ns_reg:?}");
        self.write_session_register(NsReg::I2C_LOCKED, ns_reg)
            .unwrap();
        let ns_reg: NsReg = self.read_session_register().unwrap();
        debug_now!("NSReg: {ns_reg:?}");
    }

    pub fn lock_to_i2c(&mut self, countdown: &mut impl CountDown<Time = Microseconds>) {
        let mut ns_reg_to_write = NsReg(0);
        ns_reg_to_write.set_i2c_locked(true);
        debug_now!("Writing: {ns_reg_to_write:?}");
        self.write_session_register(NsReg::I2C_LOCKED, ns_reg_to_write)
            .unwrap();
        let mut ns_reg: NsReg = self.read_session_register().unwrap();
        debug_now!("NSReg: {ns_reg:?}");
        if ns_reg.i2c_locked() {
            return;
        }
        for _i in 0..10 {
            countdown.start(Microseconds(1_000_000));
            // debug_now!("Waiting for timer");
            nb::block!(countdown.wait()).unwrap();
            let mut ns_reg_to_write = NsReg(0);
            ns_reg_to_write.set_i2c_locked(true);
            debug_now!("Writing: {ns_reg_to_write:?}");
            self.write_session_register(NsReg::I2C_LOCKED, ns_reg_to_write)
                .unwrap();
            ns_reg = self.read_session_register().unwrap();
            debug_now!("NSReg: {ns_reg:?}");
            if ns_reg.i2c_locked() {
                return;
            }
        }
        panic!("Never managed to get I2C lock");
    }

    pub fn debug_dump_sram(&mut self, countdown: &mut impl CountDown<Time = Microseconds>) {
        self.lock_to_i2c(countdown);
        let mut buffer = [0u8; 64];
        for i in 0xF8..0xFB {
            // debug_now!("Starting timer for block {i:02x}");
            countdown.start(Microseconds(10_000));
            // debug_now!("Waiting for timer");
            nb::block!(countdown.wait()).unwrap();
            let block = self.read_block_raw(i.try_into().unwrap()).unwrap();
            buffer[(i - 0xF8) * 16..][..16].copy_from_slice(&block);
        }
        debug_now!("SRAM: {}", hexstr!(&buffer));
    }

    pub fn fill_sram_with_ndef(&mut self, countdown: &mut impl CountDown<Time = Microseconds>) {
        for i in 0xF8..=0xFB {
            // debug_now!("Starting timer for block {i:02x}");
            countdown.start(Microseconds(10_000));
            // debug_now!("Waiting for timer");
            nb::block!(countdown.wait()).unwrap();
            // debug_now!("Waited for timer");
            let mut block = [0; 16];
            for j in 0..16 {
                block[j] = NDEF.get((i - 0xF8) * 16 + j).cloned().unwrap_or(0);
            }
            self.write_block_raw(i.try_into().unwrap(), block).unwrap()
        }
        self.debug_dump_sram(countdown);
    }

    pub fn wait_for_field(&mut self, countdown: &mut impl CountDown<Time = Microseconds>) {
        debug_now!("Waiting for field");
        for _i in 0..usize::MAX {
            // debug_now!("Starting timer");
            countdown.start(Microseconds(10000));
            // debug_now!("Waiting for timer");
            nb::block!(countdown.wait()).unwrap();
            // debug_now!("Waiting for field {_i}");

            let ns_reg = match self.read_session_register::<NsReg>() {
                Ok(reg) => reg,
                Err(_err) => {
                    debug_now!("Failed to read register {_i}: {_err:?}");
                    continue;
                }
            };
            // debug_now!("{ns_reg:?}");
            if ns_reg.rf_field_present() {
                break;
            }

            if self
                .fd
                .is_low()
                .map_err(|_| error_now!("Failed to test fd pin"))
                .unwrap()
            {
                debug_now!("fd is low");
                break;
            }
        }
    }

    pub fn test(&mut self, countdown: &mut impl CountDown<Time = Microseconds>) {
        self.slow_wdt(countdown);
        self.fill_sram_with_ndef(countdown);

        self.lock_to_i2c(countdown);
        self.wait_for_field(countdown);
        assert!(matches!(self.fd.is_low(), Ok(true)));

        debug_now!("Field present");

        let mut passthrough = NcReg(0);
        assert!(!passthrough.pthru_on_off());
        passthrough.set_pthru_on_off(true);
        let res = self.write_session_register(NcReg::PTHRU_ON_OFF, passthrough);
        debug_now!("Passthough set: {res:?}");
        countdown.start(Microseconds(100));
        nb::block!(countdown.wait()).unwrap();

        let ns_reg: NsReg = self.read_session_register().unwrap();
        debug_now!("NSReg: {ns_reg:?}");

        countdown.start(Microseconds(100));
        nb::block!(countdown.wait()).unwrap();

        let passthrough = self.read_session_register::<NcReg>();
        debug_now!("Passthough value: {passthrough:?}");

        self.debug_dump_sram(countdown);

        let ns_reg: NsReg = self.read_session_register().unwrap();
        debug_now!("NsReg: {ns_reg:?}");
        self.lock_to_i2c(countdown);
        self.debug_dump_sram(countdown);
    }

    pub fn read_session_register<T: SessionRegister>(&mut self) -> Result<T, E> {
        let mut buffer = [0; 1];
        self.i2c
            .write_read(self.address, &[0xFE, T::ADDRESS], &mut buffer)?;
        Ok(T::from(buffer[0]))
    }

    pub fn write_session_register<T: SessionRegister>(
        &mut self,
        mask: u8,
        data: T,
    ) -> Result<(), E> {
        self.i2c
            .write(self.address, &[0xFE, T::ADDRESS, mask, data.into()])?;
        Ok(())
    }

    pub fn read_configuration_register<T: ConfigurationRegister>(&mut self) -> Result<T, E> {
        let mut buffer = [0; 1];
        self.i2c
            .write_read(self.address, &[0x3A, T::ADDRESS], &mut buffer)?;
        Ok(T::from(buffer[0]))
    }

    pub fn write_configuration_register<T: ConfigurationRegister>(
        &mut self,
        mask: u8,
        data: T,
    ) -> Result<(), E> {
        self.i2c
            .write(self.address, &[0x3A, T::ADDRESS, mask, data.into()])?;
        Ok(())
    }
}

use nfc_device::traits::nfc;

impl<I2C, E, FD> nfc::Device for Nt3h2111<I2C, FD>
where
    I2C: WriteRead<Error = E> + Write<Error = E> + Read<Error = E>,
    E: Debug,
    FD: InputPin,
{
    fn read(&mut self, _buf: &mut [u8]) -> Result<nfc::State, nfc::Error> {
        Err(nfc::Error::NoActivity)
    }

    fn send(&mut self, _buf: &[u8]) -> Result<(), nfc::Error> {
        Err(nfc::Error::NoActivity)
    }

    fn frame_size(&self) -> usize {
        0
    }
}
