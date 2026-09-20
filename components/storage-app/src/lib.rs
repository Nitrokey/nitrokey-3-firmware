mod commands;
mod ctaphid;

use littlefs2_core::{Path, path};
use trussed_auth::{AuthClient, Pin, PinId};
use trussed_core::{
    CryptoClient, FilesystemClient, try_syscall,
    types::{EncryptedData, KeyId, Location, Mechanism, Message},
};
use zeroize::Zeroizing;

pub use commands::{Command, UnsupportedCommandError};

delog::generate_macros!();

#[non_exhaustive]
pub enum Error {
    InvalidRequest,
    SerializationFailed,
    InternalError,
    PinTooLong,
    PinTooShort,
    InvalidPin,
    PinAlreadySet,
    PinNotSet,
    PinBlocked,
}

impl From<Error> for u8 {
    fn from(error: Error) -> u8 {
        // 0 may not be returned as it indicates success
        match error {
            Error::InvalidRequest => 1,
            Error::SerializationFailed => 2,
            Error::InternalError => 3,
            Error::PinTooLong => 4,
            Error::PinTooShort => 5,
            Error::InvalidPin => 6,
            Error::PinAlreadySet => 7,
            Error::PinNotSet => 8,
            Error::PinBlocked => 9,
        }
    }
}

enum PinType {
    User,
}

impl From<PinType> for PinId {
    fn from(pin: PinType) -> PinId {
        match pin {
            PinType::User => PinId::from(0),
        }
    }
}

struct PinStatus {
    is_set: bool,
    retries: u8,
}

struct PinKey(KeyId);

impl PinKey {
    const MECHANISM: Mechanism = Mechanism::Chacha8Poly1305;
}

const USER_PIN_RETRIES: u8 = 3;
const MIN_PIN_LENGTH: usize = 6;

pub const ENCRYPTION_KEY_LEN: usize = 32;

struct EncryptionKey(Zeroizing<[u8; ENCRYPTION_KEY_LEN]>);

impl EncryptionKey {
    const AD: &[u8] = b"storage encryption key";
    const PATH: &Path = path!("encryption-key");
    const LOCATION: Location = Location::Internal;
}

impl TryFrom<&[u8]> for EncryptionKey {
    type Error = Error;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        slice
            .try_into()
            .map(|key| Self(Zeroizing::new(key)))
            .map_err(|_| Error::InternalError)
    }
}

pub trait Client: AuthClient + CryptoClient + FilesystemClient {}

impl<T: AuthClient + CryptoClient + FilesystemClient> Client for T {}

/// An encrypted storage that is managed by the [`StorageApp`][].
pub trait Storage {
    /// Initializes the storage with a new encryption key.
    ///
    /// This function should work in any state. Afterwards, the storage should still be locked but
    /// ready to be unlocked.
    fn init(&mut self, key: &[u8; ENCRYPTION_KEY_LEN]) -> Result<(), Error>;

    /// Unlocks the storage using the given encryption key.
    ///
    /// This function may only be called if the storage has been initialized and is currently
    /// locked.
    fn unlock(&mut self, key: &[u8; ENCRYPTION_KEY_LEN]) -> Result<(), Error>;

    /// Locks the storage.
    ///
    /// This function may only be called if the storage has been initialized and is currently
    /// unlocked.
    fn lock(&mut self) -> Result<(), Error>;
}

pub struct StorageApp<C, S> {
    client: C,
    storage: S,
    is_unlocked: bool,
}

fn check_pin(pin: &[u8]) -> Result<Pin, Error> {
    if pin.len() < MIN_PIN_LENGTH {
        return Err(Error::PinTooShort);
    }
    pin.try_into().map_err(|_| Error::PinTooLong)
}

impl<C: Client, S: Storage> StorageApp<C, S> {
    pub fn new(client: C, storage: S) -> Self {
        Self {
            client,
            storage,
            is_unlocked: false,
        }
    }

    fn pin_status(&mut self) -> Result<PinStatus, Error> {
        let is_set = self.is_pin_set()?;
        let mut retries = None;
        if is_set {
            retries = self.pin_retries()?;
        }
        Ok(PinStatus {
            is_set,
            retries: retries.unwrap_or(USER_PIN_RETRIES),
        })
    }

    fn is_pin_set(&mut self) -> Result<bool, Error> {
        try_syscall!(self.client.has_pin(PinType::User))
            .map(|r| r.has_pin)
            .map_err(|_| Error::InternalError)
    }

    fn pin_retries(&mut self) -> Result<Option<u8>, Error> {
        try_syscall!(self.client.pin_retries(PinType::User))
            .map(|r| r.retries)
            .map_err(|_| Error::InternalError)
    }

    fn with_pin_key<F, T>(&mut self, pin: Pin, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Self, PinKey) -> Result<T, Error>,
    {
        let key_id = try_syscall!(self.client.get_pin_key(PinType::User, pin))
            .map_err(|_| Error::InternalError)?
            .result
            .ok_or(Error::InvalidPin)?;
        let result1 = f(self, PinKey(key_id));
        let result2 = try_syscall!(self.client.delete(key_id)).map_err(|_| Error::InternalError);
        result1.and_then(|value| result2.map(|_| value))
    }

    fn set_pin<F, T>(&mut self, pin: &[u8], f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Self, PinKey) -> Result<T, Error>,
    {
        let pin = check_pin(pin)?;
        try_syscall!(
            self.client
                .set_pin(PinType::User, pin.clone(), Some(USER_PIN_RETRIES), true)
        )
        .map_err(|_| Error::InternalError)?;
        self.with_pin_key(pin, f)
    }

    fn change_pin(&mut self, old_pin: &[u8], new_pin: &[u8]) -> Result<(), Error> {
        let old_pin = check_pin(old_pin)?;
        let new_pin = check_pin(new_pin)?;
        let result = try_syscall!(self.client.change_pin(PinType::User, old_pin, new_pin))
            .map_err(|_| Error::InternalError)?;
        if result.success {
            Ok(())
        } else {
            Err(Error::InvalidPin)
        }
    }

    fn check_pin<F, T>(&mut self, pin: &[u8], f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Self, PinKey) -> Result<T, Error>,
    {
        let pin = check_pin(pin)?;
        self.with_pin_key(pin, f)
    }

    fn init_storage(&mut self, key: &EncryptionKey) -> Result<(), Error> {
        self.storage.init(&key.0)?;
        self.is_unlocked = false;
        Ok(())
    }

    fn lock_storage(&mut self) -> Result<(), Error> {
        if self.is_unlocked {
            self.storage.lock()?;
            self.is_unlocked = false;
        }
        Ok(())
    }

    fn unlock_storage(&mut self, key: &EncryptionKey) -> Result<(), Error> {
        if !self.is_unlocked {
            self.storage.unlock(&key.0)?;
            self.is_unlocked = true;
        }
        Ok(())
    }

    fn generate_encryption_key(&mut self) -> Result<EncryptionKey, Error> {
        try_syscall!(self.client.random_bytes(ENCRYPTION_KEY_LEN))
            .map_err(|_| Error::InternalError)?
            .bytes
            .as_ref()
            .try_into()
            .map_err(|_| Error::InternalError)
    }

    fn save_encryption_key(
        &mut self,
        encryption_key: &EncryptionKey,
        pin_key: PinKey,
    ) -> Result<(), Error> {
        let encrypted = try_syscall!(self.client.encrypt(
            PinKey::MECHANISM,
            pin_key.0,
            encryption_key.0.as_ref(),
            EncryptionKey::AD,
            None
        ))
        .map_err(|_| Error::InternalError)?;
        let encrypted = EncryptedData::from(encrypted);
        let mut serialized = Message::new();
        serialized.resize_to_capacity();
        let n = cbor_smol::cbor_serialize_to(&encrypted, serialized.as_mut_slice())
            .map_err(|_| Error::SerializationFailed)?;
        serialized.truncate(n);
        try_syscall!(self.client.write_file(
            EncryptionKey::LOCATION,
            EncryptionKey::PATH.into(),
            serialized,
            None
        ))
        .map_err(|_| Error::InternalError)?;
        Ok(())
    }

    fn load_encryption_key(&mut self, pin_key: PinKey) -> Result<EncryptionKey, Error> {
        let serialized = try_syscall!(
            self.client
                .read_file(EncryptionKey::LOCATION, EncryptionKey::PATH.into())
        )
        .map_err(|_| Error::InternalError)?
        .data;
        let encrypted: EncryptedData =
            cbor_smol::cbor_deserialize(&serialized).map_err(|_| Error::InternalError)?;
        let request = encrypted.decrypt(
            PinKey::MECHANISM,
            pin_key.0,
            EncryptionKey::AD
                .try_into()
                .map_err(|_| Error::InternalError)?,
        );
        let decrypted = try_syscall!(self.client.request(request))
            .map_err(|_| Error::InternalError)?
            .plaintext
            .ok_or(Error::InternalError)?;
        decrypted
            .as_ref()
            .try_into()
            .map_err(|_| Error::InternalError)
    }
}
