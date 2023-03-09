#![no_std]

#[macro_use]
extern crate delog;
generate_macros!();

use ctaphid_dispatch::{
    app::App,
    command::{Command as CtaphidCommand, VendorCommand},
    types::{AppResult, Error, Message},
};
use trussed::types::LfsStorage;
use usbd_ctaphid::constants::MESSAGE_SIZE;

const COMMAND_DEBUG: VendorCommand = VendorCommand::H73;

#[derive(Debug)]
pub enum Command {
    GetSize,
    Read,
}

impl TryFrom<u8> for Command {
    type Error = Error;

    fn try_from(command: u8) -> Result<Self, Self::Error> {
        match command {
            0 => Ok(Self::GetSize),
            1 => Ok(Self::Read),
            _ => Err(Error::InvalidCommand),
        }
    }
}

pub struct DebugApp<S> {
    storage: S,
}

impl<S: LfsStorage> DebugApp<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    fn handle(&mut self, command: Command, request: &[u8], response: &mut Message) -> AppResult {
        match command {
            Command::GetSize => {
                let size = u32::try_from(S::BLOCK_COUNT * S::BLOCK_SIZE).unwrap();
                info!("GetSize: {:#010x}", size);
                response.extend_from_slice(&size.to_be_bytes()).unwrap();
            },
            Command::Read => {
                let size = S::BLOCK_COUNT * S::BLOCK_SIZE;
                let offset: [u8; 4] = request.try_into().map_err(|_| Error::InvalidLength)?;
                let offset = usize::try_from(u32::from_be_bytes(offset)).unwrap();
                let n = response.capacity().min(MESSAGE_SIZE).min(size - offset);
                if n % S::READ_SIZE != 0 {
                    return Err(Error::InvalidLength);
                }
                response.resize_default(n).unwrap();
                info!("Read({:#010x}) ({:#010x})", offset, n);
                self.storage.read(offset, response).map_err(|_err| {
                    error!("Failed to read from storage: {:?}", _err);
                    Error::InvalidCommand
                })?;
            },
        }
        Ok(())
    }
}

impl<S: LfsStorage> App for DebugApp<S> {
    fn commands(&self) -> &'static [CtaphidCommand] {
        &[CtaphidCommand::Vendor(COMMAND_DEBUG)]
    }

    fn call(
        &mut self,
        command: CtaphidCommand,
        request: &Message,
        response: &mut Message,
    ) -> AppResult {
        if command != CtaphidCommand::Vendor(COMMAND_DEBUG) {
            error!("unsupported command: {:?}", command);
            return Err(Error::InvalidCommand);
        }
        if request.is_empty() {
            error!("missing request");
            return Err(Error::InvalidLength);
        }
        let command = Command::try_from(request[0])?;
        info!("executing command {:?}", command);
        self.handle(command, &request[1..], response)
    }
}
