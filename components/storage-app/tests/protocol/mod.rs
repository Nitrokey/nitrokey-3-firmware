use std::collections::BTreeMap;

use ciborium::Value;
use storage_app::{Client, Command, Error, Storage, StorageApp};
use trussed_core::types::Message;

fn exec<C: Client, S: Storage>(
    app: &mut StorageApp<C, S>,
    command: Command,
    request: Option<Vec<(&str, Value)>>,
) -> Result<Option<BTreeMap<String, Value>>, Error> {
    let request = request
        .map(|request| {
            let value = Value::Map(
                request
                    .into_iter()
                    .map(|(key, value)| (Value::from(key), value))
                    .collect(),
            );
            let mut v = Vec::new();
            ciborium::into_writer(&value, &mut v).unwrap();
            v
        })
        .unwrap_or_default();
    let mut response = Message::new();
    app.exec(command, &request, &mut response)?;
    if response.is_empty() {
        Ok(None)
    } else {
        let value: Value = ciborium::from_reader(response.as_slice()).unwrap();
        let items = value.into_map().unwrap();
        let map = items
            .into_iter()
            .map(|(key, value)| (key.into_text().unwrap(), value))
            .collect();
        Ok(Some(map))
    }
}

#[derive(Debug, PartialEq)]
pub struct Status {
    pub unlocked: bool,
    pub pin_set: bool,
    pub pin_retries: u8,
}

pub fn status<C: Client, S: Storage>(app: &mut StorageApp<C, S>) -> Result<Status, Error> {
    let mut response = exec(app, Command::Status, None)?.unwrap();
    let status = Status {
        unlocked: response.remove("unlocked").unwrap().into_bool().unwrap(),
        pin_set: response.remove("pin_set").unwrap().into_bool().unwrap(),
        pin_retries: response
            .remove("pin_retries")
            .unwrap()
            .into_integer()
            .unwrap()
            .try_into()
            .unwrap(),
    };
    assert!(response.is_empty(), "{response:?}");
    Ok(status)
}

pub fn set_pin<C: Client, S: Storage>(app: &mut StorageApp<C, S>, pin: &[u8]) -> Result<(), Error> {
    let response = exec(app, Command::SetPin, Some(vec![("pin", pin.into())]))?;
    assert_eq!(response, None);
    Ok(())
}

pub fn change_pin<C: Client, S: Storage>(
    app: &mut StorageApp<C, S>,
    old_pin: &[u8],
    new_pin: &[u8],
) -> Result<(), Error> {
    let response = exec(
        app,
        Command::ChangePin,
        Some(vec![
            ("old_pin", old_pin.into()),
            ("new_pin", new_pin.into()),
        ]),
    )?;
    assert_eq!(response, None);
    Ok(())
}

pub fn unlock<C: Client, S: Storage>(app: &mut StorageApp<C, S>, pin: &[u8]) -> Result<(), Error> {
    let response = exec(app, Command::Unlock, Some(vec![("pin", pin.into())]))?;
    assert_eq!(response, None);
    Ok(())
}

pub fn lock<C: Client, S: Storage>(app: &mut StorageApp<C, S>) -> Result<(), Error> {
    let response = exec(app, Command::Lock, None)?;
    assert_eq!(response, None);
    Ok(())
}
