//! Secure AES coprocessor (SAES), see Section 48 of RM0486.

use cipher::{
    Block, BlockBackend, BlockClosure, BlockDecryptMut, BlockEncryptMut, BlockSizeUser,
    ParBlocksSizeUser,
    generic_array::GenericArray,
    inout::InOut,
    typenum::{U1, U16},
};
use stm32n6::stm32n657::SAES_S;

use crate::rcc::{Peripheral, Rcc};

enum Operation {
    Encryption,
    Decryption,
}

#[derive(Clone, Copy)]
enum ChainingMode {
    Ecb,
}

impl ChainingMode {
    const fn saes_chmod(self) -> u8 {
        match self {
            Self::Ecb => 0x0,
        }
    }
}

enum Mode {
    Encryption,
    KeyDerivation,
    Decryption,
}

impl Mode {
    const fn saes_mode(self) -> u8 {
        match self {
            Self::Encryption => 0x0,
            Self::KeyDerivation => 0x1,
            Self::Decryption => 0x2,
        }
    }
}

enum KeyMode {
    Normal,
}

impl KeyMode {
    const fn saes_kmod(self) -> u8 {
        match self {
            Self::Normal => 0x0,
        }
    }
}

enum DataType {
    NoSwapping,
}

impl DataType {
    const fn saes_datatype(self) -> u8 {
        match self {
            Self::NoSwapping => 0x0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Key {
    Aes128([u8; 16]),
    Aes256([u8; 32]),
}

impl Key {
    const fn key_size(&self) -> KeySize {
        match self {
            Self::Aes128(_) => KeySize::Aes128,
            Self::Aes256(_) => KeySize::Aes256,
        }
    }

    const fn key(&self) -> &[u8] {
        match self {
            Self::Aes128(key) => key,
            Self::Aes256(key) => key,
        }
    }

    const fn word(&self, x: usize) -> u32 {
        let key = self.key();
        let i = x * 4;
        let bytes = [key[i], key[i + 1], key[i + 2], key[i + 3]];
        u32::from_be_bytes(bytes)
    }

    const fn keyr0123(&self) -> [u32; 4] {
        match self {
            Self::Aes128(_) => [self.word(3), self.word(2), self.word(1), self.word(0)],
            Self::Aes256(_) => [self.word(7), self.word(6), self.word(5), self.word(4)],
        }
    }

    const fn keyr4567(&self) -> Option<[u32; 4]> {
        match self {
            Self::Aes128(_) => None,
            Self::Aes256(_) => Some([self.word(3), self.word(2), self.word(1), self.word(0)]),
        }
    }
}

enum KeySize {
    Aes128,
    Aes256,
}

impl KeySize {
    const fn saes_keysize(self) -> bool {
        matches!(self, KeySize::Aes256)
    }
}

pub struct Saes(SAES_S);

impl Saes {
    pub fn new(saes: SAES_S, rcc: &Rcc) -> Self {
        rcc.enable(Peripheral::Saes);
        rcc.reset(Peripheral::Saes);
        Self(saes)
    }

    fn setup(&mut self, operation: Operation, chmod: ChainingMode, key: Key) {
        // See Section 48.4.9 for ECB encryption and decryption.  Ex refers to the steps for the
        // encryption setup, Dx to the steps for the decryption setup.

        // E1/D1. Disable the SAES peripheral
        self.disable();
        // E2/D2. Wait until BUSY is cleared
        while self.is_busy() {}
        // E3/D3. Initialize the SAES_CR register
        let mode = match operation {
            Operation::Encryption => Mode::Encryption,
            Operation::Decryption => Mode::KeyDerivation,
        };
        self.configure(mode, Some(chmod), key.key_size());

        if matches!(operation, Operation::Encryption) {
            // E4. Write the IV
            match chmod {
                ChainingMode::Ecb => {} // IV not required for ECB
            }
        }

        // E5/D4. Write the key into SAES_KEYRx registers
        self.write_key(key);
        // E6/D5. Wait until KEYVALID is set
        while !self.is_key_valid() {}

        if matches!(operation, Operation::Decryption) {
            // D6. Enable the SAES peripheral
            self.enable();
            // D7. Wait until the CCF flag is set
            while !self.is_computation_complete() {}
            // D8. Clear the CCF flag
            self.clear_computation_complete();
            // D9. Select chaining mode in decryption mode
            self.reconfigure(Mode::Decryption, Some(chmod));
            // D10. Write the IV
            match chmod {
                ChainingMode::Ecb => {} // IV not required for ECB
            }
        }

        // E7/D11. Enable the SAES peripheral
        self.enable();
    }

    fn disable(&mut self) {
        self.0.cr().modify(|_, w| w.en().clear_bit());
    }

    fn enable(&mut self) {
        self.0.cr().modify(|_, w| w.en().set_bit());
    }

    fn is_busy(&self) -> bool {
        self.0.sr().read().busy().bit_is_set()
    }

    fn is_key_valid(&self) -> bool {
        self.0.sr().read().keyvalid().bit_is_set()
    }

    fn is_computation_complete(&self) -> bool {
        self.0.isr().read().ccf().bit_is_set()
    }

    fn clear_computation_complete(&mut self) {
        self.0.icr().write(|w| w.ccf().set_bit());
    }

    fn configure(&mut self, mode: Mode, chmod: Option<ChainingMode>, key_size: KeySize) {
        self.0.cr().write(|w| {
            unsafe {
                if let Some(chmod) = chmod {
                    w.chmod().bits(chmod.saes_chmod());
                }
                w.mode().bits(mode.saes_mode());
                w.datatype().bits(DataType::NoSwapping.saes_datatype());
                w.keysize().bit(key_size.saes_keysize());
                w.kmod().bits(KeyMode::Normal.saes_kmod());
            }
            w
        });
    }

    fn reconfigure(&mut self, mode: Mode, chmod: Option<ChainingMode>) {
        self.0.cr().modify(|_, w| {
            unsafe {
                if let Some(chmod) = chmod {
                    w.chmod().bits(chmod.saes_chmod());
                }
                w.mode().bits(mode.saes_mode());
            }
            w
        });
    }

    fn write_key(&mut self, key: Key) {
        let [keyr0, keyr1, keyr2, keyr3] = key.keyr0123();
        self.0.keyr0().write(|w| unsafe { w.key().bits(keyr0) });
        self.0.keyr1().write(|w| unsafe { w.key().bits(keyr1) });
        self.0.keyr2().write(|w| unsafe { w.key().bits(keyr2) });
        self.0.keyr3().write(|w| unsafe { w.key().bits(keyr3) });
        if let Some([keyr4, keyr5, keyr6, keyr7]) = key.keyr4567() {
            self.0.keyr4().write(|w| unsafe { w.key().bits(keyr4) });
            self.0.keyr5().write(|w| unsafe { w.key().bits(keyr5) });
            self.0.keyr6().write(|w| unsafe { w.key().bits(keyr6) });
            self.0.keyr7().write(|w| unsafe { w.key().bits(keyr7) });
        }
    }

    fn compute_polling(&mut self, mut block: InOut<'_, '_, GenericArray<u8, U16>>) {
        // See Section 48.4.5
        // Write four input data words into the SAES_DINR register
        let buf = block.get_in();
        for i in 0..4 {
            let din = u32::from_be_bytes(*buf[i * 4..].first_chunk().unwrap());
            self.0.dinr().write(|w| unsafe { w.din().bits(din) });
        }
        // Wait until the status flag CCF is set in the SAES_ISR register
        while !self.is_computation_complete() {}
        // Then read the four data words from the SAES_DOUTR register
        let buf = block.get_out();
        for i in 0..4 {
            let dout = self.0.doutr().read().dout().bits();
            buf[i * 4..][..4].copy_from_slice(&dout.to_be_bytes());
        }
        // Clear the CCF flag by setting the CCF bit of the SAES_ICR register
        self.clear_computation_complete()
    }
}

impl BlockBackend for Saes {
    fn proc_block(&mut self, block: InOut<'_, '_, Block<Self>>) {
        self.compute_polling(block)
    }
}

impl BlockSizeUser for Saes {
    type BlockSize = U16;
}

impl ParBlocksSizeUser for Saes {
    type ParBlocksSize = U1;
}

pub struct AesDec<'a>(&'a mut Saes);

impl<'a> AesDec<'a> {
    pub fn new(saes: &'a mut Saes, key: Key) -> Self {
        saes.setup(Operation::Decryption, ChainingMode::Ecb, key);
        Self(saes)
    }
}

impl Drop for AesDec<'_> {
    fn drop(&mut self) {
        self.0.disable();
    }
}

impl BlockDecryptMut for AesDec<'_> {
    fn decrypt_with_backend_mut(&mut self, f: impl BlockClosure<BlockSize = Self::BlockSize>) {
        f.call(self.0);
    }
}

impl BlockSizeUser for AesDec<'_> {
    type BlockSize = U16;
}

pub struct AesEnc<'a>(&'a mut Saes);

impl<'a> AesEnc<'a> {
    pub fn new(saes: &'a mut Saes, key: Key) -> Self {
        saes.setup(Operation::Encryption, ChainingMode::Ecb, key);
        Self(saes)
    }
}

impl Drop for AesEnc<'_> {
    fn drop(&mut self) {
        self.0.disable();
    }
}

impl BlockEncryptMut for AesEnc<'_> {
    fn encrypt_with_backend_mut(&mut self, f: impl BlockClosure<BlockSize = Self::BlockSize>) {
        f.call(self.0);
    }
}

impl BlockSizeUser for AesEnc<'_> {
    type BlockSize = U16;
}
