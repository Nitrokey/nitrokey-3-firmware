#[cfg(not(feature = "se050"))]
use core::marker::PhantomData;

use if_chain::if_chain;
use littlefs2::{path, path::Path};
use trussed::{
    api::{Reply, Request},
    platform::Platform,
    serde_extensions::ExtensionImpl,
    service::ServiceResources,
    types::{CoreContext, Location},
    Error,
};

#[cfg(feature = "se050")]
use embedded_hal::blocking::delay::DelayUs;
#[cfg(feature = "se050")]
use se05x::{se05x::Se05X, t1::I2CForT1};
#[cfg(feature = "se050")]
use trussed_se050_backend::Se050Backend;
#[cfg(feature = "se050")]
use trussed_se050_manage::Se050ManageExtension;

#[cfg(feature = "backend-auth")]
use trussed_auth::{AuthBackend, AuthExtension, MAX_HW_KEY_LEN};

#[cfg(feature = "backend-rsa")]
use trussed_rsa_alloc::SoftwareRsa;

use trussed_chunked::ChunkedExtension;
use trussed_hkdf::{HkdfBackend, HkdfExtension};
use trussed_manage::ManageExtension;
use trussed_staging::StagingBackend;
use trussed_wrap_key_to_file::WrapKeyToFileExtension;

#[cfg(feature = "webcrypt")]
use webcrypt::hmacsha256p256::{Backend as HmacSha256P256Backend, HmacSha256P256Extension};

#[derive(trussed_derive::ExtensionDispatch)]
#[dispatch(backend_id = "Backend", extension_id = "Extension")]
#[extensions(
    Hkdf = "HkdfExtension",
    Chunked = "ChunkedExtension",
    Manage = "ManageExtension",
    WrapKeyToFile = "WrapKeyToFileExtension"
)]
#[cfg_attr(feature = "backend-auth", extensions(Auth = "AuthExtension"))]
#[cfg_attr(feature = "se050", extensions(Se050Manage = "Se050ManageExtension"))]
#[cfg_attr(
    feature = "webcrypt",
    extensions(HmacSha256P256 = "HmacSha256P256Extension")
)]
pub struct Dispatch<T: Twi, D: Delay> {
    #[cfg(feature = "backend-auth")]
    #[extensions("Auth")]
    auth: AuthBackend,

    #[dispatch(no_core)]
    #[extensions("Hkdf")]
    hkdf: HkdfBackend,

    #[cfg(feature = "webcrypt")]
    #[dispatch(no_core)]
    #[extensions("HmacSha256P256")]
    hmac_sha256_p256: HmacSha256P256Backend,

    software_rsa: SoftwareRsa,

    #[extensions("Chunked", "WrapKeyToFile")]
    staging: StagingBackend,

    #[dispatch(delegate_to = "staging", no_core)]
    #[extensions("Manage")]
    staging_manage: (),

    #[cfg(feature = "se050")]
    #[extensions("WrapKeyToFile")]
    #[cfg_attr(feature = "trussed-auth", extensions("Auth"))]
    se050: OptionalBackend<Se050Backend<T, D>>,

    #[cfg(feature = "se050")]
    #[dispatch(delegate_to = "se050", no_core)]
    #[extensions("Manage", "Se050Manage")]
    se050_manage: (),

    #[cfg(not(feature = "se050"))]
    #[dispatch(skip)]
    __: PhantomData<(T, D)>,
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

#[cfg(feature = "se050")]
const NAMESPACE: trussed_se050_backend::namespacing::Namespace = {
    use trussed_se050_backend::namespacing::*;

    Namespace(&[
        NamespaceItem {
            client: path!("admin"),
            value: NamespaceValue::Client1,
        },
        NamespaceItem {
            client: path!("opcard"),
            value: NamespaceValue::Client2,
        },
    ])
};

impl<T: Twi, D: Delay> Dispatch<T, D> {
    pub fn new(
        auth_location: Location,
        #[cfg(feature = "se050")] se050: Option<Se05X<T, D>>,
    ) -> Self {
        #[cfg(not(all(feature = "backend-auth", feature = "se050")))]
        let _ = auth_location;
        Self {
            #[cfg(feature = "backend-auth")]
            auth: AuthBackend::new(auth_location),
            hkdf: HkdfBackend,
            #[cfg(feature = "webcrypt")]
            hmac_sha256_p256: Default::default(),
            software_rsa: SoftwareRsa,
            staging: build_staging_backend(),
            staging_manage: (),
            #[cfg(feature = "se050")]
            se050: se050
                .map(|driver| Se050Backend::new(driver, auth_location, None, NAMESPACE))
                .into(),
            #[cfg(feature = "se050")]
            se050_manage: (),
            #[cfg(not(feature = "se050"))]
            __: Default::default(),
        }
    }

    #[cfg(feature = "backend-auth")]
    pub fn with_hw_key(
        auth_location: Location,
        hw_key: trussed::Bytes<MAX_HW_KEY_LEN>,
        #[cfg(feature = "se050")] se050: Option<Se05X<T, D>>,
    ) -> Self {
        #[cfg(feature = "se050")]
        // Should the backend really use the same key?
        let hw_key_se050 = hw_key.clone();
        Self {
            auth: AuthBackend::with_hw_key(auth_location, hw_key),
            hkdf: HkdfBackend,
            #[cfg(feature = "webcrypt")]
            hmac_sha256_p256: Default::default(),
            software_rsa: SoftwareRsa,
            staging: build_staging_backend(),
            staging_manage: (),
            #[cfg(feature = "se050")]
            se050: se050
                .map(|driver| {
                    Se050Backend::new(driver, auth_location, Some(hw_key_se050), NAMESPACE)
                })
                .into(),
            #[cfg(feature = "se050")]
            se050_manage: (),
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

#[derive(Debug, Clone, Copy)]
pub enum Backend {
    #[cfg(feature = "backend-auth")]
    Auth,
    Hkdf,
    #[cfg(feature = "webcrypt")]
    HmacSha256P256,
    #[cfg(feature = "backend-rsa")]
    SoftwareRsa,
    Staging,
    /// Separate BackendId to prevent non-priviledged apps from accessing the manage Extension
    StagingManage,
    #[cfg(feature = "se050")]
    Se050,
    #[cfg(feature = "se050")]
    /// Separate BackendId to prevent non-priviledged apps from accessing the manage Extension
    Se050Manage,
}

#[derive(Debug, Clone, Copy, trussed_derive::ExtensionId)]
pub enum Extension {
    #[cfg(feature = "backend-auth")]
    Auth = 0,
    Hkdf = 1,
    Chunked = 2,
    WrapKeyToFile = 3,
    Manage = 4,
    #[cfg(feature = "webcrypt")]
    HmacSha256P256 = 5,
    #[cfg(feature = "se050")]
    Se050Manage = 6,
}

pub struct OptionalBackend<T>(Option<T>);

impl<T: trussed::backend::Backend> trussed::backend::Backend for OptionalBackend<T> {
    type Context = T::Context;

    fn request<P: Platform>(
        &mut self,
        core_ctx: &mut CoreContext,
        backend_ctx: &mut Self::Context,
        request: &Request,
        resources: &mut ServiceResources<P>,
    ) -> Result<Reply, Error> {
        if let Some(backend) = self.0.as_mut() {
            backend.request(core_ctx, backend_ctx, request, resources)
        } else {
            Err(Error::RequestNotAvailable)
        }
    }
}

impl<T: ExtensionImpl<E>, E: trussed::serde_extensions::Extension> ExtensionImpl<E>
    for OptionalBackend<T>
{
    fn extension_request<P: Platform>(
        &mut self,
        core_ctx: &mut CoreContext,
        backend_ctx: &mut Self::Context,
        request: &E::Request,
        resources: &mut ServiceResources<P>,
    ) -> Result<E::Reply, Error> {
        if let Some(backend) = self.0.as_mut() {
            backend.extension_request(core_ctx, backend_ctx, request, resources)
        } else {
            Err(Error::RequestNotAvailable)
        }
    }
}

impl<T> Default for OptionalBackend<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> From<T> for OptionalBackend<T> {
    fn from(backend: T) -> Self {
        Self(Some(backend))
    }
}

impl<T> From<Option<T>> for OptionalBackend<T> {
    fn from(backend: Option<T>) -> Self {
        Self(backend)
    }
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
        assert!(should_preserve_file(path!("/attn/pub/00")));
        assert!(should_preserve_file(path!("/attn/sec/01")));
        assert!(should_preserve_file(path!("/attn/sec/02")));
        assert!(should_preserve_file(path!("/attn/sec/03")));
        assert!(should_preserve_file(path!("/attn/x5c/01")));
        assert!(should_preserve_file(path!("/attn/x5c/02")));
        assert!(should_preserve_file(path!("/attn/x5c/03")));
        assert!(!should_preserve_file(path!("/fido/dat/sec/00")));
    }
}
