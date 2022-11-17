include!(concat!(env!("OUT_DIR"), "/build_constants.rs"));

use crate::soc::types::Soc as SocT;
pub use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use core::convert::TryInto;
use core::marker::PhantomData;
use core::time::Duration;
pub use ctaphid_dispatch::app::App as CtaphidApp;
use embedded_time::duration::units::Milliseconds;
use interchange::Interchange;
use littlefs2::{const_ram_storage, fs::Allocation, fs::Filesystem};
use nfc_device::traits::nfc::Device as NfcDevice;
use rand_core::{CryptoRng, RngCore};
use trussed::types::{LfsResult, LfsStorage};
use trussed::{platform::UserInterface, store};
use usb_device::bus::UsbBus;

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
    type InternalFlashStorage: LfsStorage + 'static;
    type ExternalFlashStorage: LfsStorage;
    // VolatileStorage is always RAM
    type UsbBus: UsbBus + 'static;
    type NfcDevice: NfcDevice;
    type Rng: CryptoRng + RngCore;
    type TrussedUI: UserInterface;
    type Reboot: admin_app::Reboot;
    type UUID;

    type Duration: From<Milliseconds>;

    // cannot use dyn cortex_m::interrupt::Nr
    // cannot use actual types, those are usually Enums exported by the soc PAC
    const SYSCALL_IRQ: IrqNr;

    const SOC_NAME: &'static str;
    const BOARD_NAME: &'static str;
    const INTERFACE_CONFIG: &'static Config;

    fn device_uuid() -> &'static Self::UUID;

    unsafe fn internal_storage() -> &'static mut Storage<'static, Self::InternalFlashStorage>;
    unsafe fn external_storage() -> &'static mut Storage<'static, Self::ExternalFlashStorage>;
}

// 8KB of RAM
const_ram_storage!(VolatileStorage, 8192);

store!(
    RunnerStore,
    Internal: <SocT as Soc>::InternalFlashStorage,
    External: <SocT as Soc>::ExternalFlashStorage,
    Volatile: VolatileStorage
);

pub struct Storage<'a, S: LfsStorage> {
    pub storage: Option<S>,
    pub alloc: Option<Allocation<S>>,
    pub fs: Option<Filesystem<'a, S>>,
}

impl<'a, S: LfsStorage> Storage<'a, S> {
    pub const fn new() -> Self {
        Self {
            storage: None,
            alloc: None,
            fs: None,
        }
    }
}

pub static mut VOLATILE_STORAGE: Storage<VolatileStorage> = Storage::new();

pub struct RunnerPlatform<S: Soc> {
    rng: S::Rng,
    store: RunnerStore,
    user_interface: S::TrussedUI,
}

impl<S: Soc> RunnerPlatform<S> {
    pub fn new(rng: S::Rng, store: RunnerStore, user_interface: S::TrussedUI) -> Self {
        Self {
            rng,
            store,
            user_interface,
        }
    }
}

unsafe impl<S: Soc> trussed::platform::Platform for RunnerPlatform<S> {
    type R = S::Rng;
    type S = RunnerStore;
    type UI = S::TrussedUI;

    fn user_interface(&mut self) -> &mut Self::UI {
        &mut self.user_interface
    }

    fn rng(&mut self) -> &mut Self::R {
        &mut self.rng
    }

    fn store(&self) -> Self::S {
        self.store
    }
}

pub struct RunnerSyscall<S: Soc> {
    _marker: PhantomData<S>,
}

impl<S: Soc> Default for RunnerSyscall<S> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<S: Soc> trussed::client::Syscall for RunnerSyscall<S> {
    #[inline]
    fn syscall(&mut self) {
        rtic::pend(S::SYSCALL_IRQ);
    }
}

pub type Trussed<S> = trussed::Service<RunnerPlatform<S>>;
pub type TrussedClient<S> = trussed::ClientImplementation<RunnerSyscall<S>>;

pub type Iso14443<S> = nfc_device::Iso14443<<S as Soc>::NfcDevice>;

pub type ApduDispatch = apdu_dispatch::dispatch::ApduDispatch;
pub type CtaphidDispatch = ctaphid_dispatch::dispatch::Dispatch;

#[cfg(feature = "admin-app")]
pub type AdminApp<S> = admin_app::App<TrussedClient<S>, <S as Soc>::Reboot>;
#[cfg(feature = "oath-authenticator")]
pub type OathApp<S> = oath_authenticator::Authenticator<TrussedClient<S>>;
#[cfg(feature = "fido-authenticator")]
pub type FidoApp<S> =
    fido_authenticator::Authenticator<fido_authenticator::Conforming, TrussedClient<S>>;
#[cfg(feature = "ndef-app")]
pub type NdefApp = ndef_app::App<'static>;
#[cfg(feature = "provisioner-app")]
pub type ProvisionerApp<S> =
    provisioner_app::Provisioner<RunnerStore, <S as Soc>::InternalFlashStorage, TrussedClient<S>>;

pub trait TrussedApp<S: Soc>: Sized {
    /// non-portable resources needed by this Trussed app
    type NonPortable;

    /// the desired client ID
    const CLIENT_ID: &'static [u8];

    fn with_client(trussed: TrussedClient<S>, non_portable: Self::NonPortable) -> Self;

    fn with(trussed: &mut Trussed<S>, non_portable: Self::NonPortable) -> Self {
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
impl<S: Soc> TrussedApp<S> for OathApp<S> {
    const CLIENT_ID: &'static [u8] = b"oath\0";

    type NonPortable = ();
    fn with_client(trussed: TrussedClient<S>, _: ()) -> Self {
        Self::new(trussed)
    }
}

#[cfg(feature = "admin-app")]
impl<S: Soc> TrussedApp<S> for AdminApp<S> {
    const CLIENT_ID: &'static [u8] = b"admin\0";

    // TODO: declare uuid + version
    type NonPortable = ();
    fn with_client(trussed: TrussedClient<S>, _: ()) -> Self {
        let mut buf: [u8; 16] = [0u8; 16];
        buf.copy_from_slice(<SocT as Soc>::device_uuid());
        Self::new(trussed, buf, build_constants::CARGO_PKG_VERSION)
    }
}

#[cfg(feature = "fido-authenticator")]
impl<S: Soc> TrussedApp<S> for FidoApp<S> {
    const CLIENT_ID: &'static [u8] = b"fido\0";

    type NonPortable = ();
    fn with_client(trussed: TrussedClient<S>, _: ()) -> Self {
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

pub struct ProvisionerNonPortable<S: Soc> {
    pub store: RunnerStore,
    pub stolen_filesystem: &'static mut S::InternalFlashStorage,
    pub nfc_powered: bool,
    pub uuid: [u8; 16],
    pub rebooter: fn() -> !,
}

#[cfg(feature = "provisioner-app")]
impl<S: Soc> TrussedApp<S> for ProvisionerApp<S> {
    const CLIENT_ID: &'static [u8] = b"attn\0";

    type NonPortable = ProvisionerNonPortable<S>;
    fn with_client(
        trussed: TrussedClient<S>,
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

pub struct Apps<S: Soc> {
    #[cfg(feature = "admin-app")]
    pub admin: AdminApp<S>,
    #[cfg(feature = "fido-authenticator")]
    pub fido: FidoApp<S>,
    #[cfg(feature = "oath-authenticator")]
    pub oath: OathApp<S>,
    #[cfg(feature = "ndef-app")]
    pub ndef: NdefApp,
    #[cfg(feature = "provisioner-app")]
    pub provisioner: ProvisionerApp<S>,
}

impl<S: Soc> Apps<S> {
    pub fn new(
        trussed: &mut trussed::Service<RunnerPlatform<S>>,
        #[cfg(feature = "provisioner-app")] provisioner: ProvisionerNonPortable<S>,
    ) -> Self {
        #[cfg(feature = "admin-app")]
        let admin = AdminApp::with(trussed, ());
        #[cfg(feature = "fido-authenticator")]
        let fido = FidoApp::with(trussed, ());
        #[cfg(feature = "oath-authenticator")]
        let oath = OathApp::with(trussed, ());
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
