#[cfg(not(feature = "se050"))]
use core::marker::PhantomData;

use trussed::{
    api::{Reply, Request},
    error::Error as TrussedError,
    service::ServiceResources,
    types::{Context, Location},
    Platform,
};

use littlefs2::{path, path::Path};

use if_chain::if_chain;
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
use trussed_se050_backend::{
    manage::ManageExtension as Se050ManageExtension, Context as Se050Context, Se050Backend,
};

#[cfg(feature = "backend-auth")]
use trussed_auth::{AuthBackend, AuthContext, AuthExtension, MAX_HW_KEY_LEN};

#[cfg(feature = "backend-rsa")]
use trussed_rsa_alloc::SoftwareRsa;

use trussed_staging::{
    manage::ManageExtension, streaming::ChunkedExtension, wrap_key_to_file::WrapKeyToFileExtension,
    StagingBackend, StagingContext,
};

#[cfg(feature = "webcrypt")]
use trussed_staging::hmacsha256p256::HmacSha256P256Extension;

pub struct Dispatch<T = (), D = ()> {
    #[cfg(feature = "backend-auth")]
    auth: AuthBackend,
    staging: StagingBackend,
    #[cfg(feature = "se050")]
    se050: Option<Se050Backend<T, D>>,
    #[cfg(not(feature = "se050"))]
    __: PhantomData<(T, D)>,
}

#[derive(Default)]
pub struct DispatchContext {
    #[cfg(feature = "backend-auth")]
    auth: AuthContext,
    staging: StagingContext,
    #[cfg(feature = "se050")]
    se050: Se050Context,
}

fn should_preserve_file(file: &Path) -> bool {
    // We save all "special" objects, with an ID that is representable by a `u8`

    const DIRS: &[&Path] = &[path!("x5c"), path!("ctr"), path!("sec"), path!("pub")];

    let mut components = file.iter();
    if_chain! {
        if components.next() == Some("/".into());
        if  components.next().is_some();
        if let Some(intermediary) = components.next();
        if DIRS.contains(&&*intermediary);
        if let Some(file_name) = components.next();
        if components.next().is_none();
        if file_name.as_ref().len() <=2;
        then {
            true
        } else {
            false
        }
    }
}

fn build_staging_backend() -> StagingBackend {
    let mut backend = StagingBackend::new();
    backend.manage.should_preserve_file = |file, _location| should_preserve_file(file);
    backend
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
            staging: build_staging_backend(),
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
            staging: build_staging_backend(),
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
            Backend::Staging => {
                self.staging
                    .request(&mut ctx.core, &mut ctx.backends.staging, request, resources)
            }
            Backend::StagingManage => Err(TrussedError::RequestNotAvailable),
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
                Extension::WrapKeyToFile => {
                    ExtensionImpl::<WrapKeyToFileExtension>::extension_request_serialized(
                        &mut self.staging,
                        &mut ctx.core,
                        &mut ctx.backends.staging,
                        request,
                        resources,
                    )
                }
                #[cfg(feature = "backend-staging-hmacsha256p256")]
                Extension::HmacShaP256 => {
                    ExtensionImpl::<HmacSha256P256Extension>::extension_request_serialized(
                        &mut self.staging,
                        &mut ctx.core,
                        &mut ctx.backends.staging,
                        request,
                        resources,
                    )
                }
                #[allow(unreachable_patterns)]
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
            #[cfg(feature = "se050")]
            Backend::Se050 => match extension {
                Extension::Se050Manage => ExtensionImpl::<
                    trussed_se050_backend::manage::ManageExtension,
                >::extension_request_serialized(
                    self.se050.as_mut().ok_or(TrussedError::GeneralError)?,
                    &mut ctx.core,
                    &mut ctx.backends.se050,
                    request,
                    resources,
                ),
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
    Staging,
    StagingManage,
    #[cfg(feature = "se050")]
    Se050,
}

#[derive(Debug, Clone, Copy)]
pub enum Extension {
    #[cfg(feature = "backend-auth")]
    Auth,
    Chunked,
    WrapKeyToFile,
    Manage,
    #[cfg(feature = "backend-staging-hmacsha256p256")]
    HmacShaP256,
    #[cfg(feature = "se050")]
    Se050Manage,
}

impl From<Extension> for u8 {
    fn from(extension: Extension) -> Self {
        match extension {
            #[cfg(feature = "backend-auth")]
            Extension::Auth => 0,
            Extension::Chunked => 1,
            Extension::WrapKeyToFile => 2,
            Extension::Manage => 3,
            #[cfg(feature = "backend-staging-hmacsha256p256")]
            Extension::HmacShaP256 => 4,
            #[cfg(feature = "se050")]
            Extension::Se050Manage => 5,
        }
    }
}

impl TryFrom<u8> for Extension {
    type Error = TrussedError;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        match id {
            #[cfg(feature = "backend-auth")]
            0 => Ok(Extension::Auth),
            1 => Ok(Extension::Chunked),
            2 => Ok(Extension::WrapKeyToFile),
            3 => Ok(Extension::Manage),
            #[cfg(feature = "backend-staging-hmacsha256p256")]
            4 => Ok(Extension::HmacShaP256),
            #[cfg(feature = "se050")]
            5 => Ok(Extension::Se050Manage),
            _ => Err(TrussedError::InternalError),
        }
    }
}

#[cfg(feature = "backend-auth")]
impl<T: Twi, D: Delay> ExtensionId<AuthExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Auth;
}

impl<T: Twi, D: Delay> ExtensionId<ChunkedExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Chunked;
}

impl<T: Twi, D: Delay> ExtensionId<WrapKeyToFileExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::WrapKeyToFile;
}

#[cfg(feature = "backend-staging-hmacsha256p256")]
impl<T: Twi, D: Delay> ExtensionId<HmacSha256P256Extension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::HmacShaP256;
}

impl<T: Twi, D: Delay> ExtensionId<ManageExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Manage;
}

#[cfg(feature = "se050")]
impl<T: Twi, D: Delay> ExtensionId<Se050ManageExtension> for Dispatch<T, D> {
    type Id = Extension;

    const ID: Self::Id = Self::Id::Se050Manage;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_preserve() {
        assert!(should_preserve_file(path!("/fido/sec/00")));
        assert!(should_preserve_file(path!("/fido/x5c/00")));
        assert!(should_preserve_file(path!("/fido/sec/01")));
        assert!(should_preserve_file(path!("/fido/x5c/01")));
        assert!(!should_preserve_file(path!("/fido/dat/sec/00")));
    }
}
