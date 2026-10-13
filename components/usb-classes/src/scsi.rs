//! USB mass storage, exposed as a SCSI block device over Bulk Only Transport.

mod class;
mod xts;

use core::convert::Infallible;

use usb_device::{
    bus::{UsbBus, UsbBusAllocator},
    device::{UsbDevice, UsbDeviceState},
    UsbError,
};
use usbd_storage::transport::TransportError;

pub(crate) use self::class::State;
pub use self::{
    class::Scsi,
    xts::{Ciphers, Xts128},
};

/// Bytes per logical block. 512 is mostly assumed.
const BLOCK_SIZE_U16: u16 = 512;
pub const BLOCK_SIZE: usize = BLOCK_SIZE_U16 as _;

const MAX_LUN: u8 = 0;

/// A fixed-size block device backing the SCSI logical unit.
pub trait BlockDevice {
    type Error;

    /// Number of addressable blocks of [`BLOCK_SIZE`] bytes.
    fn blocks(&self) -> u32;

    fn read_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        self.read_blocks(lba, core::slice::from_mut(buf))
    }

    fn write_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        self.write_blocks(lba, core::slice::from_mut(buf))
    }

    fn read_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> Result<(), Self::Error>;
    fn write_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> Result<(), Self::Error>;
}

pub struct MemoryBlockDevice<'a, const N: usize>(&'a mut [[u8; BLOCK_SIZE]; N]);

impl<'a, const N: usize> MemoryBlockDevice<'a, N> {
    pub fn new(buffer: &'a mut [[u8; BLOCK_SIZE]; N]) -> Self {
        const {
            assert!(N <= (u32::MAX as usize));
        }
        Self(buffer)
    }
}

impl<const N: usize> BlockDevice for MemoryBlockDevice<'_, N> {
    type Error = Infallible;

    fn blocks(&self) -> u32 {
        const { N as _ }
    }

    fn read_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> Result<(), Self::Error> {
        buf.copy_from_slice(&self.0[lba as usize..][..buf.len()]);
        Ok(())
    }

    fn write_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> Result<(), Self::Error> {
        self.0[lba as usize..][..buf.len()].copy_from_slice(buf);
        Ok(())
    }
}

pub struct EncryptedBlockDevice<'a, B, X> {
    block_device: &'a mut B,
    xts: &'a mut X,
}

impl<'a, B: BlockDevice, X> EncryptedBlockDevice<'a, B, X> {
    pub fn new(block_device: &'a mut B, xts: &'a mut X) -> Self {
        Self { block_device, xts }
    }
}

impl<'a, B, X> BlockDevice for EncryptedBlockDevice<'a, B, X>
where
    B: BlockDevice,
    X: Xts128,
{
    type Error = B::Error;

    fn blocks(&self) -> u32 {
        self.block_device.blocks()
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        self.block_device.read_block(lba, buf)?;
        self.xts.decrypt_sector(buf, lba);
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &mut [u8; BLOCK_SIZE]) -> Result<(), Self::Error> {
        self.xts.encrypt_sector(buf, lba);
        self.block_device.write_block(lba, buf)
    }

    fn read_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> Result<(), Self::Error> {
        self.block_device.read_blocks(lba, buf)?;
        self.xts.decrypt_sectors(buf, lba);
        Ok(())
    }

    fn write_blocks(&mut self, lba: u32, buf: &mut [[u8; BLOCK_SIZE]]) -> Result<(), Self::Error> {
        self.xts.encrypt_sectors(buf, lba);
        self.block_device.write_blocks(lba, buf)
    }
}

/// Allocates the mass storage class.
pub fn setup<B: UsbBus>(bus: &UsbBusAllocator<B>) -> Scsi<'_, B> {
    // `packet_size` must be 512 when the device enumerates at high speed.
    const PACKET_SIZE: u16 = 512;
    Scsi::new(bus, PACKET_SIZE, MAX_LUN, [0; _]).expect("failed to allocate USB mass storage class")
}

pub(crate) fn poll<B, D>(
    usb_device: &mut UsbDevice<'_, B>,
    scsi: &mut Scsi<'_, B>,
    block_device: &mut D,
    state: &mut State,
) where
    B: UsbBus,
    D: BlockDevice,
{
    // A bus reset abandons any transfer that was in flight.
    if usb_device.state() == UsbDeviceState::Default {
        state.reset();
    }

    // One `poll_command` per transport poll, and `UsbDevice::poll` did one
    // too: a from-host phase ending strands undrained data in the buffer.
    for _ in 0..2 {
        let result =
            scsi.poll_command(|command| class::process_command(command, block_device, state));
        // WouldBlock is routine here: the transport polls the endpoint after the callback
        if let Err(_err) = result {
            if !matches!(_err, TransportError::Usb(UsbError::WouldBlock)) {
                warn_now!("storage: transport {_err:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use aes::{cipher::KeyInit as _, Aes128};

    use super::*;

    const BLOCKS: usize = 4;
    const KEY1: &[u8; 32] = b"0123456789abcdef0123456789abcdef";
    const KEY2: &[u8; 32] = b"deadbeefdeadbeefdeadbeefdeadbeef";

    fn block(fill: u8) -> [u8; BLOCK_SIZE] {
        [fill; _]
    }

    fn setup_xts(key: &[u8; 32]) -> Ciphers<Aes128, Aes128> {
        let cipher1 = Aes128::new_from_slice(&key[..16]).unwrap();
        let cipher2 = Aes128::new_from_slice(&key[16..]).unwrap();
        Ciphers { cipher1, cipher2 }
    }

    #[test]
    fn memory_round_trip() {
        let mut buffer = [[0; BLOCK_SIZE]; BLOCKS];
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
        let mut xts = setup_xts(KEY1);
        let mut buffer = [[0; BLOCK_SIZE]; BLOCKS];

        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &mut xts);
        encrypted_device.write_block(1, &mut block(0xC3)).unwrap();

        assert_ne!(buffer[1], block(0xC3));

        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &mut xts);
        let mut buf = block(0);
        encrypted_device.read_block(1, &mut buf).unwrap();
        assert_eq!(buf, block(0xC3));
    }

    #[test]
    fn tweak_differs_per_block() {
        let mut xts = setup_xts(KEY1);
        let mut buffer = [[0; BLOCK_SIZE]; BLOCKS];

        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &mut xts);

        encrypted_device.write_block(0, &mut block(0xFF)).unwrap();
        encrypted_device.write_block(1, &mut block(0xFF)).unwrap();

        assert_ne!(buffer[0], buffer[1]);
    }

    #[test]
    fn encrypted_different_keys() {
        let mut buffer = [[0; BLOCK_SIZE]; BLOCKS];

        let mut xts = setup_xts(KEY1);
        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &mut xts);

        encrypted_device.write_block(0, &mut block(0xFF)).unwrap();

        let mut xts = setup_xts(KEY2);
        let mut device = MemoryBlockDevice::new(&mut buffer);
        let mut encrypted_device = EncryptedBlockDevice::new(&mut device, &mut xts);

        let mut buf = block(0);
        encrypted_device.read_block(0, &mut buf).unwrap();
        assert_ne!(buf, block(0x0));
        assert_ne!(buf, block(0xFF));
    }
}
