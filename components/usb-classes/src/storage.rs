//! USB mass storage, exposed as a SCSI block device over Bulk Only Transport.

use core::borrow::BorrowMut;

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

#[cfg(feature = "std")]
pub mod host;

/// Bytes per logical block. 512 is mostly assumed.
pub const BLOCK_SIZE: u32 = 512;

const MAX_LUN: u8 = 0;

const VENDOR_ID: &[u8; 8] = b"Nitrokey";
const PRODUCT_ID: &[u8; 16] = b"Storage         ";
const PRODUCT_REVISION: &[u8; 4] = b"1.00";

// Commands that usbd-storage does not parse, but that a host expects a fixed
// write-through medium to accept.
const START_STOP_UNIT: u8 = 0x1B;
const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
const SYNCHRONIZE_CACHE_10: u8 = 0x35;

const SENSE_MEDIUM_ERROR: u8 = 0x03;
const SENSE_ILLEGAL_REQUEST: u8 = 0x05;
const ASC_UNRECOVERED_READ_ERROR: u8 = 0x11;
const ASC_WRITE_FAULT: u8 = 0x03;
const ASC_INVALID_COMMAND: u8 = 0x20;
const ASC_LBA_OUT_OF_RANGE: u8 = 0x21;

pub type StorageClass<'bus, B, Buf> = Scsi<BulkOnly<'bus, B, Buf>>;

/// A fixed-size block device backing the SCSI logical unit.
pub trait BlockDevice {
    type Error;

    /// Number of addressable blocks of [`BLOCK_SIZE`] bytes.
    fn blocks(&self) -> u32;

    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write_block(&mut self, lba: u32, buf: &[u8]) -> Result<(), Self::Error>;
}

/// Per-transfer state. A single SCSI read or write is spread across several
/// `poll_command` calls, so the progress within it has to be carried over.
pub struct State {
    /// Bytes transferred so far within the current command.
    offset: usize,
    /// The block currently being streamed.
    block: [u8; BLOCK_SIZE as usize],
    sense_key: Option<u8>,
    sense_key_code: Option<u8>,
    sense_qualifier: Option<u8>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset: 0,
            block: [0; BLOCK_SIZE as usize],
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy(u32);

    impl BlockDevice for Dummy {
        type Error = ();

        fn blocks(&self) -> u32 {
            self.0
        }

        fn read_block(&mut self, _lba: u32, _buf: &mut [u8]) -> Result<(), ()> {
            Ok(())
        }

        fn write_block(&mut self, _lba: u32, _buf: &[u8]) -> Result<(), ()> {
            Ok(())
        }
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
}

/// Handles one SCSI command against `device`.
pub fn process_command<B, Buf, D>(
    mut command: Command<ScsiCommand, StorageClass<'_, B, Buf>>,
    device: &mut D,
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

    let blocks = device.blocks();

    match command.kind {
        ScsiCommand::TestUnitReady => {
            command.pass(0);
        }
        ScsiCommand::Inquiry { .. } => {
            let mut data = [0u8; 36];
            data[0] = 0x00; // direct access block device
            data[1] = 0x80; // removable
            data[2] = 0x04; // SPC-2
            data[3] = 0x02; // response data format
            data[4] = 0x20; // 36 bytes total
            data[8..16].copy_from_slice(VENDOR_ID);
            data[16..32].copy_from_slice(PRODUCT_ID);
            data[32..36].copy_from_slice(PRODUCT_REVISION);
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
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
        }
        ScsiCommand::ReadCapacity10 => {
            let mut data = [0u8; 8];
            // Last addressable block, not the count.
            data[0..4].copy_from_slice(&u32::to_be_bytes(blocks - 1));
            data[4..8].copy_from_slice(&u32::to_be_bytes(BLOCK_SIZE));
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        ScsiCommand::ReadCapacity16 { .. } => {
            let mut data = [0u8; 16];
            data[0..8].copy_from_slice(&u64::to_be_bytes((blocks - 1) as u64));
            data[8..12].copy_from_slice(&u32::to_be_bytes(BLOCK_SIZE));
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        ScsiCommand::ReadFormatCapacities { .. } => {
            let mut data = [0u8; 12];
            data[3] = 0x08; // capacity list length
            data[4..8].copy_from_slice(&u32::to_be_bytes(blocks));
            data[8] = 0x02; // formatted media
            data[9..12].copy_from_slice(&u32::to_be_bytes(BLOCK_SIZE)[1..]);
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
        }
        ScsiCommand::Read { lba, len } => {
            let total = len as usize * BLOCK_SIZE as usize;

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

                let block_offset = state.offset % BLOCK_SIZE as usize;
                if block_offset == 0 {
                    let block = lba + (state.offset / BLOCK_SIZE as usize) as u32;
                    if device.read_block(block, &mut state.block).is_err() {
                        warn!("storage: read failed at block {}", block);
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
            let total = len as usize * BLOCK_SIZE as usize;

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

                let block_offset = state.offset % BLOCK_SIZE as usize;
                let count = command.read_data(&mut state.block[block_offset..])?;
                state.offset += count;

                if count > 0 && state.offset.is_multiple_of(BLOCK_SIZE as usize) {
                    let block = lba + (state.offset / BLOCK_SIZE as usize) as u32 - 1;
                    if device.write_block(block, &state.block).is_err() {
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
