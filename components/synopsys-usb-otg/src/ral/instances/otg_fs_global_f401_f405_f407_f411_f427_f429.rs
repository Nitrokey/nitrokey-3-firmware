#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go full speed
//!
//! Used by: stm32f401, stm32f405, stm32f407, stm32f411, stm32f427, stm32f429

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_fs_global_v1::Instance;
pub use super::super::peripherals::otg_fs_global_v1::{RegisterBlock, ResetValues};
pub use super::super::peripherals::otg_fs_global_v1::{
    CID, DIEPTXF0, DIEPTXF1, DIEPTXF2, DIEPTXF3, GAHBCFG, GCCFG, GINTMSK, GINTSTS, GNPTXSTS,
    GOTGCTL, GOTGINT, GRSTCTL, GRXFSIZ, GRXSTSP, GRXSTSR, GUSBCFG, HPTXFSIZ,
};

/// Access functions for the OTG_FS_GLOBAL peripheral instance
pub mod OTG_FS_GLOBAL {
    use super::ResetValues;

    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x50000000,
        _marker: ::core::marker::PhantomData,
    };

    /// Reset values for each field in OTG_FS_GLOBAL
    pub const reset: ResetValues = ResetValues {
        GOTGCTL: 0x00000800,
        GOTGINT: 0x00000000,
        GAHBCFG: 0x00000000,
        GUSBCFG: 0x00000A00,
        GRSTCTL: 0x20000000,
        GINTSTS: 0x04000020,
        GINTMSK: 0x00000000,
        GRXSTSR: 0x00000000,
        GRXFSIZ: 0x00000200,
        DIEPTXF0: 0x00000200,
        GNPTXSTS: 0x00080200,
        GCCFG: 0x00000000,
        CID: 0x00001000,
        HPTXFSIZ: 0x02000600,
        DIEPTXF1: 0x02000400,
        DIEPTXF2: 0x02000400,
        DIEPTXF3: 0x02000400,
        GRXSTSP: 0x00000000,
    };
}

/// Raw pointer to OTG_FS_GLOBAL
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_FS_GLOBAL: *const RegisterBlock = 0x50000000 as *const _;
