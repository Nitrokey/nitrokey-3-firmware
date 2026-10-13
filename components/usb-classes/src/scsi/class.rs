use usb_device::bus::UsbBus;
use usbd_storage::{
    subclass::{
        scsi::{self, ScsiCommand},
        Command,
    },
    transport::{
        bbb::{BulkOnly, BulkOnlyError},
        TransportError,
    },
};

use super::{BlockDevice, BLOCK_SIZE, BLOCK_SIZE_U16};

pub type Scsi<'bus, B> = scsi::Scsi<BulkOnly<'bus, B, [u8; BLOCK_SIZE]>>;

const BLOCK_SIZE_U32: u32 = BLOCK_SIZE_U16 as _;

// TODO: derive from Model and Version
const VENDOR_ID: &[u8; 8] = b"Nitrokey";
const PRODUCT_ID: &[u8; 16] = b"Storage         ";
const PRODUCT_REVISION: &[u8; 4] = b"1.00";

// Commands that usbd-storage does not parse, but that a host expects a fixed
// write-through medium to accept.
const START_STOP_UNIT: u8 = 0x1B;
const PREVENT_ALLOW_MEDIUM_REMOVAL: u8 = 0x1E;
const SYNCHRONIZE_CACHE_10: u8 = 0x35;

// See https://www.t10.org/lists/2sensekey.htm.
#[derive(Clone, Copy, Debug, Default)]
enum SenseKey {
    #[default]
    NoSense,
    MediumError,
    IllegalRequest,
}

impl SenseKey {
    const fn value(&self) -> u8 {
        match self {
            Self::NoSense => 0x00,
            Self::MediumError => 0x03,
            Self::IllegalRequest => 0x05,
        }
    }
}

impl From<SenseKey> for u8 {
    fn from(sense_key: SenseKey) -> Self {
        sense_key.value()
    }
}

// See https://www.t10.org/lists/asc-num.htm.
#[derive(Clone, Copy, Debug, Default)]
enum AdditionalSenseData {
    #[default]
    NoInformation,
    WriteFault,
    UnrecoveredReadError,
    InvalidCommand,
    LbaOutOfRange,
}

impl AdditionalSenseData {
    const fn asc(&self) -> u8 {
        match self {
            Self::NoInformation => 0x00,
            Self::WriteFault => 0x03,
            Self::UnrecoveredReadError => 0x11,
            Self::InvalidCommand => 0x20,
            Self::LbaOutOfRange => 0x21,
        }
    }

    const fn ascq(&self) -> u8 {
        0x00
    }
}

enum ProtocolError {
    WriteError,
    ReadError,
    InvalidCommand,
    LbaOutOfRange,
}

impl ProtocolError {
    const fn sense_key(&self) -> SenseKey {
        match self {
            Self::WriteError | Self::ReadError => SenseKey::MediumError,
            Self::InvalidCommand | Self::LbaOutOfRange => SenseKey::IllegalRequest,
        }
    }

    const fn additional_sense_data(&self) -> AdditionalSenseData {
        match self {
            Self::WriteError => AdditionalSenseData::WriteFault,
            Self::ReadError => AdditionalSenseData::UnrecoveredReadError,
            Self::InvalidCommand => AdditionalSenseData::InvalidCommand,
            Self::LbaOutOfRange => AdditionalSenseData::LbaOutOfRange,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SenseData {
    key: SenseKey,
    additional_data: AdditionalSenseData,
}

impl From<ProtocolError> for SenseData {
    fn from(error: ProtocolError) -> Self {
        Self {
            key: error.sense_key(),
            additional_data: error.additional_sense_data(),
        }
    }
}

const BUFFER_BLOCK_COUNT: usize = 16;
const BUFFER_SIZE: usize = BUFFER_BLOCK_COUNT * BLOCK_SIZE;

/// Per-transfer state. A single SCSI read or write is spread across several
/// `poll_command` calls, so the progress within it has to be carried over.
pub(crate) struct State {
    /// Bytes transferred so far within the current command.
    offset: usize,
    /// The block currently being streamed.
    buffer: [[u8; BLOCK_SIZE]; BUFFER_BLOCK_COUNT],
    sense_data: Option<SenseData>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset: 0,
            buffer: [[0; BLOCK_SIZE]; BUFFER_BLOCK_COUNT],
            sense_data: None,
        }
    }
}

impl State {
    pub fn reset(&mut self) {
        self.offset = 0;
        self.sense_data = None;
    }

    fn fail_with(&mut self, error: ProtocolError) {
        self.sense_data = Some(error.into());
    }
}

/// Truncate the end of the buffer if in an operation that is smaller
/// than the total length of the buffer
fn relevant_buffer(
    buf: &mut [[u8; BLOCK_SIZE]; BUFFER_BLOCK_COUNT],
    offset: usize,
    total: usize,
) -> &mut [[u8; BLOCK_SIZE]] {
    let block_start = offset - (offset % BUFFER_SIZE);
    let max_blocks_remaining = (total - block_start) / BLOCK_SIZE;
    &mut buf[..max_blocks_remaining.min(BUFFER_BLOCK_COUNT)]
}

/// True if `[lba, lba + count)` lies within the device.
fn in_bounds<D: BlockDevice>(device: &D, lba: u32, count: u32) -> bool {
    lba.checked_add(count)
        .is_some_and(|end| end <= device.blocks())
}

/// Handles one SCSI command against `device`.
pub fn process_command<B, D>(
    mut command: Command<ScsiCommand, Scsi<'_, B>>,
    device: &mut D,
    state: &mut State,
) -> Result<(), TransportError<BulkOnlyError>>
where
    B: UsbBus,
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
                state.fail_with(ProtocolError::LbaOutOfRange);
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
            let sense_data = state.sense_data.take().unwrap_or_default();
            let mut data = [0u8; 18];
            data[0] = 0x70; // current errors
            data[2] = sense_data.key.value();
            data[7] = 0x0A; // additional length, else hosts never parse ASC/ASCQ
            data[12] = sense_data.additional_data.asc();
            data[13] = sense_data.additional_data.ascq();
            command.try_write_data_all(&data)?;
            command.pass(data.len() as u32);
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
                state.fail_with(ProtocolError::LbaOutOfRange);
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
                let state_buffer = relevant_buffer(&mut state.buffer, state.offset, total);

                let buffer_offset = state.offset % BUFFER_SIZE;
                if buffer_offset == 0 {
                    let block = lba + (state.offset / BLOCK_SIZE) as u32;
                    debug!("Reading {} blocks at address {block}", state_buffer.len());
                    if let Err(_err) = device.read_blocks(block, state_buffer) {
                        warn!("storage: read failed at block {} with {_err:?}", block);
                        state.fail_with(ProtocolError::ReadError);
                        command.fail(0);
                        state.offset = 0;
                        return Ok(());
                    }
                }

                let count = command.write_data(&state_buffer.as_flattened()[buffer_offset..])?;
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
                state.fail_with(ProtocolError::LbaOutOfRange);
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
                let state_buffer = relevant_buffer(&mut state.buffer, state.offset, total);

                let buffer_offset = state.offset % BUFFER_SIZE;
                let count =
                    command.read_data(&mut state_buffer.as_flattened_mut()[buffer_offset..])?;
                state.offset += count;

                if count > 0 && state.offset.is_multiple_of(BUFFER_SIZE) {
                    let block = lba + (state.offset / BLOCK_SIZE) as u32 - 1;
                    debug!("Writing {} blocks at address {block}", state_buffer.len());
                    if device.write_blocks(block, state_buffer).is_err() {
                        warn!("storage: write failed at block {}", block);
                        state.fail_with(ProtocolError::WriteError);
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
            state.fail_with(ProtocolError::InvalidCommand);
            command.fail(0);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::in_bounds;
    use crate::scsi::{MemoryBlockDevice, BLOCK_SIZE};

    #[test]
    fn bounds() {
        let mut buf = [[0; BLOCK_SIZE]; 8];
        let device = MemoryBlockDevice::new(&mut buf);
        assert!(in_bounds(&device, 0, 8));
        assert!(in_bounds(&device, 7, 1));
        assert!(in_bounds(&device, 8, 0));
        assert!(!in_bounds(&device, 7, 2));
        assert!(!in_bounds(&device, 9, 1));
        assert!(!in_bounds(&device, u32::MAX, 1));
    }
}
