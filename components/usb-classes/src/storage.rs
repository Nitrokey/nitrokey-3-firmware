//! USB mass storage, exposed as a SCSI block device over Bulk Only Transport.

use core::{borrow::BorrowMut, convert::Infallible};

use cipher::{BlockCipher, BlockDecrypt, BlockEncrypt};
use usb_device::bus::{UsbBus, UsbBusAllocator};
use usbd_storage::{
    subclass::{
        scsi::{Scsi, ScsiCommand},
        Command,
    },
    transport::{
        bbb::{BulkOnly, BulkOnlyError},
        TransportError,
    },
};
use xts_mode::Xts128;

#[cfg(feature = "stm32n657")]
pub mod stm32n657_sdmmc;

/// Bytes per logical block. 512 is mostly assumed.
const BLOCK_SIZE_U16: u16 = 512;
pub const BLOCK_SIZE: usize = BLOCK_SIZE_U16 as _;
pub const BLOCK_SIZE_U32: u32 = BLOCK_SIZE_U16 as _;

const MAX_LUN: u8 = 0;

const VENDOR_ID: &[u8; 8] = b"Nitrokey";
const PRODUCT_ID: &[u8; 16] = b"Storage         ";
const PRODUCT_REVISION: &[u8; 4] = b"1.00";

// Commands that usbd-storage does not parse, but that a host expects a fixed
// write-through medium to accept.
const START_STOP_UNIT: u8 = 0x1B;
const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
const SYNCHRONIZE_CACHE_10: u8 = 0x35;

const SENSE_NOT_READY: u8 = 0x02;
const SENSE_MEDIUM_ERROR: u8 = 0x03;
const SENSE_ILLEGAL_REQUEST: u8 = 0x05;
const ASC_UNRECOVERED_READ_ERROR: u8 = 0x11;
const ASC_WRITE_FAULT: u8 = 0x03;
const ASC_INVALID_COMMAND: u8 = 0x20;
const ASC_LBA_OUT_OF_RANGE: u8 = 0x21;
const ASC_MEDIUM_NOT_PRESENT: u8 = 0x3A;

pub type StorageClass<'bus, B, Buf> = Scsi<BulkOnly<'bus, B, Buf>>;

/// A fixed-size block device backing the SCSI logical unit.
pub trait BlockDevice {
    type Error: core::fmt::Debug;

    /// Number of addressable blocks of [`BLOCK_SIZE`] bytes.
    fn blocks(&self) -> u32;

    fn read_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error>;
    fn write_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error>;
}

pub struct MemoryBlockDevice<'a, const N: usize>(&'a mut [u8; N]);

impl<'a, const N: usize> MemoryBlockDevice<'a, N> {
    pub fn new(buffer: &'a mut [u8; N]) -> Self {
        const {
            assert!(N.is_multiple_of(BLOCK_SIZE));
            let block_count = N / BLOCK_SIZE;
            assert!(block_count <= (u32::MAX as usize));
            assert!((block_count as u32) <= u32::MAX);
        }
        Self(buffer)
    }
}

impl<const N: usize> BlockDevice for MemoryBlockDevice<'_, N> {
    type Error = Infallible;

    fn blocks(&self) -> u32 {
        const { (N / BLOCK_SIZE) as _ }
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        let offset = lba as usize * BLOCK_SIZE;
        buf.copy_from_slice(&self.0[offset..][..BLOCK_SIZE]);
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        let offset = lba as usize * BLOCK_SIZE;
        self.0[offset..][..BLOCK_SIZE].copy_from_slice(buf);
        Ok(())
    }
}

pub struct EncryptedBlockDevice<'a, B, C: BlockCipher + BlockEncrypt + BlockDecrypt> {
    block_device: &'a mut B,
    xts: &'a Xts128<C>,
}

impl<'a, B: BlockDevice, C: BlockCipher + BlockEncrypt + BlockDecrypt>
    EncryptedBlockDevice<'a, B, C>
{
    pub fn new(block_device: &'a mut B, xts: &'a Xts128<C>) -> Self {
        Self { block_device, xts }
    }
}

impl<'a, B, C> BlockDevice for EncryptedBlockDevice<'a, B, C>
where
    B: BlockDevice,
    C: BlockCipher + BlockEncrypt + BlockDecrypt,
{
    type Error = B::Error;

    fn blocks(&self) -> u32 {
        self.block_device.blocks()
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        self.block_device.read_block(lba, buf)?;
        let tweak = xts_mode::get_tweak_default(lba.into());
        self.xts.decrypt_sector(buf, tweak);
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        let tweak = xts_mode::get_tweak_default(lba.into());
        self.xts.encrypt_sector(buf, tweak);
        self.block_device.write_block(lba, buf)
    }
}

/// Per-transfer state. A single SCSI read or write is spread across several
/// `poll_command` calls, so the progress within it has to be carried over.
pub struct State {
    /// Bytes transferred so far within the current command.
    offset: usize,
    /// The block currently being streamed.
    block: [u8; BLOCK_SIZE],
    sense_key: Option<u8>,
    sense_key_code: Option<u8>,
    sense_qualifier: Option<u8>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset: 0,
            block: [0; BLOCK_SIZE],
            sense_key: None,
            sense_key_code: None,
            sense_qualifier: None,
        }
    }
}

impl State {
    pub fn reset(&mut self) {
        self.offset = 0;
        self.sense_key = None;
        self.sense_key_code = None;
        self.sense_qualifier = None;
    }

    fn fail_with(&mut self, sense_key: u8, asc: u8) {
        self.sense_key = Some(sense_key);
        self.sense_key_code = Some(asc);
        self.sense_qualifier = Some(0x00);
    }
}

/// Allocates the mass storage class.
///
/// `packet_size` must be 512 when the device enumerates at high speed.
pub fn setup<B: UsbBus, Buf: BorrowMut<[u8]>>(
    bus: &UsbBusAllocator<B>,
    packet_size: u16,
    buf: Buf,
) -> StorageClass<'_, B, Buf> {
    Scsi::new(bus, packet_size, MAX_LUN, buf).expect("failed to allocate USB mass storage class")
}

/// True if `[lba, lba + count)` lies within the device.
fn in_bounds<D: BlockDevice>(device: &D, lba: u32, count: u32) -> bool {
    lba.checked_add(count)
        .is_some_and(|end| end <= device.blocks())
}

/// Handles one SCSI command against `device`.
pub fn process_command<B, Buf, D>(
    mut command: Command<ScsiCommand, StorageClass<'_, B, Buf>>,
    device: Option<&mut D>,
    state: &mut State,
) -> Result<(), TransportError<BulkOnlyError>>
where
    B: UsbBus,
    Buf: BorrowMut<[u8]>,
    D: BlockDevice,
{
    debug!("storage: {:?}", command.kind);

    // A host can probe more LUNs; answer "no device on this LU" for any != 0.
    if command.lun != 0 {
        match command.kind {
            ScsiCommand::Inquiry { .. } => {
                let mut data = [0u8; 36];
                data[0] = 0x7F; // PQ 0b011, device type 0x1F: not connected
                data[4] = 0x20; // remaining length
                command.try_write_data_all(&data)?;
                command.pass(data.len() as u32);
            }
            _ => {
                state.fail_with(SENSE_ILLEGAL_REQUEST, ASC_LBA_OUT_OF_RANGE);
                command.fail(0);
            }
        }
        return Ok(());
    }

    // Inquery and RequestSense are always executed even if the device is not available
    match command.kind {
        ScsiCommand::Inquiry { .. } => {
            let mut data = [0u8; 36];
            data[0] = 0x00; // direct access block device
            if device.is_none() {
                data[0] |= 0b0010_0000; // device currently not available
            }
            data[1] = 0x80; // removable
            data[2] = 0x04; // SPC-2
            data[3] = 0x02; // response data format
            data[4] = 0x20; // 36 bytes total
            data[8..16].copy_from_slice(VENDOR_ID);
            data[16..32].copy_from_slice(PRODUCT_ID);
            data[32..36].copy_from_slice(PRODUCT_REVISION);
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
            return Ok(());
        }
        ScsiCommand::RequestSense { .. } => {
            let mut data = [0u8; 18];
            data[0] = 0x70; // current errors
            data[2] = state.sense_key.unwrap_or(0);
            data[12] = state.sense_key_code.unwrap_or(0);
            data[13] = state.sense_qualifier.unwrap_or(0);
            command.try_write_data_all(&data)?;
            state.reset();
            command.pass(data.len() as u32);
            return Ok(());
        }
        _ => {}
    }

    // all other commands fail if the device is not unlocked
    let Some(device) = device else {
        state.fail_with(SENSE_NOT_READY, ASC_MEDIUM_NOT_PRESENT);
        command.fail(0);
        return Ok(());
    };

    let blocks = device.blocks();

    match command.kind {
        // these commands are handled above
        ScsiCommand::Inquiry { .. } | ScsiCommand::RequestSense { .. } => unreachable!(),
        ScsiCommand::TestUnitReady => {
            command.pass(0);
        }
        ScsiCommand::ReadCapacity10 => {
            let mut data = [0u8; 8];
            // Last addressable block, not the count.
            data[0..4].copy_from_slice(&u32::to_be_bytes(blocks - 1));
            data[4..8].copy_from_slice(&u32::to_be_bytes(BLOCK_SIZE_U32));
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        ScsiCommand::ReadCapacity16 { .. } => {
            let mut data = [0u8; 16];
            data[0..8].copy_from_slice(&u64::to_be_bytes((blocks - 1) as u64));
            data[8..12].copy_from_slice(&u32::to_be_bytes(BLOCK_SIZE_U32));
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        ScsiCommand::ReadFormatCapacities { .. } => {
            let mut data = [0u8; 12];
            data[3] = 0x08; // capacity list length
            data[4..8].copy_from_slice(&u32::to_be_bytes(blocks));
            data[8] = 0x02; // formatted media
            data[9..12].copy_from_slice(&u32::to_be_bytes(BLOCK_SIZE_U32)[1..]);
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        ScsiCommand::Read { lba, len } => {
            let total = len as usize * BLOCK_SIZE;

            if state.offset == total {
                command.pass(state.offset as u32);
                state.offset = 0;
                return Ok(());
            }

            if !in_bounds(device, lba, len as u32) {
                warn!("storage: read past end of device at lba {}", lba);
                state.fail_with(SENSE_ILLEGAL_REQUEST, ASC_LBA_OUT_OF_RANGE);
                command.fail(0);
                state.offset = 0;
                return Ok(());
            }

            // Streamed one block at a time so the staging buffer stays bounded;
            // the transport reassembles the packets into one host transfer.
            loop {
                if state.offset == total {
                    break;
                }

                let block_offset = state.offset % BLOCK_SIZE;
                if block_offset == 0 {
                    let block = lba + (state.offset / BLOCK_SIZE) as u32;
                    if let Err(_err) = device.read_block(block, &mut state.block) {
                        warn!("storage: read failed at block {} with {_err:?}", block);
                        state.fail_with(SENSE_MEDIUM_ERROR, ASC_UNRECOVERED_READ_ERROR);
                        command.fail(0);
                        state.offset = 0;
                        return Ok(());
                    }
                }

                let count = command.write_data(&state.block[block_offset..])?;
                state.offset += count;
                if count == 0 {
                    break;
                }
            }

            if state.offset == total {
                command.pass(state.offset as u32);
                state.offset = 0;
            }
        }
        ScsiCommand::Write { lba, len } => {
            let total = len as usize * BLOCK_SIZE;

            if state.offset == total {
                command.pass(state.offset as u32);
                state.offset = 0;
                return Ok(());
            }

            if !in_bounds(device, lba, len as u32) {
                warn!("storage: write past end of device at lba {}", lba);
                state.fail_with(SENSE_ILLEGAL_REQUEST, ASC_LBA_OUT_OF_RANGE);
                command.fail(0);
                state.offset = 0;
                return Ok(());
            }

            // Read at most up to the end of the block currently being filled,
            // flushing each block as it completes. Reading past a block
            // boundary in one call would desynchronise the transport, which
            // then rejects further reads with InvalidState.
            loop {
                if state.offset == total {
                    break;
                }

                let block_offset = state.offset % BLOCK_SIZE;
                let count = command.read_data(&mut state.block[block_offset..])?;
                state.offset += count;

                if count > 0 && state.offset.is_multiple_of(BLOCK_SIZE) {
                    let block = lba + (state.offset / BLOCK_SIZE) as u32 - 1;
                    if device.write_block(block, &mut state.block).is_err() {
                        warn!("storage: write failed at block {}", block);
                        state.fail_with(SENSE_MEDIUM_ERROR, ASC_WRITE_FAULT);
                        command.fail(0);
                        state.offset = 0;
                        return Ok(());
                    }
                } else {
                    break;
                }
            }

            if state.offset == total {
                command.pass(state.offset as u32);
                state.offset = 0;
            }
        }
        ScsiCommand::ModeSense6 { .. } => {
            let data = [
                0x03, // number of bytes that follow
                0x00, // SBC media type
                0x00, // not write-protected, no cache control
                0x00, // no block descriptors
            ];
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        ScsiCommand::ModeSense10 { .. } => {
            let data = [0x00, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        // Every write is flushed to the device, so there is nothing to
        // synchronise, and the medium can neither be locked nor spun down.
        ScsiCommand::Unknown {
            cmd: START_STOP_UNIT | PREVENT_ALLOW_MEDIUM_REMOVAL | SYNCHRONIZE_CACHE_10,
        } => {
            command.pass(0);
        }
        ref _unknown => {
            warn!("storage: unhandled SCSI command: {:?}", _unknown);
            state.fail_with(SENSE_ILLEGAL_REQUEST, ASC_INVALID_COMMAND);
            command.fail(0);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use aes::{cipher::KeyInit as _, Aes128};

    use super::*;

    const BLOCKS: usize = 4;
    const BUFFER_LEN: usize = BLOCKS * BLOCK_SIZE;
    const KEY1: &[u8; 32] = b"0123456789abcdef0123456789abcdef";
    const KEY2: &[u8; 32] = b"deadbeefdeadbeefdeadbeefdeadbeef";

    struct Dummy(u32);

    impl BlockDevice for Dummy {
        type Error = ();

        fn blocks(&self) -> u32 {
            self.0
        }

        fn read_block(&mut self, _lba: u32, _buf: &mut [u8; BLOCK_SIZE]) -> Result<(), ()> {
            Ok(())
        }

        fn write_block(&mut self, _lba: u32, _buf: &mut [u8; BLOCK_SIZE]) -> Result<(), ()> {
            Ok(())
        }
    }

    fn block(fill: u8) -> [u8; BLOCK_SIZE] {
        [fill; _]
    }

    fn raw_block(buffer: &[u8], lba: usize) -> [u8; BLOCK_SIZE] {
        let offset = lba * BLOCK_SIZE;
        *buffer[offset..].first_chunk().unwrap()
    }

    fn setup_xts(key: &[u8; 32]) -> Xts128<Aes128> {
        let cipher1 = Aes128::new_from_slice(&key[..16]).unwrap();
        let cipher2 = Aes128::new_from_slice(&key[16..]).unwrap();
        Xts128::new(cipher1, cipher2)
    }

    #[test]
    fn bounds() {
        let device = Dummy(8);
        assert!(in_bounds(&device, 0, 8));
        assert!(in_bounds(&device, 7, 1));
        assert!(in_bounds(&device, 8, 0));
        assert!(!in_bounds(&device, 7, 2));
        assert!(!in_bounds(&device, 9, 1));
        assert!(!in_bounds(&device, u32::MAX, 1));
    }

    #[test]
    fn memory_round_trip() {
        let mut buffer = [0; BUFFER_LEN];
        let mut device = MemoryBlockDevice::new(&mut buffer);
        assert_eq!(device.blocks(), u32::try_from(BLOCKS).unwrap());

        device.write_block(2, &mut block(0xA5)).unwrap();

        let mut buf = block(0);
        device.read_block(2, &mut buf).unwrap();
        assert_eq!(buf, block(0xA5));

        device.read_block(1, &mut buf).unwrap();
        assert_eq!(buf, block(0));
    }

    #[test]
    fn encrypted_round_trip() {
        let xts = setup_xts(KEY1);
        let mut buffer = [0; BUFFER_LEN];

        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &xts);
        encrypted_device.write_block(1, &mut block(0xC3)).unwrap();

        assert_ne!(raw_block(&buffer, 1), block(0xC3));

        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &xts);
        let mut buf = block(0);
        encrypted_device.read_block(1, &mut buf).unwrap();
        assert_eq!(buf, block(0xC3));
    }

    #[test]
    fn tweak_differs_per_block() {
        let xts = setup_xts(KEY1);
        let mut buffer = [0; BUFFER_LEN];

        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &xts);

        encrypted_device.write_block(0, &mut block(0xFF)).unwrap();
        encrypted_device.write_block(1, &mut block(0xFF)).unwrap();

        assert_ne!(raw_block(&buffer, 0), raw_block(&buffer, 1));
    }

    #[test]
    fn encrypted_different_keys() {
        let mut buffer = [0; BUFFER_LEN];

        let xts = setup_xts(KEY1);
        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &xts);

        encrypted_device.write_block(0, &mut block(0xFF)).unwrap();

        let xts = setup_xts(KEY2);
        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &xts);

        let mut buf = block(0);
        encrypted_device.read_block(0, &mut buf).unwrap();
        assert_ne!(buf, block(0x0));
        assert_ne!(buf, block(0xFF));
    }
}
