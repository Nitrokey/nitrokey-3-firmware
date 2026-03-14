//! Boot and security control (BSEC), see Section 4 of RM0486.

use stm32n6::stm32n657::BSEC;

pub struct Bsec(BSEC);

impl Bsec {
    pub fn new(bsec: BSEC) -> Self {
        Self(bsec)
    }

    /// Returns the unique device ID.
    ///
    /// See Section 79 (Device electronic signature) and Section 5 (OTP mapping) of RM0486.
    pub fn uid(&self) -> [u32; 3] {
        let id0 = self.0.fvr(5).read().bits();
        let id1 = self.0.fvr(6).read().bits();
        let id2 = self.0.fvr(7).read().bits();
        [id0, id1, id2]
    }
}
