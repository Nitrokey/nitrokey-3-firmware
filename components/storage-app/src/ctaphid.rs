use ctaphid_app::{App, Command, Error, VendorCommand};
use heapless_bytes::BytesView;

use crate::{Client, Storage, StorageApp};

const COMMAND_STORAGE: VendorCommand = VendorCommand::H73;

impl<C: Client, S: Storage> App<'_> for StorageApp<C, S> {
    fn commands(&self) -> &'static [Command] {
        &[Command::Vendor(COMMAND_STORAGE)]
    }

    fn call(
        &mut self,
        command: Command,
        request: &[u8],
        response: &mut BytesView,
    ) -> Result<(), Error> {
        if command != Command::Vendor(COMMAND_STORAGE) {
            return Err(Error::InvalidCommand);
        }
        let (&subcommand, data) = request.split_first().ok_or(Error::InvalidLength)?;
        let subcommand = subcommand.try_into().map_err(|_| Error::InvalidCommand)?;

        let n = response.len();
        response.push(0).map_err(|_| Error::InvalidLength)?;
        if let Err(err) = self.exec(subcommand, data, response) {
            response.as_mut_slice()[n] = err.into();
        }
        Ok(())
    }
}
