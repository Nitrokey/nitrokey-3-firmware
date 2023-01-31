use heapless_bytes::Bytes;
use littlefs2::driver::Storage;

use lfs_backup::{BackupBackend, FSBackupError, Result, MAX_DUMP_BLOB_LENGTH};

use crate::soc::types::Soc as SocT;
use crate::types::Soc;

pub struct EFSBackupBackend<'a> {
    extflash: &'a mut <SocT as Soc>::ExternalFlashStorage,
    initial_offset: usize,
    offset: usize,
    len: usize,
}

impl<'a> EFSBackupBackend<'a> {
    pub fn new(
        extflash: &'a mut <SocT as Soc>::ExternalFlashStorage,
        offset: usize,
        len: usize,
    ) -> Self {
        Self {
            extflash,
            initial_offset: offset.clone(),
            offset,
            len,
        }
    }
}

impl<'a> BackupBackend for EFSBackupBackend<'a> {
    // would be good to get this from ext-flash directly
    const RW_SIZE: usize = 256;

    fn write(&mut self, content: &[u8]) -> Result<usize> {
        let len =
            content.len() + ((Self::RW_SIZE - (content.len() % Self::RW_SIZE)) % Self::RW_SIZE);
        let mut data: Bytes<MAX_DUMP_BLOB_LENGTH> = Bytes::from_slice(content).unwrap();

        data.resize(len, 0x00)
            .map_err(|_| FSBackupError::BackendWriteErr)?;

        let count = self
            .extflash
            .write(self.offset, &data)
            .map_err(|_| FSBackupError::BackendWriteErr)?;
        self.offset += data.len();

        //trace_now!("W-addr: {} len: {} datalen: {}", self.offset, content.len(), data.len());
        //trace_now!("W-data: {} = {:?}", data);

        Ok(count)
    }

    fn read<const N: usize>(&mut self, len: usize) -> Result<Bytes<N>> {
        let mut output = Bytes::<N>::default();
        output.resize_default(len).expect("assuming: N > len");

        self.extflash
            .read(self.offset, &mut output)
            .map_err(|_| FSBackupError::BackendReadErr)?;

        self.offset +=
            output.len() + ((Self::RW_SIZE - (output.len() % Self::RW_SIZE)) % Self::RW_SIZE);

        //trace_now!("R-addr: {} len {:}", self.offset, output.len());
        //trace_now!("R-data {:?}", output);

        Ok(output)
    }

    fn erase(&mut self) -> Result<usize> {
        self.extflash
            .erase(self.initial_offset, self.len)
            .map_err(|_| FSBackupError::BackendEraseErr)?;
        self.offset = self.initial_offset.clone();
        Ok(self.len)
    }

    fn reset(&mut self) {
        self.offset = self.initial_offset;
    }
}
