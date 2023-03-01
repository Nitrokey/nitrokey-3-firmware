use trussed::{
    api::{reply, request, Reply, Request},
    backend::BackendId,
    error::Error,
    platform::Platform,
    serde_extensions::ExtensionDispatch,
    service::ServiceResources,
    types,
};
#[cfg(feature = "backend-auth")]
use trussed::{backend::Backend as _, serde_extensions::ExtensionImpl as _};
#[cfg(feature = "backend-auth")]
use trussed_auth::{AuthBackend, AuthContext, MAX_HW_KEY_LEN};

#[derive(Debug)]
pub struct Dispatch {
    #[cfg(feature = "backend-auth")]
    auth: AuthBackend,
}

impl Dispatch {
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(feature = "backend-auth")]
    pub fn with_hw_key(hw_key: types::Bytes<MAX_HW_KEY_LEN>) -> Self {
        Self {
            auth: AuthBackend::with_hw_key(trussed::types::Location::Internal, hw_key),
        }
    }
}

impl Default for Dispatch {
    fn default() -> Self {
        Self {
            #[cfg(feature = "backend-auth")]
            auth: AuthBackend::new(trussed::types::Location::Internal),
        }
    }
}

impl ExtensionDispatch for Dispatch {
    type Context = Context;
    type BackendId = Backend;
    type ExtensionId = Extension;

    fn core_request<P: Platform>(
        &mut self,
        backend: &Self::BackendId,
        ctx: &mut types::Context<Self::Context>,
        request: &Request,
        resources: &mut ServiceResources<P>,
    ) -> Result<Reply, Error> {
        let _ = (backend, &ctx, request, &resources);
        match backend {
            #[cfg(feature = "backend-auth")]
            Backend::Auth => {
                self.auth
                    .request(&mut ctx.core, &mut ctx.backends.auth, request, resources)
            }
            #[allow(unused)]
            _ => Err(Error::RequestNotAvailable),
        }
    }

    fn extension_request<P: Platform>(
        &mut self,
        backend: &Self::BackendId,
        extension: &Self::ExtensionId,
        ctx: &mut types::Context<Self::Context>,
        request: &request::SerdeExtension,
        resources: &mut ServiceResources<P>,
    ) -> Result<reply::SerdeExtension, Error> {
        let _ = (backend, extension, &ctx, request, &resources);
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
            #[allow(unused)]
            _ => Err(Error::RequestNotAvailable),
        }
    }
}

#[derive(Default)]
pub struct Context {
    #[cfg(feature = "backend-auth")]
    auth: AuthContext,
}

pub enum Backend {
    #[cfg(feature = "backend-auth")]
    Auth,
}

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
    type Error = Error;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            #[cfg(feature = "backend-auth")]
            0 => Ok(Self::Auth),
            _ => Err(Error::InternalError),
        }
    }
}

#[allow(unused)]
pub const BACKENDS_DEFAULT: &[BackendId<Backend>] = &[];
#[cfg(feature = "backend-auth")]
pub const BACKENDS_AUTH: &[BackendId<Backend>] =
    &[BackendId::Custom(Backend::Auth), BackendId::Core];
