use std::{cell::RefCell, rc::Rc};

use storage_app::{Error, Storage, StorageApp};
use trussed::{
    backend::BackendId,
    virt::{self, Client, StoreConfig},
};
use trussed_auth::AuthExtension;
use trussed_auth_backend::{AuthBackend, FilesystemLayout};
use trussed_core::types::Location;
use trussed_derive::{ExtensionDispatch, ExtensionId};

pub enum Backend {
    Auth,
}

#[derive(ExtensionId)]
pub enum Extension {
    Auth = 0,
}

#[derive(ExtensionDispatch)]
#[dispatch(backend_id = "Backend", extension_id = "Extension")]
#[extensions(Auth = "AuthExtension")]
pub struct Dispatch {
    #[dispatch(no_core)]
    #[extensions("Auth")]
    auth: AuthBackend,
}

impl Dispatch {
    fn new() -> Self {
        Self {
            auth: AuthBackend::new(Location::Internal, FilesystemLayout::V1),
        }
    }
}

#[derive(Default)]
pub struct StorageData {
    pub init_count: u8,
    pub lock_count: u8,
    pub unlock_count: u8,
    pub is_unlocked: bool,
}

pub struct DummyStorage {
    data: Rc<RefCell<StorageData>>,
    key: Option<[u8; 32]>,
}

impl Storage for DummyStorage {
    fn init(&mut self, key: &[u8; 32]) -> Result<(), Error> {
        let mut data = self.data.borrow_mut();
        data.init_count += 1;
        data.is_unlocked = false;
        self.key = Some(*key);
        Ok(())
    }

    fn unlock(&mut self, key: &[u8; 32]) -> Result<(), Error> {
        let mut data = self.data.borrow_mut();
        if data.is_unlocked || self.key.as_ref() != Some(key) {
            return Err(Error::InternalError);
        }
        data.unlock_count += 1;
        data.is_unlocked = true;
        Ok(())
    }

    fn lock(&mut self) -> Result<(), Error> {
        let mut data = self.data.borrow_mut();
        if self.key.is_none() || !data.is_unlocked {
            return Err(Error::InternalError);
        }
        data.lock_count += 1;
        data.is_unlocked = false;
        Ok(())
    }
}

pub type App<'a> = StorageApp<Client<'a, Dispatch>, DummyStorage>;

pub fn with_app<F: FnOnce(&mut App<'_>, Rc<RefCell<StorageData>>)>(f: F) {
    virt::with_platform(StoreConfig::ram(), |platform| {
        platform.run_client_with_backends(
            "storage",
            Dispatch::new(),
            &[BackendId::Custom(Backend::Auth), BackendId::Core],
            |client| {
                let data = Rc::new(RefCell::new(StorageData::default()));
                let storage = DummyStorage {
                    data: data.clone(),
                    key: None,
                };
                let mut app = App::new(client, storage);
                f(&mut app, data)
            },
        )
    })
}
