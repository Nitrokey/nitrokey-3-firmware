use usb_classes::storage::{BlockDevice, State, StorageClass, BLOCK_SIZE};
use usb_device::{
    bus::{UsbBus, UsbBusAllocator},
    device::{UsbDevice, UsbDeviceState},
};

pub struct Storage;

impl storage_app::Storage for Storage {
    fn init(&mut self, _key: &[u8; 32]) -> Result<(), storage_app::Error> {
        info!("Storage initialized");
        Ok(())
    }

    fn unlock(&mut self, _key: &[u8; 32]) -> Result<(), storage_app::Error> {
        // TODO: implement
        info!("Storage unlocked");
        Ok(())
    }

    fn lock(&mut self) -> Result<(), storage_app::Error> {
        // TODO: implement
        info!("Storage locked");
        Ok(())
    }
}

const BLOCK_COUNT: u32 = 2;
pub const BUFFER_LEN: usize = (BLOCK_COUNT as usize) * (BLOCK_SIZE as usize);

struct RamBlockDevice<'a> {
    backing: &'a mut [u8; BUFFER_LEN],
}

impl<'a> RamBlockDevice<'a> {
    fn new(buffer: &'a mut [u8; BUFFER_LEN]) -> Self {
        Self { backing: buffer }
    }
}

fn offset(lba: u32) -> usize {
    lba as usize * BLOCK_SIZE as usize
}

impl BlockDevice for RamBlockDevice<'_> {
    type Error = ();

    fn blocks(&self) -> u32 {
        BLOCK_COUNT
    }

    fn read_block(&mut self, lba: u32, buf: &mut [u8]) -> Result<(), Self::Error> {
        let offset = offset(lba);
        buf.copy_from_slice(&self.backing[offset..offset + buf.len()]);
        Ok(())
    }

    fn write_block(&mut self, lba: u32, buf: &[u8]) -> Result<(), Self::Error> {
        let offset = offset(lba);
        self.backing[offset..offset + buf.len()].copy_from_slice(buf);
        Ok(())
    }
}

pub struct UsbStorage<'a, B: UsbBus> {
    pub scsi: StorageClass<'a, B, [u8; 512]>,
    state: State,
    block_device: RamBlockDevice<'a>,
}

impl<'a, B: UsbBus> UsbStorage<'a, B> {
    pub fn new(usb_bus: &'a UsbBusAllocator<B>, buffer: &'a mut [u8; BUFFER_LEN]) -> Self {
        Self {
            scsi: usb_classes::storage::setup(usb_bus, 512, [0; 512]),
            state: State::default(),
            block_device: RamBlockDevice::new(buffer),
        }
    }

    pub fn poll(&mut self, device: &mut UsbDevice<'_, B>) {
        // TODO: check storage state

        if device.state() == UsbDeviceState::Default {
            self.state.reset();
        }

        // One `poll_command` per transport poll, and `UsbDevice::poll` did one
        // too: a from-host phase ending strands undrained data in the buffer.
        for _ in 0..2 {
            let result = self.scsi.poll_command(|command| {
                usb_classes::storage::process_command(
                    command,
                    &mut self.block_device,
                    &mut self.state,
                )
            });
            if let Err(_err) = result {
                warn!("storage: {_err:?}");
            }
        }
    }
}
