//! True random number generator (RNG), see Section 47 of RM0486.

use rand_core::{CryptoRng, Error, RngCore, impls};
use stm32n6::stm32n657::RNG_S;

use crate::rcc::{Peripheral, Rcc};

pub struct Rng(RNG_S);

impl Rng {
    /// Enables the RNG with the hardware default NIST-compliant configuration.
    pub fn new(rng: RNG_S, rcc: &Rcc) -> Self {
        rcc.enable(Peripheral::Rng);
        rcc.reset(Peripheral::Rng);
        rng.cr().modify(|_, w| w.rngen().set_bit());
        Self(rng)
    }

    /// Blocks until a valid random word is available.
    pub fn next_word(&mut self) -> u32 {
        loop {
            let sr = self.0.sr().read();
            if sr.seis().bit_is_set() || sr.ceis().bit_is_set() {
                // Auto-reset is enabled, so clearing the flags restarts generation.
                self.0
                    .sr()
                    .modify(|_, w| w.seis().clear_bit().ceis().clear_bit());
                continue;
            }
            if sr.drdy().bit_is_set() {
                let word = self.0.dr().read().bits();
                // A zero word means a seed error occurred after polling.
                if word != 0 {
                    return word;
                }
            }
        }
    }
}

impl RngCore for Rng {
    fn next_u32(&mut self) -> u32 {
        self.next_word()
    }

    fn next_u64(&mut self) -> u64 {
        impls::next_u64_via_u32(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for Rng {}
