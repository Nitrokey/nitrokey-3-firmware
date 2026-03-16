use core::sync::atomic::{AtomicBool, Ordering};

use boards::ui::{
    buttons::UserPresence,
    rgb_led::{Intensities, RgbLed},
};
use embedded_hal::digital::v2::{InputPin as _, OutputPin as _, PinState};
use stm32n6::stm32n657::GPIOG_S;
use stm32n657_hal::gpio::{Input, Output, PinC13, PinG0, PinG10, PinG8, PullDown, PushPull};
use trussed::platform::consent;

static PANIC_LED_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub type RedLedPin<M = Output<PushPull>> = PinG10<M>;
pub type GreenLedPin<M = Output<PushPull>> = PinG0<M>;
pub type BlueLedPin<M = Output<PushPull>> = PinG8<M>;

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
        let red = red.into_push_pull_output();
        let green = green.into_push_pull_output();
        let blue = blue.into_push_pull_output();
        Self::new(red, green, blue)
    }
}

impl RgbLed for Led {
    fn set_panic_led() {
        if PANIC_LED_INITIALIZED.load(Ordering::Relaxed) {
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

pub type ButtonPin<M = Input<PullDown>> = PinC13<M>;

pub struct Button(ButtonPin);

impl Button {
    pub fn new(pin: ButtonPin) -> Self {
        Self(pin)
    }

    pub fn init<M>(pin: ButtonPin<M>) -> Self {
        Self::new(pin.into_pull_down_input())
    }
}

impl UserPresence for Button {
    fn check_user_presence(&mut self) -> consent::Level {
        if self.0.is_high().unwrap_or(false) {
            consent::Level::Normal
        } else {
            consent::Level::None
        }
    }
}
