use interchange::{Channel, Requester, Responder};
use usb_classes::storage::{BlockDevice, State, StorageClass, BLOCK_SIZE};
use usb_device::{
    bus::{UsbBus, UsbBusAllocator},
    device::{UsbDevice, UsbDeviceState},
};

pub type StorageChannel = Channel<StorageAction, ()>;
pub type StorageRequester<'a> = Requester<'a, StorageAction, ()>;
pub type StorageResponder<'a> = Responder<'a, StorageAction, ()>;

pub enum StorageAction {
    Lock,
    Unlock([u8; 32]),
}

pub struct Storage {
    rq: StorageRequester<'static>,
}

impl Storage {
    pub fn new(rq: StorageRequester<'static>) -> Self {
        Self { rq }
    }

    fn send(&mut self, action: StorageAction) -> Result<(), storage_app::Error> {
        // discard any replies to free the channel
        self.rq.take_response();
        self.rq
            .request(action)
            .map_err(|_| storage_app::Error::InternalError)
    }
}

impl storage_app::Storage for Storage {
    fn init(&mut self, _key: &[u8; 32]) -> Result<(), storage_app::Error> {
        info!("Storage initialized");
        Ok(())
    }

    fn unlock(&mut self, key: &[u8; 32]) -> Result<(), storage_app::Error> {
        info!("Storage unlocked");
        self.send(StorageAction::Unlock(*key))
    }

    fn lock(&mut self) -> Result<(), storage_app::Error> {
        info!("Storage locked");
        self.send(StorageAction::Lock)
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
    responder: StorageResponder<'a>,
    encryption_key: Option<[u8; 32]>,
}

impl<'a, B: UsbBus> UsbStorage<'a, B> {
    pub fn new(
        usb_bus: &'a UsbBusAllocator<B>,
        buffer: &'a mut [u8; BUFFER_LEN],
        responder: StorageResponder<'a>,
    ) -> Self {
        Self {
            scsi: usb_classes::storage::setup(usb_bus, 512, [0; 512]),
            state: State::default(),
            block_device: RamBlockDevice::new(buffer),
            responder,
            encryption_key: None,
        }
    }

    pub fn poll<F>(&mut self, device: &mut UsbDevice<'_, B>, force_reset: F)
    where
        F: FnOnce(&mut UsbDevice<'_, B>) -> usb_device::Result<()>,
    {
        if let Some(action) = self.responder.take_request() {
            self.responder.respond(()).ok();
            self.encryption_key = match action {
                StorageAction::Lock => None,
                StorageAction::Unlock(key) => Some(key),
            };
            if let Err(_err) = (force_reset)(device) {
                warn!("Failed to trigger USB force reset: {_err:?}");
            }
        }

        if device.state() == UsbDeviceState::Default {
            self.state.reset();
        }

        // One `poll_command` per transport poll, and `UsbDevice::poll` did one
        // too: a from-host phase ending strands undrained data in the buffer.
        for _ in 0..2 {
            let result = self.scsi.poll_command(|command| {
                usb_classes::storage::process_command(
                    command,
                    self.encryption_key.map(|_| &mut self.block_device),
                    &mut self.state,
                )
            });
            if let Err(_err) = result {
                warn!("storage: {_err:?}");
            }
        }
    }
}
