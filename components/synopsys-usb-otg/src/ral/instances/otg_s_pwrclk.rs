#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go full speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_fs_pwrclk::Instance;
pub use super::super::peripherals::otg_fs_pwrclk::PCGCCTL;
pub use super::super::peripherals::otg_fs_pwrclk::{RegisterBlock, ResetValues};

/// Access functions for the OTG_FS_PWRCLK peripheral instance
pub mod OTG_FS_PWRCLK {
    use super::ResetValues;

    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x50000e00,
        _marker: ::core::marker::PhantomData,
    };

    /// Reset values for each field in OTG_FS_PWRCLK
    pub const reset: ResetValues = ResetValues {
        PCGCCTL: 0x00000000,
    };
}

/// Raw pointer to OTG_FS_PWRCLK
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_FS_PWRCLK: *const RegisterBlock = 0x50000e00 as *const _;

/// Access functions for the OTG_HS_PWRCLK peripheral instance
pub mod OTG_HS_PWRCLK {
    use super::ResetValues;

    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x40040e00,
        _marker: ::core::marker::PhantomData,
    };

    /// Reset values for each field in OTG_HS_PWRCLK
    pub const reset: ResetValues = ResetValues {
        PCGCCTL: 0x00000000,
    };
}

/// Raw pointer to OTG_HS_PWRCLK
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_HS_PWRCLK: *const RegisterBlock = 0x40040e00 as *const _;
