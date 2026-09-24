//! Power control (PWR), see Section 13 of RM0486.

use stm32n6::stm32n657::PWR_S;

pub struct Pwr(PWR_S);

impl Pwr {
    pub fn new(pwr: PWR_S) -> Self {
        Self(pwr)
    }

    /// Validates the VDD33USB supply, required before using the USB HS PHYs.
    pub fn enable_usb_supply(&self) {
        self.0.svmcr3().modify(|_, w| w.usb33vmen().set_bit());
        while self.0.svmcr3().read().usb33rdy().bit_is_clear() {}
        self.0.svmcr3().modify(|_, w| w.usb33sv().set_bit());
    }

    pub fn enable_mmc_vddio(&self) {
        self.0.svmcr2().modify(|_, w| w.vddio5vmen().set_bit());
        self.0.svmcr1().modify(|_, w| w.vddio4vmen().set_bit());
        while self.0.svmcr2().read().vddio5rdy().bit_is_clear() {}
        while self.0.svmcr1().read().vddio4rdy().bit_is_clear() {}
        self.0.svmcr2().modify(|_, w| w.vddio5sv().set_bit());
        self.0.svmcr1().modify(|_, w| w.vddio4sv().set_bit());
    }
}
