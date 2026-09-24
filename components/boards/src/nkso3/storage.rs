use aes::{
    cipher::{Key, KeyInit as _},
    Aes128,
};
use interchange::{Channel, Requester, Responder};
use usb_classes::storage::{
    EncryptedBlockDevice, MemoryBlockDevice, State, StorageClass, BLOCK_SIZE,
};
use usb_device::{
    bus::{UsbBus, UsbBusAllocator},
    device::{UsbDevice, UsbDeviceState},
};
use xts_mode::Xts128;

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

pub const BUFFER_LEN: usize = 2 * BLOCK_SIZE;

pub struct UsbStorage<'a, B: UsbBus> {
    pub scsi: StorageClass<'a, B, [u8; 512]>,
    state: State,
    block_device: MemoryBlockDevice<'a, BUFFER_LEN>,
    responder: StorageResponder<'a>,
    xts: Option<Xts128<Aes128>>,
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
            block_device: MemoryBlockDevice::new(buffer),
            responder,
            xts: None,
        }
    }

    pub fn poll<F>(&mut self, device: &mut UsbDevice<'_, B>, force_reset: F)
    where
        F: FnOnce(&mut UsbDevice<'_, B>) -> usb_device::Result<()>,
    {
        if let Some(action) = self.responder.take_request() {
            self.responder.respond(()).ok();
            self.xts = match action {
                StorageAction::Lock => None,
                StorageAction::Unlock(key) => {
                    let cipher1 = Aes128::new(Key::<Aes128>::from_slice(&key[..16]));
                    let cipher2 = Aes128::new(Key::<Aes128>::from_slice(&key[16..]));
                    Some(Xts128::new(cipher1, cipher2))
                }
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
                let mut block_device = self
                    .xts
                    .as_ref()
                    .map(|xts| EncryptedBlockDevice::new(&mut self.block_device, xts));
                usb_classes::storage::process_command(
                    command,
                    block_device.as_mut(),
                    &mut self.state,
                )
            });
            if let Err(_err) = result {
                warn!("storage: {_err:?}");
            }
        }
    }
}
