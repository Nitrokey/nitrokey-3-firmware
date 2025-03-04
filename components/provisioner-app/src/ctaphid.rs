use crate::{Instruction, Provisioner};
use core::convert::TryFrom;
use ctaphid_app::{App, Command, Error, VendorCommand};
use heapless_bytes::Bytes;
use trussed::{client, store::Store};

const COMMAND_PROVISIONER: VendorCommand = VendorCommand::H71;

impl<S, T, const N: usize> App<'_, N> for Provisioner<S, T>
where
    S: Store,
    T: client::CryptoClient,
{
    fn commands(&self) -> &'static [Command] {
        &[Command::Vendor(COMMAND_PROVISIONER)]
    }

    fn call(
        &mut self,
        command: Command,
        request: &[u8],
        response: &mut Bytes<N>,
    ) -> Result<(), Error> {
        if command != Command::Vendor(COMMAND_PROVISIONER) {
            return Err(Error::InvalidCommand);
        }
        if request.is_empty() {
            return Err(Error::InvalidLength);
        }
        Instruction::try_from(request[0])
            .and_then(|instruction| self.handle(instruction, &request[1..], response))
            .map_err(|_| Error::InvalidCommand)
    }
}
