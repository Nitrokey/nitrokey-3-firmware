use core::marker::PhantomData;

use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs;
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

#[cfg(feature = "se050-backend-random")]
use embedded_hal::blocking::delay::DelayUs;
#[cfg(feature = "se050-backend-random")]
use se050::{se050::Se050, t1::I2CForT1};

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
    #[cfg(feature = "se050-backend-random")]
    se050: se050_backend_random::BackendRandom<T, D>,
    #[cfg(not(feature = "se050-backend-random"))]
    __: PhantomData<(T, D)>,
}

#[derive(Default)]
pub struct DispatchContext {
    #[cfg(feature = "backend-auth")]
    auth: AuthContext,
    #[cfg(feature = "backend-staging")]
    staging: StagingContext,
}

impl<T: Twi, D: Delay> Dispatch<T, D> {
    pub fn new(
        auth_location: Location,
        #[cfg(feature = "se050-backend-random")] se050: Se050<T, D>,
    ) -> Self {
        #[cfg(not(feature = "backend-auth"))]
        let _ = auth_location;
        Self {
            #[cfg(feature = "backend-auth")]
            auth: AuthBackend::new(auth_location),
            #[cfg(feature = "backend-staging")]
            staging: StagingBackend::new(),
            #[cfg(feature = "se050-backend-random")]
            se050: se050_backend_random::BackendRandom::new(se050),
            #[cfg(not(feature = "se050-backend-random"))]
            __: Default::default(),
        }
    }

    #[cfg(feature = "backend-auth")]
    pub fn with_hw_key(
        auth_location: Location,
        hw_key: Bytes<MAX_HW_KEY_LEN>,
        #[cfg(feature = "se050-backend-random")] se050: Se050<T, D>,
    ) -> Self {
        Self {
            auth: AuthBackend::with_hw_key(auth_location, hw_key),
            #[cfg(feature = "backend-staging")]
            staging: StagingBackend::new(),
            #[cfg(feature = "se050-backend-random")]
            se050: se050_backend_random::BackendRandom::new(se050),
            #[cfg(not(feature = "se050-backend-random"))]
            __: Default::default(),
        }
    }
}

// HACK around #[cfg] for where clauses. See https://users.rust-lang.org/t/cfg-on-where-clause-items/90292

#[cfg(feature = "se050-backend-random")]
pub trait Twi: I2CForT1 {}
#[cfg(feature = "se050-backend-random")]
impl<T: I2CForT1> Twi for T {}
#[cfg(feature = "se050-backend-random")]
pub trait Delay: DelayUs<u32> {}
#[cfg(feature = "se050-backend-random")]
impl<D: DelayUs<u32>> Delay for D {}

#[cfg(not(feature = "se050-backend-random"))]
pub trait Twi {}
#[cfg(not(feature = "se050-backend-random"))]
impl<T> Twi for T {}

#[cfg(not(feature = "se050-backend-random"))]
pub trait Delay {}
#[cfg(not(feature = "se050-backend-random"))]
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
            #[cfg(feature = "se050-backend-random")]
            Backend::Se050 => self
                .se050
                .request(&mut ctx.core, &mut (), request, resources),
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
    #[cfg(feature = "se050-backend-random")]
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
