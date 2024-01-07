use apdu_dispatch::{
    command::SIZE as ApduCommandSize, response::SIZE as ApduResponseSize, App as ApduApp,
};
use ctaphid_dispatch::app::App as CtaphidApp;
use littlefs2::{fs::Allocation, io::Result as LfsResult};
use trussed::{
    api::{reply, request, Reply, Request},
    backend::{Backend as _, BackendId},
    client::{ClientBuilder, ClientImplementation},
    interrupt::InterruptFlag,
    platform::{Platform, Store},
    serde_extensions::{ExtensionDispatch, ExtensionId, ExtensionImpl},
    service::ServiceResources,
    types::Context,
    Error as TrussedError,
};
use trussed_staging::{
    manage::ManageExtension, streaming::ChunkedExtension, StagingBackend, StagingContext,
};

use apps::InitStatus;
use utils::RamStorage;

use super::{
    nk3am::{
        self,
        ui::{HardwareButtons, RgbLed},
        DummyNfc, InternalFlashStorage,
    },
    Board,
};
use crate::{
    soc::{nrf52840::Nrf52, Soc as _},
    store::{impl_storage_pointers, RunnerStore},
    types::{self, RunnerSyscall, Trussed},
};

#[cfg(feature = "se050")]
compile_error!("NKPK does not support se050");

pub struct NKPK;

impl Board for NKPK {
    type Soc = Nrf52;

    type Apps = Apps;
    type Dispatch = Dispatch;

    type NfcDevice = DummyNfc;
    type Buttons = HardwareButtons;
    type Led = RgbLed;

    type Twi = ();
    type Se050Timer = ();

    const BOARD_NAME: &'static str = "NKPK";
    const USB_PRODUCT: &'static str = "Nitrokey Passkey";

    fn init_apps(
        trussed: &mut Trussed<Self>,
        init_status: InitStatus,
        store: &RunnerStore<Self>,
        nfc_powered: bool,
    ) -> Self::Apps
    where
        Self: Sized,
    {
        Apps::new(trussed, init_status, store, nfc_powered)
    }

    fn init_dispatch(_hw_key: Option<&[u8]>) -> Self::Dispatch {
        Default::default()
    }

    fn prepare_ifs(ifs: &mut Self::InternalStorage) {
        ifs.format_journal_blocks();
    }

    fn recover_ifs(
        ifs_storage: &mut Self::InternalStorage,
        ifs_alloc: &mut Allocation<Self::InternalStorage>,
        efs_storage: &mut Self::ExternalStorage,
    ) -> LfsResult<()> {
        let _ = (ifs_alloc, efs_storage);
        error_now!("IFS (nkpk) mount-fail");
        info_now!("recovering from journal");
        ifs_storage.recover_from_journal();
        Ok(())
    }
}

// TODO: do we really want to mirror the NK3AM EFS?
pub type ExternalFlashStorage = RamStorage<nk3am::ExternalFlashStorage, 256>;

impl_storage_pointers!(
    NKPK,
    Internal = InternalFlashStorage,
    External = ExternalFlashStorage,
);

type Client = ClientImplementation<RunnerSyscall<Nrf52>, Dispatch>;
type AdminApp = admin_app::App<Client, Nrf52, apps::AdminStatus, ()>;
type FidoApp = fido_authenticator::Authenticator<fido_authenticator::Conforming, Client>;
#[cfg(feature = "provisioner")]
type ProvisionerApp = provisioner_app::Provisioner<RunnerStore<NKPK>, InternalFlashStorage, Client>;

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    Staging,
    /// Separate BackendId to prevent non-priviledged apps from accessing the manage Extension
    StagingManage,
}

#[derive(Debug, Clone, Copy)]
pub enum Extension {
    Chunked,
    Manage,
}

impl From<Extension> for u8 {
    fn from(extension: Extension) -> Self {
        match extension {
            Extension::Chunked => 0,
            Extension::Manage => 1,
        }
    }
}

impl TryFrom<u8> for Extension {
    type Error = TrussedError;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            0 => Ok(Extension::Chunked),
            1 => Ok(Extension::Manage),
            _ => Err(TrussedError::InternalError),
        }
    }
}

#[derive(Default)]
pub struct DispatchContext {
    staging: StagingContext,
}

pub struct Dispatch {
    staging: StagingBackend,
}

impl Dispatch {
    fn new() -> Self {
        let mut staging = StagingBackend::new();
        staging.manage.should_preserve_file =
            |file, _location| apps::dispatch::should_preserve_file(file);
        Self { staging }
    }
}

impl Default for Dispatch {
    fn default() -> Self {
        Self::new()
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
            Backend::Staging => {
                self.staging
                    .request(&mut ctx.core, &mut ctx.backends.staging, request, resources)
            }
            Backend::StagingManage => Err(TrussedError::RequestNotAvailable),
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
            Backend::Staging => match extension {
                Extension::Chunked => {
                    ExtensionImpl::<ChunkedExtension>::extension_request_serialized(
                        &mut self.staging,
                        &mut ctx.core,
                        &mut ctx.backends.staging,
                        request,
                        resources,
                    )
                }
                _ => Err(TrussedError::RequestNotAvailable),
            },
            Backend::StagingManage => match extension {
                Extension::Manage => {
                    ExtensionImpl::<ManageExtension>::extension_request_serialized(
                        &mut self.staging,
                        &mut ctx.core,
                        &mut ctx.backends.staging,
                        request,
                        resources,
                    )
                }
                _ => Err(TrussedError::RequestNotAvailable),
            },
        }
    }
}

impl ExtensionId<ChunkedExtension> for Dispatch {
    type Id = Extension;
    const ID: Self::Id = Self::Id::Chunked;
}

impl ExtensionId<ManageExtension> for Dispatch {
    type Id = Extension;
    const ID: Self::Id = Self::Id::Manage;
}

pub struct Apps {
    admin: AdminApp,
    fido: FidoApp,
    #[cfg(feature = "provisioner")]
    provisioner: ProvisionerApp,
}

impl Apps {
    fn new(
        trussed: &mut Trussed<NKPK>,
        init_status: InitStatus,
        store: &RunnerStore<NKPK>,
        nfc_powered: bool,
    ) -> Self {
        let admin = Self::admin(trussed, init_status, store, nfc_powered);
        let fido = Self::fido(trussed);
        #[cfg(feature = "provisioner")]
        let provisioner = Self::provisioner(trussed, store, nfc_powered);

        Self {
            admin,
            fido,
            #[cfg(feature = "provisioner")]
            provisioner,
        }
    }

    fn client(
        trussed: &mut Trussed<NKPK>,
        id: &str,
        backends: &'static [BackendId<Backend>],
        interrupt: Option<&'static InterruptFlag>,
    ) -> Client {
        ClientBuilder::new(id)
            .backends(backends)
            .interrupt(interrupt)
            .prepare(trussed)
            .unwrap()
            .build(Default::default())
    }

    fn admin(
        trussed: &mut Trussed<NKPK>,
        init_status: InitStatus,
        store: &RunnerStore<NKPK>,
        nfc_powered: bool,
    ) -> AdminApp {
        const VERSION: u32 = utils::VERSION.encode();
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        let backends = &[BackendId::Custom(Backend::StagingManage), BackendId::Core];
        let client = Self::client(trussed, "admin", backends, Some(&INTERRUPT));

        let mut ifs_blocks = u8::MAX;
        let mut efs_blocks = u16::MAX;
        if !nfc_powered {
            if let Ok(n) = store.ifs().available_blocks() {
                if let Ok(n) = u8::try_from(n) {
                    ifs_blocks = n;
                }
            }
            if let Ok(n) = store.efs().available_blocks() {
                if let Ok(n) = u16::try_from(n) {
                    efs_blocks = n;
                }
            }
        }
        let efs_blocks = efs_blocks.to_be_bytes();

        let status = [
            init_status.bits(),
            ifs_blocks,
            efs_blocks[0],
            efs_blocks[1],
            apps::Variant::Nrf52.into(),
        ];
        AdminApp::with_default_config(
            client,
            *Nrf52::device_uuid(),
            VERSION,
            utils::VERSION_STRING,
            status,
        )
    }

    fn fido(trussed: &mut Trussed<NKPK>) -> FidoApp {
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        let backends = &[BackendId::Custom(Backend::Staging), BackendId::Core];
        let client = Self::client(trussed, "fido", backends, Some(&INTERRUPT));
        FidoApp::new(
            client,
            fido_authenticator::Conforming {},
            fido_authenticator::Config {
                max_msg_size: usbd_ctaphid::constants::MESSAGE_SIZE,
                skip_up_timeout: Some(core::time::Duration::from_secs(2)),
                max_resident_credential_count: Some(10),
                large_blobs: None,
            },
        )
    }

    #[cfg(feature = "provisioner")]
    fn provisioner(
        trussed: &mut Trussed<NKPK>,
        store: &RunnerStore<NKPK>,
        nfc_powered: bool,
    ) -> ProvisionerApp {
        use apps::Reboot as _;
        static INTERRUPT: InterruptFlag = InterruptFlag::new();
        let client = Self::client(trussed, "provisioner", &[], Some(&INTERRUPT));
        let stolen_filesystem = unsafe { crate::store::steal_internal_storage::<NKPK>() };
        ProvisionerApp::new(
            client,
            *store,
            stolen_filesystem,
            nfc_powered,
            *Nrf52::device_uuid(),
            Nrf52::reboot_to_firmware_update,
        )
    }
}

impl types::Apps for Apps {
    fn apdu_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn ApduApp<ApduCommandSize, ApduResponseSize>]) -> T,
    {
        f(&mut [
            &mut self.admin,
            &mut self.fido,
            #[cfg(feature = "provisioner")]
            &mut self.provisioner,
        ])
    }

    fn ctaphid_dispatch<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut [&mut dyn CtaphidApp<'static>]) -> T,
    {
        f(&mut [
            &mut self.admin,
            &mut self.fido,
            #[cfg(feature = "provisioner")]
            &mut self.provisioner,
        ])
    }
}
