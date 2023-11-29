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
#[cfg(feature = "se050")]
use embedded_hal::blocking::delay::DelayUs;
use littlefs2::path;
use serde::{Deserialize, Serialize};
use trussed::{
    backend::BackendId, client::ClientBuilder, interrupt::InterruptFlag, platform::Syscall,
    store::filestore::ClientFilestore, types::Path, ClientImplementation, Platform, Service,
};

pub use admin_app::Reboot;
use admin_app::{ConfigValueMut, ResetSignalAllocation};
use trussed::types::Location;

#[cfg(feature = "webcrypt")]
use webcrypt::{PeekingBypass, Webcrypt};

mod dispatch;
use dispatch::Backend;
pub use dispatch::Dispatch;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &Default::default()
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Config {
    #[serde(default, rename = "f", skip_serializing_if = "is_default")]
    fido: FidoConfig,
}

impl admin_app::Config for Config {
    fn field(&mut self, key: &str) -> Option<ConfigValueMut<'_>> {
        let (app, key) = key.split_once('.')?;
        match app {
            "fido" => self.fido.field(key),
            _ => None,
        }
    }

    fn reset_client_id(
        &self,
        key: &str,
    ) -> Option<(&'static Path, &'static ResetSignalAllocation)> {
        match key {
            #[cfg(feature = "factory-reset")]
            "opcard" => Some((path!("opcard"), &OPCARD_RESET_SIGNAL)),
            _ => None,
        }
    }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct FidoConfig {
    #[serde(default, rename = "t", skip_serializing_if = "is_default")]
    disable_skip_up_timeout: bool,
}

impl admin_app::Config for FidoConfig {
    fn field(&mut self, key: &str) -> Option<ConfigValueMut<'_>> {
        match key {
            "disable_skip_up_timeout" => {
                Some(ConfigValueMut::Bool(&mut self.disable_skip_up_timeout))
            }
            _ => None,
        }
    }
}

pub trait Runner {
    type Syscall: Syscall + 'static;

    type Reboot: Reboot;
    type Store: trussed::store::Store;
    #[cfg(feature = "provisioner-app")]
    type Filesystem: trussed::types::LfsStorage + 'static;
    #[cfg(feature = "se050")]
    type Twi: se05x::t1::I2CForT1 + 'static;
    #[cfg(feature = "se050")]
    type Se050Timer: DelayUs<u32> + 'static;
    #[cfg(not(feature = "se050"))]
    type Twi: 'static;
    #[cfg(not(feature = "se050"))]
    type Se050Timer: 'static;

    fn uuid(&self) -> [u8; 16];
    fn is_efs_available(&self) -> bool;
}

pub struct Data<R: Runner> {
    pub admin: AdminData<R>,
    #[cfg(feature = "provisioner-app")]
    pub provisioner: ProvisionerData<R>,
    pub _marker: PhantomData<R>,
}

type Client<R> = ClientImplementation<
    <R as Runner>::Syscall,
    Dispatch<<R as Runner>::Twi, <R as Runner>::Se050Timer>,
>;

type AdminApp<R> = admin_app::App<Client<R>, <R as Runner>::Reboot, AdminStatus, Config>;
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
            admin,
            #[cfg(feature = "provisioner-app")]
            provisioner,
            ..
        } = data;

        let admin = AdminApp::<R>::new(runner, &mut make_client, admin, &());

        #[cfg(feature = "webcrypt")]
        let webcrypt_fido_bypass = PeekingBypass::new(
            App::new(runner, &mut make_client, (), &admin.config().fido),
            App::new(runner, &mut make_client, (), &()),
        );

        Self {
            #[cfg(all(feature = "fido-authenticator", not(feature = "webcrypt")))]
            fido: App::new(runner, &mut make_client, (), &admin.config().fido),
            #[cfg(feature = "ndef-app")]
            ndef: NdefApp::new(),
            #[cfg(feature = "secrets-app")]
            oath: App::new(runner, &mut make_client, (), &()),
            #[cfg(feature = "opcard")]
            opcard: App::new(runner, &mut make_client, (), &()),
            #[cfg(feature = "piv-authenticator")]
            piv: App::new(runner, &mut make_client, (), &()),
            #[cfg(feature = "provisioner-app")]
            provisioner: App::new(runner, &mut make_client, provisioner, &()),
            #[cfg(feature = "webcrypt")]
            webcrypt: webcrypt_fido_bypass,
            admin,
        }
    }

    pub fn with_service<P: Platform>(
        runner: &R,
        trussed: &mut Service<P, Dispatch<R::Twi, R::Se050Timer>>,
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
    type Config: admin_app::Config;

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
        config: &Self::Config,
    ) -> Self {
        let backends = Self::backends(runner, config);
        Self::with_client(
            runner,
            make_client(Self::CLIENT_ID, backends, Self::interrupt()),
            data,
            config,
        )
    }

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data, config: &Self::Config)
        -> Self;

    fn backends(runner: &R, config: &Self::Config) -> &'static [BackendId<Backend>] {
        let _ = (runner, config);
        const BACKENDS_DEFAULT: &[BackendId<Backend>] = &[];
        BACKENDS_DEFAULT
    }

    fn interrupt() -> Option<&'static InterruptFlag> {
        None
    }
}

#[derive(Copy, Clone)]
pub enum Variant {
    Usbip,
    Lpc55,
    Nrf52,
}

impl From<Variant> for u8 {
    fn from(variant: Variant) -> Self {
        match variant {
            Variant::Usbip => 0,
            Variant::Lpc55 => 1,
            Variant::Nrf52 => 2,
        }
    }
}

pub struct AdminData<R: Runner> {
    pub store: R::Store,
    pub init_status: u8,
    pub ifs_blocks: u8,
    pub efs_blocks: u16,
    pub variant: Variant,
}

impl<R: Runner> AdminData<R> {
    pub fn new(store: R::Store, variant: Variant) -> Self {
        Self {
            store,
            init_status: 0,
            ifs_blocks: u8::MAX,
            efs_blocks: u16::MAX,
            variant,
        }
    }
}

pub type AdminStatus = [u8; 5];

impl<R: Runner> AdminData<R> {
    fn status(&self) -> AdminStatus {
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

const ADMIN_APP_CLIENT_ID: &str = "admin";

impl<R: Runner> App<R> for AdminApp<R> {
    const CLIENT_ID: &'static str = ADMIN_APP_CLIENT_ID;

    type Data = AdminData<R>;
    type Config = ();

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data, _: &()) -> Self {
        const VERSION: u32 = utils::VERSION.encode();
        // TODO: use CLIENT_ID directly
        let mut filestore = ClientFilestore::new(ADMIN_APP_CLIENT_ID.into(), data.store);
        Self::load(
            trussed,
            &mut filestore,
            runner.uuid(),
            VERSION,
            utils::VERSION_STRING,
            data.status(),
        )
    }
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
    }

    fn backends(runner: &R, _config: &()) -> &'static [BackendId<Backend>] {
        const BACKENDS_ADMIN: &[BackendId<Backend>] = &[
            BackendId::Custom(Backend::StagingManage),
            #[cfg(feature = "se050-test-app")]
            BackendId::Custom(Backend::Se050),
            BackendId::Core,
        ];
        let _ = runner;
        BACKENDS_ADMIN
    }
}

#[cfg(feature = "fido-authenticator")]
impl<R: Runner> App<R> for FidoApp<R> {
    const CLIENT_ID: &'static str = "fido";

    type Data = ();
    type Config = FidoConfig;

    fn with_client(runner: &R, trussed: Client<R>, _: (), config: &Self::Config) -> Self {
        let skip_up_timeout = if config.disable_skip_up_timeout {
            None
        } else {
            Some(core::time::Duration::from_secs(2))
        };
        let large_blobs = if cfg!(feature = "test") && runner.is_efs_available() {
            Some(fido_authenticator::LargeBlobsConfig {
                location: Location::External,
                max_size: 4096,
            })
        } else {
            None
        };
        fido_authenticator::Authenticator::new(
            trussed,
            fido_authenticator::Conforming {},
            fido_authenticator::Config {
                max_msg_size: usbd_ctaphid::constants::MESSAGE_SIZE,
                skip_up_timeout,
                max_resident_credential_count: Some(10),
                large_blobs,
            },
        )
    }
    fn interrupt() -> Option<&'static InterruptFlag> {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        Some(&INTERRUPT)
    }

    fn backends(_runner: &R, _config: &Self::Config) -> &'static [BackendId<Backend>] {
        &[BackendId::Custom(Backend::Staging), BackendId::Core]
    }
}

#[cfg(feature = "webcrypt")]
impl<R: Runner> App<R> for WebcryptApp<R> {
    const CLIENT_ID: &'static str = "webcrypt";

    type Data = ();
    type Config = ();

    fn with_client(runner: &R, trussed: Client<R>, _: (), _: &()) -> Self {
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
    fn backends(runner: &R, _: &()) -> &'static [BackendId<Backend>] {
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
    type Config = ();

    fn with_client(runner: &R, trussed: Client<R>, _: (), _: &()) -> Self {
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
    fn backends(runner: &R, _: &()) -> &'static [BackendId<Backend>] {
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

#[cfg(feature = "factory-reset")]
static OPCARD_RESET_SIGNAL: ResetSignalAllocation = ResetSignalAllocation::new();

#[cfg(feature = "opcard")]
impl<R: Runner> App<R> for OpcardApp<R> {
    const CLIENT_ID: &'static str = "opcard";

    type Data = ();
    type Config = ();

    fn with_client(runner: &R, trussed: Client<R>, _: (), _: &()) -> Self {
        let uuid = runner.uuid();
        let mut options = opcard::Options::default();
        options.button_available = true;
        // See scd/app-openpgp.c in GnuPG for the manufacturer IDs
        options.manufacturer = 0x000Fu16.to_be_bytes();
        options.serial = [uuid[0], uuid[1], uuid[2], uuid[3]];
        options.storage = trussed::types::Location::External;
        #[cfg(feature = "factory-reset")]
        {
            options.reset_signal = Some(&OPCARD_RESET_SIGNAL);
        }
        Self::new(trussed, options)
    }
    fn backends(runner: &R, _: &()) -> &'static [BackendId<Backend>] {
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
    type Config = ();

    fn with_client(runner: &R, trussed: Client<R>, _: (), _: &()) -> Self {
        Self::new(
            trussed,
            piv_authenticator::Options::default().uuid(Some(runner.uuid())),
        )
    }
    fn backends(runner: &R, _: &()) -> &'static [BackendId<Backend>] {
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
    type Config = ();

    fn with_client(runner: &R, trussed: Client<R>, data: Self::Data, _: &()) -> Self {
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

#[cfg(test)]
mod tests {
    use super::{Config, FidoConfig};
    use cbor_smol::{cbor_serialize_bytes, Bytes};

    #[test]
    fn test_config_size() {
        let config = Config {
            fido: FidoConfig {
                disable_skip_up_timeout: true,
            },
        };
        let data: Bytes<1024> = cbor_serialize_bytes(&config).unwrap();
        // littlefs2 is most efficient with files < 1/4 of the block size.  The block sizes are 512
        // bytes for LPC55 and 256 bytes for NRF52.  As the block count is only problematic on the
        // LPC55, this could be increased to 128 if necessary.
        assert!(data.len() < 64, "{}: {}", data.len(), hex::encode(&data));
    }
}
