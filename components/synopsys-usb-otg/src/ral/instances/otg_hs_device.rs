#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446

#[cfg(not(feature = "nosync"))]
pub use super::super::peripherals::otg_hs_device::Instance;
pub use super::super::peripherals::otg_hs_device::{RegisterBlock, ResetValues};
pub use super::super::peripherals::otg_hs_device::{
    DAINT, DAINTMSK, DCFG, DCTL, DEACHINT, DEACHINTMSK, DIEPCTL0, DIEPCTL1, DIEPCTL2, DIEPCTL3,
    DIEPCTL4, DIEPCTL5, DIEPDMA1, DIEPDMA2, DIEPDMA3, DIEPDMA4, DIEPDMA5, DIEPEACHMSK1, DIEPEMPMSK,
    DIEPINT0, DIEPINT1, DIEPINT2, DIEPINT3, DIEPINT4, DIEPINT5, DIEPMSK, DIEPTSIZ0, DIEPTSIZ1,
    DIEPTSIZ2, DIEPTSIZ3, DIEPTSIZ4, DIEPTSIZ5, DOEPCTL0, DOEPCTL1, DOEPCTL2, DOEPCTL3, DOEPCTL4,
    DOEPCTL5, DOEPEACHMSK1, DOEPINT0, DOEPINT1, DOEPINT2, DOEPINT3, DOEPINT4, DOEPINT5, DOEPMSK,
    DOEPTSIZ0, DOEPTSIZ1, DOEPTSIZ2, DOEPTSIZ3, DOEPTSIZ4, DOEPTSIZ5, DSTS, DTHRCTL, DTXFSTS0,
    DTXFSTS1, DTXFSTS2, DTXFSTS3, DTXFSTS4, DTXFSTS5, DVBUSDIS, DVBUSPULSE,
};

/// Access functions for the OTG_HS_DEVICE peripheral instance
pub mod OTG_HS_DEVICE {
    use super::ResetValues;

    #[cfg(not(feature = "nosync"))]
    use super::Instance;

    #[cfg(not(feature = "nosync"))]
    const INSTANCE: Instance = Instance {
        addr: 0x40040800,
        _marker: ::core::marker::PhantomData,
    };

    /// Reset values for each field in OTG_HS_DEVICE
    pub const reset: ResetValues = ResetValues {
        DCFG: 0x02200000,
        DCTL: 0x00000000,
        DSTS: 0x00000010,
        DIEPMSK: 0x00000000,
        DOEPMSK: 0x00000000,
        DAINT: 0x00000000,
        DAINTMSK: 0x00000000,
        DVBUSDIS: 0x000017D7,
        DVBUSPULSE: 0x000005B8,
        DTHRCTL: 0x00000000,
        DIEPEMPMSK: 0x00000000,
        DEACHINT: 0x00000000,
        DEACHINTMSK: 0x00000000,
        DIEPEACHMSK1: 0x00000000,
        DOEPEACHMSK1: 0x00000000,
        DIEPCTL0: 0x00000000,
        DIEPCTL1: 0x00000000,
        DIEPCTL2: 0x00000000,
        DIEPCTL3: 0x00000000,
        DIEPCTL4: 0x00000000,
        DIEPCTL5: 0x00000000,
        DIEPINT0: 0x00000080,
        DIEPINT1: 0x00000080,
        DIEPINT2: 0x00000080,
        DIEPINT3: 0x00000080,
        DIEPINT4: 0x00000080,
        DIEPINT5: 0x00000080,
        DIEPTSIZ0: 0x00000000,
        DIEPDMA1: 0x00000000,
        DIEPDMA2: 0x00000000,
        DIEPDMA3: 0x00000000,
        DIEPDMA4: 0x00000000,
        DIEPDMA5: 0x00000000,
        DTXFSTS0: 0x00000000,
        DTXFSTS1: 0x00000000,
        DTXFSTS2: 0x00000000,
        DTXFSTS3: 0x00000000,
        DTXFSTS4: 0x00000000,
        DTXFSTS5: 0x00000000,
        DIEPTSIZ1: 0x00000000,
        DIEPTSIZ2: 0x00000000,
        DIEPTSIZ3: 0x00000000,
        DIEPTSIZ4: 0x00000000,
        DIEPTSIZ5: 0x00000000,
        DOEPCTL0: 0x00008000,
        DOEPCTL1: 0x00000000,
        DOEPCTL2: 0x00000000,
        DOEPCTL3: 0x00000000,
        DOEPCTL4: 0x00000000,
        DOEPCTL5: 0x00000000,
        DOEPINT0: 0x00000080,
        DOEPINT1: 0x00000080,
        DOEPINT2: 0x00000080,
        DOEPINT3: 0x00000080,
        DOEPINT4: 0x00000080,
        DOEPINT5: 0x00000080,
        DOEPTSIZ0: 0x00000000,
        DOEPTSIZ1: 0x00000000,
        DOEPTSIZ2: 0x00000000,
        DOEPTSIZ3: 0x00000000,
        DOEPTSIZ4: 0x00000000,
        DOEPTSIZ5: 0x00000000,
    };
}

/// Raw pointer to OTG_HS_DEVICE
///
/// Dereferencing this is unsafe because you are not ensured unique
/// access to the peripheral, so you may encounter data races with
/// other users of this peripheral. It is up to you to ensure you
/// will not cause data races.
///
/// This constant is provided for ease of use in unsafe code: you can
/// simply call for example `write_reg!(gpio, GPIOA, ODR, 1);`.
pub const OTG_HS_DEVICE: *const RegisterBlock = 0x40040800 as *const _;
