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
    AuxIrq, AuxIrqMask, FifoAccess, FifoIrq, FifoIrqMask, FifoWordCnt, MainIrq, MainIrqMask,
    NfcCfg, NfcRats, NfcStatus, NfcTxen, NfcTxenValue, Register, ResetSilence, UserCfg0, UserCfg1,
    UserCfg2, VoutEnCfg, VoutResCfg,
};

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
    pub atqa: u16,
    pub sak1: u8,
    pub sak2: u8,
    pub tl: u8,
    pub t0: u8,
    pub ta: u8,
    pub tb: u8,
    pub tc: u8,
    pub vout_reg_cfg: u8,
}

pub trait Led: Send + Sync {
    fn set_red(&mut self, intensity: u8);

    fn set_green(&mut self, intensity: u8);

    fn set_blue(&mut self, intensity: u8);

    fn set_color(&mut self, red: u8, green: u8, blue: u8) {
        self.set_red(red);
        self.set_green(green);
        self.set_blue(blue);
    }
}

#[derive(Debug, Default)]
struct DebugState {
    was_fifo_full_once: bool,
    rx_done_reached: bool,
    framing_error_reached: bool,
}

pub struct Fm11nt082c<I2C, CSN, IRQ, Timer> {
    i2c: I2C,
    csn: CSN,
    timer: Timer,
    irq: IRQ,
    current_frame_size: usize,
    offset: usize,
    packet: [u8; 256],
    debug_state: DebugState,
    led: &'static mut dyn Led,
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

    /// Data must contain the address of the eeprom encoded in big endian followed by the data
    ///
    /// This function waits for the eeprom write to succeed.
    pub fn write_eeprom(&mut self, data: &[u8]) -> Result<(), I2C::BusError> {
        debug_now!("Writing eeprom");
        self.device.i2c.write(ADDRESS, data)?;
        self.device.timer.start(Microseconds(10_000));
        nb::block!(self.device.timer.wait()).unwrap();
        loop {
            let res = self.device.i2c.write_read(ADDRESS, &[0x00, 0x00], &mut [0]);
            debug_now!("Waiting for finished write: {res:?}");
            let Err(err) = res else {
                break;
            };
            if err.is_address_nack() {
                continue;
            }
            return Err(err);
        }
        Ok(())
    }

    pub fn configure(&mut self, conf: Configuration) -> Result<(), I2C::BusError> {
        // // NOT documented in the datasheet
        // // FIXME: use bitlfags to document what is being configured
        // const REGU_CONFIG: u8 = (0b11 << 4) | (0b10 << 2) | (0b11 << 0);
        // // In the example code, FM11_E2_REGU_CFG_ADDR, not documented in the datasheet
        // const REGU_ADDR: u16 = 0x0391;
        // let buf = &mut [0; 1];

        // self.device
        //     .i2c
        //     .write_read(ADDRESS, &REGU_CONFIG.to_be_bytes(), buf)?;
        // debug_now!("REGU config: {buf:02x?}");

        // let [regu_addr1, regu_addr2] = REGU_ADDR.to_be_bytes();
        // let buf = &[regu_addr1, regu_addr2, REGU_CONFIG];

        // self.write_eeprom(buf)?;
        // let buf = &mut [0; 1];
        // self.device
        //     .i2c
        //     .write_read(ADDRESS, &REGU_CONFIG.to_be_bytes(), buf)?;
        // debug_now!("REGU config: {buf:02x?}");

        const ATQA_ADDR: u16 = 0x03BC;
        debug_now!("Entering configuration");

        let buf = &mut [0; 4];
        self.device
            .i2c
            .write_read(ADDRESS, &ATQA_ADDR.to_be_bytes(), buf)?;
        debug_now!("ATQA config: {buf:02x?}");

        let [atqa_addr1, atqa_addr2] = ATQA_ADDR.to_be_bytes();
        let [atqa1, atqa2] = conf.atqa.to_be_bytes();
        let buf = &[atqa_addr1, atqa_addr2, atqa1, atqa2, conf.sak1, conf.sak2];

        self.write_eeprom(buf)?;
        debug_now!("Wrote ATQA");
        let buf = &mut [0; 4];
        self.device
            .i2c
            .write_read(ADDRESS, &ATQA_ADDR.to_be_bytes(), buf)?;
        debug_now!("ATQA config: {}", hexstr!(buf));

        const NFC_CONFIGURATION_ADDRESS: u16 = 0x03B0;
        let [nfc_conf_address1, nfc_conf_address2] = NFC_CONFIGURATION_ADDRESS.to_be_bytes();
        let buf = &[
            nfc_conf_address1,
            nfc_conf_address2,
            conf.tl,
            conf.t0,
            conf.vout_reg_cfg,
            ADDRESS,
            conf.ta,
            conf.tb,
            conf.tc,
        ];
        debug_now!("Expected nfc config: {}", hexstr!(buf));
        self.write_eeprom(buf)?;
        let buf = &mut [0; 12];
        self.device
            .i2c
            .write_read(ADDRESS, &NFC_CONFIGURATION_ADDRESS.to_be_bytes(), buf)?;
        debug_now!("NFC config: {}", hexstr!(buf));

        const SERIAL_NUMBER_ADDRESS: u16 = 0x0000;

        let buf = &mut [0; 13];
        self.device
            .i2c
            .write_read(ADDRESS, &SERIAL_NUMBER_ADDRESS.to_be_bytes(), &mut buf[..9])?;
        debug_now!("serial number: {}", hexstr!(buf));

        self.device
            .i2c
            .write_read(ADDRESS, &ATQA_ADDR.to_be_bytes(), &mut buf[9..])?;
        debug_now!("serial number: {}", hexstr!(buf));

        const CT_ADDRESS: u16 = 0x03C0;

        let buf_ct = &mut [0; 8];
        self.device
            .i2c
            .write_read(ADDRESS, &CT_ADDRESS.to_be_bytes(), buf_ct)?;
        debug_now!("serial number: {}", hexstr!(buf_ct));

        let crc8_value = crc8(buf);
        debug_now!("Calculated crc8 value: {crc8_value:02x?}");
        const CRC8_ADDRESS: u16 = 0x03BB;
        let [crc8_address1, crc8_address2] = CRC8_ADDRESS.to_be_bytes();
        self.write_eeprom(&[crc8_address1, crc8_address2, crc8_value])?;
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
        debug!("Frame size: {}", self.device.current_frame_size);
        let aux_irq = self.read_register::<AuxIrq>();
        debug!("{:02x?}", aux_irq);
        if aux_irq.unwrap().framing_error() {
            self.device.debug_state.framing_error_reached = true;
        }
        debug!("{:02x?}", self.read_register::<AuxIrqMask>());
        // debug!("{:02x?}", self.read_register::<FifoAccess>());
        // debug!("{:02x?}", self.read_register::<FifoClear>());
        debug!("{:02x?}", self.read_register::<FifoIrq>());
        debug!("{:02x?}", self.read_register::<FifoIrqMask>());
        debug!("WORDCOUNT: {:02x?}", self.read_register::<FifoWordCnt>());
        debug!("{:02x?}", self.read_register::<MainIrq>());
        debug!("{:02x?}", self.read_register::<MainIrqMask>());
        debug!("{:02x?}", self.read_register::<NfcCfg>());
        debug!("{:02x?}", self.read_register::<NfcRats>());
        // debug!("{:02x?}", self.read_register::<NfcTxen>());
        // debug!("{:02x?}", self.read_register::<ResetSilence>());
        debug!("{:02x?}", self.read_register::<registers::Status>());
        debug!("{:02x?}", self.read_register::<UserCfg0>());
        debug!("{:02x?}", self.read_register::<UserCfg1>());
        debug!("{:02x?}", self.read_register::<UserCfg2>());
        debug!("{:02x?}", self.read_register::<VoutEnCfg>());
        debug!("{:02x?}", self.read_register::<VoutResCfg>());
        debug!("{:02x?}", self.read_register::<NfcStatus>());
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

impl<I2C, CSN: OutputPin, IRQ: InputPin, Timer> Fm11nt082c<I2C, CSN, IRQ, Timer>
where
    I2C: I2CBus,
    CSN::Error: Debug,
    IRQ::Error: Debug,
    Timer: CountDown<Time = Microseconds>,
{
    pub fn new(i2c: I2C, csn: CSN, irq: IRQ, timer: Timer, led: &'static mut dyn Led) -> Self {
        led.set_red(0);
        led.set_blue(0);
        led.set_green(255);
        Self {
            i2c,
            csn,
            irq,
            timer,
            current_frame_size: 128,
            offset: 0,
            packet: [0; 256],
            led,
            debug_state: Default::default(),
        }
    }

    pub fn set_led_state(&mut self) {
        debug!("{:?}", self.debug_state);
        match self.debug_state {
            DebugState {
                rx_done_reached: false,
                framing_error_reached: false,
                ..
            } => self.led.set_color(0, 0, 0),
            DebugState {
                rx_done_reached: true,
                framing_error_reached: false,
                ..
            } => self.led.set_color(0, 255, 0),
            DebugState {
                rx_done_reached: false,
                framing_error_reached: true,
                ..
            } => self.led.set_color(0, 0, 255),
            DebugState {
                rx_done_reached: true,
                framing_error_reached: true,
                ..
            } => self.led.set_color(0, 255, 255),
        }
    }

    pub fn set_led(&mut self, led: &'static mut dyn Led) {
        self.led = led;
    }

    pub fn init(&mut self) -> Result<(), I2C::BusError> {
        let mut txn = self.txn();
        txn.write_register(ResetSilence(0xCC))?;
        txn.write_register(NfcTxen(0x88))?;
        txn.dump_registers();
        // Undocumeted check
        let user_cfg0 = UserCfg0(0x91);
        let user_cfg1 = UserCfg1(0x82);
        let user_cfg2 = UserCfg2(0x21);
        debug_now!("{user_cfg0:02x?}\n{user_cfg1:02x?}\n{user_cfg2:02x?}");
        let usercfg_chk_word = !(user_cfg0.0 ^ user_cfg1.0 ^ user_cfg2.0);
        debug_now!("CHK word: {usercfg_chk_word:02x}");
        txn.write_eeprom(&[
            0x03,
            0x90,
            user_cfg0.0,
            user_cfg1.0,
            user_cfg2.0,
            usercfg_chk_word,
        ])?;
        txn.write_eeprom(&[0x03, 0xB8, user_cfg0.0, user_cfg1.0, user_cfg2.0])?;
        debug_now!("Reading eeprom");
        let mut buf = [0; 2];
        txn.device
            .i2c
            .write_read(ADDRESS, &[0x03, 0x90], &mut buf)?;
        debug_now!("EEprom read: {}", hexstr!(&buf));
        txn.write_register(user_cfg0)?;
        txn.write_register(user_cfg1)?;
        txn.write_register(user_cfg2)?;

        txn.write_register(AuxIrq(0))?;
        let mut fifo_irq_mask = FifoIrqMask(0xF3);
        // fifo_irq_mask.set_water_level_mask(false);
        // fifo_irq_mask.set_full_mask(false);
        // fifo_irq_mask.set_empty_mask(false);
        debug_now!("{fifo_irq_mask:?}");
        txn.write_register(fifo_irq_mask)?;

        let mut main_irq_mask = MainIrqMask(0x44);
        // main_irq_mask.set_rx_start_mask(false);
        // main_irq_mask.set_rx_done_mask(false);
        // main_irq_mask.set_tx_done_mask(false);
        // main_irq_mask.set_fifo_flag_mask(false);
        debug_now!("{main_irq_mask:02x?}");
        txn.write_register(main_irq_mask)?;

        let aux_irq_mask = AuxIrqMask(0);
        debug_now!("{aux_irq_mask:02x?}");
        txn.write_register(aux_irq_mask)?;

        debug_now!("{:02x?}", txn.read_register::<MainIrqMask>());

        let mut t0 = T0(0);
        t0.set_tc_transmitted(true);
        t0.set_tb_transmitted(true);
        t0.set_ta_transmitted(true);
        // Means 256 bytes FSCI
        t0.set_fsci(0x8);

        let mut ta = Ta(0);
        ta.set_same_bitrate_both_direction(true);
        ta.set_same_bitrate_poll_2(true);
        ta.set_same_bitrate_listen_2(true);

        let mut tb = Tb(0);
        // FWT = 256 * 16/fc * 2^FWI
        tb.set_fwi(7);
        // SFGT = 256 * 16/fc * 2^SFGI
        tb.set_sfgi(8);

        // Same values as old chip
        assert_eq!(0x78, t0.0);
        assert_eq!(0b10010001, ta.0);
        assert_eq!(0x78, tb.0);

        txn.configure(Configuration {
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

            // configaration of current limiting resistance impedance when power output
            vout_reg_cfg: 0,
        })?;

        // txn.device.write_fifo(&[0x00, 0x00, 0x00])?;
        // debug_now!(
        //     "AFTER WRITING: {:02x?}",
        //     txn.read_register::<FifoWordCnt>()?.fifo_wordcnt(),
        // );
        // txn.write_register(NfcTxen(0x55))?;

        Ok(())
    }

    /// Get a transaction
    ///
    /// While the transaction is open, the `csn` stays low
    fn txn<'a>(&'a mut self) -> Txn<'a, I2C, CSN, IRQ, Timer> {
        self.csn.set_low().unwrap();
        self.timer.start(Microseconds::new(250));
        nb::block!(self.timer.wait()).unwrap();
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
        self.set_led_state();
        self.dump_registers();
        let main_irq = self.read_register::<MainIrq>()?;
        let fifo_irq = self.read_register::<FifoIrq>()?;

        let mut new_session = false;

        if main_irq.active_flag() {
            self.offset = 0;
            new_session = true;
        }

        if main_irq.rx_start() {
            self.offset = 0;
            self.current_frame_size = fsdi_to_frame_size(self.read_register::<NfcRats>()?.fsdi());
            debug!(
                "Rx start  ============================================================================: {}",
                self.current_frame_size
            );
        }

        // Case where the full packet is available
        if main_irq.rx_done() {
            self.debug_state.rx_done_reached = true;
            debug!("RX Done");
            let count = self.read_register::<FifoWordCnt>()?.fifo_wordcnt();
            if count == 32 {
                self.debug_state.was_fifo_full_once = true;
            }
            debug!("WORD_COUNT: {count:02x?}");
            let count = count.min(24);
            if count > 0 {
                self.read_fifo(count)?;
                self.offset += count as usize;
            }

            if self.offset <= 2 {
                // too few bytes, ignore..
                info!("RxDone read too few ({})", hex_str!(&buf[..self.offset]));
                self.offset = 0;
            } else {
                info!("RxDone");
                let l = self.offset - 2;
                buf[..l].copy_from_slice(&self.packet[..l]);
                self.offset = 0;
                if new_session {
                    debug!("New session read suscessfull");
                    return Ok(Ok(NfcState::NewSession(l as u8)));
                } else {
                    debug!("Continue read successfull");
                    return Ok(Ok(NfcState::Continue(l as u8)));
                }
            }
        }

        let rf_status = self.read_register::<NfcStatus>()?;
        debug!("bare Count: {:02x?}", self.read_register::<FifoWordCnt>());
        debug!("water_level: {fifo_irq:?}");
        if !rf_status.nfc_tx() {
            let count = self.read_register::<FifoWordCnt>()?.fifo_wordcnt();
            if count == 32 {
                self.debug_state.was_fifo_full_once = true;
            }
            let count = count.min(24);
            debug!("Second Count: {count:02x}");
            self.read_fifo(count)?;
            self.offset += count as usize;
        }

        debug!("Packet {}", self.offset);
        debug!("{}", hexstr!(&self.packet[..self.offset]));

        if new_session {
            debug!("NewSession read incomplete");
            Ok(Err(NfcError::NewSession))
        } else {
            debug!("No activity read incomplete");
            Ok(Err(NfcError::NoActivity))
        }
    }

    fn write_fifo(&mut self, data: &[u8]) -> Result<(), I2C::BusError> {
        self.txn().write_fifo(data)
    }

    /// Returns true for sucess
    fn wait_for_transmission(&mut self) -> Result<bool, I2C::BusError> {
        self.write_register(NfcTxen(NfcTxenValue::SendBackData.into()))?;

        let mut nfc_status = self.read_register::<NfcStatus>()?;
        for _ in 0..100 {
            if nfc_status.nfc_tx() {
                debug!("Chip is transmitting");
                break;
            } else {
                // chip is not transmitting yet
                nfc_status = self.read_register::<NfcStatus>()?;
            }
        }

        if !nfc_status.nfc_tx() {
            debug_now!("Chip never started transmitting");
            return Ok(false);
        }

        let mut current_count = self.read_register::<FifoWordCnt>()?.fifo_wordcnt();
        if current_count == 32 {
            self.debug_state.was_fifo_full_once = true;
        }

        let mut fifo_irq = self.read_register::<FifoIrq>()?;
        if current_count < 8 {
            return Ok(true);
        }
        for _ in 0..300 {
            if fifo_irq.water_level() {
                break;
            }
            current_count = self.read_register::<FifoWordCnt>()?.fifo_wordcnt();
            if current_count == 32 {
                self.debug_state.was_fifo_full_once = true;
            }
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
        if !self.wait_for_transmission()? {
            return Ok(Err(NfcError::NoActivity));
        }
        let (chunks, rem) = rem.as_chunks::<24>();
        for chunk in chunks {
            self.write_fifo(chunk)?;
            if !self.wait_for_transmission()? {
                return Ok(Err(NfcError::NoActivity));
            }
        }

        if !rem.is_empty() {
            self.write_fifo(rem)?;
            if !self.wait_for_transmission()? {
                return Ok(Err(NfcError::NoActivity));
            }
        }
        Ok(Ok(()))
    }

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
    fn read(&mut self, buf: &mut [u8]) -> Result<NfcState, NfcError> {
        // debug_now!("Polling read");
        self.read_packet(buf).unwrap()
    }

    fn send(&mut self, buf: &[u8]) -> Result<(), NfcError> {
        debug_now!("Sending");
        self.send_packet(buf).unwrap()
    }

    fn frame_size(&self) -> usize {
        self.current_frame_size
    }
}
