use core::sync::atomic::{AtomicBool, Ordering};

use embedded_hal::digital::v2::{OutputPin as _, PinState};
use stm32n6::stm32n657::GPIOG_S;
use stm32n657_hal::gpio::{Output, PinB10, PinG1, PinG10, PushPull};
use trussed_core::types::consent;

use crate::ui::{
    buttons::UserPresence,
    rgb_led::{Intensities, RgbLed},
};

static PANIC_LED_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub type RedLedPin<M = Output<PushPull>> = PinG10<M>;
pub type GreenLedPin<M = Output<PushPull>> = PinG1<M>;
pub type BlueLedPin<M = Output<PushPull>> = PinB10<M>;

/// RGB LED D1, active low: any non-zero intensity turns the LED on.
pub struct Led {
    red: RedLedPin,
    green: GreenLedPin,
    blue: BlueLedPin,
}

impl Led {
    pub fn new(red: RedLedPin, green: GreenLedPin, blue: BlueLedPin) -> Self {
        PANIC_LED_INITIALIZED.store(true, Ordering::Relaxed);
        let mut led = Self { red, green, blue };
        led.set(Intensities::from(0));
        led
    }

    pub fn init<MR, MG, MB>(
        red: RedLedPin<MR>,
        green: GreenLedPin<MG>,
        blue: BlueLedPin<MB>,
    ) -> Self {
        Self::new(
            red.into_push_pull_output(),
            green.into_push_pull_output(),
            blue.into_push_pull_output(),
        )
    }
}

impl RgbLed for Led {
    fn set_panic_led() {
        if PANIC_LED_INITIALIZED.load(Ordering::Relaxed) {
            // SAFETY: only reached from the panic and hard fault handlers.
            unsafe {
                GPIOG_S::steal().bsrr().write(|w| w.br10().set_bit());
            }
        }
    }

    fn red(&mut self, intensity: u8) {
        self.red.set_state(led_pin_state(intensity)).ok();
    }

    fn green(&mut self, intensity: u8) {
        self.green.set_state(led_pin_state(intensity)).ok();
    }

    fn blue(&mut self, intensity: u8) {
        self.blue.set_state(led_pin_state(intensity)).ok();
    }
}

fn led_pin_state(intensity: u8) -> PinState {
    PinState::from(intensity == 0)
}

pub struct DummyButton;

impl UserPresence for DummyButton {
    fn check_user_presence(&mut self) -> consent::Level {
        consent::Level::Normal
    }
}
