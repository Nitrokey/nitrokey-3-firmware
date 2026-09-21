use core::{marker::PhantomData, ops::Range};

use bitflags::{Flags, bitflags};

use crate::{
    gpio::*,
    rcc::Rcc,
    sdmmc::{
        self, BusWidth, Disabled, DpsmState, Enabled, Error, FIFO_SIZE, PowerCtrl, Resp,
        ResponseBits, SdMmc, SdMmcMaster, TransferDir, TransferMode,
    },
};
use stm32n6::stm32n657::{SDMMC1_S, SDMMC2_S};

bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct Class: u32 {
        const ERASE = 0x00000020;
    }
}

#[derive(Default)]
pub struct CardInfo {
    card_type: CardType,
    /// Rel Card Add
    rca: u16,
    class: Class,
    block_number: u32,
    block_size: u32,
    log_block_number: u32,
    log_block_size: u32,
}

pub struct MmcMaster<P, Pins, S> {
    sdmmc: SdMmcMaster<P, S>,
    state: State,
    card_info: CardInfo,
    cid: [u32; 4],
    csd: [u32; 4],
    ext_csd: [u32; 128],
    errorstate: Error,
    pins: Pins,
    _state: PhantomData<S>,
}

#[derive(Default, PartialEq)]
enum CardType {
    #[default]
    LowCapacity,
    HighCapacity,
}

#[expect(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Reset = 0,
    Ready = 1,
    Timeout = 2,
    Busy = 3,
    Programming = 4,
    Receiving = 5,
    Transfer = 6,
    Error = 0xF,
}

impl State {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

impl<P: SdMmc, Pins: MmcPins<Peripheral = P>> MmcMaster<P, Pins, Disabled> {
    pub fn new(peripheral: P, pins: Pins) -> Self {
        Self {
            errorstate: Error::empty(),
            sdmmc: SdMmcMaster::new(peripheral),
            state: State::Reset,
            card_info: CardInfo::default(),
            cid: [0; 4],
            csd: [0; 4],
            ext_csd: [0; 128],
            pins,
            _state: PhantomData,
        }
    }

    pub fn enable(self, rcc: &Rcc) -> Result<MmcMaster<P, Pins, Enabled>, Error> {
        let init = sdmmc::SdMMCInit {
            clock_edge: sdmmc::ClockEdge::Rising,
            clock_power_save: sdmmc::ClockPowerSave::Disable,
            bus_wide: Pins::WIDTH,
            hardware_flow_control: sdmmc::HardwareFlowControl::Disable,
            clock_div: 41, // TODO get proper clock divider
            is_transceiver_present: 0,
        };

        self.sdmmc.peripheral.enable_clk(rcc);
        let sdmmc = self.sdmmc.enable(init);
        let mut this = MmcMaster {
            errorstate: Error::empty(),
            sdmmc,
            state: State::Ready,
            card_info: CardInfo::default(),
            cid: [0; 4],
            csd: [0; 4],
            ext_csd: [0; 128],
            pins: self.pins,
            _state: PhantomData,
        };
        this.sdmmc.power_state_on();

        this.power_on()?;

        this.init_card()?;

        if let Err(err) = this.sdmmc.cmd_block_len(BLOCK_SIZE) {
            this.sdmmc.clear_static_flags();
            this.errorstate |= err;
            this.state = State::Ready;
            return Err(err);
        }
        Ok(this)
    }
}

#[expect(unused)]
#[derive(Default)]
struct Csd {
    /// CSD structure                         
    csd_struct: u8,
    ///System specification version          
    sys_spec_version: u8,
    /// Reserved                              
    reserved1: u8,
    /// Data read access time 1               
    taac: u8,
    /// Data read access time 2 in CLK cycles
    nsac: u8,
    /// Max. bus clock frequency              
    max_bus_clk_frec: u8,
    /// Card command classes                  
    card_comd_classes: u16,
    ///Max. read data block length           
    rd_block_len: u8,
    /// Partial blocks for read allowed       
    part_block_read: u8,
    /// Write block misalignment              
    wr_block_misalign: u8,
    /// Read block misalignment               
    rd_block_misalign: u8,
    /// DSR implemented                       
    dsr_impl: u8,
    /// Reserved                              
    reserved2: u8,
    /// Device Size                           
    device_size: u32,
    ///Max. read current @ VDD min           
    max_rd_current_vdd_min: u8,
    /// Max. read current @ VDD max           
    max_rd_current_vdd_max: u8,
    /// Max. write current @ VDD min          
    max_wr_current_vdd_min: u8,
    /// Max. write current @ VDD max          
    max_wr_current_vdd_max: u8,
    /// Device size multiplier                
    device_size_mul: u8,
    /// Erase group size                      
    erase_gr_size: u8,
    /// Erase group size multiplier           
    erase_gr_mul: u8,
    /// Write protect group size              
    wr_protect_gr_size: u8,
    /// Write protect group enable            
    wr_protect_gr_enable: u8,
    /// Manufacturer default ECC              
    man_defl_ec_c: u8,
    /// Write speed factor                    
    wr_speed_fact: u8,
    /// Max. write data block length          
    max_wr_block_len: u8,
    /// Partial blocks for write allowed      
    write_block_pa_partial: u8,
    /// Reserved                              
    reserved3: u8,
    /// Content protection application        
    content_protect_appli: u8,
    /// File format group                     
    file_format_group: u8,
    /// Copy flag (OTP)                       
    copy_flag: u8,
    /// Permanent write protection            
    perm_wr_protect: u8,
    /// Temporary write protection            
    temp_wr_protect: u8,
    /// File format                           
    file_format: u8,
    /// ECC code                              
    ecc: u8,
    /// CSD CRC                               
    csd_crc: u8,
    /// Always 1                              
    reserved4: u8,
}
impl<P: SdMmc, Pins: MmcPins<Peripheral = P>> MmcMaster<P, Pins, Enabled> {
    fn power_on(&mut self) -> Result<(), Error> {
        self.sdmmc.cmd_go_idle_state()?;

        let mut valid_voltage = 0;
        let mut count = 0;
        let mut response = ResponseBits::empty();
        while valid_voltage == 0 {
            count += 1;
            if count == 0xFFFF {
                return Err(Error::INVALID_VOLTRANGE);
            }
            self.sdmmc.cmd_op_condition(0xC0000080)?;
            response = self.sdmmc.get_response(Resp::Resp1);
            valid_voltage = response.bits() >> 31;
        }

        if response.bits() & 0xFF000000 == 0xC0000000 {
            self.card_info.card_type = CardType::HighCapacity;
        } else {
            self.card_info.card_type = CardType::LowCapacity;
        }
        Ok(())
    }

    fn init_card(&mut self) -> Result<(), Error> {
        if self.sdmmc.power() == PowerCtrl::Off {
            return Err(Error::REQUEST_NOT_APPLICABLE);
        }

        self.sdmmc.cmd_send_cid()?;
        self.cid[0] = self.sdmmc.get_response(Resp::Resp1).bits();
        self.cid[1] = self.sdmmc.get_response(Resp::Resp2).bits();
        self.cid[2] = self.sdmmc.get_response(Resp::Resp3).bits();
        self.cid[3] = self.sdmmc.get_response(Resp::Resp4).bits();
        self.card_info.rca = self.sdmmc.cmd_set_rel_add()?;

        self.sdmmc.cmd_send_csd()?;
        self.csd[0] = self.sdmmc.get_response(Resp::Resp1).bits();
        self.csd[1] = self.sdmmc.get_response(Resp::Resp2).bits();
        self.csd[2] = self.sdmmc.get_response(Resp::Resp3).bits();
        self.csd[3] = self.sdmmc.get_response(Resp::Resp4).bits();

        self.card_info.class =
            Class::from_bits_retain(self.sdmmc.get_response(Resp::Resp2).bits() >> 20);

        self.sdmmc
            .cmd_select_deselect((self.card_info.rca as u32) << 16)?;

        let _csd = self.get_card_csd()?;

        if let Err(err) = self
            .sdmmc
            .cmd_send_status((self.card_info.rca as u32) << 16)
        {
            self.errorstate |= err;
        }

        self.ext_csd = self.get_card_ext_csd()?;

        if let Err(err) = self
            .sdmmc
            .cmd_send_status((self.card_info.rca as u32) << 16)
        {
            self.errorstate |= err;
        }

        Ok(())
    }

    fn get_card_csd(&mut self) -> Result<Csd, Error> {
        let mut csd = Csd {
            csd_struct: ((self.csd[0] & 0xC0000000) >> 30)
                .try_into()
                .expect("value to fit in u8"),
            sys_spec_version: ((self.csd[0] & 0x3C000000) >> 26)
                .try_into()
                .expect("value to fit in u8"),
            reserved1: ((self.csd[0] & 0x03000000) >> 24)
                .try_into()
                .expect("value to fit in u8"),
            taac: ((self.csd[0] & 0x00FF0000) >> 16)
                .try_into()
                .expect("value to fit in u8"),
            nsac: ((self.csd[0] & 0x0000FF00) >> 8)
                .try_into()
                .expect("value to fit in u8"),
            max_bus_clk_frec: (self.csd[0] & 0x000000FF)
                .try_into()
                .expect("value to fit in u8"),
            card_comd_classes: ((self.csd[1] & 0xFFF00000) >> 20)
                .try_into()
                .expect("value to fit in u8"),
            rd_block_len: ((self.csd[1] & 0x000F0000) >> 16)
                .try_into()
                .expect("value to fit in u8"),
            part_block_read: ((self.csd[1] & 0x00008000) >> 15)
                .try_into()
                .expect("value to fit in u8"),
            wr_block_misalign: ((self.csd[1] & 0x00004000) >> 14)
                .try_into()
                .expect("value to fit in u8"),
            rd_block_misalign: ((self.csd[1] & 0x00002000) >> 13)
                .try_into()
                .expect("value to fit in u8"),
            dsr_impl: ((self.csd[1] & 0x00001000) >> 12)
                .try_into()
                .expect("value to fit in u8"),
            erase_gr_size: ((self.csd[2] & 0x00004000) >> 14)
                .try_into()
                .expect("value to fit in u8"),
            erase_gr_mul: ((self.csd[2] & 0x00003F80) >> 7)
                .try_into()
                .expect("value to fit in u8"),
            wr_protect_gr_size: (self.csd[2] & 0x0000007F)
                .try_into()
                .expect("value to fit in u8"),
            wr_protect_gr_enable: ((self.csd[3] & 0x80000000) >> 31)
                .try_into()
                .expect("value to fit in u8"),
            man_defl_ec_c: ((self.csd[3] & 0x60000000) >> 29)
                .try_into()
                .expect("value to fit in u8"),
            wr_speed_fact: ((self.csd[3] & 0x1C000000) >> 26)
                .try_into()
                .expect("value to fit in u8"),
            max_wr_block_len: ((self.csd[3] & 0x03C00000) >> 22)
                .try_into()
                .expect("value to fit in u8"),
            write_block_pa_partial: ((self.csd[3] & 0x00200000) >> 21)
                .try_into()
                .expect("value to fit in u8"),
            content_protect_appli: ((self.csd[3] & 0x00010000) >> 16)
                .try_into()
                .expect("value to fit in u8"),
            file_format_group: ((self.csd[3] & 0x00008000) >> 15)
                .try_into()
                .expect("value to fit in u8"),
            copy_flag: ((self.csd[3] & 0x00004000) >> 14)
                .try_into()
                .expect("value to fit in u8"),
            perm_wr_protect: ((self.csd[3] & 0x00002000) >> 13)
                .try_into()
                .expect("value to fit in u8"),
            temp_wr_protect: ((self.csd[3] & 0x00001000) >> 12)
                .try_into()
                .expect("value to fit in u8"),
            file_format: ((self.csd[3] & 0x00000C00) >> 10)
                .try_into()
                .expect("value to fit in u8"),
            ecc: ((self.csd[3] & 0x00000300) >> 8)
                .try_into()
                .expect("value to fit in u8"),
            csd_crc: ((self.csd[3] & 0x000000FE) >> 1)
                .try_into()
                .expect("value to fit in u8"),
            reserved3: 0,
            reserved4: 1,
            ..Default::default()
        };

        let block_number = self.read_ext_csd(212, 0x0FFFFFFF)?;

        match self.card_info.card_type {
            CardType::LowCapacity => {
                csd.device_size =
                    ((self.csd[1] & 0x000003FF) << 2) | ((self.csd[2] & 0xC0000000) >> 30);
                csd.max_rd_current_vdd_min = ((self.csd[2] & 0x38000000) >> 27) as u8;
                csd.max_rd_current_vdd_max = ((self.csd[2] & 0x07000000) >> 24) as u8;
                csd.max_wr_current_vdd_min = ((self.csd[2] & 0x00E00000) >> 21) as u8;
                csd.max_wr_current_vdd_max = ((self.csd[2] & 0x001C0000) >> 18) as u8;
                csd.device_size_mul = ((self.csd[2] & 0x00038000) >> 15) as u8;
                self.card_info.block_number =
                    (csd.device_size + 1) * (1 << ((csd.device_size_mul & 0x7) + 2));
                self.card_info.block_size = 1 << csd.rd_block_len & 0xF;

                self.card_info.log_block_number =
                    self.card_info.block_number * self.card_info.block_size / BLOCK_SIZE;
                self.card_info.log_block_size = BLOCK_SIZE;
            }
            CardType::HighCapacity => {
                self.card_info.block_number = block_number;
                self.card_info.log_block_number = block_number;
                self.card_info.block_size = BLOCK_SIZE;
                self.card_info.log_block_size = BLOCK_SIZE;
            }
        }
        Ok(csd)
    }

    fn enable_dctrl(&mut self) {
        self.sdmmc
            .peripheral
            .dctrl()
            .write(|w| unsafe { w.bits(0) });
    }

    fn read_ext_csd(&mut self, field_index: u16, _timeout: u32) -> Result<u32, Error> {
        let mut ret = 0;
        self.errorstate.clear();
        self.enable_dctrl();

        self.sdmmc.config_data(sdmmc::ConfigData {
            data_time_out: 0xFFFFFFFF,
            data_len: 512,
            data_block_size: sdmmc::DataBlockSize::B512,
            transfer_dir: TransferDir::ToSdMmc,
            transfer_mode: TransferMode::Block,
            dpsm: DpsmState::Enable,
        });
        self.sdmmc.cmd_trans_enable();
        if let Err(err) = self.sdmmc.cmd_send_ext_csd(0) {
            self.errorstate |= err;
            self.sdmmc.clear_static_flags();
            return Err(err);
        }
        let mut star;

        let mut dataremaining = 512;

        let mut i = 0;
        while {
            star = self.sdmmc.peripheral.star().read();
            !(star.rxoverr().bit()
                | star.dcrcfail().bit()
                | star.dtimeout().bit()
                | star.dataend().bit())
        } {
            if star.rxfifohf().bit() && dataremaining >= FIFO_SIZE {
                for count in 0..FIFO_SIZE / 4 {
                    let tmp = self.sdmmc.read_fifo();
                    if i + count == field_index as usize / 4 {
                        ret = tmp;
                    }
                }
                i += 8;
                dataremaining -= FIFO_SIZE;
            }

            if false
            /* TODO: timeout */
            {
                self.sdmmc.clear_static_flags();
                self.errorstate |= Error::TIMEOUT;
                self.state = State::Ready;
                return Err(Error::TIMEOUT);
            }
        }

        self.sdmmc.cmd_trans_disable();

        let star = self.sdmmc.peripheral.star().read();
        if star.dtimeout().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::TIMEOUT;
            self.state = State::Ready;
            return Err(Error::TIMEOUT);
        } else if star.dcrcfail().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::DATA_CRC_FAIL;
            self.state = State::Ready;
            return Err(Error::DATA_CRC_FAIL);
        } else if star.rxoverr().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::RX_OVERRUN;
            self.state = State::Ready;
            return Err(Error::RX_OVERRUN);
        }

        if let Err(err) = self
            .sdmmc
            .cmd_send_status((self.card_info.rca as u32) << 16)
        {
            self.errorstate |= err;
        }

        self.sdmmc.clear_static_flags();
        self.state = State::Ready;
        Ok(ret)
    }

    fn get_card_ext_csd(&mut self) -> Result<[u32; 128], Error> {
        assert_eq!(self.state, State::Ready);
        self.errorstate.clear();
        self.state = State::Busy;
        self.enable_dctrl();

        let config = sdmmc::ConfigData {
            data_time_out: 0xFFFFFFFF,
            data_len: 512,
            data_block_size: sdmmc::DataBlockSize::B512,
            transfer_dir: TransferDir::ToSdMmc,
            transfer_mode: TransferMode::Block,
            dpsm: DpsmState::Disable,
        };
        self.sdmmc.config_data(config);
        self.sdmmc.cmd_trans_enable();
        let mut tmpbuf = [0; 128];

        if let Err(err) = self.sdmmc.cmd_send_ext_csd(0) {
            self.sdmmc.clear_static_flags();
            self.errorstate |= err;
            self.state = State::Ready;
            return Err(err);
        }

        let mut dataremaining = 512;
        let mut offset = 0;

        let mut star;
        while {
            star = self.sdmmc.peripheral.star().read();
            !(star.rxoverr().bit()
                | star.dcrcfail().bit()
                | star.dtimeout().bit()
                | star.dataend().bit())
        } {
            if star.rxfifohf().bit() && dataremaining >= FIFO_SIZE {
                for _ in 0..FIFO_SIZE / 4 {
                    tmpbuf[offset] = self.sdmmc.read_fifo();
                    offset += 1;
                }
                dataremaining -= FIFO_SIZE;
            }
            // TODO: timeout
        }

        self.sdmmc.cmd_trans_disable();

        let star = self.sdmmc.peripheral.star().read();
        if star.dtimeout().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::TIMEOUT;
            self.state = State::Ready;
            return Err(Error::TIMEOUT);
        } else if star.dcrcfail().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::DATA_CRC_FAIL;
            self.state = State::Ready;
            return Err(Error::DATA_CRC_FAIL);
        } else if star.rxoverr().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::RX_OVERRUN;
            self.state = State::Ready;
            return Err(Error::RX_OVERRUN);
        }

        self.sdmmc.clear_static_flags();
        self.state = State::Ready;

        Ok(tmpbuf)
    }
}

const BLOCK_SIZE: u32 = 512;

impl<P: SdMmc, Pins: MmcPins<Peripheral = P>> MmcMaster<P, Pins, Enabled> {
    pub fn free(mut self) -> SdMmcMaster<P, Enabled> {
        self.sdmmc.power_state_off();
        self.sdmmc
    }

    pub fn read_blocks(
        &mut self,
        buffer: &mut [[u8; BLOCK_SIZE as _]],
        raw_address: u32,
    ) -> Result<(), Error> {
        if !self.state.is_ready() {
            return Err(Error::BUSY);
        }
        self.errorstate.clear();

        if raw_address + buffer.len() as u32 > self.card_info.log_block_number {
            return Err(Error::ADDR_OUTOF_RANGE);
        }

        if !raw_address.is_multiple_of(8) {
            return Err(Error::ADDR_MISALIGNED);
        }

        self.state = State::Busy;
        self.enable_dctrl();

        let address = if self.card_info.card_type == CardType::HighCapacity {
            raw_address * BLOCK_SIZE
        } else {
            raw_address
        };

        self.sdmmc.config_data(sdmmc::ConfigData {
            data_time_out: 0xFFFFFFFF,
            data_len: buffer.len() as u32 * BLOCK_SIZE,
            data_block_size: sdmmc::DataBlockSize::B512,
            transfer_dir: TransferDir::ToSdMmc,
            transfer_mode: TransferMode::Block,
            dpsm: DpsmState::Disable,
        });
        self.sdmmc.cmd_trans_enable();

        let cmd_res = match buffer.len() {
            0 => panic!("Reading 0 blocks"),
            1 => self.sdmmc.cmd_read_single_block(address),
            _ => self.sdmmc.cmd_read_multi_block(address),
        };

        if let Err(err) = cmd_res {
            self.sdmmc.clear_static_flags();
            self.state = State::Ready;
            self.errorstate |= err;
            return Err(err);
        }

        let mut star;
        let mut dataremaining = buffer.len() * BLOCK_SIZE as usize;
        let mut offset = 0;
        let buf = buffer.as_flattened_mut();
        while {
            star = self.sdmmc.peripheral.star().read();
            !(star.rxoverr().bit()
                | star.dcrcfail().bit()
                | star.dtimeout().bit()
                | star.dataend().bit())
        } {
            if star.rxfifohf().bit() && dataremaining > FIFO_SIZE {
                for _ in 0..FIFO_SIZE / 4 {
                    let data = self.sdmmc.read_fifo();
                    buf[offset..][..4].copy_from_slice(&data.to_le_bytes());
                    offset += 4;
                }
                dataremaining -= FIFO_SIZE;
            }

            // TODO: timeout
        }

        self.sdmmc.cmd_trans_disable();

        if star.dataend().bit() && buffer.len() > 1 {
            if let Err(err) = self.sdmmc.cmd_stop_transfer() {
                self.sdmmc.clear_static_flags();
                self.state = State::Ready;
                self.errorstate |= err;
                return Err(err);
            }
        }
        if star.dtimeout().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::TIMEOUT;
            self.state = State::Ready;
            return Err(Error::TIMEOUT);
        } else if star.dcrcfail().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::DATA_CRC_FAIL;
            self.state = State::Ready;
            return Err(Error::DATA_CRC_FAIL);
        } else if star.rxoverr().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::RX_OVERRUN;
            self.state = State::Ready;
            return Err(Error::RX_OVERRUN);
        }

        self.sdmmc.clear_static_flags();
        self.state = State::Ready;
        Ok(())
    }

    pub fn write_blocks(
        &mut self,
        buffer: &[[u8; BLOCK_SIZE as _]],
        raw_address: u32,
    ) -> Result<(), Error> {
        if !self.state.is_ready() {
            return Err(Error::BUSY);
        }
        self.errorstate.clear();

        if raw_address + buffer.len() as u32 > self.card_info.log_block_number {
            return Err(Error::ADDR_OUTOF_RANGE);
        }

        if !raw_address.is_multiple_of(8) {
            return Err(Error::ADDR_MISALIGNED);
        }

        self.state = State::Busy;
        self.enable_dctrl();

        let address = if self.card_info.card_type == CardType::HighCapacity {
            raw_address * BLOCK_SIZE
        } else {
            raw_address
        };

        self.sdmmc.config_data(sdmmc::ConfigData {
            data_time_out: 0xFFFFFFFF,
            data_len: buffer.len() as u32 * BLOCK_SIZE,
            data_block_size: sdmmc::DataBlockSize::B512,
            transfer_dir: TransferDir::ToCard,
            transfer_mode: TransferMode::Block,
            dpsm: DpsmState::Disable,
        });
        self.sdmmc.cmd_trans_enable();

        let cmd_res = match buffer.len() {
            0 => panic!("Reading 0 blocks"),
            1 => self.sdmmc.cmd_write_single_block(address),
            _ => self.sdmmc.cmd_write_multi_block(address),
        };

        if let Err(err) = cmd_res {
            self.sdmmc.clear_static_flags();
            self.state = State::Ready;
            self.errorstate |= err;
            return Err(err);
        }

        let mut star;
        let mut dataremaining = buffer.len() * BLOCK_SIZE as usize;
        let mut offset = 0;
        let buf = buffer.as_flattened();
        while {
            star = self.sdmmc.peripheral.star().read();
            !(star.rxoverr().bit()
                | star.dcrcfail().bit()
                | star.dtimeout().bit()
                | star.dataend().bit())
        } {
            if star.rxfifohf().bit() && dataremaining > FIFO_SIZE {
                for _ in 0..FIFO_SIZE / 4 {
                    self.sdmmc
                        .write_fifo(u32::from_le_bytes(buf[offset..][..4].try_into().unwrap()));
                    offset += 4;
                }
                dataremaining -= FIFO_SIZE;
            }

            // TODO: timeout
        }

        self.sdmmc.cmd_trans_disable();

        if star.dataend().bit() && buffer.len() > 1 {
            if let Err(err) = self.sdmmc.cmd_stop_transfer() {
                self.sdmmc.clear_static_flags();
                self.state = State::Ready;
                self.errorstate |= err;
                return Err(err);
            }
        }
        if star.dtimeout().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::TIMEOUT;
            self.state = State::Ready;
            return Err(Error::TIMEOUT);
        } else if star.dcrcfail().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::DATA_CRC_FAIL;
            self.state = State::Ready;
            return Err(Error::DATA_CRC_FAIL);
        } else if star.rxoverr().bit() {
            self.sdmmc.clear_static_flags();
            self.errorstate |= Error::RX_OVERRUN;
            self.state = State::Ready;
            return Err(Error::RX_OVERRUN);
        }

        self.sdmmc.clear_static_flags();
        self.state = State::Ready;
        Ok(())
    }

    pub fn erase(&mut self, blocks_addr: Range<u32>) -> Result<(), Error> {
        if !self.state.is_ready() {
            return Err(Error::BUSY);
        }
        assert!(blocks_addr.end >= blocks_addr.start);
        self.errorstate.clear();

        if (self.ext_csd[61 / 4] >> 8) & 0xFF != 0
            && !(blocks_addr.start.is_multiple_of(8) && blocks_addr.end.is_multiple_of(8))
        {
            return Err(Error::ADDR_MISALIGNED);
        }

        self.state = State::Busy;

        if !self.card_info.class.contains(Class::ERASE) {
            self.sdmmc.clear_static_flags();
            self.state = State::Ready;
            return Err(Error::REQUEST_NOT_APPLICABLE);
        }

        if self
            .sdmmc
            .get_response(Resp::Resp1)
            .contains(ResponseBits::SDMMC_CARD_LOCKED)
        {
            self.sdmmc.clear_static_flags();
            self.state = State::Ready;
            return Err(Error::LOCK_UNLOCK_FAILED);
        }

        let blocks = match self.card_info.card_type {
            CardType::HighCapacity => blocks_addr.start * 8..blocks_addr.end * 8,
            CardType::LowCapacity => blocks_addr,
        };

        if let Err(err) = self.sdmmc.cmd_erase_start_add(blocks.start) {
            self.sdmmc.clear_static_flags();
            self.state = State::Ready;
            return Err(err);
        }

        if let Err(err) = self.sdmmc.cmd_erase_end_add(blocks.end) {
            self.sdmmc.clear_static_flags();
            self.state = State::Ready;
            return Err(err);
        }

        if let Err(err) = self.sdmmc.cmd_erase(0) {
            self.sdmmc.clear_static_flags();
            self.state = State::Ready;
            return Err(err);
        }
        self.state = State::Ready;
        Ok(())
    }
}

pub trait D0 {
    type Peripheral;
}
pub trait D1 {
    type Peripheral;
}
pub trait D2 {
    type Peripheral;
}
pub trait D3 {
    type Peripheral;
}
pub trait D4 {
    type Peripheral;
}
pub trait D5 {
    type Peripheral;
}
pub trait D6 {
    type Peripheral;
}
pub trait D7 {
    type Peripheral;
}
pub trait Cmd {
    type Peripheral;
}
pub trait Ck {
    type Peripheral;
}

macro_rules! pins {
    ($(($pin:ident, $alt:ident, $function:ident, $peripheral:ident)),*$(,)?) => {$(
        impl $function for $pin<Alternate<PullUp, $alt>> {
          type Peripheral = $peripheral;
        }
    )*};
}

pins! {
    (PinA0, ALTERNATE_FUNCTION_11, Cmd, SDMMC2_S),
    (PinB4, ALTERNATE_FUNCTION_11, D3, SDMMC2_S),
    (PinB8, ALTERNATE_FUNCTION_11, D0, SDMMC2_S),
    (PinB9, ALTERNATE_FUNCTION_11, D2, SDMMC2_S),
    (PinB13, ALTERNATE_FUNCTION_11, D6, SDMMC2_S),
    (PinC0, ALTERNATE_FUNCTION_11, D2, SDMMC2_S),
    (PinC1, ALTERNATE_FUNCTION_11, D5, SDMMC2_S),
    (PinC2, ALTERNATE_FUNCTION_11, Ck, SDMMC2_S),
    (PinC3, ALTERNATE_FUNCTION_11, Cmd, SDMMC2_S),
    (PinC4, ALTERNATE_FUNCTION_11, D0, SDMMC2_S),
    (PinC5, ALTERNATE_FUNCTION_11, D1, SDMMC2_S),
    (PinC6, ALTERNATE_FUNCTION_10, D6, SDMMC1_S),
    (PinC6, ALTERNATE_FUNCTION_11, D6, SDMMC2_S),
    // (PinC6, ALTERNATE_FUNCTION_12, D0dir, SDMMC1_S),
    (PinC7, ALTERNATE_FUNCTION_10, D7, SDMMC1_S),
    (PinC7, ALTERNATE_FUNCTION_11, D7, SDMMC2_S),
    // (PinC7, ALTERNATE_FUNCTION_12, D123dir, SDMMC1_S),
    (PinC8, ALTERNATE_FUNCTION_10, D0, SDMMC1_S),
    (PinC9, ALTERNATE_FUNCTION_10, D1, SDMMC1_S),
    (PinC10, ALTERNATE_FUNCTION_10, D2, SDMMC1_S),
    (PinC11, ALTERNATE_FUNCTION_10, D3, SDMMC1_S),
    (PinC12, ALTERNATE_FUNCTION_10, Ck, SDMMC1_S),
    (PinD2, ALTERNATE_FUNCTION_11, Ck, SDMMC2_S),
    (PinD5, ALTERNATE_FUNCTION_11, D7, SDMMC2_S),
    (PinD11, ALTERNATE_FUNCTION_10, D0, SDMMC1_S),
    (PinD15, ALTERNATE_FUNCTION_10, D0, SDMMC1_S),
    (PinE4, ALTERNATE_FUNCTION_11, D3, SDMMC2_S),
    (PinE15, ALTERNATE_FUNCTION_11, D0, SDMMC1_S),
    (PinG8, ALTERNATE_FUNCTION_11, D1, SDMMC2_S),
    (PinH2, ALTERNATE_FUNCTION_10, Cmd, SDMMC1_S),
    (PinH8, ALTERNATE_FUNCTION_11, D1, SDMMC2_S),
    (PinH9, ALTERNATE_FUNCTION_10, D4, SDMMC1_S),
    (PinH9, ALTERNATE_FUNCTION_11, D4, SDMMC2_S),
    // (PinH9, ALTERNATE_FUNCTION_12, Ckin, SDMMC1_S),
}

pub trait MmcPins {
    type Peripheral;
    const WIDTH: BusWidth;
}

impl<P, P1, P2, P3> MmcPins for (P1, P2, P3)
where
    P1: Cmd<Peripheral = P>,
    P2: Ck<Peripheral = P>,
    P3: D0<Peripheral = P>,
{
    type Peripheral = P;
    const WIDTH: BusWidth = BusWidth::OneBit;
}

impl<P, P1, P2, P3, P4, P5, P6> MmcPins for (P1, P2, P3, P4, P5, P6)
where
    P1: Cmd<Peripheral = P>,
    P2: Ck<Peripheral = P>,
    P3: D0<Peripheral = P>,
    P4: D1<Peripheral = P>,
    P5: D2<Peripheral = P>,
    P6: D3<Peripheral = P>,
{
    type Peripheral = P;
    const WIDTH: BusWidth = BusWidth::FourBit;
}

impl<P, P1, P2, P3, P4, P5, P6, P7, P8, P9, P10> MmcPins
    for (P1, P2, P3, P4, P5, P6, P7, P8, P9, P10)
where
    P1: Cmd<Peripheral = P>,
    P2: Ck<Peripheral = P>,
    P3: D0<Peripheral = P>,
    P4: D1<Peripheral = P>,
    P5: D2<Peripheral = P>,
    P6: D3<Peripheral = P>,
    P7: D4<Peripheral = P>,
    P8: D5<Peripheral = P>,
    P9: D6<Peripheral = P>,
    P10: D7<Peripheral = P>,
{
    type Peripheral = P;
    const WIDTH: BusWidth = BusWidth::EightBit;
}
