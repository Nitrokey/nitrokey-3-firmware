#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_hs_global::Instance;
pub use super::super::peripherals::otg_hs_global::{RegisterBlock, ResetValues};
pub use super::super::peripherals::otg_hs_global::{
    CID, DIEPTXF1, DIEPTXF2, DIEPTXF3, DIEPTXF4, DIEPTXF5, GAHBCFG, GCCFG, GINTMSK, GINTSTS,
    GNPTXFSIZ, GNPTXSTS, GOTGCTL, GOTGINT, GRSTCTL, GRXFSIZ, GRXSTSP, GRXSTSR, GUSBCFG, HPTXFSIZ,
    PHYCR,
};

/// Access functions for the OTG_HS_GLOBAL peripheral instance
pub mod OTG_HS_GLOBAL {
    use super::ResetValues;

    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x40040000,
        _marker: ::core::marker::PhantomData,
    };

    /// Reset values for each field in OTG_HS_GLOBAL
    pub const reset: ResetValues = ResetValues {
        GOTGCTL: 0x00000800,
        GOTGINT: 0x00000000,
        GAHBCFG: 0x00000000,
        GUSBCFG: 0x00000A00,
        GRSTCTL: 0x20000000,
        GINTSTS: 0x04000020,
        GINTMSK: 0x00000000,
        GRXSTSR: 0x00000000,
        GRXSTSP: 0x00000000,
        GRXFSIZ: 0x00000200,
        GNPTXFSIZ: 0x00000200,
        GNPTXSTS: 0x00080200,
        PHYCR: 0x00000000,
        GCCFG: 0x00000000,
        CID: 0x00001200,
        HPTXFSIZ: 0x02000600,
        DIEPTXF1: 0x02000400,
        DIEPTXF2: 0x02000400,
        DIEPTXF3: 0x02000400,
        DIEPTXF4: 0x02000400,
        DIEPTXF5: 0x02000400,
    };
}

/// Raw pointer to OTG_HS_GLOBAL
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_HS_GLOBAL: *const RegisterBlock = 0x40040000 as *const _;
