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

#[derive(Debug)]
pub struct Dispatch {
    #[cfg(feature = "backend-auth")]
    auth: AuthBackend,
}

#[derive(Debug, Default)]
pub struct DispatchContext {
    #[cfg(feature = "backend-auth")]
    auth: AuthContext,
}

impl Dispatch {
    pub fn new(auth_location: Location) -> Self {
        #[cfg(not(feature = "backend-auth"))]
        let _ = auth_location;
        Self {
            #[cfg(feature = "backend-auth")]
            auth: AuthBackend::new(auth_location),
        }
    }
    #[cfg(feature = "backend-auth")]
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
            #[cfg(feature = "backend-auth")]
            Backend::Auth => {
                self.auth
                    .request(&mut ctx.core, &mut ctx.backends.auth, request, resources)
            }
            #[cfg(feature = "backend-rsa")]
            Backend::SoftwareRsa => SoftwareRsa.request(&mut ctx.core, &mut (), request, resources),
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
            },
            #[cfg(feature = "backend-rsa")]
            Backend::SoftwareRsa => Err(TrussedError::RequestNotAvailable),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    #[cfg(feature = "backend-auth")]
    Auth,
    #[cfg(feature = "backend-rsa")]
    SoftwareRsa,
}

#[derive(Debug, Clone, Copy)]
pub enum Extension {
    #[cfg(feature = "backend-auth")]
    Auth,
}

impl From<Extension> for u8 {
    fn from(extension: Extension) -> Self {
        match extension {
            #[cfg(feature = "backend-auth")]
            Extension::Auth => 0,
        }
    }
}

impl TryFrom<u8> for Extension {
    type Error = TrussedError;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            #[cfg(feature = "backend-auth")]
            0 => Ok(Extension::Auth),
            _ => Err(TrussedError::InternalError),
        }
    }
}

#[cfg(feature = "backend-auth")]
impl ExtensionId<AuthExtension> for Dispatch {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Auth;
}
