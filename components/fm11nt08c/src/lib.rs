#![no_std]

use core::fmt::Debug;

#[macro_use]
extern crate delog;
generate_macros!();

mod crc8;
pub mod registers;

use crc8::crc8;
use embedded_hal::{
    blocking::i2c::{Read, Write, WriteRead},
    digital::v2::{InputPin, OutputPin},
    timer::CountDown,
};
use embedded_time::duration::Microseconds;
use nfc_device::traits::nfc::{Error as NfcError, State as NfcState};
use registers::{
    FifoAccess, FifoIrq, FifoIrqMask, FifoWordCnt, MainIrq, MainIrqMask, NfcRats, NfcStatus,
    NfcTxen, NfcTxenValue, Register, ResetSilence, UserCfg0, UserCfg1, UserCfg2, VoutResCfg,
};

// bits: 4 (rx_done), 5 (rx_start), 6 (active_flag), 7 (power_on_flag) enabled; bit 1 (fifo_flag)
const MAIN_IRQ_MASK_ACTIVE: u8 = 0x0D;
// bits: upper 4 = 1 (rfu), 3 (water_level), 2 (overflow)
const FIFO_IRQ_MASK_ACTIVE: u8 = 0xF3;

const BLOCK_SIZE: usize = 16;

bitfield::bitfield! {
    struct T0(u8);
    impl Debug;
    pub tc_transmitted, set_tc_transmitted: 6;
    pub tb_transmitted, set_tb_transmitted: 5;
    pub ta_transmitted, set_ta_transmitted: 4;
    pub fsci, set_fsci: 3,0;
}

bitfield::bitfield! {
    struct Ta(u8);
    impl Debug;
    pub same_bitrate_both_direction, set_same_bitrate_both_direction: 7;
    pub same_bitrate_poll_8, set_same_bitrate_poll_8: 6;
    pub same_bitrate_poll_4, set_same_bitrate_poll_4: 5;
    pub same_bitrate_poll_2, set_same_bitrate_poll_2: 4;
    pub rfu, _: 3;
    pub same_bitrate_listen_8, set_same_bitrate_listen_8: 2;
    pub same_bitrate_listen_4, set_same_bitrate_listen_4: 1;
    pub same_bitrate_listen_2, set_same_bitrate_listen_2: 0;
}

bitfield::bitfield! {
    struct Tb(u8);
    impl Debug;
    pub fwi, set_fwi: 7,4;
    pub sfgi, set_sfgi: 3,0;
}

bitfield::bitfield! {
    struct Tc(u8);
    impl Debug;
    pub fwi, set_fwi: 7,4;
    pub sfgi, set_sfgi: 3,0;
}

fn fsdi_to_frame_size(fsdi: u8) -> usize {
    match fsdi {
        0 => 16,
        1 => 24,
        2 => 32,
        3 => 40,
        4 => 48,
        5 => 64,
        6 => 96,
        7 => 128,
        _ => 256,
    }
}

pub trait I2CError: Debug {
    fn is_address_nack(&self) -> bool;
    fn is_data_nack(&self) -> bool;
}

pub trait I2CBus:
    Read<Error = <Self as I2CBus>::BusError>
    + Write<Error = <Self as I2CBus>::BusError>
    + WriteRead<Error = <Self as I2CBus>::BusError>
{
    type BusError: I2CError;
}

impl<T, E> I2CBus for T
where
    E: I2CError,
    T: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
{
    type BusError = E;
}

mod i2cimpl;

const ADDRESS: u8 = 0x57;

pub struct Configuration {
    pub user_cfg0: UserCfg0,
    pub user_cfg1: UserCfg1,
    pub user_cfg2: UserCfg2,
    pub atqa: u16,
    pub sak1: u8,
    pub sak2: u8,
    pub tl: u8,
    pub t0: u8,
    pub ta: u8,
    pub tb: u8,
    pub tc: u8,
    pub vout_reg_cfg: VoutResCfg,
}

pub struct Fm11nt082c<I2C, CSN, IRQ, Timer> {
    i2c: I2C,
    csn: CSN,
    timer: Timer,
    #[allow(unused)]
    irq: IRQ,
    current_frame_size: usize,
    offset: usize,
    packet: [u8; 256],
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

impl<I2C, CSN: OutputPin, IRQ: InputPin, Timer> Txn<'_, I2C, CSN, IRQ, Timer>
where
    I2C: I2CBus,
    CSN::Error: Debug,
    IRQ::Error: Debug,
    Timer: CountDown<Time = Microseconds>,
{
    pub fn read_register_raw(&mut self, address: u16) -> Result<u8, I2C::BusError> {
        let buf = &mut [0u8];
        self.device
            .i2c
            .write_read(ADDRESS, &addr_to_bytes(address), buf)?;
        Ok(buf[0])
    }

    pub fn read_eeprom(&mut self, address: u16, buf: &mut [u8]) -> Result<(), I2C::BusError> {
        self.device
            .i2c
            .write_read(ADDRESS, &addr_to_bytes(address), buf)
    }

    /// Returns whether there should be a wait before the next transactio
    fn write_page(
        &mut self,
        address: u16,
        data: &[u8],
        checked: bool,
    ) -> Result<bool, I2C::BusError> {
        if checked {
            let buf = &mut [0; BLOCK_SIZE][..data.len()];
            self.read_eeprom(address, buf)?;
            if buf == data {
                debug!("Not writing data thanks to check");
                return Ok(false);
            }
        }
        let mut buf = [0; BLOCK_SIZE + 2];
        buf[0] = address.to_be_bytes()[0];
        buf[1] = address.to_be_bytes()[1];
        buf[2..][..data.len()].copy_from_slice(data);
        self.device.i2c.write(ADDRESS, &buf[..data.len() + 2])?;
        Ok(true)
    }

    /// If checked is true, only write when the data is not the same
    pub fn write_eeprom(
        &mut self,
        mut address: u16,
        mut data: &[u8],
        checked: bool,
    ) -> Result<(), I2C::BusError> {
        let mut should_wait = false;
        if !address.is_multiple_of(BLOCK_SIZE as _) {
            let offset = BLOCK_SIZE - (address as usize % BLOCK_SIZE);
            if data.len() > offset {
                let tmp = data.split_at(offset);
                data = tmp.1;
                should_wait |= self.write_page(address, tmp.0, checked)?;
                address += offset as u16;
            } else {
                should_wait |= self.write_page(address, data, checked)?;
                data = &[];
            }
        }
        for (idx, chunk) in data.chunks(BLOCK_SIZE).enumerate() {
            let adr = address + (idx * BLOCK_SIZE) as u16;
            should_wait |= self.write_page(adr, chunk, checked)?;
        }
        if should_wait {
            self.device.timer.start(Microseconds::new(10_000));
            nb::block!(self.device.timer.wait()).unwrap();
        }
        Ok(())
    }

    pub fn configure(&mut self, conf: Configuration) -> Result<(), I2C::BusError> {
        debug!("Configure");
        // // NOT documented in the datasheet
        // // FIXME: use bitlfags to document what is being configured
        // const REGU_CONFIG: u8 = (0b11 << 4) | (0b10 << 2) | (0b11 << 0);
        // // In the example code, FM11_E2_REGU_CFG_ADDR, not documented in the datasheet
        // const REGU_ADDR: u16 = 0x0391;
        // let buf = &mut [0; 1];

        let mut crc8_buf = [0; 13];
        const SERIAL_NUMBER_ADDRESS: u16 = 0x0000;
        self.read_eeprom(SERIAL_NUMBER_ADDRESS, &mut crc8_buf[..9])?;
        crc8_buf[9..].copy_from_slice(&[
            conf.atqa.to_be_bytes()[0],
            conf.atqa.to_be_bytes()[1],
            conf.sak1,
            conf.sak2,
        ]);
        let crc8 = crc8(&crc8_buf);
        let config_buf = [
            conf.tl,
            conf.t0,
            conf.vout_reg_cfg.0,
            ADDRESS,
            conf.ta,
            conf.tb,
            conf.tc,
            0x1E, // default value
            conf.user_cfg0.0,
            conf.user_cfg1.0,
            conf.user_cfg2.0,
            crc8,
            conf.atqa.to_be_bytes()[0],
            conf.atqa.to_be_bytes()[1],
            conf.sak1,
            conf.sak2,
        ];
        self.write_eeprom(0x3B0, &config_buf, true)?;

        Ok(())
    }

    pub fn write_register_raw(&mut self, value: u8, address: u16) -> Result<(), I2C::BusError> {
        let [b1, b2] = addr_to_bytes(address);
        let buf = [b1, b2, value];
        self.device.i2c.write(ADDRESS, &buf)
    }

    pub fn read_register<R: Register>(&mut self) -> Result<R, I2C::BusError> {
        self.read_register_raw(R::ADDRESS).map(R::from)
    }

    pub fn write_register<R: Register>(&mut self, value: R) -> Result<(), I2C::BusError> {
        self.write_register_raw(value.into(), R::ADDRESS)
    }

    fn write_fifo(&mut self, data: &[u8]) -> Result<(), I2C::BusError> {
        let len = data.len() + 2;
        // Max length of FIFO (32 bytes + address)
        let mut buf = [0; 32 + 2];
        buf[..2].copy_from_slice(&FifoAccess::ADDRESS.to_be_bytes());
        buf[2..][..data.len()].copy_from_slice(data);
        self.device.i2c.write(ADDRESS, &buf[..len])?;
        Ok(())
    }

    fn dump_registers(&mut self) {
        // return;
        // debug!("Frame size: {}", self.device.current_frame_size);
        // let aux_irq = self.read_register::<AuxIrq>();
        // debug!("{:02x?}", aux_irq);
        // debug!("{:02x?}", self.read_register::<AuxIrqMask>());
        // // debug!("{:02x?}", self.read_register::<FifoAccess>());
        // // debug!("{:02x?}", self.read_register::<FifoClear>());
        // debug!("{:02x?}", self.read_register::<FifoIrq>());
        // debug!("{:02x?}", self.read_register::<FifoIrqMask>());
        // debug!("WORDCOUNT: {:02x?}", self.read_register::<FifoWordCnt>());
        // debug!("{:02x?}", self.read_register::<MainIrq>());
        // debug!("{:02x?}", self.read_register::<MainIrqMask>());
        // debug!("{:02x?}", self.read_register::<NfcCfg>());
        // debug!("{:02x?}", self.read_register::<NfcRats>());
        // // debug!("{:02x?}", self.read_register::<NfcTxen>());
        // // debug!("{:02x?}", self.read_register::<ResetSilence>());
        // debug!("{:02x?}", self.read_register::<registers::Status>());
        // debug!("{:02x?}", self.read_register::<UserCfg0>());
        // debug!("{:02x?}", self.read_register::<UserCfg1>());
        // debug!("{:02x?}", self.read_register::<UserCfg2>());
        // debug!("{:02x?}", self.read_register::<VoutEnCfg>());
        // debug!("{:02x?}", self.read_register::<VoutResCfg>());
        // debug!("{:02x?}", self.read_register::<NfcStatus>());
    }
}

// impl<'a, I2C, CSN, IRQ, Timer> Drop for Txn<'a, I2C, CSN, IRQ, Timer>
// where
//     CSN: OutputPin,
//     CSN::Error: Debug,
// {
//     fn drop(&mut self) {
//         // self.device.csn.set_high().unwrap();
//     }
// }

impl<I2C, CSN: OutputPin, IRQ: InputPin, Timer> Fm11nt082c<I2C, CSN, IRQ, Timer>
where
    I2C: I2CBus,
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
            current_frame_size: 128,
            offset: 0,
            packet: [0; 256],
        }
    }

    pub fn close(mut self) -> (I2C, CSN, IRQ, Timer) {
        self.csn.set_high().unwrap();
        (self.i2c, self.csn, self.irq, self.timer)
    }

    /// Initialize the chip.
    pub fn init(&mut self, configure: bool) -> Result<(), I2C::BusError> {
        debug!("Init");
        self.csn.set_low().unwrap();
        let mut txn = self.txn();

        if configure {
            let user_cfg0 = UserCfg0(0x91);
            let user_cfg1 = UserCfg1(0x82);
            let user_cfg2 = UserCfg2(0x21);

            let usercfg_chk_word = !(user_cfg0.0 ^ user_cfg1.0 ^ user_cfg2.0);
            txn.write_eeprom(
                0x0390,
                &[user_cfg0.0, user_cfg1.0, user_cfg2.0, usercfg_chk_word],
                true,
            )?;

            let mut t0 = T0(0);
            t0.set_tc_transmitted(true);
            t0.set_tb_transmitted(true);
            t0.set_ta_transmitted(true);
            // Means 256 bytes FSCI
            t0.set_fsci(0x8);

            let ta = Ta(0x80);

            let mut tb = Tb(0);
            // FWT = 256 * 16/fc * 2^FWI
            tb.set_fwi(7);
            // SFGT = 256 * 16/fc * 2^SFGI
            tb.set_sfgi(8);

            // Same values as old chip
            assert_eq!(0x78, t0.0);
            assert_eq!(0x80, ta.0);
            assert_eq!(0x78, tb.0);

            txn.configure(Configuration {
                user_cfg0,
                user_cfg1,
                user_cfg2,
                atqa: 0x4400,
                sak1: 0x04,
                sak2: 0x20,
                // Length (5 = TL + T0 + TA + TB + TC)
                tl: 0x05,
                t0: t0.0,
                ta: ta.0,
                tb: tb.0,
                // No advanced protocol features supported
                // DID not supported
                // NAD not supported
                tc: 0x00,
                // current limiting resistance impedance when power output
                vout_reg_cfg: VoutResCfg(0xF0),
            })?;
        }

        txn.write_register(MainIrqMask(MAIN_IRQ_MASK_ACTIVE))?;
        txn.write_register(FifoIrqMask(FIFO_IRQ_MASK_ACTIVE))?;
        txn.write_register(NfcTxen(0x77))?;
        txn.write_register(ResetSilence(0x55))?;

        //self.timer.start(Microseconds::new(500));
        //nb::block!(self.timer.wait()).unwrap();
        Ok(())
    }

    /// Get a transaction
    ///
    /// While the transaction is open, the `csn` stays low
    fn txn<'a>(&'a mut self) -> Txn<'a, I2C, CSN, IRQ, Timer> {
        // self.csn.set_low().unwrap();
        // self.timer.start(Microseconds::new(250));
        // nb::block!(self.timer.wait()).unwrap();
        Txn { device: self }
    }
    pub fn read_register_raw(&mut self, address: u16) -> Result<u8, I2C::BusError> {
        self.txn().read_register_raw(address)
    }

    pub fn write_register_raw(&mut self, value: u8, address: u16) -> Result<(), I2C::BusError> {
        self.txn().write_register_raw(value, address)
    }

    pub fn read_register<R: Register>(&mut self) -> Result<R, I2C::BusError> {
        self.txn().read_register()
    }

    pub fn write_register<R: Register>(&mut self, value: R) -> Result<(), I2C::BusError> {
        self.txn().write_register_raw(value.into(), R::ADDRESS)
    }

    pub fn read_fifo(&mut self, count: u8) -> Result<(), I2C::BusError> {
        let txn = self.txn();
        let buf: &mut [u8] = &mut txn.device.packet[txn.device.offset..][..count as usize];
        txn.device
            .i2c
            .write_read(ADDRESS, &FifoAccess::ADDRESS.to_be_bytes(), buf)?;
        // txn.write_register(FifoClear(0))?;
        Ok(())
    }

    pub fn read_packet(
        &mut self,
        buf: &mut [u8],
    ) -> Result<Result<NfcState, NfcError>, I2C::BusError> {
        // self.set_led_state();
        // self.dump_registers();
        let main_irq = self.read_register::<MainIrq>()?;
        let fifo_irq = self.read_register::<FifoIrq>()?;
        if fifo_irq.overflow() {
            error!("FM11 FIFO overflow during RX: data lost");
        }

        self.write_register(MainIrqMask(MAIN_IRQ_MASK_ACTIVE))?;

        let mut new_session = false;

        if main_irq.active_flag() {
            self.offset = 0;
            new_session = true;
        }

        if main_irq.rx_start() {
            //     error!("1");
            self.offset = 0;
            self.current_frame_size = fsdi_to_frame_size(self.read_register::<NfcRats>()?.fsdi());
            //     debug!(
            //     "Rx start  ============================================================================: {}",
            //     self.current_frame_size
            // );
        }
        let count = self.read_register::<FifoWordCnt>()?.fifo_wordcnt();
        debug!(
            "WORD_COUNT: {count:02x?}, offset: {:02x},{}{}{}{}",
            self.offset,
            if main_irq.rx_done() { " rx_done " } else { "" },
            if main_irq.rx_start() {
                " rx_start "
            } else {
                ""
            },
            if fifo_irq.water_level() {
                " water_level "
            } else {
                ""
            },
            if fifo_irq.overflow() {
                " overflow "
            } else {
                ""
            },
        );

        // Case where the full packet is available
        if main_irq.rx_done() {
            // error!("RxDone reached");
            if count == 32 {
                //         error!("Fifo FULL");
            }
            // let count = count.min(24);
            if count > 0 {
                self.read_fifo(count)?;
                self.offset += count as usize;
            }

            if self.offset <= 2 {
                // too few bytes, ignore..
                //         info!("RxDone read too few ({})", hex_str!(&buf[..self.offset]));
                self.offset = 0;
            } else {
                //         info!("RxDone");
                let l = self.offset - 2;
                buf[..l].copy_from_slice(&self.packet[..l]);
                self.offset = 0;
                if new_session {
                    //             debug!("New session read suscessfull");
                    return Ok(Ok(NfcState::NewSession(l as u8)));
                } else {
                    //             debug!("Continue read successfull");
                    return Ok(Ok(NfcState::Continue(l as u8)));
                }
            }
        }

        let rf_status = self.read_register::<NfcStatus>()?;
        // debug!("bare Count: {:02x?}", self.read_register::<FifoWordCnt>());
        // debug!("water_level: {fifo_irq:?}");
        if !rf_status.nfc_tx() {
            let count = count.min(24);
            if count > 0 {
                self.read_fifo(count)?;
                self.offset += count as usize;
            }
        }

        // debug!("Packet {}", self.offset);
        // debug!("{}", hexstr!(&self.packet[..self.offset]));

        if new_session {
            //     debug!("NewSession read incomplete");
            Ok(Err(NfcError::NewSession))
        } else {
            //     debug!("No activity read incomplete");
            Ok(Err(NfcError::NoActivity))
        }
    }

    fn write_fifo(&mut self, data: &[u8]) -> Result<(), I2C::BusError> {
        self.txn().write_fifo(data)
    }

    #[allow(unused)]
    /// Returns true for sucess
    fn wait_for_transmission(&mut self) -> Result<bool, I2C::BusError> {
        self.write_register(NfcTxen(NfcTxenValue::SendBackData.into()))?;

        let mut nfc_status = self.read_register::<NfcStatus>()?;
        loop {
            if nfc_status.nfc_tx() {
                //         debug!("Chip is transmitting");
                break;
            } else {
                // chip is not transmitting yet
                nfc_status = self.read_register::<NfcStatus>()?;
            }
        }

        if !nfc_status.nfc_tx() {
            //     debug_now!("Chip never started transmitting");
            return Ok(false);
        }

        let mut current_count = self.read_register::<FifoWordCnt>()?.fifo_wordcnt();

        let mut fifo_irq = self.read_register::<FifoIrq>()?;
        if current_count < 8 {
            return Ok(true);
        }
        for _ in 0..300 {
            if fifo_irq.water_level() {
                break;
            }
            current_count = self.read_register::<FifoWordCnt>()?.fifo_wordcnt();
            if current_count < 8 {
                return Ok(true);
            }
            fifo_irq = self.read_register::<FifoIrq>()?;
        }
        return Ok(false);
    }

    fn send_packet(&mut self, buf: &[u8]) -> Result<Result<(), NfcError>, I2C::BusError> {
        // FIFO size is 32 bytes, but wait_for_transmissions waits for the waterlevel to trigger, which is at 8 bytes
        // So we only send 24 bytes at a time after the first transmission
        let (first_chunk, rem) = buf.split_at_checked(32).unwrap_or((&buf, &[]));
        self.write_fifo(first_chunk)?;
        self.write_register(NfcTxen::new(NfcTxenValue::SendBackData))?;
        let chunks = rem.chunks(24);
        for chunk in chunks {
            'this_chunk: loop {
                if self.read_register::<FifoWordCnt>()?.fifo_wordcnt() <= 8 {
                    self.write_fifo(chunk)?;
                    break 'this_chunk;
                }
            }
        }

        // debug_now!("Sent Packet");
        Ok(Ok(()))
    }

    #[allow(unused)]
    fn dump_registers(&mut self) {
        self.txn().dump_registers();
    }
}

impl<I2C, CSN: OutputPin, IRQ: InputPin, Timer> nfc_device::traits::nfc::Device
    for Fm11nt082c<I2C, CSN, IRQ, Timer>
where
    I2C: I2CBus,
    CSN::Error: Debug,
    IRQ::Error: Debug,
    Timer: CountDown<Time = Microseconds>,
{
    #[inline(never)]
    fn read(&mut self, buf: &mut [u8]) -> Result<NfcState, NfcError> {
        // Don't unwrap I2C bus errors into a panic — they happen
        // occasionally on chip startup and are recoverable. Surface as
        // NoActivity so the iso14443 driver retries on the next poll.
        match self.read_packet(buf) {
            Ok(inner) => inner,
            Err(_e) => Err(NfcError::NoActivity),
        }
    }

    #[inline(never)]
    fn send(&mut self, buf: &[u8]) -> Result<(), NfcError> {
        match self.send_packet(buf) {
            Ok(inner) => inner,
            Err(_e) => Err(NfcError::NoActivity),
        }
    }

    fn frame_size(&self) -> usize {
        self.current_frame_size
    }
}
