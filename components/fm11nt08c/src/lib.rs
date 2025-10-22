#![no_std]

use core::fmt::Debug;

#[macro_use]
extern crate delog;
generate_macros!();

pub mod registers;

use embedded_hal::{
    blocking::i2c::{Read, Write, WriteRead},
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use embedded_time::duration::Microseconds;
use nfc_device::traits::nfc::{Error as NfcError, State as NfcState};
use registers::{
    AuxIrq, FifoIrqMask, FifoWordCnt, MainIrq, MainIrqMask, NfcCfg, NfcRats, NfcStatus, NfcTxen,
    Register, UserCfg0, UserCfg1, UserCfg2, VoutMode,
};

const ADDRESS: u8 = 0x57;

pub struct Configuration {
    pub regu: u8,
    pub atqa: u16,
    pub sak1: u8,
    pub sak2: u8,
    pub tl: u8,
    pub t0: u8,
    pub ta: u8,
    pub tb: u8,
    pub tc: u8,
    pub nfc: u8,
}

pub struct Fm11nt082c<I2C, CSN, IRQ, Timer> {
    i2c: I2C,
    csn: CSN,
    timer: Timer,
    irq: IRQ,
}

pub struct Txn<'a, I2C, CSN, IRQ, Timer>
where
    CSN: OutputPin,
    CSN::Error: Debug,
{
    device: &'a mut Fm11nt082c<I2C, CSN, IRQ, Timer>,
}

fn addr_to_bytes(addr: u16) -> [u8; 2] {
    let [b1, b2] = addr.to_be_bytes();

    [b1, b2]
}

impl<I2CError, I2C, CSN: OutputPin, IRQ: InputPin, Timer> Txn<'_, I2C, CSN, IRQ, Timer>
where
    I2C: WriteRead<Error = I2CError> + Write<Error = I2CError> + Read<Error = I2CError>,
    I2CError: Debug,
    CSN::Error: Debug,
    IRQ::Error: Debug,
{
    pub fn read_register_raw(&mut self, address: u16) -> Result<u8, I2CError> {
        let buf = &mut [0u8];
        self.device
            .i2c
            .write_read(ADDRESS, &addr_to_bytes(address), buf)?;
        Ok(buf[0])
    }

    // pub fn configure(&mut self, conf: Configuration) -> Result<(), I2CError> {
    //     //         let [addrb1, addrb2] =
    //     //         let [atqab1, atqab2] = conf.atqa.to_be_bytes();
    //     //         let buf = [
    //     //             atqab1,atqab2,
    //     //         ];

    //     // self.device.i2c.write(ADDRESS, )
    // }

    pub fn write_register_raw(&mut self, value: u8, address: u16) -> Result<(), I2CError> {
        let [b1, b2] = addr_to_bytes(address);
        let buf = [b1, b2, value];
        self.device.i2c.write(ADDRESS, &buf)
    }

    pub fn read_register<R: Register>(&mut self) -> Result<R, I2CError> {
        self.read_register_raw(R::ADDRESS).map(R::from)
    }

    pub fn write_register<R: Register>(&mut self, value: R) -> Result<(), I2CError> {
        self.write_register_raw(value.into(), R::ADDRESS)
    }
}

impl<'a, I2C, CSN, IRQ, Timer> Drop for Txn<'a, I2C, CSN, IRQ, Timer>
where
    CSN: OutputPin,
    CSN::Error: Debug,
{
    fn drop(&mut self) {
        self.device.csn.set_high().unwrap();
    }
}

impl<I2CError, I2C, CSN: OutputPin, IRQ: InputPin, Timer> Fm11nt082c<I2C, CSN, IRQ, Timer>
where
    I2C: WriteRead<Error = I2CError> + Write<Error = I2CError> + Read<Error = I2CError>,
    I2CError: Debug,
    CSN::Error: Debug,
    IRQ::Error: Debug,
    Timer: CountDown<Time = Microseconds>,
{
    pub fn new(i2c: I2C, csn: CSN, irq: IRQ, timer: Timer) -> Self {
        Self {
            i2c,
            csn,
            irq,
            timer,
        }
    }

    pub fn init(&mut self) -> Result<(), I2CError> {
        let mut txn = self.txn();
        txn.write_register(NfcTxen(0x88))?;
        let mut user_cfg0 = txn.read_register::<UserCfg0>()?;
        debug_now!("{user_cfg0:?}");
        user_cfg0.set_vout_mode(VoutMode::EnabledAfterPowerOn);
        txn.write_register(user_cfg0)?;
        debug_now!("{:?}", txn.read_register::<UserCfg0>());
        let mut user_cfg_1 = txn.read_register::<UserCfg1>()?;
        debug_now!("{:?}", txn.read_register::<UserCfg2>());
        debug_now!("{:?}", txn.read_register::<NfcCfg>());
        debug_now!("{:?}", txn.read_register::<NfcStatus>());
        debug_now!("{:?}", txn.read_register::<NfcRats>());
        debug_now!("{:?}", txn.read_register::<FifoWordCnt>());
        debug_now!("{:?}", txn.read_register::<NfcTxen>());

        let buf = &mut [0; 4];
        txn.device
            .i2c
            .write_read(ADDRESS, &addr_to_bytes(0x3BC), buf)?;
        debug_now!("{buf:02x?}");

        txn.write_register(AuxIrq(0))?;
        let mut fifo_irq_mask = FifoIrqMask(0);
        fifo_irq_mask.set_water_level_mask(true);
        fifo_irq_mask.set_full_mask(true);
        debug_now!("{fifo_irq_mask:?}");
        txn.write_register(fifo_irq_mask)?;

        let mut main_irq_mask = MainIrqMask(0xFF);
        main_irq_mask.set_rx_start_mask(false);
        main_irq_mask.set_rx_done_mask(false);
        main_irq_mask.set_tx_done_mask(false);
        main_irq_mask.set_fifo_flag_mask(false);
        debug_now!("{main_irq_mask:?}");
        txn.write_register(main_irq_mask)?;
        debug_now!("{:02x?}", txn.read_register::<MainIrqMask>());

        user_cfg_1.set_nfc_mode(registers::NfcMode::Iso14443_4);
        txn.write_register(user_cfg_1)?;

        debug_now!("{:?}", txn.read_register::<UserCfg1>());
        Ok(())
    }

    fn txn<'a>(&'a mut self) -> Txn<'a, I2C, CSN, IRQ, Timer> {
        self.csn.set_low().unwrap();
        self.timer.start(Microseconds::new(250));
        nb::block!(self.timer.wait()).unwrap();
        Txn { device: self }
    }
    pub fn read_register_raw(&mut self, address: u16) -> Result<u8, I2CError> {
        self.txn().read_register_raw(address)
    }

    pub fn write_register_raw(&mut self, value: u8, address: u16) -> Result<(), I2CError> {
        self.txn().write_register_raw(value, address)
    }

    pub fn read_register<R: Register>(&mut self) -> Result<R, I2CError> {
        self.txn().read_register()
    }

    pub fn write_register<R: Register>(&mut self, value: R) -> Result<(), I2CError> {
        self.txn().write_register_raw(value.into(), R::ADDRESS)
    }

    pub fn read_packet(&mut self, buf: &mut [u8]) -> Result<NfcState, I2CError> {
        let main_irq = self.read_register::<MainIrq>();
        debug_now!("{main_irq:?}");
        let aux_irq = self.read_register::<AuxIrq>();
        debug_now!("{aux_irq:?}");
        todo!()
    }
}

impl<I2CError, I2C, CSN: OutputPin, IRQ: InputPin, Timer> nfc_device::traits::nfc::Device
    for Fm11nt082c<I2C, CSN, IRQ, Timer>
where
    I2C: WriteRead<Error = I2CError> + Write<Error = I2CError> + Read<Error = I2CError>,
    I2CError: Debug,
    CSN::Error: Debug,
    IRQ::Error: Debug,
    Timer: CountDown<Time = Microseconds>,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<NfcState, NfcError> {
        self.read_packet(buf).map_err(|_| NfcError::NoActivity)
    }

    fn send(&mut self, _buf: &[u8]) -> Result<(), NfcError> {
        todo!()
    }

    fn frame_size(&self) -> usize {
        128
    }
}
