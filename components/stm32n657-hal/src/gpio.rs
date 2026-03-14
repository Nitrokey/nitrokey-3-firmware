//! General-purpose I/Os (GPIO), see Section 15 of RM0486.

use core::convert::Infallible;

use embedded_hal::digital::v2::OutputPin;
use stm32n6::stm32n657::{GPIOG_S, RCC};

pub struct GpioG(GPIOG_S);

impl GpioG {
    pub fn new(gpiog: GPIOG_S, rcc: &RCC) -> Self {
        rcc.ahb4ensr().write(|w| w.gpiogens().set_bit());
        Self(gpiog)
    }
}

pub trait PinMode {}

pub struct NoMode;

impl PinMode for NoMode {}

pub struct OutputMode;

impl PinMode for OutputMode {}

macro_rules! impl_pin {
    ($gpio:ident, $pin:ident, $mode:ident, $ot:ident, $bs:ident, $br:ident) => {
        pub struct $pin<'a, M: PinMode> {
            gpio: &'a $gpio,
            _mode: M,
        }

        impl<'a> $pin<'a, NoMode> {
            pub fn new(gpio: &'a $gpio) -> Self {
                Self {
                    gpio,
                    _mode: NoMode,
                }
            }

            pub fn into_push_pull_output(self) -> $pin<'a, OutputMode> {
                // mode: 01 = general-purpose output mode
                self.gpio
                    .0
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b01) });
                // ot: 0 = output push-pull
                self.gpio.0.otyper().modify(|_, w| w.$ot().clear_bit());
                $pin {
                    gpio: self.gpio,
                    _mode: OutputMode,
                }
            }
        }

        impl OutputPin for $pin<'_, OutputMode> {
            type Error = Infallible;

            fn set_high(&mut self) -> Result<(), Self::Error> {
                self.gpio.0.bsrr().write(|w| w.$bs().set_bit());
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), Self::Error> {
                self.gpio.0.bsrr().write(|w| w.$br().set_bit());
                Ok(())
            }
        }
    };
}

impl_pin!(GpioG, PinG0, mode0, ot0, bs0, br0);
impl_pin!(GpioG, PinG8, mode8, ot8, bs8, br8);
impl_pin!(GpioG, PinG10, mode10, ot10, bs10, br10);
