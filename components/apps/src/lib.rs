#![no_std]

use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use ctaphid_dispatch::app::App as CtaphidApp;
use trussed::{
    pipe::TrussedInterchange, platform::Syscall, types::PathBuf, ClientImplementation,
    Interchange as _, Platform, Service,
};

#[cfg(feature = "admin-app")]
pub use admin_app::Reboot;

pub trait Runner {
    type Syscall: Syscall + Default;

    #[cfg(feature = "admin-app")]
    type Reboot: Reboot;
    #[cfg(feature = "provisioner-app")]
    type Store: trussed::store::Store;
    #[cfg(feature = "provisioner-app")]
    type Filesystem: trussed::types::LfsStorage + 'static;

    fn uuid(&self) -> [u8; 16];
    fn version(&self) -> u32;
}

type Client<R> = ClientImplementation<<R as Runner>::Syscall>;

#[cfg(feature = "admin-app")]
type AdminApp<R> = admin_app::App<Client<R>, <R as Runner>::Reboot>;
#[cfg(feature = "fido-authenticator")]
type FidoApp<R> = fido_authenticator::Authenticator<fido_authenticator::Conforming, Client<R>>;
#[cfg(feature = "ndef-app")]
type NdefApp = ndef_app::App<'static>;
#[cfg(feature = "oath-authenticator")]
type OathApp<R> = oath_authenticator::Authenticator<Client<R>>;
#[cfg(feature = "opcard")]
type OpcardApp<R> = opcard::Card<Client<R>>;
#[cfg(feature = "provisioner-app")]
type ProvisionerApp<R> =
    provisioner_app::Provisioner<<R as Runner>::Store, <R as Runner>::Filesystem, Client<R>>;

pub struct Apps<R: Runner> {
    #[cfg(feature = "admin-app")]
    admin: AdminApp<R>,
    #[cfg(feature = "fido-authenticator")]
    fido: FidoApp<R>,
    #[cfg(feature = "ndef-app")]
    ndef: NdefApp,
    #[cfg(feature = "oath-authenticator")]
    oath: OathApp<R>,
    #[cfg(feature = "opcard")]
    opcard: OpcardApp<R>,
    #[cfg(feature = "provisioner-app")]
    provisioner: ProvisionerApp<R>,
}

impl<R: Runner> Apps<R> {
    pub fn new<P: Platform>(
        runner: &R,
        trussed: &mut Service<P>,
        #[cfg(feature = "provisioner-app")] provisioner: ProvisionerNonPortable<R>,
    ) -> Self {
        Self {
            #[cfg(feature = "admin-app")]
            admin: App::with(runner, trussed, ()),
            #[cfg(feature = "fido-authenticator")]
            fido: App::with(runner, trussed, ()),
            #[cfg(feature = "ndef-app")]
            ndef: NdefApp::new(),
            #[cfg(feature = "oath-authenticator")]
            oath: App::with(runner, trussed, ()),
            #[cfg(feature = "opcard")]
            opcard: App::with(runner, trussed, ()),
            #[cfg(feature = "provisioner-app")]
            provisioner: App::with(runner, trussed, provisioner),
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
            #[cfg(feature = "oath-authenticator")]
            &mut self.oath,
        ])
    }
}

trait App<R: Runner>: Sized {
    /// non-portable resources needed by this Trussed app
    type NonPortable;

    /// the desired client ID
    const CLIENT_ID: &'static [u8];

    fn with_client(runner: &R, trussed: Client<R>, non_portable: Self::NonPortable) -> Self;

    fn with<P: Platform>(runner: &R, trussed: &mut Service<P>, non_portable: Self::NonPortable) -> Self {
        let (trussed_requester, trussed_responder) =
            TrussedInterchange::claim().expect("could not setup TrussedInterchange");

        let mut client_id = PathBuf::new();
        client_id.push(Self::CLIENT_ID.try_into().unwrap());
        assert!(trussed.add_endpoint(trussed_responder, client_id).is_ok());

        let syscaller = R::Syscall::default();
        let trussed_client = ClientImplementation::new(trussed_requester, syscaller);

        Self::with_client(runner, trussed_client, non_portable)
    }
}

#[cfg(feature = "admin-app")]
impl<R: Runner> App<R> for AdminApp<R> {
    const CLIENT_ID: &'static [u8] = b"admin\0";

    type NonPortable = ();

    fn with_client(runner: &R, trussed: Client<R>, _: ()) -> Self {
        Self::new(trussed, runner.uuid(), runner.version())
    }
}

#[cfg(feature = "fido-authenticator")]
impl<R: Runner> App<R> for FidoApp<R> {
    const CLIENT_ID: &'static [u8] = b"fido\0";

    type NonPortable = ();

    fn with_client(_runner: &R, trussed: Client<R>, _: ()) -> Self {
        fido_authenticator::Authenticator::new(
            trussed,
            fido_authenticator::Conforming {},
            fido_authenticator::Config {
                max_msg_size: usbd_ctaphid::constants::MESSAGE_SIZE,
                skip_up_timeout: Some(core::time::Duration::from_secs(2)),
            },
        )
    }
}

#[cfg(feature = "oath-authenticator")]
impl<R: Runner> App<R> for OathApp<R> {
    const CLIENT_ID: &'static [u8] = b"oath\0";

    type NonPortable = ();

    fn with_client(_runner: &R, trussed: Client<R>, _: ()) -> Self {
        Self::new(trussed)
    }
}

#[cfg(feature = "opcard")]
impl<R: Runner> App<R> for OpcardApp<R> {
    const CLIENT_ID: &'static [u8] = b"opcard\0";

    type NonPortable = ();

    fn with_client(runner: &R, trussed: Client<R>, _: ()) -> Self {
        let uuid = runner.uuid();
        let mut options = opcard::Options::default();
        options.serial = [0xa0, 0x20, uuid[0], uuid[1]];
        // TODO: set manufacturer to Nitrokey
        Self::new(trussed, options)
    }
}

#[cfg(feature = "provisioner-app")]
pub struct ProvisionerNonPortable<R: Runner> {
    pub store: R::Store,
    pub stolen_filesystem: &'static mut R::Filesystem,
    pub nfc_powered: bool,
    pub rebooter: fn() -> !,
}

#[cfg(feature = "provisioner-app")]
impl<R: Runner> App<R> for ProvisionerApp<R> {
    const CLIENT_ID: &'static [u8] = b"attn\0";

    type NonPortable = ProvisionerNonPortable<R>;

    fn with_client(runner: &R, trussed: Client<R>, non_portable: Self::NonPortable) -> Self {
        let uuid = runner.uuid();
        Self::new(
            trussed,
            non_portable.store,
            non_portable.stolen_filesystem,
            non_portable.nfc_powered,
            uuid,
            non_portable.rebooter,
        )
    }
}
