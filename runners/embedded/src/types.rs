include!(concat!(env!("OUT_DIR"), "/build_constants.rs"));

use crate::soc::types::Soc as SocT;
pub use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use bitflags::bitflags;
pub use ctaphid_dispatch::app::App as CtaphidApp;
use littlefs2::{const_ram_storage, fs::Allocation, fs::Filesystem};
use trussed::types::{LfsResult, LfsStorage};
use trussed::{platform, store};
pub mod usbnfc;

#[derive(Clone, Copy)]
pub struct IrqNr {
    pub i: u16,
}
unsafe impl cortex_m::interrupt::InterruptNumber for IrqNr {
    fn number(self) -> u16 {
        self.i
    }
}

pub struct Config {
    pub card_issuer: &'static [u8; 13],
    pub usb_product: &'static str,
    pub usb_manufacturer: &'static str,
    pub usb_serial: &'static str,
    // pub usb_release: u16 --> taken from utils::VERSION::usb_release()
    pub usb_id_vendor: u16,
    pub usb_id_product: u16,
}

pub trait Soc {
    type InternalFlashStorage;
    type ExternalFlashStorage;
    // VolatileStorage is always RAM
    type UsbBus;
    type NfcDevice;
    type Rng;
    type TrussedUI;
    type Reboot;
    type UUID;

    type Duration;
    type Instant;

    // cannot use dyn cortex_m::interrupt::Nr
    // cannot use actual types, those are usually Enums exported by the soc PAC
    const SYSCALL_IRQ: IrqNr;

    const SOC_NAME: &'static str;
    const BOARD_NAME: &'static str;
    const INTERFACE_CONFIG: &'static Config;

    fn device_uuid() -> &'static Self::UUID;
}

pub struct Runner;

impl apps::Runner for Runner {
    type Syscall = RunnerSyscall;
    type Reboot = <SocT as Soc>::Reboot;
    #[cfg(feature = "provisioner")]
    type Store = RunnerStore;
    #[cfg(feature = "provisioner")]
    type Filesystem = <SocT as Soc>::InternalFlashStorage;

    fn uuid(&self) -> [u8; 16] {
        *<SocT as Soc>::device_uuid()
    }
}

// 8KB of RAM
const_ram_storage!(VolatileStorage, 8192);

store!(
    RunnerStore,
    Internal: <SocT as Soc>::InternalFlashStorage,
    External: <SocT as Soc>::ExternalFlashStorage,
    Volatile: VolatileStorage
);

pub static mut INTERNAL_STORAGE: Option<<SocT as Soc>::InternalFlashStorage> = None;
pub static mut INTERNAL_FS_ALLOC: Option<Allocation<<SocT as Soc>::InternalFlashStorage>> = None;
pub static mut INTERNAL_FS: Option<Filesystem<<SocT as Soc>::InternalFlashStorage>> = None;
pub static mut EXTERNAL_STORAGE: Option<<SocT as Soc>::ExternalFlashStorage> = None;
pub static mut EXTERNAL_FS_ALLOC: Option<Allocation<<SocT as Soc>::ExternalFlashStorage>> = None;
pub static mut EXTERNAL_FS: Option<Filesystem<<SocT as Soc>::ExternalFlashStorage>> = None;
pub static mut VOLATILE_STORAGE: Option<VolatileStorage> = None;
pub static mut VOLATILE_FS_ALLOC: Option<Allocation<VolatileStorage>> = None;
pub static mut VOLATILE_FS: Option<Filesystem<VolatileStorage>> = None;

platform!(
    RunnerPlatform,
    R: <SocT as Soc>::Rng,
    S: RunnerStore,
    UI: <SocT as Soc>::TrussedUI,
);

#[derive(Default)]
pub struct RunnerSyscall {}

impl trussed::client::Syscall for RunnerSyscall {
    #[inline]
    fn syscall(&mut self) {
        rtic::pend(<SocT as Soc>::SYSCALL_IRQ);
    }
}

pub type Trussed = trussed::Service<RunnerPlatform>;

pub type Iso14443 = nfc_device::Iso14443<<SocT as Soc>::NfcDevice>;

pub type ApduDispatch = apdu_dispatch::dispatch::ApduDispatch;
pub type CtaphidDispatch = ctaphid_dispatch::dispatch::Dispatch;

pub type Apps = apps::Apps<Runner>;

bitflags! {
    #[derive(Default)]
    pub struct InitStatus: u8 {
        const NFC_ERROR = 0b00000001;
        const INTERNAL_FLASH_ERROR = 0b00000010;
        const EXTERNAL_FLASH_ERROR = 0b00000100;
        const MIGRATION_ERROR = 0b00001000;
    }
}

#[derive(Debug)]
pub struct DelogFlusher {}

impl delog::Flusher for DelogFlusher {
    fn flush(&self, _msg: &str) {
        #[cfg(feature = "log-rtt")]
        rtt_target::rprint!(_msg);

        #[cfg(feature = "log-semihosting")]
        cortex_m_semihosting::hprint!(_msg).ok();

        // TODO: re-enable?
        // #[cfg(feature = "log-serial")]
        // see https://git.io/JLARR for the plan on how to improve this once we switch to RTIC 0.6
        // rtic::pend(hal::raw::Interrupt::MAILBOX);
    }
}

pub static DELOG_FLUSHER: DelogFlusher = DelogFlusher {};

#[derive(PartialEq)]
pub enum BootMode {
    NFCPassive,
    Full,
}

pub struct DummyPinError {}
pub struct DummyPin {}
impl DummyPin {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DummyPin {
    fn default() -> Self {
        Self::new()
    }
}

impl embedded_hal::digital::v2::OutputPin for DummyPin {
    type Error = DummyPinError;
    fn set_low(&mut self) -> Result<(), DummyPinError> {
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), DummyPinError> {
        Ok(())
    }
}
