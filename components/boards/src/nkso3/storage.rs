use core::cell::RefCell;

use crate::soc::stm32n6::mmc::{self, Mmc};

use aes::{
    cipher::{Key, KeyInit as _},
    Aes128,
};
use cipher::{typenum::U16, BlockCipher, BlockClosure, BlockDecrypt, BlockEncrypt, BlockSizeUser};
use interchange::{Channel, Requester, Responder};
use stm32n657_hal::saes::{self, Saes};
use usb_classes::storage::{
    stm32n657_sdmmc::MmcStorage, EncryptedBlockDevice, State, StorageClass, BLOCK_SIZE,
};
use usb_device::{
    bus::{UsbBus, UsbBusAllocator},
    device::{UsbDevice, UsbDeviceState},
    UsbError,
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

/// DWT cycles for one XTS-AES-128 sector encryption, averaged over 16
pub fn xts_bench_cycles() -> u32 {
    let cipher1 = Aes128::new(Key::<Aes128>::from_slice(&[1; 16]));
    let cipher2 = Aes128::new(Key::<Aes128>::from_slice(&[2; 16]));
    let xts = Xts128::new(cipher1, cipher2);
    let mut block = [0x5au8; BLOCK_SIZE];
    let start = cortex_m::peripheral::DWT::cycle_count();
    for i in 0..16u32 {
        xts.encrypt_sector(&mut block, xts_mode::get_tweak_default(i.into()));
    }
    cortex_m::peripheral::DWT::cycle_count().wrapping_sub(start) / 16
}

enum Mode {
    Hardware,
    Software,
}

enum Cipher<'a> {
    Hardware {
        key: [u8; 16],
        saes: RefCell<&'a mut Saes>,
    },
    Software(&'a Aes128),
}

impl BlockCipher for Cipher<'_> {}

impl BlockDecrypt for Cipher<'_> {
    fn decrypt_with_backend(&self, f: impl BlockClosure<BlockSize = Self::BlockSize>) {
        match self {
            Self::Hardware { key, saes } => {
                let mut saes = saes.borrow_mut();
                saes::AesDec::new(*saes, saes::Key::Aes128(*key)).decrypt_with_backend(f);
            }
            Self::Software(cipher) => cipher.decrypt_with_backend(f),
        }
    }
}

impl BlockEncrypt for Cipher<'_> {
    fn encrypt_with_backend(&self, f: impl BlockClosure<BlockSize = Self::BlockSize>) {
        match self {
            Self::Hardware { key, saes } => {
                let mut saes = saes.borrow_mut();
                saes::AesEnc::new(*saes, saes::Key::Aes128(*key)).encrypt_with_backend(f);
            }
            Self::Software(cipher) => cipher.encrypt_with_backend(f),
        }
    }
}

impl BlockSizeUser for Cipher<'_> {
    type BlockSize = U16;
}

enum Ciphers {
    Hardware {
        key1: [u8; 16],
        cipher2: Aes128,
        saes: Saes,
    },
    Software {
        cipher1: Aes128,
        cipher2: Aes128,
        saes: Saes,
    },
}

impl Ciphers {
    fn new(key: [u8; 32], saes: Saes, mode: Mode) -> Self {
        match mode {
            Mode::Hardware => {
                let key1 = *key.first_chunk().unwrap();
                let cipher2 = Aes128::new(Key::<Aes128>::from_slice(&key[16..]));
                Self::Hardware {
                    key1,
                    cipher2,
                    saes,
                }
            }
            Mode::Software => {
                let cipher1 = Aes128::new(Key::<Aes128>::from_slice(&key[..16]));
                let cipher2 = Aes128::new(Key::<Aes128>::from_slice(&key[16..]));
                Self::Software {
                    cipher1,
                    cipher2,
                    saes,
                }
            }
        }
    }

    fn lock(self) -> Saes {
        match self {
            Self::Hardware { saes, .. } => saes,
            Self::Software { saes, .. } => saes,
        }
    }

    fn xts(&mut self) -> Xts128<Cipher<'_>> {
        let (cipher1, cipher2) = match self {
            Self::Hardware {
                key1,
                cipher2,
                saes,
            } => (
                Cipher::Hardware {
                    key: *key1,
                    saes: RefCell::new(saes),
                },
                Cipher::Software(cipher2),
            ),
            Self::Software {
                cipher1, cipher2, ..
            } => (Cipher::Software(cipher1), Cipher::Software(cipher2)),
        };
        Xts128::new(cipher1, cipher2)
    }
}

struct EncryptionState {
    saes: Option<Saes>,
    ciphers: Option<Ciphers>,
}

impl EncryptionState {
    fn new(saes: Saes) -> Self {
        Self {
            saes: Some(saes),
            ciphers: None,
        }
    }

    fn lock(&mut self) {
        if let Some(ciphers) = self.ciphers.take() {
            self.saes = Some(ciphers.lock());
        }
    }

    fn unlock(&mut self, key: [u8; 32], mode: Mode) {
        if let Some(saes) = self.saes.take() {
            self.ciphers = Some(Ciphers::new(key, saes, mode));
        }
    }

    fn xts(&mut self) -> Option<Xts128<Cipher<'_>>> {
        self.ciphers.as_mut().map(Ciphers::xts)
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
        responder: StorageResponder<'a>,
        saes: Saes,
    ) -> Self {
        Self {
            scsi: usb_classes::storage::setup(usb_bus, 512, [0; 512]),
            state: State::default(),
            block_device: MmcStorage::new(mmc),
            responder,
            encryption: EncryptionState::new(saes),
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
                StorageAction::Unlock(key) => self.encryption.unlock(key, Mode::Hardware),
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
                let xts = self.encryption.xts();
                let mut block_device = xts
                    .as_ref()
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
