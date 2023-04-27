use trussed::{
    api::{Reply, Request},
    error::Error as TrussedError,
    service::ServiceResources,
    types::{Context, Location},
    Platform,
};

#[cfg(feature = "backend-auth")]
use trussed::{
    api::{reply, request},
    backend::Backend as _,
    serde_extensions::{ExtensionDispatch, ExtensionId, ExtensionImpl},
    Bytes,
};
#[cfg(feature = "backend-auth")]
use trussed_auth::{AuthBackend, AuthContext, AuthExtension, MAX_HW_KEY_LEN};

#[cfg(feature = "backend-rsa")]
use trussed_rsa_alloc::SoftwareRsa;

#[cfg(feature = "backend-staging")]
use trussed_staging::{
    streaming::ChunkedExtension, wrap_key_to_file::WrapKeyToFileExtension, StagingBackend,
    StagingContext,
};

#[derive(Debug)]
pub struct Dispatch {
    #[cfg(feature = "backend-auth")]
    auth: AuthBackend,
    #[cfg(feature = "backend-staging")]
    staging: StagingBackend,
}

#[derive(Default)]
pub struct DispatchContext {
    #[cfg(feature = "backend-auth")]
    auth: AuthContext,
    #[cfg(feature = "backend-staging")]
    staging: StagingContext,
}

impl Dispatch {
    pub fn new(auth_location: Location) -> Self {
        #[cfg(not(feature = "backend-auth"))]
        let _ = auth_location;
        Self {
            #[cfg(feature = "backend-auth")]
            auth: AuthBackend::new(auth_location),
            #[cfg(feature = "backend-staging")]
            staging: StagingBackend::new(),
        }
    }

    #[cfg(feature = "backend-auth")]
    pub fn with_hw_key(auth_location: Location, hw_key: Bytes<MAX_HW_KEY_LEN>) -> Self {
        Self {
            auth: AuthBackend::with_hw_key(auth_location, hw_key),
            #[cfg(feature = "backend-staging")]
            staging: StagingBackend::new(),
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
            #[cfg(feature = "backend-auth")]
            Backend::Auth => {
                self.auth
                    .request(&mut ctx.core, &mut ctx.backends.auth, request, resources)
            }
            #[cfg(feature = "backend-rsa")]
            Backend::SoftwareRsa => SoftwareRsa.request(&mut ctx.core, &mut (), request, resources),
            #[cfg(feature = "backend-staging")]
            Backend::Staging => {
                self.staging
                    .request(&mut ctx.core, &mut ctx.backends.staging, request, resources)
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
            #[cfg(feature = "backend-auth")]
            Backend::Auth => match extension {
                Extension::Auth => self.auth.extension_request_serialized(
                    &mut ctx.core,
                    &mut ctx.backends.auth,
                    request,
                    resources,
                ),
                #[allow(unreachable_patterns)]
                _ => Err(TrussedError::RequestNotAvailable),
            },
            #[cfg(feature = "backend-rsa")]
            Backend::SoftwareRsa => Err(TrussedError::RequestNotAvailable),
            #[cfg(feature = "backend-staging")]
            Backend::Staging => match extension {
                Extension::Chunked => <StagingBackend as ExtensionImpl<ChunkedExtension>>::extension_request_serialized(
                    &mut self.staging,
                    &mut ctx.core,
                    &mut ctx.backends.staging,
                    request,
                    resources,
                ),
                Extension::WrapKeyToFile => <StagingBackend as ExtensionImpl<WrapKeyToFileExtension>>::extension_request_serialized(
                    &mut self.staging,
                    &mut ctx.core,
                    &mut ctx.backends.staging,
                    request,
                    resources,
                ),
                #[allow(unreachable_patterns)]
                _ => Err(TrussedError::RequestNotAvailable),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    #[cfg(feature = "backend-auth")]
    Auth,
    #[cfg(feature = "backend-rsa")]
    SoftwareRsa,
    #[cfg(feature = "backend-staging")]
    Staging,
}

#[derive(Debug, Clone, Copy)]
pub enum Extension {
    #[cfg(feature = "backend-auth")]
    Auth,
    #[cfg(feature = "backend-staging")]
    Chunked,
    #[cfg(feature = "backend-staging")]
    WrapKeyToFile,
}

impl From<Extension> for u8 {
    fn from(extension: Extension) -> Self {
        match extension {
            #[cfg(feature = "backend-auth")]
            Extension::Auth => 0,
            #[cfg(feature = "backend-staging")]
            Extension::Chunked => 1,
            #[cfg(feature = "backend-staging")]
            Extension::WrapKeyToFile => 2,
        }
    }
}

impl TryFrom<u8> for Extension {
    type Error = TrussedError;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            #[cfg(feature = "backend-auth")]
            0 => Ok(Extension::Auth),
            #[cfg(feature = "backend-staging")]
            1 => Ok(Extension::Chunked),
            #[cfg(feature = "backend-staging")]
            2 => Ok(Extension::WrapKeyToFile),
            _ => Err(TrussedError::InternalError),
        }
    }
}

#[cfg(feature = "backend-auth")]
impl ExtensionId<AuthExtension> for Dispatch {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Auth;
}

#[cfg(feature = "backend-staging")]
impl ExtensionId<ChunkedExtension> for Dispatch {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Chunked;
}

#[cfg(feature = "backend-staging")]
impl ExtensionId<WrapKeyToFileExtension> for Dispatch {
    type Id = Extension;

    const ID: Self::Id = Self::Id::WrapKeyToFile;
}
