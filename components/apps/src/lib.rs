#![no_std]

#[cfg(feature = "secrets-app")]
const SECRETS_APP_CREDENTIALS_COUNT_LIMIT: u16 = 50;
#[cfg(feature = "webcrypt")]
const WEBCRYPT_APP_CREDENTIALS_COUNT_LIMIT: u16 = 50;

use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use core::marker::PhantomData;
use ctaphid_dispatch::app::App as CtaphidApp;
use trussed::{
    backend::BackendId, client::ClientBuilder, interrupt::InterruptFlag, platform::Syscall,
    ClientImplementation, Platform, Service,
};

#[cfg(feature = "admin-app")]
pub use admin_app::Reboot;
use trussed::types::Location;

#[cfg(feature = "webcrypt")]
use webcrypt::{PeekingBypass, Webcrypt};

mod dispatch;
use dispatch::Backend;
pub use dispatch::Dispatch;

pub trait Runner {
    type Syscall: Syscall + 'static;

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
#[cfg(feature = "secrets-app")]
type SecretsApp<R> = secrets_app::Authenticator<Client<R>>;
#[cfg(feature = "webcrypt")]
type WebcryptApp<R> = webcrypt::Webcrypt<Client<R>>;
#[cfg(feature = "opcard")]
type OpcardApp<R> = opcard::Card<Client<R>>;
#[cfg(feature = "piv-authenticator")]
type PivApp<R> = piv_authenticator::Authenticator<Client<R>>;
#[cfg(feature = "provisioner-app")]
type ProvisionerApp<R> =
    provisioner_app::Provisioner<<R as Runner>::Store, <R as Runner>::Filesystem, Client<R>>;

#[repr(u8)]
pub enum CustomStatus {
    #[cfg(feature = "secrets-app")]
    ReverseHotpSuccess = 0,
    #[cfg(feature = "secrets-app")]
    ReverseHotpError = 1,
}

impl From<CustomStatus> for u8 {
    fn from(status: CustomStatus) -> Self {
        status as _
    }
}

impl TryFrom<u8> for CustomStatus {
    type Error = UnknownStatusError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            #[cfg(feature = "secrets-app")]
            0 => Ok(Self::ReverseHotpSuccess),
            #[cfg(feature = "secrets-app")]
            1 => Ok(Self::ReverseHotpError),
            _ => Err(UnknownStatusError(value)),
        }
    }
}

pub struct UnknownStatusError(u8);

pub struct Apps<R: Runner> {
    #[cfg(feature = "admin-app")]
    admin: AdminApp<R>,
    #[cfg(all(feature = "fido-authenticator", not(feature = "webcrypt")))]
    fido: FidoApp<R>,
    #[cfg(feature = "ndef-app")]
    ndef: NdefApp,
    #[cfg(feature = "secrets-app")]
    oath: SecretsApp<R>,
    #[cfg(feature = "opcard")]
    opcard: OpcardApp<R>,
    #[cfg(feature = "piv-authenticator")]
    piv: PivApp<R>,
    #[cfg(feature = "provisioner-app")]
    provisioner: ProvisionerApp<R>,
    #[cfg(feature = "webcrypt")]
    webcrypt: PeekingBypass<'static, FidoApp<R>, WebcryptApp<R>>,
    /// Avoid compilation error if no feature is used.
    /// Without it, the type parameter `R` is not used
    _compile_no_feature: PhantomData<R>,
}

impl<R: Runner> Apps<R> {
    pub fn new(
        runner: &R,
        mut make_client: impl FnMut(
            &str,
            &'static [BackendId<Backend>],
            Option<&'static InterruptFlag>,
        ) -> Client<R>,
        data: Data<R>,
    ) -> Self {
        let _ = (runner, &mut make_client);
        let Data {
            #[cfg(feature = "admin-app")]
            admin,
            #[cfg(feature = "provisioner-app")]
            provisioner,
            ..
        } = data;
        #[cfg(feature = "webcrypt")]
        let webcrypt_fido_bypass = PeekingBypass::new(
            App::new(runner, &mut make_client, ()),
            App::new(runner, &mut make_client, ()),
        );
        Self {
            #[cfg(feature = "admin-app")]
            admin: App::new(runner, &mut make_client, admin),
            #[cfg(all(feature = "fido-authenticator", not(feature = "webcrypt")))]
            fido: App::new(runner, &mut make_client, ()),
            #[cfg(feature = "ndef-app")]
            ndef: NdefApp::new(),
            #[cfg(feature = "secrets-app")]
            oath: App::new(runner, &mut make_client, ()),
            #[cfg(feature = "opcard")]
            opcard: App::new(runner, &mut make_client, ()),
            #[cfg(feature = "piv-authenticator")]
            piv: App::new(runner, &mut make_client, ()),
            #[cfg(feature = "provisioner-app")]
            provisioner: App::new(runner, &mut make_client, provisioner),
            #[cfg(feature = "webcrypt")]
            webcrypt: webcrypt_fido_bypass,
            _compile_no_feature: PhantomData::default(),
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
            |id, backends, interrupt| {
                ClientBuilder::new(id)
                    .backends(backends)
                    .interrupt(interrupt)
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
            #[cfg(feature = "secrets-app")]
            &mut self.oath,
            #[cfg(feature = "opcard")]
            &mut self.opcard,
            #[cfg(feature = "piv-authenticator")]
            &mut self.piv,
            #[cfg(all(feature = "fido-authenticator", not(feature = "webcrypt")))]
            &mut self.fido,
            #[cfg(feature = "admin-app")]
            &mut self.admin,
            #[cfg(feature = "provisioner-app")]
            &mut self.provisioner,
            // #[cfg(feature = "webcrypt")]
            // &mut self.webcrypt,
        ])
    }

    pub fn ctaphid_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn CtaphidApp<'static>]) -> T,
    {
        f(&mut [
            #[cfg(feature = "webcrypt")]
            &mut self.webcrypt,
            #[cfg(all(feature = "fido-authenticator", not(feature = "webcrypt")))]
            &mut self.fido,
            #[cfg(feature = "admin-app")]
            &mut self.admin,
            #[cfg(feature = "secrets-app")]
            &mut self.oath,
            #[cfg(feature = "provisioner-app")]
            &mut self.provisioner,
        ])
    }
}

#[cfg(feature = "trussed-usbip")]
impl<R: Runner> trussed_usbip::Apps<'static, Client<R>, Dispatch> for Apps<R> {
    type Data = (R, Data<R>);

    fn new<B>(builder: &B, (runner, data): (R, Data<R>)) -> Self
    where
        B: trussed_usbip::ClientBuilder<Client<R>, Dispatch>,
    {
        Self::new(
            &runner,
            move |id, backends, _| builder.build(id, backends),
            data,
        )
    }

    fn with_ctaphid_apps<T>(
        &mut self,
        f: impl FnOnce(&mut [&mut dyn CtaphidApp<'static>]) -> T,
    ) -> T {
        self.ctaphid_dispatch(f)
    }

    #[cfg(feature = "trussed-usbip-ccid")]
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
        make_client: impl FnOnce(
            &str,
            &'static [BackendId<Backend>],
            Option<&'static InterruptFlag>,
        ) -> Client<R>,
        data: Self::Data,
    ) -> Self {
        let backends = Self::backends(runner);
        Self::with_client(
            runner,
            make_client(Self::CLIENT_ID, backends, Self::interrupt()),
            data,
        )
    }

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data) -> Self;

    fn backends(runner: &R) -> &'static [BackendId<Backend>] {
        let _ = runner;
        const BACKENDS_DEFAULT: &[BackendId<Backend>] = &[];
        BACKENDS_DEFAULT
    }

    fn interrupt() -> Option<&'static InterruptFlag> {
        None
    }
}

#[cfg(feature = "admin-app")]
#[derive(Copy, Clone)]
pub enum Variant {
    Usbip,
    Lpc55,
    Nrf52,
}

#[cfg(feature = "admin-app")]
impl From<Variant> for u8 {
    fn from(variant: Variant) -> Self {
        match variant {
            Variant::Usbip => 0,
            Variant::Lpc55 => 1,
            Variant::Nrf52 => 2,
        }
    }
}

#[cfg(feature = "admin-app")]
pub struct AdminData {
    pub init_status: u8,
    pub ifs_blocks: u8,
    pub efs_blocks: u16,
    pub variant: Variant,
}

#[cfg(feature = "admin-app")]
impl AdminData {
    pub fn new(variant: Variant) -> Self {
        Self {
            init_status: 0,
            ifs_blocks: u8::MAX,
            efs_blocks: u16::MAX,
            variant,
        }
    }
}

#[cfg(feature = "admin-app")]
pub type AdminStatus = [u8; 5];

#[cfg(feature = "admin-app")]
impl AdminData {
    fn encode(&self) -> AdminStatus {
        let efs_blocks = self.efs_blocks.to_be_bytes();
        [
            self.init_status,
            self.ifs_blocks,
            efs_blocks[0],
            efs_blocks[1],
            self.variant.into(),
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
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
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
                max_resident_credential_count: Some(10),
            },
        )
    }
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
    }
}

#[cfg(feature = "webcrypt")]
impl<R: Runner> App<R> for WebcryptApp<R> {
    const CLIENT_ID: &'static str = "webcrypt";

    type Data = ();

    fn with_client(runner: &R, trussed: Client<R>, _: ()) -> Self {
        let uuid = runner.uuid();
        Webcrypt::new_with_options(
            trussed,
            webcrypt::Options::new(
                Location::External,
                [uuid[0], uuid[1], uuid[2], uuid[3]],
                WEBCRYPT_APP_CREDENTIALS_COUNT_LIMIT,
            ),
        )
    }
    fn backends(runner: &R) -> &'static [BackendId<Backend>] {
        const BACKENDS_WEBCRYPT: &[BackendId<Backend>] = &[
            BackendId::Custom(Backend::SoftwareRsa),
            BackendId::Custom(Backend::Staging),
            BackendId::Custom(Backend::Auth),
            BackendId::Core,
        ];
        let _ = runner;
        BACKENDS_WEBCRYPT
    }
}

#[cfg(feature = "secrets-app")]
impl<R: Runner> App<R> for SecretsApp<R> {
    const CLIENT_ID: &'static str = "secrets";

    type Data = ();

    fn with_client(runner: &R, trussed: Client<R>, _: ()) -> Self {
        let uuid = runner.uuid();
        let options = secrets_app::Options::new(
            Location::External,
            CustomStatus::ReverseHotpSuccess.into(),
            CustomStatus::ReverseHotpError.into(),
            [uuid[0], uuid[1], uuid[2], uuid[3]],
            SECRETS_APP_CREDENTIALS_COUNT_LIMIT,
        );
        Self::new(trussed, options)
    }
    fn backends(runner: &R) -> &'static [BackendId<Backend>] {
        const BACKENDS_OATH: &[BackendId<Backend>] =
            &[BackendId::Custom(Backend::Auth), BackendId::Core];
        let _ = runner;
        BACKENDS_OATH
    }
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
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
        // See scd/app-openpgp.c in GnuPG for the manufacturer IDs
        options.manufacturer = 0x000Fu16.to_be_bytes();
        options.serial = [uuid[0], uuid[1], uuid[2], uuid[3]];
        options.storage = trussed::types::Location::External;
        Self::new(trussed, options)
    }
    fn backends(runner: &R) -> &'static [BackendId<Backend>] {
        const BACKENDS_OPCARD: &[BackendId<Backend>] = &[
            BackendId::Custom(Backend::SoftwareRsa),
            BackendId::Custom(Backend::Auth),
            BackendId::Custom(Backend::Staging),
            BackendId::Core,
        ];
        let _ = runner;
        BACKENDS_OPCARD
    }
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
    }
}

#[cfg(feature = "piv-authenticator")]
impl<R: Runner> App<R> for PivApp<R> {
    const CLIENT_ID: &'static str = "piv";

    type Data = ();

    fn with_client(runner: &R, trussed: Client<R>, _: ()) -> Self {
        Self::new(
            trussed,
            piv_authenticator::Options::default().uuid(Some(runner.uuid())),
        )
    }
    fn backends(runner: &R) -> &'static [BackendId<Backend>] {
        const BACKENDS_PIV: &[BackendId<Backend>] = &[
            BackendId::Custom(Backend::SoftwareRsa),
            BackendId::Custom(Backend::Auth),
            BackendId::Custom(Backend::Staging),
            BackendId::Core,
        ];
        let _ = runner;
        BACKENDS_PIV
    }
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
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
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
    }
}
