use crate::soc::stm32n6::mmc::{self, Mmc};

use aes::{cipher::KeyInit as _, Aes128};
use interchange::{Channel, Requester, Responder};
use stm32n657_hal::cryp::{self, Cryp};
use usb_classes::storage::{
    stm32n657_sdmmc::MmcStorage, xts::Xts128, EncryptedBlockDevice, State, StorageClass, BLOCK_SIZE,
};
use usb_device::{
    bus::{UsbBus, UsbBusAllocator},
    device::{UsbDevice, UsbDeviceState},
    UsbError,
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

pub const BUFFER_LEN: usize = 2 * BLOCK_SIZE;

/// DWT cycles for one XTS-AES-128 sector encryption, averaged over 16
pub fn xts_bench_cycles(cryp: Cryp) -> (u32, Cryp) {
    let mut xts = XtsCiphers::new(cryp, [1; 32]);
    let mut block = [0x5au8; BLOCK_SIZE];
    let start = cortex_m::peripheral::DWT::cycle_count();
    for i in 0..16u32 {
        xts.encrypt_sector(&mut block, i.into());
    }
    let cycles = cortex_m::peripheral::DWT::cycle_count().wrapping_sub(start) / 16;
    (cycles, xts.lock())
}

struct XtsCiphers {
    cryp: Cryp,
    key1: [u8; 16],
    cipher2: Aes128,
}

impl XtsCiphers {
    fn new(cryp: Cryp, key: [u8; 32]) -> Self {
        let (key1, key2) = key.split_first_chunk().unwrap();
        let cipher2 = Aes128::new_from_slice(key2).unwrap();
        Self {
            cryp,
            key1: *key1,
            cipher2,
        }
    }

    fn lock(self) -> Cryp {
        self.cryp
    }
}

impl Xts128 for XtsCiphers {
    type C1Dec<'a> = cryp::AesDec<'a>;
    type C1Enc<'a> = cryp::AesEnc<'a>;
    type C2<'a> = Aes128;

    fn with_dec<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::C1Dec<'_>, &mut Self::C2<'_>),
    {
        let mut cipher1 = cryp::AesDec::new(&mut self.cryp, cryp::Key::Aes128(self.key1));
        f(&mut cipher1, &mut self.cipher2)
    }

    fn with_enc<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self::C1Enc<'_>, &mut Self::C2<'_>),
    {
        let mut cipher1 = cryp::AesEnc::new(&mut self.cryp, cryp::Key::Aes128(self.key1));
        f(&mut cipher1, &mut self.cipher2)
    }
}

// This should be an enum:
// ```
// enum EncryptionState {
//     Locked(Cryp),
//     Unlocked(XtsCiphers),
// }
// ```
// But this is not possible due to ownership limitations. As a workaround, each field represents
// a variant, so exactly one field must be non-null at any point.
struct EncryptionState {
    locked: Option<Cryp>,
    unlocked: Option<XtsCiphers>,
}

impl EncryptionState {
    fn new(cryp: Cryp) -> Self {
        Self {
            locked: Some(cryp),
            unlocked: None,
        }
    }

    fn lock(&mut self) {
        if let Some(xts) = self.unlocked.take() {
            self.locked = Some(xts.lock());
        }
    }

    fn unlock(&mut self, key: [u8; 32]) {
        if let Some(cryp) = self.locked.take() {
            self.unlocked = Some(XtsCiphers::new(cryp, key));
        }
    }

    fn xts(&mut self) -> Option<&mut XtsCiphers> {
        self.unlocked.as_mut()
    }
}

pub struct UsbStorage<'a, B: UsbBus> {
    pub scsi: StorageClass<'a, B, [u8; 512]>,
    state: State,
    block_device: MmcStorage<mmc::Peripheral, mmc::Pins>,
    responder: StorageResponder<'a>,
    encryption: EncryptionState,
}

impl<'a, B: UsbBus> UsbStorage<'a, B> {
    pub fn new(
        usb_bus: &'a UsbBusAllocator<B>,
        mmc: Mmc,
        cryp: Cryp,
        responder: StorageResponder<'a>,
    ) -> Self {
        Self {
            scsi: usb_classes::storage::setup(usb_bus, 512, [0; 512]),
            state: State::default(),
            block_device: MmcStorage::new(mmc),
            responder,
            encryption: EncryptionState::new(cryp),
        }
    }

    pub fn poll<F>(&mut self, device: &mut UsbDevice<'_, B>, force_reset: F)
    where
        F: FnOnce(&mut UsbDevice<'_, B>) -> usb_device::Result<()>,
    {
        if let Some(action) = self.responder.take_request() {
            self.responder.respond(()).ok();
            match action {
                StorageAction::Lock => self.encryption.lock(),
                StorageAction::Unlock(key) => self.encryption.unlock(key),
            }
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
                    .encryption
                    .xts()
                    .map(|xts| EncryptedBlockDevice::new(&mut self.block_device, xts));
                usb_classes::storage::process_command(
                    command,
                    block_device.as_mut(),
                    &mut self.state,
                )
            });
            // WouldBlock is routine here: the transport polls the endpoint after the callback
            if let Err(_err) = result {
                if !matches!(
                    _err,
                    usb_classes::storage::StorageTransportError::Usb(UsbError::WouldBlock)
                ) {
                    warn_now!("storage: transport {_err:?}");
                }
            }
        }
    }
}
