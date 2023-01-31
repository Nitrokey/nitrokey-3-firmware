#![no_std]

use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use core::marker::PhantomData;
use ctaphid_dispatch::app::App as CtaphidApp;
use trussed::{
    pipe::TrussedInterchange, platform::Syscall, types::PathBuf, ClientImplementation,
    Interchange as _, Platform, Service,
};

#[cfg(feature = "admin-app")]
pub use admin_app::Reboot;

pub trait Runner {
    type Syscall: Syscall;

    #[cfg(feature = "admin-app")]
    type Reboot: Reboot;
    #[cfg(feature = "provisioner-app")]
    type Store: trussed::store::Store;
    #[cfg(feature = "provisioner-app")]
    type Filesystem: trussed::types::LfsStorage + 'static;

    fn uuid(&self) -> [u8; 16];
    fn version(&self) -> u32;
    fn full_version(&self) -> &'static str;
}

pub struct NonPortable<R: Runner> {
    #[cfg(feature = "admin-app")]
    pub admin: AdminAppNonPortable,
    #[cfg(feature = "provisioner-app")]
    pub provisioner: ProvisionerNonPortable<R>,
    pub _marker: PhantomData<R>,
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
    pub fn new(
        runner: &R,
        mut make_client: impl FnMut(&[u8]) -> Client<R>,
        non_portable: NonPortable<R>,
    ) -> Self {
        let NonPortable {
            #[cfg(feature = "admin-app")]
            admin,
            #[cfg(feature = "provisioner-app")]
            provisioner,
            ..
        } = non_portable;
        Self {
            #[cfg(feature = "admin-app")]
            admin: App::new(runner, &mut make_client, admin),
            #[cfg(feature = "fido-authenticator")]
            fido: App::new(runner, &mut make_client, ()),
            #[cfg(feature = "ndef-app")]
            ndef: NdefApp::new(),
            #[cfg(feature = "oath-authenticator")]
            oath: App::new(runner, &mut make_client, ()),
            #[cfg(feature = "opcard")]
            opcard: App::new(runner, &mut make_client, ()),
            #[cfg(feature = "provisioner-app")]
            provisioner: App::new(runner, &mut make_client, provisioner),
        }
    }

    pub fn with_service<P: Platform>(
        runner: &R,
        trussed: &mut Service<P>,
        non_portable: NonPortable<R>,
    ) -> Self
    where
        R::Syscall: Default,
    {
        Self::new(
            runner,
            |id| {
                let (trussed_requester, trussed_responder) =
                    TrussedInterchange::claim().expect("could not setup TrussedInterchange");

                let mut client_id = PathBuf::new();
                client_id.push(id.try_into().unwrap());
                assert!(trussed.add_endpoint(trussed_responder, client_id).is_ok());

                let syscaller = R::Syscall::default();
                ClientImplementation::new(trussed_requester, syscaller)
            },
            non_portable,
        )
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
            #[cfg(feature = "provisioner-app")]
            &mut self.provisioner,
        ])
    }
}

#[cfg(feature = "trussed-usbip")]
impl<R: Runner> trussed_usbip::Apps<Client<R>, (&R, NonPortable<R>)> for Apps<R> {
    fn new(make_client: impl Fn(&str) -> Client<R>, (runner, data): (&R, NonPortable<R>)) -> Self {
        Self::new(
            runner,
            move |id| {
                let id = core::str::from_utf8(id).expect("invalid client id");
                make_client(id)
            },
            data,
        )
    }

    fn with_ctaphid_apps<T>(&mut self, f: impl FnOnce(&mut [&mut dyn CtaphidApp]) -> T) -> T {
        self.ctaphid_dispatch(f)
    }

    fn with_ccid_apps<T>(
        &mut self,
        f: impl FnOnce(&mut [&mut dyn apdu_dispatch::App<ApduCommandSize, ApduResponseSize>]) -> T,
    ) -> T {
        self.apdu_dispatch(f)
    }
}

trait App<R: Runner>: Sized {
    /// non-portable resources needed by this Trussed app
    type NonPortable;

    /// the desired client ID
    const CLIENT_ID: &'static [u8];

    fn new(
        runner: &R,
        make_client: impl FnOnce(&[u8]) -> Client<R>,
        non_portable: Self::NonPortable,
    ) -> Self {
        Self::with_client(runner, make_client(Self::CLIENT_ID), non_portable)
    }

    fn with_client(runner: &R, trussed: Client<R>, non_portable: Self::NonPortable) -> Self;
}

#[cfg(feature = "admin-app")]
#[derive(Default)]
pub struct AdminAppNonPortable {
    pub init_status: u8,
}

#[cfg(feature = "admin-app")]
impl<R: Runner> App<R> for AdminApp<R> {
    const CLIENT_ID: &'static [u8] = b"admin\0";

    type NonPortable = AdminAppNonPortable;

    fn with_client(runner: &R, trussed: Client<R>, non_portable: Self::NonPortable) -> Self {
        Self::new(
            trussed,
            runner.uuid(),
            runner.version(),
            runner.full_version(),
            non_portable.init_status,
        )
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
        options.button_available = true;
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
