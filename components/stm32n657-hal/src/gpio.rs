//! General-purpose I/Os (GPIO), see Section 15 of RM0486.

use core::{convert::Infallible, marker::PhantomData};

use embedded_hal::digital::v2::{InputPin, OutputPin};
use stm32n6::stm32n657::{GPIOC_S, GPIOG_S};

use crate::rcc::{Peripheral, Rcc};

pub struct Input<M> {
    _marker: PhantomData<M>,
}

pub struct Floating;

pub struct PullDown;

pub struct Output<M> {
    _marker: PhantomData<M>,
}

pub struct PushPull;

macro_rules! impl_gpio {
    ($gpio:ident, $GPIO:ident, [
        $($pin:ident: $Pin:ident = ($mode:ident, $ot:ident, $pupd:ident, $id:ident, $bs:ident, $br:ident),)*
    ]) => {
        pub struct $gpio {
            $(
                pub $pin: $Pin<Input<Floating>>,
            )*
        }

        impl $gpio {
            pub fn new(gpio: $GPIO, rcc: &Rcc) -> Self {
                let _ = gpio;
                rcc.enable(Peripheral::$gpio);
                Self {
                    $(
                        $pin: $Pin {
                            _marker: Default::default(),
                        },
                    )*
                }
            }
        }

        $(
            impl_pin!($GPIO, $Pin, $mode, $ot, $pupd, $id, $bs, $br);
        )*
    }
}

macro_rules! impl_pin {
    ($GPIO:ident, $pin:ident, $mode:ident, $ot:ident, $pupd:ident, $id:ident, $bs:ident, $br:ident) => {
        pub struct $pin<M> {
            _marker: PhantomData<M>,
        }

        impl<M> $pin<M> {
            fn gpio(&self) -> $GPIO {
                // SAFETY: This struct can only be constructed by consuming the peripheral, so
                // there can be no other instances accessing the same pin.
                unsafe { $GPIO::steal() }
            }

            pub fn into_pull_down_input(self) -> $pin<Input<PullDown>> {
                // mode: 00 = general-purpose input mode
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b00) });
                // pupd: 10 = pull-down
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(0b10) });
                $pin {
                    _marker: Default::default(),
                }
            }

            pub fn into_push_pull_output(self) -> $pin<Output<PushPull>> {
                // mode: 01 = general-purpose output mode
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b01) });
                // ot: 0 = output push-pull
                self.gpio().otyper().modify(|_, w| w.$ot().clear_bit());
                // pp: 00 = no pull-up, pull-down
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(0b00) });
                $pin {
                    _marker: Default::default(),
                }
            }
        }

        impl<M> $pin<Input<M>> {
            fn input(&self) -> bool {
                self.gpio().idr().read().$id().bit()
            }
        }

        impl<M> InputPin for $pin<Input<M>> {
            type Error = Infallible;

            fn is_high(&self) -> Result<bool, Self::Error> {
                Ok(self.input())
            }

            fn is_low(&self) -> Result<bool, Self::Error> {
                Ok(!self.input())
            }
        }

        impl<M> OutputPin for $pin<Output<M>> {
            type Error = Infallible;

            fn set_high(&mut self) -> Result<(), Self::Error> {
                self.gpio().bsrr().write(|w| w.$bs().set_bit());
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), Self::Error> {
                self.gpio().bsrr().write(|w| w.$br().set_bit());
                Ok(())
            }
        }
    };
}

impl_gpio!(GpioC, GPIOC_S, [
    c13: PinC13 = (mode13, ot13, pupd13, id13, bs13, br13),
]);
impl_gpio!(GpioG, GPIOG_S, [
    g0: PinG0 = (mode0, ot0, pupd0, id0, bs0, br0),
    g8: PinG8 = (mode8, ot8, pupd8, id8, bs8, br8),
    g10: PinG10 = (mode10, ot10, pupd10, id10, bs10, br10),
]);
