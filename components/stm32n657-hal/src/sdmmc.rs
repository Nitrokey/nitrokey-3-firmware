use core::{marker::PhantomData, ops::Deref};

use bitflags::bitflags;
use stm32n6::stm32n657::{SDMMC1_S, SDMMC2_S, sdmmc1};

use crate::{
    rcc::{Peripheral, Rcc},
    utils::enum_u,
};

pub trait SdMmc: Deref<Target = sdmmc1::RegisterBlock> {
    fn enable_clk(&self, rcc: &Rcc);
}

impl SdMmc for SDMMC1_S {
    fn enable_clk(&self, rcc: &Rcc) {
        rcc.enable(Peripheral::Sdmmc1);
    }
}
impl SdMmc for SDMMC2_S {
    fn enable_clk(&self, rcc: &Rcc) {
        rcc.enable(Peripheral::Sdmmc2);
    }
}

/// Specifies the SDMMC_CCK clock transition on which Data and Command change.g
#[derive(Clone, Copy, Debug)]
pub enum ClockEdge {
    Rising,
    #[doc(alias = "SDMMC_CLKCR_NEGEDGE")]
    Falling,
}

impl ClockEdge {
    fn bit(&self) -> bool {
        matches!(self, Self::Falling)
    }
}

/// Specifies whether SDMMC Clock output is enabled or disabled when the bus is idle
#[derive(Clone, Copy, Debug)]
pub enum ClockPowerSave {
    Disable,
    #[doc(alias = "SDMMC_CLKCR_PWRSAV")]
    Enable,
}

impl ClockPowerSave {
    fn bit(&self) -> bool {
        matches!(self, Self::Enable)
    }
}

/// Specifies the SDMMC bus width
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum BusWidth {
    OneBit = 0b00,
    #[doc(alias = "SDMMC_CLKCR_WIDBUS_0")]
    FourBit = 0b01,
    #[doc(alias = "SDMMC_CLKCR_WIDBUS_1")]
    EightBit = 0b10,
}

impl BusWidth {
    pub fn is_wide(self) -> bool {
        !matches!(self, Self::OneBit)
    }
}

/// Specifies whether the SDMMC hardware flow control is enabled or disabled
#[derive(Clone, Copy, Debug)]
pub enum HardwareFlowControl {
    Disable,
    #[doc(alias = "SDMMC_CLKCR_HWFC_EN")]
    Enable,
}

impl HardwareFlowControl {
    fn bit(&self) -> bool {
        matches!(self, Self::Enable)
    }
}

/// Specifies whether there is a transceiver present
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum TransceiverPresent {
    Unknown = 0,
    NotPresent = 1,
    Present = 2,
}

pub struct SdMMCInit {
    pub clock_edge: ClockEdge,
    pub clock_power_save: ClockPowerSave,
    pub bus_wide: BusWidth,
    pub hardware_flow_control: HardwareFlowControl,
    /// Specifies the clock frequency of the SDMMC controller.
    /// This parameter has to be in 0..=1023
    pub clock_div: u16,
    pub is_transceiver_present: u32,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Response {
    No = 0b00,
    /// Short response, expect CMDREND or CRCFAIL
    #[doc(alias = "SDMMC_CMD_WAITRESP_0")]
    Short = 0b01,
    /// Short response, expect CMDREND (no CRC)
    ShortNotCrc = 0b10,
    #[doc(alias = "SDMMC_CMD_WAITRESP")]
    Long = 0b11,
}

#[derive(Clone, Copy, Debug)]
pub enum WaitForInterrupt {
    No,
    #[doc(alias = "SDMMC_CMD_WAITINT")]
    It,
    #[doc(alias = "SDMMC_CMD_WAITPEND")]
    Pend,
}

impl WaitForInterrupt {
    fn pend_bit(&self) -> bool {
        matches!(self, Self::Pend)
    }
    fn interrupt_bit(&self) -> bool {
        matches!(self, Self::It)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Cpsm {
    Disable,
    #[doc(alias = "SDMMC_CMD_CPSMEN")]
    Enable,
}

impl Cpsm {
    fn bit(&self) -> bool {
        matches!(self, Self::Enable)
    }
}

pub trait CommandIndex: Into<u8> + Copy {}

impl CommandIndex for CmdIndex {}
impl From<CmdIndex> for u8 {
    fn from(val: CmdIndex) -> Self {
        val as _
    }
}

impl CommandIndex for SdCardCommand {}
impl From<SdCardCommand> for u8 {
    fn from(val: SdCardCommand) -> Self {
        val as _
    }
}

impl CommandIndex for MmcCommand {}
impl From<MmcCommand> for u8 {
    fn from(val: MmcCommand) -> Self {
        val as _
    }
}

pub struct Command<C = CmdIndex> {
    pub argument: u32,
    /// Must be within 0..=64
    pub cmd_index: C,
    pub response: Response,
    pub wait_for_interrupt: WaitForInterrupt,
    pub cpsm: Cpsm,
}

const CHECK_PATTERN: u32 = 0x000001AA;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum DataBlockSize {
    /// Block size of 1 bytes
    B1 = 0b0000,
    /// Block size of 2 bytes
    B2 = 0b0001,
    /// Block size of 4 bytes
    B4 = 0b0010,
    /// Block size of 8 bytes
    B8 = 0b0011,
    /// Block size of 16 bytes
    B16 = 0b0100,
    /// Block size of 32 bytes
    B32 = 0b0101,
    /// Block size of 64 bytes
    B64 = 0b0110,
    /// Block size of 128 bytes
    B128 = 0b0111,
    /// Block size of 256 bytes
    B256 = 0b1000,
    /// Block size of 512 bytes
    B512 = 0b1001,
    /// Block size of 1024 bytes
    B1024 = 0b1010,
    /// Block size of 2048 bytes
    B2048 = 0b1011,
    /// Block size of 4096 bytes
    B4096 = 0b1100,
    /// Block size of 8192 bytes
    B8192 = 0b1101,
    /// Block size of 16384 bytes
    B16384 = 0b1110,
    Reserved = 0b1111,
}

#[derive(Clone, Copy, Debug)]
pub enum TransferDir {
    ToCard,
    #[doc(alias = "SDMMC_DCTRL_DTDIR")]
    ToSdMmc,
}

impl TransferDir {
    fn bit(&self) -> bool {
        matches!(self, Self::ToSdMmc)
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum TransferMode {
    /// Block  data transfer ending on block count
    Block = 0b00,
    /// SDIO multibyte transfer
    Sdio = 0b01,
    /// eMMC stream data transfer
    #[doc(alias = "SDMMC_DCTRL_DTMODE_1")]
    Stream = 0b10,
    /// Block data transfer ending with STOP_TRANSMISSION
    UntilStop = 0b11,
}

#[derive(Clone, Copy, Debug)]
pub enum DpsmState {
    Disable = 0,
    #[doc(alias = "SDMMC_DCTRL_DTEN")]
    Enable,
}

impl DpsmState {
    fn bit(&self) -> bool {
        matches!(self, Self::Enable)
    }
}

pub struct ConfigData {
    /// Data timeout period in card bus clock periods
    pub data_time_out: u32,
    /// Number of bytes to be transfered
    pub data_len: u32,

    pub data_block_size: DataBlockSize,
    pub transfer_dir: TransferDir,
    pub transfer_mode: TransferMode,
    pub dpsm: DpsmState,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Error: u32 {
        ///  Command response received (but CRC check failed)
        const CMD_CRC_FAIL = 0x00000001;
        ///  Data block sent/received (CRC check failed)
        const DATA_CRC_FAIL = 0x00000002;
        ///  Command response timeout
        const CMD_RSP_TIMEOUT = 0x00000004;
        ///  Data timeout
        const DATA_TIMEOUT = 0x00000008;
        ///  Transmit FIFO underrun
        const TX_UNDERRUN = 0x00000010;
        ///  Receive FIFO overrun
        const RX_OVERRUN = 0x00000020;
        ///  Misaligned address
        const ADDR_MISALIGNED = 0x00000040;
        ///  Transferred block length is not allowed for the card or the number of transferred bytes does not match the block length
        const BLOCK_LEN_ERR = 0x00000080;
        ///  An error in the sequence of erase command occurs
        const ERASE_SEQ_ERR = 0x00000100;
        ///  An invalid selection for erase groups
        const BAD_ERASE_PARAM = 0x00000200;
        ///  Attempt to program a write protect block
        const WRITE_PROT_VIOLATION = 0x00000400;
        ///  Sequence or password error has been detected in unlock command or if there was an attempt to access a locked card
        const LOCK_UNLOCK_FAILED = 0x00000800;
        ///  CRC check of the previous command failed
        const COM_CRC_FAILED = 0x00001000;
        ///  Command is not legal for the card state
        const ILLEGAL_CMD = 0x00002000;
        ///  Card internal ECC was applied but failed to correct the data
        const CARD_ECC_FAILED = 0x00004000;
        ///  Internal card controller error
        const CC_ERR = 0x00008000;
        ///  General or unknown error
        const GENERAL_UNKNOWN_ERR = 0x00010000;
        ///  The card could not sustain data reading in stream rmode
        const STREAM_READ_UNDERRUN = 0x00020000;
        ///  The card could not sustain data programming in stream mode
        const STREAM_WRITE_OVERRUN = 0x00040000;
        ///  CID/CSD overwrite error
        const CID_CSD_OVERWRITE = 0x00080000;
        ///  Only partial address space was erased
        const WP_ERASE_SKIP = 0x00100000;
        ///  Command has been executed without using internal ECC
        const CARD_ECC_DISABLED = 0x00200000;
        ///  Erase sequence was cleared before executing because an out of erase sequence command was received
        const ERASE_RESET = 0x00400000;
        ///  Error in sequence of authentication
        const AKE_SEQ_ERR = 0x00800000;
        ///  Error in case of invalid voltage range
        const INVALID_VOLTRANGE = 0x01000000;
        ///  Error when addressed block is out of range
        const ADDR_OUTOF_RANGE = 0x02000000;
        ///  Error when command request is not applicable
        const REQUEST_NOT_APPLICABLE = 0x04000000;
        ///  the used parameter is not valid
        const INVALID_PARAMETER = 0x08000000;
        ///  Error when feature is not insupported
        const UNSUPPORTED_FEATURE = 0x10000000;
        ///  Error when transfer process is busy
        const BUSY = 0x20000000;
        ///  Error while DMA transfer
        const DMA = 0x40000000;
        ///  Timeout error
        const TIMEOUT = 0x80000000;
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum CmdIndex {
    ///  Resets the SD memory card.                                                               
    GoIdleState = 0,
    ///  Sends host capacity support information and activates the card's initialization process.
    SendOpCond = 1,
    ///  Asks any card connected to the host to send the CID numbers on the CMD line.             
    AllSendCid = 2,
    ///  Asks the card to publish a new relative address (RCA).                                   
    SetRelAddr = 3,
    ///  Programs the DSR of all cards.                                                           
    SetDsr = 4,
    ///  Sends host capacity support information (HCS) and asks the accessed card to send its operating condition register (OCR) content in the response on the CMD line.
    SdmmcSenOpCond = 5,
    ///  Checks switchable function (mode 0) and switch card function (mode 1).                   
    HsSwitch = 6,
    ///  Selects the card by its own relative address and gets deselected by any other address    
    SelDeselCard = 7,
    ///  Sends SD Memory Card interface condition, which includes host supply voltage information  and asks the card whether card supports voltage.                      
    HsSendExtCsd = 8,
    ///  Addressed card sends its card specific data (CSD) on the CMD line.                       
    SendCsd = 9,
    ///  Addressed card sends its card identification (CID) on the CMD line.                      
    SendCid = 10,
    ///  SD card Voltage switch to 1.8V mode.                                                     
    VoltageSwitch = 11,
    ///  Forces the card to stop transmission.                                                    
    StopTransmission = 12,
    ///  Addressed card sends its status register.                                                
    SendStatus = 13,
    ///  Reserved                                                                                 
    HsBustestRead = 14,
    ///  Sends an addressed card into the inactive state.                                         
    GoInactiveState = 15,
    ///  Sets the block length (in bytes for SDSC) for all following block commands (read, write, lock). Default block length is fixed to 512 Bytes. Not effective        
    /// for SDHS and SDXC.                                                                       
    SetBlockLen = 16,
    ///  Reads single block of size selected by SET_BLOCKLEN in case of SDSC, and a block of fixed 512 bytes in case of SDHC and SDXC.                                    
    ReadSingleBlock = 17,
    ///  Continuously transfers data blocks from card to host until interrupted by  STOP_TRANSMISSION command.                                                            
    ReadMultiBlock = 18,
    ///  64 bytes tuning pattern is sent for SDR50 and SDR104.                                    
    HsBustestWrite = 19,
    ///  Speed class control command.                                                             
    WriteDatUntilStop = 20,
    ///  Specify block count for CMD18 and CMD25.                                                 
    SetBlockCount = 23,
    ///  Writes single block of size selected by SET_BLOCKLEN in case of SDSC, and a block of fixed 512 bytes in case of SDHC and SDXC.                                   
    WriteSingleBlock = 24,
    ///  Continuously writes blocks of data until a STOP_TRANSMISSION follows.                    
    WriteMultBlock = 25,
    ///  Reserved for manufacturers.                                                              
    ProgCid = 26,
    ///  Programming of the programmable bits of the CSD.                                         
    ProgCsd = 27,
    ///  Sets the write protection bit of the addressed group.                                    
    SetWriteProt = 28,
    ///  Clears the write protection bit of the addressed group.                                  
    ClrWriteProt = 29,
    ///  Asks the card to send the status of the write protection bits.                           
    SendWriteProt = 30,
    ///  Sets the address of the first write block to be erased. (For SD card only).              
    SdEraseGrpStart = 32,
    ///  Sets the address of the last write block of the continuous range to be erased.           
    SdEraseGrpEnd = 33,
    ///  Sets the address of the first write block to be erased. Reserved for each command system set by switch function command (CMD6).                                  
    EraseGrpStart = 35,
    ///  Sets the address of the last write block of the continuous range to be erased. Reserved for each command system set by switch function command (CMD6).           
    EraseGrpEnd = 36,
    ///  Reserved for SD security applications.                                                   
    Erase = 38,
    ///  SD card doesn't support it (Reserved).                                                   
    FastIo = 39,
    ///  SD card doesn't support it (Reserved).                                                   
    GoIrqState = 40,
    ///  Sets/resets the password or lock/unlock the card. The size of the data block is set by the SET_BLOCK_LEN command.                                                
    LockUnlock = 42,
    ///  Indicates to the card that the next command is an application specific command rather than a standard command.                                                   
    AppCmd = 55,
    ///  Used either to transfer a data block to the card or to get a data block from the card for general purpose/application specific commands.                         
    GenCmd = 56,
    ///  No command                                                                               
    NoCmd = 64,
}

/// SD Card Specific security commands.
/// [`CmdIndex::AppCmd`][] should be sent before sending these commands.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum SdCardCommand {
    ///  (ACMD6) Defines the data bus width to be used for data transfer. The allowed data bus widths are given in SCR register.                                                   
    AppSdSetBuswidth = 6,
    ///  (ACMD13) Sends the SD status.                                                            
    SdAppStatus = 13,
    ///  (ACMD22) Sends the number of the written (without errors) write blocks. Responds with 32bit+CRC data block.                                                               
    SdAppSendNumWriteBlocks = 22,
    ///  (ACMD41) Sends host capacity support information (HCS) and asks the accessed card to send its operating condition register (OCR) content in the response on the CMD line.
    SdAppOpCond = 41,
    ///  (ACMD42) Connect/Disconnect the 50 KOhm pull-up resistor on CD/DAT3 (pin 1) of the card  
    SdAppSetClrCardDetect = 42,
    ///  Reads the SD Configuration Register (SCR).                                               
    SdAppSendScr = 51,
    ///  For SD I/O card only, reserved for security specification.                               
    SdmmcRwDirect = 52,
    ///  For SD I/O card only, reserved for security specification.                               
    SdmmcRwExtended = 53,
}

/// MMC Specific commands.
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum MmcCommand {
    MmcSleepAwake = 5,
}

/// Error card status R1 (OCR register)
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ErrorcardStatus {
    AddrOutOfRange = 0x80000000,
    AddrMisaligned = 0x40000000,
    BlockLenErr = 0x20000000,
    EraseSeqErr = 0x10000000,
    BadEraseParam = 0x08000000,
    WriteProtViolation = 0x04000000,
    LockUnlockFailed = 0x01000000,
    ComCrcFailed = 0x00800000,
    IllegalCmd = 0x00400000,
    CardEccFailed = 0x00200000,
    CcError = 0x00100000,
    GeneralUnknownError = 0x00080000,
    StreamReadUnderrun = 0x00040000,
    StreamWriteOverrun = 0x00020000,
    CidCsdOverwrite = 0x00010000,
    WpEraseSkip = 0x00008000,
    CardEccDisabled = 0x00004000,
    EraseReset = 0x00002000,
    AkeSeqError = 0x00000008,
    Errorbits = 0xFDFFE008,
}

/// Masks for the R6 Response
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum ErrorCardR6Response {
    GeneralUnknownError = 0x00002000,
    IllegalCmd = 0x00004000,
    ComCrcFailed = 0x00008000,
}

pub struct Disabled;
pub struct Enabled;

pub struct SdMmcMaster<P, S> {
    pub(crate) peripheral: P,
    _state: PhantomData<S>,
}

impl<P: SdMmc, C> SdMmcMaster<P, C> {
    pub fn init(&mut self, init: SdMMCInit) {
        self.peripheral.clkcr().modify(|_, w| unsafe {
            w.clkdiv()
                .bits(init.clock_div)
                .pwrsav()
                .bit(init.clock_power_save.bit())
                .widbus()
                .bits(init.bus_wide as u8)
                .negedge()
                .bit(init.clock_edge.bit())
                .hwfc_en()
                .bit(init.hardware_flow_control.bit())
                .ddr()
                .bit(false)
                // TODO: check (true is requried for higher bus speed)
                .busspeed()
                .bit(false)
                .selclkrx()
                .bits(0)
        });
    }
}

impl<P: SdMmc> SdMmcMaster<P, Disabled> {
    pub fn new(peripheral: P) -> Self {
        Self {
            peripheral,
            _state: PhantomData,
        }
    }

    pub fn enable(mut self, init: SdMMCInit) -> SdMmcMaster<P, Enabled> {
        self.init(init);
        SdMmcMaster {
            peripheral: self.peripheral,
            _state: PhantomData,
        }
    }
}

enum_u!(
    #[repr(u8)]
    #[derive(Clone, Copy, Debug)]
    pub enum PowerCtrl {
        Off = 0x0,
        Reserved = 01,
        On = 0b11,
        /// Disalbe the SDMMC and stops the clock card
        Cycle = 0b10,
    }
);

enum_u!(
    #[repr(u32)]
    /// Response registers
    #[derive(Clone, Copy, Debug)]
    pub enum Resp {
        Resp1 = 0x0,
        Resp2 = 0x4,
        Resp3 = 0x8,
        Resp4 = 0xC,
    }
);

pub(crate) const fn calc_timeout(timeout_seconds: u32) -> u32 {
    // TODO: get real system freq
    let system_freq = 64_000_000;

    timeout_seconds * (system_freq / 8 / 1000)
    // return 10;
}

const CMD_TIMEOUT: u32 = 5000;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ResponseBits: u32 {
        const OCR_ADDR_OUT_OF_RANGE     = 0x80000000;
        const OCR_ADDR_MISALIGNED       = 0x40000000;
        const OCR_BLOCK_LEN_ERR         = 0x20000000;
        const OCR_ERASE_SEQ_ERR         = 0x10000000;
        const OCR_BAD_ERASE_PARAM       = 0x08000000;
        const OCR_WRITE_PROT_VIOLATION  = 0x04000000;
        const OCR_LOCK_UNLOCK_FAILED    = 0x01000000;
        const OCR_COM_CRC_FAILED        = 0x00800000;
        const OCR_ILLEGAL_CMD           = 0x00400000;
        const OCR_CARD_ECC_FAILED       = 0x00200000;
        const OCR_CC_ERROR              = 0x00100000;
        const OCR_GENERAL_UNKNOWN_ERROR = 0x00080000;
        const OCR_STREAM_READ_UNDERRUN  = 0x00040000;
        const OCR_STREAM_WRITE_OVERRUN  = 0x00020000;
        const OCR_CID_CSD_OVERWRITE     = 0x00010000;
        const OCR_WP_ERASE_SKIP         = 0x00008000;
        const OCR_CARD_ECC_DISABLED     = 0x00004000;
        const OCR_ERASE_RESET           = 0x00002000;
        const OCR_AKE_SEQ_ERROR         = 0x00000008;
        const OCR_ERRORBITS             = 0xFDFFE008;
        const R6_GENERAL_UNKNOWN_ERROR  = 0x00002000;
        const R6_ILLEGAL_CMD            = 0x00004000;
        const R6_COM_CRC_FAILED         = 0x00008000;

        /// this is the reserved for future use in spec RFU
        const SDIO_R5_ERROR                   = 0x00000400;
        /// Out of range error
        const SDIO_R5_OUT_OF_RANGE            = 0x00000100;
        /// Invalid function number
        const SDIO_R5_INVALID_FUNCTION_NUMBER = 0x00000200;
        /// General or an unknown error
        const SDIO_R5_GENERAL_UNKNOWN_ERROR   = 0x00000800;
        /// SDIO Card current state
        ///  * 00=DIS (card not selected)
        ///  * 01=CMD (data line free)
        ///  * 10=TRN (transfer on data lines)
        const SDIO_R5_IO_CURRENT_STATE        = 0x00003000;
        /// Illegal command error
        const SDIO_R5_ILLEGAL_CMD             = 0x00004000;
        /// CRC check of previous cmd failed
        const SDIO_R5_COM_CRC_FAILED          = 0x00008000;

        const SDIO_R5_ERRORBITS = (Self::SDIO_R5_COM_CRC_FAILED.bits()          |
                                                      Self::SDIO_R5_ILLEGAL_CMD.bits()             |
                                                      Self::SDIO_R5_GENERAL_UNKNOWN_ERROR.bits()   |
                                                      Self::SDIO_R5_INVALID_FUNCTION_NUMBER.bits() |
                                                      Self::SDIO_R5_OUT_OF_RANGE.bits());
        const SDMMC_WIDE_BUS_SUPPORT          = 0x00040000;
        const SDMMC_SINGLE_BUS_SUPPORT        = 0x00010000;
        const SDMMC_CARD_LOCKED               = 0x02000000;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ReadWaitMode {
    /// Read Wait control using SDMMC_DATA2
    Data2,
    /// Read Wait control by stopping SDMMCCLK
    Clk,
}

impl ReadWaitMode {
    fn bit(self) -> bool {
        matches!(self, Self::Clk)
    }
}

impl<P: SdMmc> SdMmcMaster<P, Enabled> {
    pub fn clear_static_flags(&mut self) {
        self.peripheral.icr().write(|w| {
            w.ccrcfailc()
                .bit(true)
                .dcrcfailc()
                .bit(true)
                .ctimeoutc()
                .bit(true)
                .dtimeoutc()
                .bit(true)
                .txunderrc()
                .bit(true)
                .rxoverrc()
                .bit(true)
                .cmdrendc()
                .bit(true)
                .cmdsentc()
                .bit(true)
                .dataendc()
                .bit(true)
                .dholdc()
                .bit(true)
                .dbckendc()
                .bit(true)
                .dabortc()
                .bit(true)
                .busyd0endc()
                .bit(true)
                .sdioitc()
                .bit(true)
                .ackfailc()
                .bit(true)
                .acktimeoutc()
                .bit(true)
                .vswendc()
                .bit(true)
                .ckstopc()
                .bit(true)
                .idmatec()
                .bit(true)
                .idmabtcc()
                .bit(true)
        });
    }

    pub fn read_fifo(&mut self) -> u32 {
        // The C hal only reads/writes the 0x80 register for FIFO
        self.peripheral.fifor0().read().bits()
    }

    pub fn write_fifo(&mut self, value: u32) {
        self.peripheral.fifor0().write(|w| unsafe { w.bits(value) });
    }

    pub fn power_state_on(&mut self) {
        self.peripheral
            .power()
            .modify(|_, w| unsafe { w.pwrctrl().bits(PowerCtrl::On as _) });
    }
    pub fn power_state_off(&mut self) {
        self.peripheral
            .power()
            .modify(|_, w| unsafe { w.pwrctrl().bits(PowerCtrl::Off as _) });
    }
    pub fn power_state_cycle(&mut self) {
        self.peripheral
            .power()
            .modify(|_, w| unsafe { w.pwrctrl().bits(PowerCtrl::Cycle as _) });
    }

    pub fn power(&self) -> PowerCtrl {
        self.peripheral
            .power()
            .read()
            .pwrctrl()
            .bits()
            .try_into()
            .expect("All power values to be supported by the enum")
    }

    pub fn send_command<C: CommandIndex>(&mut self, command: Command<C>) {
        self.peripheral
            .argr()
            .write(|w| unsafe { w.bits(command.argument) });
        self.peripheral.cmdr().modify(|_, w| unsafe {
            w.cmdindex()
                .bits(command.cmd_index.into())
                .waitresp()
                .bits(command.response as u8)
                .waitint()
                .bit(command.wait_for_interrupt.interrupt_bit())
                .waitpend()
                .bit(command.wait_for_interrupt.pend_bit())
                .cpsmen()
                .bit(command.cpsm.bit())
                .cmdsuspend()
                .bit(false)
        });
    }

    pub fn command_response(&mut self) -> u8 {
        self.peripheral.respcmdr().read().respcmd().bits()
    }

    pub fn get_response(&mut self, resp: Resp) -> ResponseBits {
        match resp {
            Resp::Resp1 => ResponseBits::from_bits_retain(self.peripheral.resp1r().read().bits()),
            Resp::Resp2 => ResponseBits::from_bits_retain(self.peripheral.resp2r().read().bits()),
            Resp::Resp3 => ResponseBits::from_bits_retain(self.peripheral.resp3r().read().bits()),
            Resp::Resp4 => ResponseBits::from_bits_retain(self.peripheral.resp4r().read().bits()),
        }
    }

    pub fn data_counter(&mut self) -> u32 {
        self.peripheral.dcntr().read().bits()
    }

    /// Why is this the same as read_fifo?
    pub fn fifo_count(&mut self) -> u32 {
        // The C hal only reads/writes the 0x80 register for FIFO
        self.peripheral.fifor0().read().bits()
    }

    pub fn set_read_wait_mode(&mut self, read_wait_mode: ReadWaitMode) {
        self.peripheral
            .dctrl()
            .modify(|_, w| w.rwmod().bit(read_wait_mode.bit()));
    }

    pub fn get_cmd_resp(&mut self) -> u8 {
        self.peripheral.respcmdr().read().bits() as u8
    }

    /// Checks for error conditions for R1 response.
    pub fn get_cmd_resp1<C: CommandIndex>(&mut self, cmd: C, timeout: u32) -> Result<(), Error> {
        let mut count = calc_timeout(timeout);
        let mut star;
        loop {
            count -= 1;
            if count == 0 {
                return Err(Error::TIMEOUT);
            }

            star = self.peripheral.star().read();
            let flags = star.ccrcfail().bit()
                | star.cmdrend().bit()
                | star.ctimeout().bit()
                | star.busyd0end().bit();
            if flags && !star.cpsmact().bit() {
                break;
            }
        }

        if star.ctimeout().bit() {
            self.peripheral.icr().modify(|_, w| w.ctimeoutc().bit(true));
            return Err(Error::CMD_RSP_TIMEOUT);
        }

        if star.ccrcfail().bit() {
            self.peripheral.icr().modify(|_, w| w.ccrcfailc().bit(true));
            return Err(Error::CMD_CRC_FAIL);
        }

        self.clear_static_flags();
        if self.get_cmd_resp() != cmd.into() {
            return Err(Error::CMD_CRC_FAIL);
        }

        let resp1 = self.get_response(Resp::Resp1);

        if resp1 & ResponseBits::OCR_ERRORBITS == ResponseBits::empty() {
            return Ok(());
        }

        if resp1.contains(ResponseBits::OCR_ADDR_OUT_OF_RANGE) {
            return Err(Error::ADDR_OUTOF_RANGE);
        } else if resp1.contains(ResponseBits::OCR_ADDR_MISALIGNED) {
            return Err(Error::ADDR_MISALIGNED);
        } else if resp1.contains(ResponseBits::OCR_BLOCK_LEN_ERR) {
            return Err(Error::BLOCK_LEN_ERR);
        } else if resp1.contains(ResponseBits::OCR_ERASE_SEQ_ERR) {
            return Err(Error::ERASE_SEQ_ERR);
        } else if resp1.contains(ResponseBits::OCR_BAD_ERASE_PARAM) {
            return Err(Error::BAD_ERASE_PARAM);
        } else if resp1.contains(ResponseBits::OCR_WRITE_PROT_VIOLATION) {
            return Err(Error::WRITE_PROT_VIOLATION);
        } else if resp1.contains(ResponseBits::OCR_LOCK_UNLOCK_FAILED) {
            return Err(Error::LOCK_UNLOCK_FAILED);
        } else if resp1.contains(ResponseBits::OCR_COM_CRC_FAILED) {
            return Err(Error::COM_CRC_FAILED);
        } else if resp1.contains(ResponseBits::OCR_ILLEGAL_CMD) {
            return Err(Error::ILLEGAL_CMD);
        } else if resp1.contains(ResponseBits::OCR_CARD_ECC_FAILED) {
            return Err(Error::CARD_ECC_FAILED);
        } else if resp1.contains(ResponseBits::OCR_CC_ERROR) {
            return Err(Error::CC_ERR);
        } else if resp1.contains(ResponseBits::OCR_STREAM_READ_UNDERRUN) {
            return Err(Error::STREAM_READ_UNDERRUN);
        } else if resp1.contains(ResponseBits::OCR_STREAM_WRITE_OVERRUN) {
            return Err(Error::STREAM_WRITE_OVERRUN);
        } else if resp1.contains(ResponseBits::OCR_CID_CSD_OVERWRITE) {
            return Err(Error::CID_CSD_OVERWRITE);
        } else if resp1.contains(ResponseBits::OCR_WP_ERASE_SKIP) {
            return Err(Error::WP_ERASE_SKIP);
        } else if resp1.contains(ResponseBits::OCR_CARD_ECC_DISABLED) {
            return Err(Error::CARD_ECC_DISABLED);
        } else if resp1.contains(ResponseBits::OCR_ERASE_RESET) {
            return Err(Error::ERASE_RESET);
        } else if resp1.contains(ResponseBits::OCR_AKE_SEQ_ERROR) {
            return Err(Error::AKE_SEQ_ERR);
        }
        Err(Error::GENERAL_UNKNOWN_ERR)
    }

    pub fn get_cmd_resp2(&mut self) -> Result<(), Error> {
        let mut count = calc_timeout(CMD_TIMEOUT);

        let mut star;
        loop {
            star = self.peripheral.star().read();
            let flags = star.ccrcfail().bit() | star.cmdrend().bit() | star.ctimeout().bit();
            if flags && !star.cpsmact().bit() {
                break;
            }
            count -= 1;
            if count == 0 {
                return Err(Error::TIMEOUT);
            }
        }
        if star.ctimeout().bit() {
            self.peripheral.icr().modify(|_, w| w.ctimeoutc().bit(true));
            return Err(Error::CMD_RSP_TIMEOUT);
        }

        if star.ccrcfail().bit() {
            self.peripheral.icr().modify(|_, w| w.ccrcfailc().bit(true));
            return Err(Error::CMD_CRC_FAIL);
        }

        self.clear_static_flags();
        Ok(())
    }

    pub fn get_cmd_resp3(&mut self) -> Result<(), Error> {
        let mut count = calc_timeout(CMD_TIMEOUT);

        let mut star;
        loop {
            star = self.peripheral.star().read();
            let flags = star.ccrcfail().bit() | star.cmdrend().bit() | star.ctimeout().bit();
            if flags && !star.cpsmact().bit() {
                break;
            }
            count -= 1;
            if count == 0 {
                return Err(Error::TIMEOUT);
            }
        }

        if star.ctimeout().bit() {
            self.peripheral.icr().modify(|_, w| w.ctimeoutc().bit(true));
            return Err(Error::CMD_RSP_TIMEOUT);
        }

        self.clear_static_flags();
        Ok(())
    }

    pub fn get_cmd_resp4(&mut self, response: &mut u32) -> Result<(), Error> {
        let mut count = calc_timeout(CMD_TIMEOUT);

        let mut star;
        loop {
            star = self.peripheral.star().read();
            let flags = star.ccrcfail().bit() | star.cmdrend().bit() | star.ctimeout().bit();
            if flags && !star.cpsmact().bit() {
                break;
            }
            count -= 1;
            if count == 0 {
                return Err(Error::TIMEOUT);
            }
        }

        if star.ctimeout().bit() {
            self.peripheral.icr().modify(|_, w| w.ctimeoutc().bit(true));
            return Err(Error::CMD_RSP_TIMEOUT);
        }
        self.clear_static_flags();
        let resp = self.get_response(Resp::Resp1);
        *response = resp.bits();

        Ok(())
    }

    pub fn get_cmd_resp5<C: CommandIndex>(
        &mut self,
        cmd: C,
        response: Option<&mut u8>,
    ) -> Result<(), Error> {
        let mut count = calc_timeout(CMD_TIMEOUT);

        let mut star;
        loop {
            star = self.peripheral.star().read();
            let flags = star.ccrcfail().bit() | star.cmdrend().bit() | star.ctimeout().bit();
            if flags && !star.cpsmact().bit() {
                break;
            }
            count -= 1;
            if count == 0 {
                return Err(Error::TIMEOUT);
            }
        }

        if star.ctimeout().bit() {
            self.peripheral.icr().modify(|_, w| w.ctimeoutc().bit(true));
            return Err(Error::CMD_RSP_TIMEOUT);
        }

        if star.ccrcfail().bit() {
            self.peripheral.icr().modify(|_, w| w.ccrcfailc().bit(true));
            return Err(Error::CMD_CRC_FAIL);
        }

        if self.get_cmd_resp() != cmd.into() {
            return Err(Error::CMD_CRC_FAIL);
        }

        self.clear_static_flags();

        let resp5 = self.get_response(Resp::Resp1);
        if (resp5 & ResponseBits::SDIO_R5_ERRORBITS).is_empty() {
            if let Some(response) = response {
                *response = (resp5.bits() & 0xFF) as u8;
            }
            return Ok(());
        }
        if resp5.contains(ResponseBits::SDIO_R5_OUT_OF_RANGE) {
            return Err(Error::ADDR_OUTOF_RANGE);
        }
        if resp5.contains(ResponseBits::SDIO_R5_INVALID_FUNCTION_NUMBER) {
            return Err(Error::INVALID_PARAMETER);
        }
        if resp5.contains(ResponseBits::SDIO_R5_ILLEGAL_CMD) {
            return Err(Error::ILLEGAL_CMD);
        }
        if resp5.contains(ResponseBits::SDIO_R5_COM_CRC_FAILED) {
            return Err(Error::COM_CRC_FAILED);
        }
        Err(Error::GENERAL_UNKNOWN_ERR)
    }

    pub fn get_cmd_resp6(&mut self, cmd: CmdIndex) -> Result<u16, Error> {
        let mut count = calc_timeout(CMD_TIMEOUT);

        let mut star;
        loop {
            star = self.peripheral.star().read();
            let flags = star.ccrcfail().bit() | star.cmdrend().bit() | star.ctimeout().bit();
            if flags && !star.cpsmact().bit() {
                break;
            }
            count -= 1;
            if count == 0 {
                return Err(Error::TIMEOUT);
            }
        }

        if star.ctimeout().bit() {
            self.peripheral.icr().modify(|_, w| w.ctimeoutc().bit(true));
            return Err(Error::CMD_RSP_TIMEOUT);
        }

        if star.ccrcfail().bit() {
            self.peripheral.icr().modify(|_, w| w.ccrcfailc().bit(true));
            return Err(Error::CMD_CRC_FAIL);
        }

        if self.get_cmd_resp() != cmd.into() {
            return Err(Error::CMD_CRC_FAIL);
        }

        self.clear_static_flags();
        let response_r1 = self.get_response(Resp::Resp1);
        if (response_r1
            & (ResponseBits::R6_GENERAL_UNKNOWN_ERROR
                | ResponseBits::R6_ILLEGAL_CMD
                | ResponseBits::R6_COM_CRC_FAILED))
            .is_empty()
        {
            return Ok((response_r1.bits() >> 16) as u16);
        }

        if response_r1.contains(ResponseBits::R6_ILLEGAL_CMD) {
            return Err(Error::ILLEGAL_CMD);
        }

        if response_r1.contains(ResponseBits::R6_COM_CRC_FAILED) {
            return Err(Error::COM_CRC_FAILED);
        }

        Err(Error::GENERAL_UNKNOWN_ERR)
    }

    pub fn get_cmd_resp7(&mut self) -> Result<(), Error> {
        let mut count = calc_timeout(CMD_TIMEOUT);

        let mut star;
        loop {
            star = self.peripheral.star().read();
            let flags = star.ccrcfail().bit() | star.cmdrend().bit() | star.ctimeout().bit();
            if flags && !star.cpsmact().bit() {
                break;
            }
            count -= 1;
            if count == 0 {
                return Err(Error::TIMEOUT);
            }
        }

        if star.ctimeout().bit() {
            self.peripheral.icr().modify(|_, w| w.ctimeoutc().bit(true));
            return Err(Error::CMD_RSP_TIMEOUT);
        }
        if star.ccrcfail().bit() {
            self.peripheral.icr().modify(|_, w| w.ccrcfailc().bit(true));
            return Err(Error::CMD_CRC_FAIL);
        }
        if star.cmdrend().bit() {
            self.peripheral.icr().modify(|_, w| w.cmdrendc().bit(true));
        }
        Ok(())
    }

    pub fn get_cmd_error(&mut self) -> Result<(), Error> {
        let mut count = calc_timeout(CMD_TIMEOUT);

        loop {
            count -= 1;
            if count == 0 {
                error_now!("Count timeout");
                return Err(Error::TIMEOUT);
            }

            let star = self.peripheral.star().read();

            if star.cmdsent().bit() {
                break;
            }
        }

        self.clear_static_flags();
        Ok(())
    }

    pub fn cmd_short1_nowfi_cpsm<C: CommandIndex>(
        &mut self,
        command: C,
        arg: u32,
    ) -> Result<(), Error> {
        self.send_command(Command {
            argument: arg,
            cmd_index: command,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp1(command, CMD_TIMEOUT)
    }

    pub fn cmd_block_len(&mut self, block_size: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::SetBlockLen, block_size)
    }

    pub fn cmd_block_count(&mut self, block_count: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::SetBlockCount, block_count)
    }

    pub fn cmd_read_single_block(&mut self, read_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::ReadSingleBlock, read_addr)
    }

    pub fn cmd_read_multi_block(&mut self, read_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::ReadMultiBlock, read_addr)
    }

    pub fn cmd_write_single_block(&mut self, write_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::WriteSingleBlock, write_addr)
    }

    pub fn cmd_write_multi_block(&mut self, write_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::WriteMultBlock, write_addr)
    }

    pub fn cmd_sd_erase_start_add(&mut self, start_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::SdEraseGrpStart, start_addr)
    }

    pub fn cmd_sd_erase_end_add(&mut self, end_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::SdEraseGrpEnd, end_addr)
    }

    pub fn cmd_erase_start_add(&mut self, start_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::EraseGrpStart, start_addr)
    }

    pub fn cmd_erase_end_add(&mut self, end_addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::EraseGrpEnd, end_addr)
    }

    pub fn cmd_erase(&mut self, erase_type: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::Erase, erase_type)
    }

    pub fn cmd_stop_transfer(&mut self) -> Result<(), Error> {
        let command = CmdIndex::StopTransmission;

        self.peripheral
            .cmdr()
            .modify(|_, w| w.cmdstop().bit(true).cmdtrans().bit(false));

        self.send_command(Command {
            argument: 0,
            cmd_index: command,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        let res = self.get_cmd_resp1(command, CMD_TIMEOUT);

        self.peripheral.cmdr().modify(|_, w| w.cmdstop().bit(false));
        if res == Err(Error::ADDR_OUTOF_RANGE) {
            return Ok(());
        }

        res
    }

    pub fn cmd_select_deselect(&mut self, addr: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::SelDeselCard, addr)
    }

    pub fn cmd_go_idle_state(&mut self) -> Result<(), Error> {
        self.send_command(Command {
            argument: 0,
            cmd_index: CmdIndex::GoIdleState,
            response: Response::No,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_error()
    }

    pub fn cmd_oper_cond(&mut self) -> Result<(), Error> {
        self.send_command(Command {
            argument: CHECK_PATTERN,
            cmd_index: CmdIndex::HsSendExtCsd,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp7()
    }

    /// Send the Application command to verify that that the next command
    /// is an application specific com-mand rather than a standard command
    /// and check the response.
    pub fn cmd_add_command(&mut self, argument: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::AppCmd, argument)
    }

    /// SD Card Specific security command.
    /// [`CmdIndex::AppCmd`][] should be sent before sending this command.
    pub fn cmd_app_oper_command(&mut self, argument: u32) -> Result<(), Error> {
        self.send_command(Command {
            argument,
            cmd_index: SdCardCommand::SdAppOpCond,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp3()
    }

    /// SD Card Specific security command.
    /// [`CmdIndex::AppCmd`][] should be sent before sending this command.
    pub fn cmd_bus_wdith(&mut self, bus_width: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(SdCardCommand::AppSdSetBuswidth, bus_width)
    }

    /// SD Card Specific security command.
    /// [`CmdIndex::AppCmd`][] should be sent before sending this command.
    pub fn cmd_send_scr(&mut self) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(SdCardCommand::SdAppSendScr, 0)
    }

    pub fn cmd_send_cid(&mut self) -> Result<(), Error> {
        let command = CmdIndex::AllSendCid;
        self.send_command(Command {
            argument: 0,
            cmd_index: command,
            response: Response::Long,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp2()
    }

    /// `argument` is the card RCA shifted by 16
    pub fn cmd_send_csd(&mut self, argument: u32) -> Result<(), Error> {
        let command = CmdIndex::SendCsd;
        self.send_command(Command {
            argument,
            cmd_index: command,
            response: Response::Long,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp2()
    }

    pub fn cmd_set_rel_add(&mut self) -> Result<u16, Error> {
        let cmd_index = CmdIndex::SetRelAddr;
        self.send_command(Command {
            argument: 0,
            cmd_index,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });
        self.get_cmd_resp6(cmd_index)
    }

    /// Send the Set Relative Address command to MMC card (not SD card).
    pub fn set_rel_add_mmc(&mut self, rca: u16) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::SetRelAddr, (rca as u32) << 16)
    }

    /// Send the Sleep command to MMC card (not SD card).
    pub fn cmd_sleep_mmc(&mut self, argument: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(MmcCommand::MmcSleepAwake, argument)
    }

    pub fn cmd_send_status(&mut self, argument: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::SendStatus, argument)
    }

    /// SD Card Specific security command.
    /// [`CmdIndex::AppCmd`][] should be sent before sending this command.
    pub fn cmd_status_register(&mut self) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(SdCardCommand::SdAppStatus, 0)
    }

    /// Sends host capacity support information and activates the card's
    /// initialization process. Send SDMMC_CMD_SEND_OP_COND command
    pub fn cmd_op_condition(&mut self, argument: u32) -> Result<(), Error> {
        self.send_command(Command {
            argument,
            cmd_index: CmdIndex::SendOpCond,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp3()
    }

    pub fn cmd_switch(&mut self, argument: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::HsSwitch, argument)
    }

    pub fn cmd_voltage_switch(&mut self) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::VoltageSwitch, 0)
    }

    pub fn cmd_send_ext_csd(&mut self, argument: u32) -> Result<(), Error> {
        self.cmd_short1_nowfi_cpsm(CmdIndex::HsSendExtCsd, argument)
    }

    /// SD Card Specific security command.
    /// [`CmdIndex::AppCmd`][] should be sent before sending this command.
    pub fn sdio_cmd_read_write_direct(
        &mut self,
        argument: u32,
        response: &mut u8,
    ) -> Result<(), Error> {
        self.send_command(Command {
            argument,
            cmd_index: SdCardCommand::SdmmcRwDirect,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp5(SdCardCommand::SdmmcRwDirect, Some(response))
    }

    /// SD Card Specific security command.
    /// [`CmdIndex::AppCmd`][] should be sent before sending this command.
    pub fn sdio_cmd_read_write_extended(&mut self, argument: u32) -> Result<(), Error> {
        self.send_command(Command {
            argument,
            cmd_index: SdCardCommand::SdmmcRwExtended,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp5(SdCardCommand::SdmmcRwDirect, None)
    }

    /// SD Card Specific security command.
    /// [`CmdIndex::AppCmd`][] should be sent before sending this command.
    pub fn cmd_send_operation_condition(
        &mut self,
        argument: u32,
        response: &mut u32,
    ) -> Result<(), Error> {
        self.send_command(Command {
            argument,
            cmd_index: CmdIndex::SdmmcSenOpCond,
            response: Response::Short,
            wait_for_interrupt: WaitForInterrupt::No,
            cpsm: Cpsm::Enable,
        });

        self.get_cmd_resp4(response)
    }

    pub fn config_data(&mut self, config: ConfigData) {
        self.peripheral
            .dtimer()
            .write(|w| unsafe { w.bits(config.data_time_out) });
        self.peripheral
            .dlenr()
            .write(|w| unsafe { w.bits(config.data_len) });
        self.peripheral.dctrl().modify(|_, w| unsafe {
            w.dblocksize()
                .bits(config.data_block_size as u8)
                .dtdir()
                .bit(config.transfer_dir.bit())
                .dtmode()
                .bits(config.transfer_mode as u8)
                .dten()
                .bit(config.dpsm.bit())
        });
    }

    pub fn cmd_trans_enable(&mut self) {
        self.peripheral.cmdr().modify(|_, w| w.cmdtrans().bit(true));
    }
    pub fn cmd_trans_disable(&mut self) {
        self.peripheral
            .cmdr()
            .modify(|_, w| w.cmdtrans().bit(false));
    }
}

pub const FIFO_SIZE: usize = 512;
