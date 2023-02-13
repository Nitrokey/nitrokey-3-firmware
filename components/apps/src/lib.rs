#![no_std]

use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use core::marker::PhantomData;
use ctaphid_dispatch::app::App as CtaphidApp;
use trussed::{client::ClientBuilder, platform::Syscall, ClientImplementation, Platform, Service};

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
}

pub struct Data<R: Runner> {
    #[cfg(feature = "admin-app")]
    pub admin: AdminData,
    #[cfg(feature = "provisioner-app")]
    pub provisioner: ProvisionerData<R>,
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
    pub fn new(runner: &R, mut make_client: impl FnMut(&[u8]) -> Client<R>, data: Data<R>) -> Self {
        let Data {
            #[cfg(feature = "admin-app")]
            admin,
            #[cfg(feature = "provisioner-app")]
            provisioner,
            ..
        } = data;
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

    pub fn with_service<P: Platform>(runner: &R, trussed: &mut Service<P>, data: Data<R>) -> Self
    where
        R::Syscall: Default,
    {
        Self::new(
            runner,
            |id| {
                ClientBuilder::new(id)
                    .prepare(trussed)
                    .unwrap()
                    .build(R::Syscall::default())
            },
            data,
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
impl<R: Runner, D: trussed::backend::Dispatch> trussed_usbip::Apps<Client<R>, D> for Apps<R> {
    type Data = (R, Data<R>);

    fn new<B>(builder: &B, (runner, data): (R, Data<R>)) -> Self
    where
        B: trussed_usbip::ClientBuilder<Client<R>, D>,
    {
        Self::new(
            &runner,
            move |id| {
                let id = core::str::from_utf8(id).expect("invalid client id");
                builder.build(id, &[])
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
    /// additional data needed by this Trussed app
    type Data;

    /// the desired client ID
    const CLIENT_ID: &'static [u8];

    fn new(runner: &R, make_client: impl FnOnce(&[u8]) -> Client<R>, data: Self::Data) -> Self {
        Self::with_client(runner, make_client(Self::CLIENT_ID), data)
    }

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data) -> Self;
}

#[cfg(feature = "admin-app")]
#[derive(Default)]
pub struct AdminData {
    pub init_status: u8,
}

#[cfg(feature = "admin-app")]
impl<R: Runner> App<R> for AdminApp<R> {
    const CLIENT_ID: &'static [u8] = b"admin\0";

    type Data = AdminData;

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data) -> Self {
        const VERSION: u32 = utils::VERSION.encode();
        Self::new(
            trussed,
            runner.uuid(),
            VERSION,
            utils::VERSION_STRING,
            data.init_status,
        )
    }
}

#[cfg(feature = "fido-authenticator")]
impl<R: Runner> App<R> for FidoApp<R> {
    const CLIENT_ID: &'static [u8] = b"fido\0";

    type Data = ();

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

    type Data = ();

    fn with_client(_runner: &R, trussed: Client<R>, _: ()) -> Self {
        Self::new(trussed)
    }
}

#[cfg(feature = "opcard")]
impl<R: Runner> App<R> for OpcardApp<R> {
    const CLIENT_ID: &'static [u8] = b"opcard\0";

    type Data = ();

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
pub struct ProvisionerData<R: Runner> {
    pub store: R::Store,
    pub stolen_filesystem: &'static mut R::Filesystem,
    pub nfc_powered: bool,
    pub rebooter: fn() -> !,
}

#[cfg(feature = "provisioner-app")]
impl<R: Runner> App<R> for ProvisionerApp<R> {
    const CLIENT_ID: &'static [u8] = b"attn\0";

    type Data = ProvisionerData<R>;

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data) -> Self {
        let uuid = runner.uuid();
        Self::new(
            trussed,
            data.store,
            data.stolen_filesystem,
            data.nfc_powered,
            uuid,
            data.rebooter,
        )
    }
}
