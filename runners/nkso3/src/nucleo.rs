use core::sync::atomic::{AtomicBool, Ordering};

use boards::ui::rgb_led::RgbLed;
use embedded_hal::digital::v2::{OutputPin as _, PinState};
use stm32n6::stm32n657::GPIOG_S;
use stm32n657_hal::gpio::{GpioG, OutputMode, PinG0, PinG10, PinG8};

static PANIC_LED_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct Led {
    red: PinG10<'static, OutputMode>,
    green: PinG0<'static, OutputMode>,
    blue: PinG8<'static, OutputMode>,
}

impl Led {
    pub fn new(gpiog: &'static GpioG) -> Self {
        let mut red = PinG10::new(gpiog).into_push_pull_output();
        let mut green = PinG0::new(gpiog).into_push_pull_output();
        let mut blue = PinG8::new(gpiog).into_push_pull_output();
        red.set_high().ok();
        green.set_high().ok();
        blue.set_high().ok();
        PANIC_LED_INITIALIZED.store(true, Ordering::Relaxed);
        Self { red, green, blue }
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
