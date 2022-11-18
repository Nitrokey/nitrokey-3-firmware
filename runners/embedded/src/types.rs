include!(concat!(env!("OUT_DIR"), "/build_constants.rs"));

use crate::soc::types::Soc as SocT;
pub use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use core::convert::TryInto;
use core::time::Duration;
pub use ctaphid_dispatch::app::App as CtaphidApp;
use interchange::Interchange;
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
    // pub usb_release: u16 --> taken from build_constants::USB_RELEASE
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
pub type TrussedClient = trussed::ClientImplementation<RunnerSyscall>;

pub type Iso14443 = nfc_device::Iso14443<<SocT as Soc>::NfcDevice>;

pub type ApduDispatch = apdu_dispatch::dispatch::ApduDispatch;
pub type CtaphidDispatch = ctaphid_dispatch::dispatch::Dispatch;

#[cfg(feature = "admin-app")]
pub type AdminApp = admin_app::App<TrussedClient, <SocT as Soc>::Reboot>;
#[cfg(feature = "oath-authenticator")]
pub type OathApp = oath_authenticator::Authenticator<TrussedClient>;
#[cfg(feature = "fido-authenticator")]
pub type FidoApp = fido_authenticator::Authenticator<fido_authenticator::Conforming, TrussedClient>;
#[cfg(feature = "ndef-app")]
pub type NdefApp = ndef_app::App<'static>;
#[cfg(feature = "opcard")]
pub type OpcardApp = opcard::Card<TrussedClient>;
#[cfg(feature = "provisioner-app")]
pub type ProvisionerApp =
    provisioner_app::Provisioner<RunnerStore, <SocT as Soc>::InternalFlashStorage, TrussedClient>;

pub trait TrussedApp: Sized {
    /// non-portable resources needed by this Trussed app
    type NonPortable;

    /// the desired client ID
    const CLIENT_ID: &'static [u8];

    fn with_client(trussed: TrussedClient, non_portable: Self::NonPortable) -> Self;

    fn with(trussed: &mut Trussed, non_portable: Self::NonPortable) -> Self {
        let (trussed_requester, trussed_responder) =
            trussed::pipe::TrussedInterchange::claim().expect("could not setup TrussedInterchange");

        let mut client_id = littlefs2::path::PathBuf::new();
        client_id.push(Self::CLIENT_ID.try_into().unwrap());
        assert!(trussed.add_endpoint(trussed_responder, client_id).is_ok());

        let syscaller = RunnerSyscall::default();
        let trussed_client = TrussedClient::new(trussed_requester, syscaller);

        Self::with_client(trussed_client, non_portable)
    }
}

#[cfg(feature = "oath-authenticator")]
impl TrussedApp for OathApp {
    const CLIENT_ID: &'static [u8] = b"oath\0";

    type NonPortable = ();
    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        Self::new(trussed)
    }
}

#[cfg(feature = "opcard")]
impl TrussedApp for OpcardApp {
    const CLIENT_ID: &'static [u8] = b"opcard\0";

    type NonPortable = ();
    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        let uuid = <SocT as Soc>::device_uuid();
        let mut options = opcard::Options::default();
        options.serial = [0xa0, 0x10, uuid[0], uuid[1]];
        // TODO: set manufacturer to Nitrokey
        Self::new(trussed, options)
    }
}

#[cfg(feature = "admin-app")]
impl TrussedApp for AdminApp {
    const CLIENT_ID: &'static [u8] = b"admin\0";

    // TODO: declare uuid + version
    type NonPortable = ();
    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        let mut buf: [u8; 16] = [0u8; 16];
        buf.copy_from_slice(<SocT as Soc>::device_uuid());
        Self::new(trussed, buf, build_constants::CARGO_PKG_VERSION)
    }
}

#[cfg(feature = "fido-authenticator")]
impl TrussedApp for FidoApp {
    const CLIENT_ID: &'static [u8] = b"fido\0";

    type NonPortable = ();
    fn with_client(trussed: TrussedClient, _: ()) -> Self {
        fido_authenticator::Authenticator::new(
            trussed,
            fido_authenticator::Conforming {},
            fido_authenticator::Config {
                max_msg_size: usbd_ctaphid::constants::MESSAGE_SIZE,
                skip_up_timeout: Some(Duration::from_secs(2)),
            },
        )
    }
}

pub struct ProvisionerNonPortable {
    pub store: RunnerStore,
    pub stolen_filesystem: &'static mut <SocT as Soc>::InternalFlashStorage,
    pub nfc_powered: bool,
    pub uuid: [u8; 16],
    pub rebooter: fn() -> !,
}

#[cfg(feature = "provisioner-app")]
impl TrussedApp for ProvisionerApp {
    const CLIENT_ID: &'static [u8] = b"attn\0";

    type NonPortable = ProvisionerNonPortable;
    fn with_client(
        trussed: TrussedClient,
        ProvisionerNonPortable {
            store,
            stolen_filesystem,
            nfc_powered,
            uuid,
            rebooter,
        }: Self::NonPortable,
    ) -> Self {
        Self::new(
            trussed,
            store,
            stolen_filesystem,
            nfc_powered,
            uuid,
            rebooter,
        )
    }
}

pub struct Apps {
    #[cfg(feature = "admin-app")]
    pub admin: AdminApp,
    #[cfg(feature = "fido-authenticator")]
    pub fido: FidoApp,
    #[cfg(feature = "oath-authenticator")]
    pub oath: OathApp,
    #[cfg(feature = "opcard")]
    pub opcard: OpcardApp,
    #[cfg(feature = "ndef-app")]
    pub ndef: NdefApp,
    #[cfg(feature = "provisioner-app")]
    pub provisioner: ProvisionerApp,
}

impl Apps {
    pub fn new(
        trussed: &mut trussed::Service<RunnerPlatform>,
        #[cfg(feature = "provisioner-app")] provisioner: ProvisionerNonPortable,
    ) -> Self {
        #[cfg(feature = "admin-app")]
        let admin = AdminApp::with(trussed, ());
        #[cfg(feature = "fido-authenticator")]
        let fido = FidoApp::with(trussed, ());
        #[cfg(feature = "oath-authenticator")]
        let oath = OathApp::with(trussed, ());
        #[cfg(feature = "opcard")]
        let opcard = OpcardApp::with(trussed, ());
        #[cfg(feature = "ndef-app")]
        let ndef = NdefApp::new();
        #[cfg(feature = "provisioner-app")]
        let provisioner = ProvisionerApp::with(trussed, provisioner);

        Self {
            #[cfg(feature = "admin-app")]
            admin,
            #[cfg(feature = "fido-authenticator")]
            fido,
            #[cfg(feature = "oath-authenticator")]
            oath,
            #[cfg(feature = "opcard")]
            opcard,
            #[cfg(feature = "ndef-app")]
            ndef,
            #[cfg(feature = "provisioner-app")]
            provisioner,
        }
    }

    pub fn apdu_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn ApduApp<ApduCommandSize, ApduResponseSize>]) -> T,
    {
        f(&mut [
            #[cfg(feature = "ndef-app")]
            &mut self.ndef,
            #[cfg(feature = "oath-authenticator")]
            &mut self.oath,
            #[cfg(feature = "opcard")]
            &mut self.opcard,
            #[cfg(feature = "fido-authenticator")]
            &mut self.fido,
            #[cfg(feature = "admin-app")]
            &mut self.admin,
            #[cfg(feature = "provisioner-app")]
            &mut self.provisioner,
        ])
    }

    pub fn ctaphid_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn CtaphidApp]) -> T,
    {
        f(&mut [
            #[cfg(feature = "fido-authenticator")]
            &mut self.fido,
            #[cfg(feature = "admin-app")]
            &mut self.admin,
        ])
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
