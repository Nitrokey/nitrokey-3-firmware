//! USB peripheral driver for Synopsys USB OTG peripherals.

#![no_std]

#[cfg(all(feature = "fs", feature = "hs"))]
compile_error!("choose only one USB mode");

#[cfg(not(any(feature = "fs", feature = "hs")))]
compile_error!("select USB mode feature (fs/hs)");

mod endpoint;
mod endpoint_memory;

mod target;

/// USB peripheral driver.
pub mod bus;

pub use crate::bus::UsbBus;

mod ral;
mod transition;

/// A trait for device-specific USB peripherals. Implement this to add support for a new hardware
/// platform. Peripherals that have this trait must have the same register block as STM32 USB OTG
/// peripherals.
pub unsafe trait UsbPeripheral: Send + Sync {
    /// Pointer to the register block
    const REGISTERS: *const ();

    /// true for High Speed variants of the peripheral, false for Full Speed
    const HIGH_SPEED: bool;

    /// FIFO size in 32-bit words
    const FIFO_DEPTH_WORDS: usize;

    /// Number of (bidirectional) endpoints
    const ENDPOINT_COUNT: usize;

    /// Enables USB device on its peripheral bus
    fn enable();

    /// AHB frequency in hertz
    fn ahb_frequency_hz(&self) -> u32;

    /// Returns PHY type that should be used for USB peripheral
    fn phy_type(&self) -> PhyType {
        PhyType::InternalFullSpeed
    }

    /// Performs initial setup of the internal high-speed PHY
    ///
    /// This function should turn on LDO and PLL and wait for PHY clock to become stable.
    fn setup_internal_hs_phy(&self) {}
}

/// USB PHY type
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PhyType {
    /// Internal Full-Speed PHY
    ///
    /// Available on most High-Speed peripherals.
    InternalFullSpeed,
    /// Internal High-Speed PHY
    ///
    /// Available on a few STM32 chips.
    InternalHighSpeed,
    /// External ULPI High-Speed PHY
    ExternalHighSpeed,
}
