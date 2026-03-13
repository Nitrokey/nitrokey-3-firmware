use stm32n6::stm32n657::BSEC;

pub fn uid(bsec: &BSEC) -> [u32; 3] {
    // See Section 79 (Device electronic signature), Section 4 (Boot and security control (BSEC))
    // and Section 5 (OTP mapping (OTP)) of the reference manual RM0486.
    let id0 = bsec.fvr(5).read().bits();
    let id1 = bsec.fvr(6).read().bits();
    let id2 = bsec.fvr(7).read().bits();
    [id0, id1, id2]
}
