#[cfg(not(feature = "se050"))]
use core::marker::PhantomData;

use trussed::{
    api::{Reply, Request},
    error::Error as TrussedError,
    service::ServiceResources,
    types::{Context, Location},
    Platform,
};

#[cfg(any(
    feature = "backend-auth",
    feature = "backend-rsa",
    feature = "backend-staging"
))]
use trussed::{
    api::{reply, request},
    backend::Backend as _,
    serde_extensions::{ExtensionDispatch, ExtensionId, ExtensionImpl},
    Bytes,
};

#[cfg(feature = "se050")]
use embedded_hal::blocking::delay::DelayUs;
#[cfg(feature = "se050")]
use se05x::{se05x::Se05X, t1::I2CForT1};
#[cfg(feature = "se050")]
use trussed_se050_backend::{manage::ManageExtension, Context as Se050Context, Se050Backend};

#[cfg(feature = "backend-auth")]
use trussed_auth::{AuthBackend, AuthContext, AuthExtension, MAX_HW_KEY_LEN};

#[cfg(feature = "backend-rsa")]
use trussed_rsa_alloc::SoftwareRsa;

#[cfg(feature = "backend-staging")]
use trussed_staging::{
    streaming::ChunkedExtension, wrap_key_to_file::WrapKeyToFileExtension, StagingBackend,
    StagingContext,
};

#[cfg(all(feature = "webcrypt", feature = "backend-staging"))]
use trussed_staging::hmacsha256p256::HmacSha256P256Extension;

pub struct Dispatch<T = (), D = ()> {
    #[cfg(feature = "backend-auth")]
    auth: AuthBackend,
    #[cfg(feature = "backend-staging")]
    staging: StagingBackend,
    #[cfg(feature = "se050")]
    se050: Option<trussed_se050_backend::Se050Backend<T, D>>,
    #[cfg(not(feature = "se050"))]
    __: PhantomData<(T, D)>,
}

#[derive(Default)]
pub struct DispatchContext {
    #[cfg(feature = "backend-auth")]
    auth: AuthContext,
    #[cfg(feature = "backend-staging")]
    staging: StagingContext,
    #[cfg(feature = "se050")]
    se050: Se050Context,
}

impl<T: Twi, D: Delay> Dispatch<T, D> {
    pub fn new(
        auth_location: Location,
        #[cfg(feature = "se050")] se050: Option<Se05X<T, D>>,
    ) -> Self {
        #[cfg(not(feature = "backend-auth"))]
        let _ = auth_location;
        Self {
            #[cfg(feature = "backend-auth")]
            auth: AuthBackend::new(auth_location),
            #[cfg(feature = "backend-staging")]
            staging: StagingBackend::new(),
            #[cfg(feature = "se050")]
            se050: se050.map(trussed_se050_backend::Se050Backend::new),
            #[cfg(not(feature = "se050"))]
            __: Default::default(),
        }
    }

    #[cfg(feature = "backend-auth")]
    pub fn with_hw_key(
        auth_location: Location,
        hw_key: Bytes<MAX_HW_KEY_LEN>,
        #[cfg(feature = "se050")] se050: Option<Se05X<T, D>>,
    ) -> Self {
        Self {
            auth: AuthBackend::with_hw_key(auth_location, hw_key),
            #[cfg(feature = "backend-staging")]
            staging: StagingBackend::new(),
            #[cfg(feature = "se050")]
            se050: se050.map(trussed_se050_backend::Se050Backend::new),
            #[cfg(not(feature = "se050"))]
            __: Default::default(),
        }
    }
}

// HACK around #[cfg] for where clauses. See https://users.rust-lang.org/t/cfg-on-where-clause-items/90292

#[cfg(feature = "se050")]
pub trait Twi: I2CForT1 {}
#[cfg(feature = "se050")]
impl<T: I2CForT1> Twi for T {}
#[cfg(feature = "se050")]
pub trait Delay: DelayUs<u32> {}
#[cfg(feature = "se050")]
impl<D: DelayUs<u32>> Delay for D {}

#[cfg(not(feature = "se050"))]
pub trait Twi {}
#[cfg(not(feature = "se050"))]
impl<T> Twi for T {}

#[cfg(not(feature = "se050"))]
pub trait Delay {}
#[cfg(not(feature = "se050"))]
impl<D> Delay for D {}

impl<T: Twi, D: Delay> ExtensionDispatch for Dispatch<T, D> {
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
            #[cfg(feature = "se050")]
            Backend::Se050 => self
                .se050
                .as_mut()
                .ok_or(TrussedError::GeneralError)?
                .request(&mut ctx.core, &mut ctx.backends.se050, request, resources),
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
        #[allow(unreachable_patterns)]
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
                #[cfg(feature = "webcrypt")]
                Extension::HmacShaP256 => <StagingBackend as ExtensionImpl<HmacSha256P256Extension>>::extension_request_serialized(
                    &mut self.staging,
                    &mut ctx.core,
                    &mut ctx.backends.staging,
                    request,
                    resources,
                ),
                #[allow(unreachable_patterns)]
                _ => Err(TrussedError::RequestNotAvailable),
            },
            #[cfg(feature = "se050")]
            Backend::Se050 => {
                match extension {
                    Extension::Se050Manage => <Se050Backend<_,_> as ExtensionImpl<ManageExtension>>::extension_request_serialized(
                        self.se050.as_mut().ok_or(TrussedError::GeneralError)?,
                        &mut ctx.core,
                        &mut ctx.backends.se050,
                        request,
                        resources
                    ),
                    _ => Err(TrussedError::RequestNotAvailable),
                }
            }
            _ => Err(TrussedError::RequestNotAvailable),
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
    #[cfg(feature = "se050")]
    Se050,
}

#[derive(Debug, Clone, Copy)]
pub enum Extension {
    #[cfg(feature = "backend-auth")]
    Auth,
    #[cfg(feature = "backend-staging")]
    Chunked,
    #[cfg(feature = "backend-staging")]
    WrapKeyToFile,
    #[cfg(feature = "backend-staging")]
    HmacShaP256,
    #[cfg(feature = "se050")]
    Se050Manage,
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
            #[cfg(feature = "backend-staging")]
            Extension::HmacShaP256 => 3,
            #[cfg(feature = "se050")]
            Extension::Se050Manage => 4,
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
            #[cfg(feature = "backend-staging")]
            3 => Ok(Extension::HmacShaP256),
            #[cfg(feature = "se050")]
            4 => Ok(Extension::Se050Manage),
            _ => Err(TrussedError::InternalError),
        }
    }
}

#[cfg(feature = "backend-auth")]
impl<T: Twi, D: Delay> ExtensionId<AuthExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Auth;
}

#[cfg(feature = "backend-staging")]
impl<T: Twi, D: Delay> ExtensionId<ChunkedExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Chunked;
}

#[cfg(feature = "backend-staging")]
impl<T: Twi, D: Delay> ExtensionId<WrapKeyToFileExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::WrapKeyToFile;
}

#[cfg(all(feature = "backend-staging", feature = "webcrypt"))]
impl<T: Twi, D: Delay> ExtensionId<HmacSha256P256Extension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::HmacShaP256;
}

#[cfg(feature = "se050")]
impl<T: Twi, D: Delay> ExtensionId<ManageExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Se050Manage;
}
