use heapless_bytes::BytesView;
use serde::{Deserialize, Serialize};

use crate::{Client, Error, Storage, StorageApp};

pub struct UnsupportedCommandError;

#[non_exhaustive]
pub enum Command {
    Status,
    Unlock,
    Lock,
    SetPin,
    ChangePin,
}

impl TryFrom<u8> for Command {
    type Error = UnsupportedCommandError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x00 => Self::Status,
            0x01 => Self::Unlock,
            0x02 => Self::Lock,
            0x03 => Self::SetPin,
            0x04 => Self::ChangePin,
            _ => {
                return Err(UnsupportedCommandError);
            }
        })
    }
}

#[derive(Serialize)]
struct StatusResponse {
    unlocked: bool,
    pin_set: bool,
    pin_retries: u8,
}

#[derive(Deserialize)]
struct UnlockRequest<'a> {
    pin: &'a [u8],
}

#[derive(Deserialize)]
struct SetPinRequest<'a> {
    pin: &'a [u8],
}

#[derive(Deserialize)]
struct ChangePinRequest<'a> {
    old_pin: &'a [u8],
    new_pin: &'a [u8],
}

fn deserialize<'a, T: Deserialize<'a>>(data: &'a [u8]) -> Result<T, Error> {
    cbor_smol::cbor_deserialize(data).map_err(|_| Error::InvalidRequest)
}

fn serialize<T: Serialize>(object: &T, buf: &mut BytesView) -> Result<(), Error> {
    let n = buf.len();
    buf.resize_to_capacity();
    let m = cbor_smol::cbor_serialize_to(object, &mut buf.as_mut_slice()[n..])
        .map_err(|_| Error::SerializationFailed)?;
    buf.truncate(n + m);
    Ok(())
}

impl<C: Client, S: Storage> StorageApp<C, S> {
    pub fn exec(
        &mut self,
        command: Command,
        data: &[u8],
        buf: &mut BytesView,
    ) -> Result<(), Error> {
        match command {
            Command::Status => {
                if !data.is_empty() {
                    return Err(Error::InvalidRequest);
                }
                let response = self.cmd_status()?;
                serialize(&response, buf)
            }
            Command::Unlock => {
                let request = deserialize(data)?;
                self.cmd_unlock(request)
            }
            Command::Lock => {
                if !data.is_empty() {
                    return Err(Error::InvalidRequest);
                }
                self.cmd_lock()
            }
            Command::SetPin => {
                let request = deserialize(data)?;
                self.cmd_set_pin(request)
            }
            Command::ChangePin => {
                let request = deserialize(data)?;
                self.cmd_change_pin(request)
            }
        }
    }

    fn cmd_status(&mut self) -> Result<StatusResponse, Error> {
        let pin_status = self.pin_status()?;
        Ok(StatusResponse {
            unlocked: self.is_unlocked,
            pin_set: pin_status.is_set,
            pin_retries: pin_status.retries,
        })
    }

    fn cmd_unlock(&mut self, request: UnlockRequest<'_>) -> Result<(), Error> {
        if !self.is_pin_set()? {
            return Err(Error::PinNotSet);
        }
        if self.pin_retries()? == Some(0) {
            return Err(Error::PinBlocked);
        }
        self.check_pin(request.pin, |app, pin_key| {
            let encryption_key = app.load_encryption_key(pin_key)?;
            app.unlock_storage(&encryption_key)
        })
    }

    fn cmd_lock(&mut self) -> Result<(), Error> {
        self.lock_storage()
    }

    fn cmd_set_pin(&mut self, request: SetPinRequest<'_>) -> Result<(), Error> {
        if self.is_pin_set()? {
            return Err(Error::PinAlreadySet);
        }
        // TODO: Should we clear the PIN if the encryption key generation failed?
        self.set_pin(request.pin, |app, pin_key| {
            let encryption_key = app.generate_encryption_key()?;
            app.save_encryption_key(&encryption_key, pin_key)?;
            app.init_storage(&encryption_key)
        })
    }

    fn cmd_change_pin(&mut self, request: ChangePinRequest<'_>) -> Result<(), Error> {
        if !self.is_pin_set()? {
            return Err(Error::PinNotSet);
        }
        if self.pin_retries()? == Some(0) {
            return Err(Error::PinBlocked);
        }
        self.change_pin(request.old_pin, request.new_pin)
    }
}
