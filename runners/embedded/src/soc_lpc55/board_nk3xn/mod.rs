pub mod button;
pub mod led;

pub use led::set_panic_led;

pub const BOARD_NAME: &'static str = "nk3xn";

pub type PwmTimer =
    lpc55_hal::peripherals::ctimer::Ctimer3<lpc55_hal::typestates::init_state::Unknown>;
pub type ButtonsTimer =
    lpc55_hal::peripherals::ctimer::Ctimer1<lpc55_hal::typestates::init_state::Unknown>;
