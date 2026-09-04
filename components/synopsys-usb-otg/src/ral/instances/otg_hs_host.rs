#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446, stm32f469

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_hs_host::Instance;
pub use super::super::peripherals::otg_hs_host::{RegisterBlock, ResetValues};
pub use super::super::peripherals::otg_hs_host::{
    HAINT, HAINTMSK, HCCHAR0, HCCHAR1, HCCHAR10, HCCHAR11, HCCHAR2, HCCHAR3, HCCHAR4, HCCHAR5,
    HCCHAR6, HCCHAR7, HCCHAR8, HCCHAR9, HCDMA0, HCDMA1, HCDMA10, HCDMA11, HCDMA2, HCDMA3, HCDMA4,
    HCDMA5, HCDMA6, HCDMA7, HCDMA8, HCDMA9, HCFG, HCINT0, HCINT1, HCINT10, HCINT11, HCINT2, HCINT3,
    HCINT4, HCINT5, HCINT6, HCINT7, HCINT8, HCINT9, HCINTMSK0, HCINTMSK1, HCINTMSK10, HCINTMSK11,
    HCINTMSK2, HCINTMSK3, HCINTMSK4, HCINTMSK5, HCINTMSK6, HCINTMSK7, HCINTMSK8, HCINTMSK9,
    HCSPLT0, HCSPLT1, HCSPLT10, HCSPLT11, HCSPLT2, HCSPLT3, HCSPLT4, HCSPLT5, HCSPLT6, HCSPLT7,
    HCSPLT8, HCSPLT9, HCTSIZ0, HCTSIZ1, HCTSIZ10, HCTSIZ11, HCTSIZ2, HCTSIZ3, HCTSIZ4, HCTSIZ5,
    HCTSIZ6, HCTSIZ7, HCTSIZ8, HCTSIZ9, HFIR, HFNUM, HPRT, HPTXSTS,
};

/// Access functions for the OTG_HS_HOST peripheral instance
pub mod OTG_HS_HOST {
    use super::ResetValues;

    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x40040400,
        _marker: ::core::marker::PhantomData,
    };

    /// Reset values for each field in OTG_HS_HOST
    pub const reset: ResetValues = ResetValues {
        HCFG: 0x00000000,
        HFIR: 0x0000EA60,
        HFNUM: 0x00003FFF,
        HPTXSTS: 0x00080100,
        HAINT: 0x00000000,
        HAINTMSK: 0x00000000,
        HPRT: 0x00000000,
        HCCHAR0: 0x00000000,
        HCCHAR1: 0x00000000,
        HCCHAR2: 0x00000000,
        HCCHAR3: 0x00000000,
        HCCHAR4: 0x00000000,
        HCCHAR5: 0x00000000,
        HCCHAR6: 0x00000000,
        HCCHAR7: 0x00000000,
        HCCHAR8: 0x00000000,
        HCCHAR9: 0x00000000,
        HCCHAR10: 0x00000000,
        HCCHAR11: 0x00000000,
        HCSPLT0: 0x00000000,
        HCSPLT1: 0x00000000,
        HCSPLT2: 0x00000000,
        HCSPLT3: 0x00000000,
        HCSPLT4: 0x00000000,
        HCSPLT5: 0x00000000,
        HCSPLT6: 0x00000000,
        HCSPLT7: 0x00000000,
        HCSPLT8: 0x00000000,
        HCSPLT9: 0x00000000,
        HCSPLT10: 0x00000000,
        HCSPLT11: 0x00000000,
        HCINT0: 0x00000000,
        HCINT1: 0x00000000,
        HCINT2: 0x00000000,
        HCINT3: 0x00000000,
        HCINT4: 0x00000000,
        HCINT5: 0x00000000,
        HCINT6: 0x00000000,
        HCINT7: 0x00000000,
        HCINT8: 0x00000000,
        HCINT9: 0x00000000,
        HCINT10: 0x00000000,
        HCINT11: 0x00000000,
        HCINTMSK0: 0x00000000,
        HCINTMSK1: 0x00000000,
        HCINTMSK2: 0x00000000,
        HCINTMSK3: 0x00000000,
        HCINTMSK4: 0x00000000,
        HCINTMSK5: 0x00000000,
        HCINTMSK6: 0x00000000,
        HCINTMSK7: 0x00000000,
        HCINTMSK8: 0x00000000,
        HCINTMSK9: 0x00000000,
        HCINTMSK10: 0x00000000,
        HCINTMSK11: 0x00000000,
        HCTSIZ0: 0x00000000,
        HCTSIZ1: 0x00000000,
        HCTSIZ2: 0x00000000,
        HCTSIZ3: 0x00000000,
        HCTSIZ4: 0x00000000,
        HCTSIZ5: 0x00000000,
        HCTSIZ6: 0x00000000,
        HCTSIZ7: 0x00000000,
        HCTSIZ8: 0x00000000,
        HCTSIZ9: 0x00000000,
        HCTSIZ10: 0x00000000,
        HCTSIZ11: 0x00000000,
        HCDMA0: 0x00000000,
        HCDMA1: 0x00000000,
        HCDMA2: 0x00000000,
        HCDMA3: 0x00000000,
        HCDMA4: 0x00000000,
        HCDMA5: 0x00000000,
        HCDMA6: 0x00000000,
        HCDMA7: 0x00000000,
        HCDMA8: 0x00000000,
        HCDMA9: 0x00000000,
        HCDMA10: 0x00000000,
        HCDMA11: 0x00000000,
    };
}

/// Raw pointer to OTG_HS_HOST
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_HS_HOST: *const RegisterBlock = 0x40040400 as *const _;
