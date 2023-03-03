#![no_std]

use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use core::marker::PhantomData;
use ctaphid_dispatch::app::App as CtaphidApp;
use trussed::{
    api::{reply, request, Reply, Request},
    backend::{Backend as _, BackendId},
    client::ClientBuilder,
    error::Error as TrussedError,
    platform::Syscall,
    serde_extensions::{ExtensionDispatch, ExtensionId, ExtensionImpl},
    service::ServiceResources,
    types::{Context, Location},
    Bytes, ClientImplementation, Platform, Service,
};
use trussed_auth::{AuthBackend, AuthContext, AuthExtension, MAX_HW_KEY_LEN};

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

type Client<R> = ClientImplementation<<R as Runner>::Syscall, Dispatch>;

#[cfg(feature = "admin-app")]
type AdminApp<R> = admin_app::App<Client<R>, <R as Runner>::Reboot, AdminStatus>;
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

#[derive(Debug)]
pub struct Dispatch {
    auth: AuthBackend,
}

#[derive(Debug, Default)]
pub struct DispatchContext {
    auth: AuthContext,
}

impl Dispatch {
    pub fn new(auth_location: Location) -> Self {
        Self {
            auth: AuthBackend::new(auth_location),
        }
    }
    pub fn with_hw_key(auth_location: Location, hw_key: Bytes<MAX_HW_KEY_LEN>) -> Self {
        Self {
            auth: AuthBackend::with_hw_key(auth_location, hw_key),
        }
    }
}

impl ExtensionDispatch for Dispatch {
    type Context = DispatchContext;
    type BackendId = Backend;
    type ExtensionId = Extension;

    fn core_request<P: Platform>(
        &mut self,
        backend: &Self::BackendId,
        ctx: &mut Context<Self::Context>,
        request: &Request,
        resources: &mut ServiceResources<P>,
    ) -> Result<Reply, TrussedError> {
        match backend {
            Backend::Auth => {
                self.auth
                    .request(&mut ctx.core, &mut ctx.backends.auth, request, resources)
            }
        }
    }

    fn extension_request<P: Platform>(
        &mut self,
        backend: &Self::BackendId,
        extension: &Self::ExtensionId,
        ctx: &mut Context<Self::Context>,
        request: &request::SerdeExtension,
        resources: &mut ServiceResources<P>,
    ) -> Result<reply::SerdeExtension, TrussedError> {
        match backend {
            Backend::Auth => match extension {
                Extension::Auth => self.auth.extension_request_serialized(
                    &mut ctx.core,
                    &mut ctx.backends.auth,
                    request,
                    resources,
                ),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Auth,
}

#[derive(Debug, Clone, Copy)]
pub enum Extension {
    Auth,
}

impl From<Extension> for u8 {
    fn from(extension: Extension) -> Self {
        match extension {
            Extension::Auth => 0,
        }
    }
}

impl TryFrom<u8> for Extension {
    type Error = TrussedError;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            0 => Ok(Extension::Auth),
            _ => Err(TrussedError::InternalError),
        }
    }
}

impl ExtensionId<AuthExtension> for Dispatch {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Auth;
}

const BACKENDS_DEFAULT: &[BackendId<Backend>] = &[];

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
        mut make_client: impl FnMut(&str, &'static [BackendId<Backend>]) -> Client<R>,
        data: Data<R>,
    ) -> Self {
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

    pub fn with_service<P: Platform>(
        runner: &R,
        trussed: &mut Service<P, Dispatch>,
        data: Data<R>,
    ) -> Self
    where
        R::Syscall: Default,
    {
        Self::new(
            runner,
            |id, backends| {
                ClientBuilder::new(id)
                    .backends(backends)
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
impl<R: Runner> trussed_usbip::Apps<Client<R>, Dispatch> for Apps<R> {
    type Data = (R, Data<R>);

    fn new<B>(builder: &B, (runner, data): (R, Data<R>)) -> Self
    where
        B: trussed_usbip::ClientBuilder<Client<R>, Dispatch>,
    {
        Self::new(
            &runner,
            move |id, backends| builder.build(id, backends),
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
    const CLIENT_ID: &'static str;

    fn new(
        runner: &R,
        make_client: impl FnOnce(&str, &'static [BackendId<Backend>]) -> Client<R>,
        data: Self::Data,
    ) -> Self {
        let backends = Self::backends(runner);
        Self::with_client(runner, make_client(Self::CLIENT_ID, backends), data)
    }

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data) -> Self;

    fn backends(runner: &R) -> &'static [BackendId<Backend>] {
        let _ = runner;
        BACKENDS_DEFAULT
    }
}

#[cfg(feature = "admin-app")]
pub struct AdminData {
    pub init_status: u8,
    pub ifs_blocks: u8,
    pub efs_blocks: u16,
}

#[cfg(feature = "admin-app")]
impl Default for AdminData {
    fn default() -> Self {
        Self {
            init_status: 0,
            ifs_blocks: u8::MAX,
            efs_blocks: u16::MAX,
        }
    }
}

#[cfg(feature = "admin-app")]
pub type AdminStatus = [u8; 4];

#[cfg(feature = "admin-app")]
impl AdminData {
    fn encode(&self) -> AdminStatus {
        let efs_blocks = self.efs_blocks.to_be_bytes();
        [
            self.init_status,
            self.ifs_blocks,
            efs_blocks[0],
            efs_blocks[1],
        ]
    }
}

#[cfg(feature = "admin-app")]
impl<R: Runner> App<R> for AdminApp<R> {
    const CLIENT_ID: &'static str = "admin";

    type Data = AdminData;

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data) -> Self {
        const VERSION: u32 = utils::VERSION.encode();
        Self::new(
            trussed,
            runner.uuid(),
            VERSION,
            utils::VERSION_STRING,
            data.encode(),
        )
    }
}

#[cfg(feature = "fido-authenticator")]
impl<R: Runner> App<R> for FidoApp<R> {
    const CLIENT_ID: &'static str = "fido";

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
    const CLIENT_ID: &'static str = "oath";

    type Data = ();

    fn with_client(_runner: &R, trussed: Client<R>, _: ()) -> Self {
        let mut options = oath_authenticator::Options::default();
        options.location = trussed::types::Location::Internal;
        Self::with_options(trussed, options)
    }
}

#[cfg(feature = "opcard")]
impl<R: Runner> App<R> for OpcardApp<R> {
    const CLIENT_ID: &'static str = "opcard";

    type Data = ();

    fn with_client(runner: &R, trussed: Client<R>, _: ()) -> Self {
        let uuid = runner.uuid();
        let mut options = opcard::Options::default();
        options.button_available = true;
        options.serial = [0xa0, 0x20, uuid[0], uuid[1]];
        options.storage = trussed::types::Location::Internal;
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
    const CLIENT_ID: &'static str = "attn";

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
