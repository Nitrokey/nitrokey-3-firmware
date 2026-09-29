// Cryptographic processor (CRYP), see Section 49 of RM0486.

use cipher::{
    Array, Block, BlockSizeUser, InOut, InOutBuf, ParBlocks, ParBlocksSizeUser,
    block::{
        BlockCipherDecBackend, BlockCipherDecClosure, BlockCipherDecrypt, BlockCipherEncBackend,
        BlockCipherEncClosure, BlockCipherEncrypt,
    },
    common::typenum::{U16, U32},
};
use stm32n6::stm32n657::CRYP_S;

use crate::rcc::{Peripheral, Rcc};

enum Operation {
    Decryption,
    Encryption,
}

#[derive(Clone, Copy)]
enum ChainingMode {
    Ecb,
}

enum AlgoMode {
    Ecb,
    KeyDerivation,
}

impl AlgoMode {
    const fn cryp_algomode(self) -> u8 {
        match self {
            Self::Ecb => 0x4,
            Self::KeyDerivation => 0x7,
        }
    }
}

impl From<ChainingMode> for AlgoMode {
    fn from(mode: ChainingMode) -> Self {
        match mode {
            ChainingMode::Ecb => Self::Ecb,
        }
    }
}

enum Mode {
    Decryption,
    Encryption,
}

impl Mode {
    const fn cryp_algodir(self) -> bool {
        matches!(self, Self::Decryption)
    }
}

enum KeyMode {
    Normal,
}

impl KeyMode {
    const fn cryp_kmod(self) -> u8 {
        match self {
            Self::Normal => 0x0,
        }
    }
}

enum DataType {
    NoSwapping,
}

impl DataType {
    const fn cryp_datatype(self) -> u8 {
        match self {
            Self::NoSwapping => 0x0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Key {
    Aes128([u8; 16]),
    Aes192([u8; 24]),
    Aes256([u8; 32]),
}

impl Key {
    const fn key_size(&self) -> KeySize {
        match self {
            Self::Aes128(_) => KeySize::Aes128,
            Self::Aes192(_) => KeySize::Aes192,
            Self::Aes256(_) => KeySize::Aes256,
        }
    }

    const fn key(&self) -> &[u8] {
        match self {
            Self::Aes128(key) => key,
            Self::Aes192(key) => key,
            Self::Aes256(key) => key,
        }
    }

    const fn word(&self, x: usize) -> u32 {
        let key = self.key();
        let i = x * 4;
        let bytes = [key[i], key[i + 1], key[i + 2], key[i + 3]];
        u32::from_be_bytes(bytes)
    }

    const fn k3rr(&self) -> u32 {
        match self {
            Self::Aes128(_) => self.word(3),
            Self::Aes192(_) => self.word(5),
            Self::Aes256(_) => self.word(7),
        }
    }

    const fn k3lr(&self) -> u32 {
        match self {
            Self::Aes128(_) => self.word(2),
            Self::Aes192(_) => self.word(4),
            Self::Aes256(_) => self.word(6),
        }
    }

    const fn k2rr(&self) -> u32 {
        match self {
            Self::Aes128(_) => self.word(1),
            Self::Aes192(_) => self.word(3),
            Self::Aes256(_) => self.word(5),
        }
    }

    const fn k2lr(&self) -> u32 {
        match self {
            Self::Aes128(_) => self.word(0),
            Self::Aes192(_) => self.word(2),
            Self::Aes256(_) => self.word(4),
        }
    }

    const fn k1rr(&self) -> Option<u32> {
        match self {
            Self::Aes128(_) => None,
            Self::Aes192(_) => Some(self.word(1)),
            Self::Aes256(_) => Some(self.word(3)),
        }
    }

    const fn k1lr(&self) -> Option<u32> {
        match self {
            Self::Aes128(_) => None,
            Self::Aes192(_) => Some(self.word(0)),
            Self::Aes256(_) => Some(self.word(2)),
        }
    }

    const fn k0rr(&self) -> Option<u32> {
        match self {
            Self::Aes128(_) | Self::Aes192(_) => None,
            Self::Aes256(_) => Some(self.word(1)),
        }
    }

    const fn k0lr(&self) -> Option<u32> {
        match self {
            Self::Aes128(_) | Self::Aes192(_) => None,
            Self::Aes256(_) => Some(self.word(0)),
        }
    }
}

enum KeySize {
    Aes128,
    Aes192,
    Aes256,
}

impl KeySize {
    const fn cryp_keysize(self) -> u8 {
        match self {
            Self::Aes128 => 0x0,
            Self::Aes192 => 0x1,
            Self::Aes256 => 0x2,
        }
    }
}

pub struct Cryp(CRYP_S);

impl Cryp {
    pub fn new(saes: CRYP_S, rcc: &Rcc) -> Self {
        rcc.enable(Peripheral::Cryp);
        rcc.reset(Peripheral::Cryp);
        Self(saes)
    }

    fn setup_dec(&self, chmod: ChainingMode, key: Key) -> CrypDec<'_> {
        self.setup(Operation::Decryption, chmod, key);
        CrypDec(self)
    }

    fn setup_enc(&self, chmod: ChainingMode, key: Key) -> CrypEnc<'_> {
        self.setup(Operation::Encryption, chmod, key);
        CrypEnc(self)
    }

    fn setup(&self, operation: Operation, chmod: ChainingMode, key: Key) {
        // See Section 49.4.9 for ECB encryption and decryption.  Ex refers to the steps for the
        // encryption setup, Dx to the steps for the decryption setup.

        // E1/D1. Disable the SAES peripheral
        self.disable();
        // E2/D2. Flush the FIFOs
        self.flush_fifos();
        // E3/D3. Initialize the CRYP_CR register
        let algomode = match operation {
            Operation::Encryption => chmod.into(),
            Operation::Decryption => AlgoMode::KeyDerivation,
        };
        self.configure(Mode::Encryption, algomode, key.key_size());

        if matches!(operation, Operation::Encryption) {
            // E4. Write the IV
            match chmod {
                ChainingMode::Ecb => {} // IV not required for ECB
            }
        }

        // E5/D4. Write the key
        self.write_key(key);
        // E6/D5. Wait until KEYVALID is set
        while !self.is_key_valid() {}

        if matches!(operation, Operation::Decryption) {
            // D6. Enable the SAES peripheral
            self.enable();
            // D7. Wait until the busy flag is cleared and select chaining mode in decryption mode
            while self.is_busy() {}
            self.reconfigure(Mode::Decryption, chmod.into());
            // D8. Write the IV
            match chmod {
                ChainingMode::Ecb => {} // IV not required for ECB
            }
        }

        // E7/D9. Enable the CRYP peripheral
        self.enable();
    }

    fn disable(&self) {
        self.0.cr().modify(|_, w| w.crypen().clear_bit());
    }

    fn enable(&self) {
        self.0.cr().modify(|_, w| w.crypen().set_bit());
    }

    fn is_busy(&self) -> bool {
        self.0.sr().read().busy().bit_is_set()
    }

    fn is_output_fifo_empty(&self) -> bool {
        self.0.sr().read().ofne().bit_is_clear()
    }

    fn is_input_fifo_full(&self) -> bool {
        self.0.sr().read().ifnf().bit_is_clear()
    }

    fn is_key_valid(&self) -> bool {
        self.0.sr().read().keyvalid().bit_is_set()
    }

    fn flush_fifos(&self) {
        self.0.cr().modify(|_, w| w.fflush().set_bit());
    }

    fn configure(&self, mode: Mode, algomode: AlgoMode, key_size: KeySize) {
        self.0.cr().write(|w| {
            unsafe {
                w.algomode().bits(algomode.cryp_algomode());
                w.algodir().bit(mode.cryp_algodir());
                w.datatype().bits(DataType::NoSwapping.cryp_datatype());
                w.keysize().bits(key_size.cryp_keysize());
                w.kmod().bits(KeyMode::Normal.cryp_kmod());
            }
            w
        });
    }

    fn reconfigure(&self, mode: Mode, algomode: AlgoMode) {
        self.0.cr().modify(|_, w| {
            unsafe {
                w.algomode().bits(algomode.cryp_algomode());
                w.algodir().bit(mode.cryp_algodir());
            }
            w
        });
    }

    fn write_key(&self, key: Key) {
        self.0.k3rr().write(|w| unsafe { w.k().bits(key.k3rr()) });
        self.0.k3lr().write(|w| unsafe { w.k().bits(key.k3lr()) });
        self.0.k2rr().write(|w| unsafe { w.k().bits(key.k2rr()) });
        self.0.k2lr().write(|w| unsafe { w.k().bits(key.k2lr()) });
        if let Some(k1rr) = key.k1rr() {
            self.0.k1rr().write(|w| unsafe { w.k().bits(k1rr) });
        }
        if let Some(k1lr) = key.k1lr() {
            self.0.k1lr().write(|w| unsafe { w.k().bits(k1lr) });
        }
        if let Some(k0rr) = key.k0rr() {
            self.0.k0rr().write(|w| unsafe { w.k().bits(k0rr) });
        }
        if let Some(k0lr) = key.k0lr() {
            self.0.k0lr().write(|w| unsafe { w.k().bits(k0lr) });
        }
    }

    fn compute_polling(&self, mut block: InOut<'_, '_, Array<u8, U16>>) {
        // See Section 49.4.5.
        // Wait until the not-full flag IFNF is set
        while self.is_input_fifo_full() {}
        // Write data in the input FIFO
        self.write_block(block.get_in());
        // Wait until the not-empty flag OFNE is set
        while self.is_output_fifo_empty() {}
        // Read the output FIFO
        self.read_block(block.get_out());
    }

    fn compute_polling_multi(&self, mut blocks: InOutBuf<'_, '_, Array<u8, U16>>) {
        let n = blocks.len();
        let mut i = 0;
        let mut j = 0;
        while j < n {
            // Write data to the input FIFO until it is full
            while i < n && !self.is_input_fifo_full() {
                self.write_block(&blocks.get_in()[i]);
                i += 1;
            }

            // Wait until the output FIFO is not empty
            while self.is_output_fifo_empty() {}

            // Read data from the output FIFO until it is empty
            while j < n && !self.is_output_fifo_empty() {
                self.read_block(&mut blocks.get_out()[j]);
                j += 1;
            }

            if i < n {
                // Wait until the input FIFO is not full
                while self.is_input_fifo_full() {}
            }
        }
    }

    fn write_block(&self, block: &Array<u8, U16>) {
        for i in 0..4 {
            let din = u32::from_be_bytes(*block[i * 4..].first_chunk().unwrap());
            self.0.dinr().write(|w| unsafe { w.din().bits(din) });
        }
    }

    fn read_block(&self, block: &mut Array<u8, U16>) {
        for i in 0..4 {
            let dout = self.0.doutr().read().dout().bits();
            block[i * 4..][..4].copy_from_slice(&dout.to_be_bytes());
        }
    }
}

struct CrypDec<'a>(&'a Cryp);

impl Drop for CrypDec<'_> {
    fn drop(&mut self) {
        self.0.disable();
    }
}

impl BlockCipherDecBackend for CrypDec<'_> {
    fn decrypt_block(&self, block: InOut<'_, '_, Block<Self>>) {
        self.0.compute_polling(block)
    }

    fn decrypt_par_blocks(&self, blocks: InOut<'_, '_, ParBlocks<Self>>) {
        self.0.compute_polling_multi(blocks.into_buf())
    }

    fn decrypt_tail_blocks(&self, blocks: InOutBuf<'_, '_, Block<Self>>) {
        self.0.compute_polling_multi(blocks)
    }
}

impl BlockSizeUser for CrypDec<'_> {
    type BlockSize = U16;
}

impl ParBlocksSizeUser for CrypDec<'_> {
    type ParBlocksSize = U32;
}

struct CrypEnc<'a>(&'a Cryp);

impl Drop for CrypEnc<'_> {
    fn drop(&mut self) {
        self.0.disable();
    }
}

impl BlockCipherEncBackend for CrypEnc<'_> {
    fn encrypt_block(&self, block: InOut<'_, '_, Block<Self>>) {
        self.0.compute_polling(block)
    }

    fn encrypt_par_blocks(&self, blocks: InOut<'_, '_, ParBlocks<Self>>) {
        self.0.compute_polling_multi(blocks.into_buf())
    }

    fn encrypt_tail_blocks(&self, blocks: InOutBuf<'_, '_, Block<Self>>) {
        self.0.compute_polling_multi(blocks)
    }
}

impl BlockSizeUser for CrypEnc<'_> {
    type BlockSize = U16;
}

impl ParBlocksSizeUser for CrypEnc<'_> {
    type ParBlocksSize = U32;
}

pub struct AesDec<'a> {
    cryp: &'a Cryp,
    key: Key,
}

impl<'a> AesDec<'a> {
    pub fn new(cryp: &'a Cryp, key: Key) -> Self {
        Self { cryp, key }
    }
}

impl BlockCipherDecrypt for AesDec<'_> {
    fn decrypt_with_backend(&self, f: impl BlockCipherDecClosure<BlockSize = Self::BlockSize>) {
        let backend = self.cryp.setup_dec(ChainingMode::Ecb, self.key);
        f.call(&backend)
    }
}

impl BlockSizeUser for AesDec<'_> {
    type BlockSize = U16;
}

pub struct AesEnc<'a> {
    cryp: &'a Cryp,
    key: Key,
}

impl<'a> AesEnc<'a> {
    pub fn new(cryp: &'a Cryp, key: Key) -> Self {
        Self { cryp, key }
    }
}

impl BlockCipherEncrypt for AesEnc<'_> {
    fn encrypt_with_backend(&self, f: impl BlockCipherEncClosure<BlockSize = Self::BlockSize>) {
        let backend = self.cryp.setup_enc(ChainingMode::Ecb, self.key);
        f.call(&backend)
    }
}

impl BlockSizeUser for AesEnc<'_> {
    type BlockSize = U16;
}
