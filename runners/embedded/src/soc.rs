#[cfg(feature = "soc-lpc55")]
pub mod lpc55;

#[cfg(feature = "soc-nrf52840")]
pub mod nrf52840;

pub fn set_panic_led() {
    #[cfg(feature = "soc-lpc55")]
    lpc55::board::set_panic_led();
    #[cfg(feature = "soc-nrf52840")]
    nrf52840::board::set_panic_led();
}
