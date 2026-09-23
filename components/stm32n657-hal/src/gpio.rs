//! General-purpose I/Os (GPIO), see Section 15 of RM0486.

use core::{convert::Infallible, marker::PhantomData};

use embedded_hal::digital::v2::{InputPin, OutputPin};
use stm32n6::stm32n657::{GPIOA_S, GPIOB_S, GPIOC_S, GPIOD_S, GPIOE_S, GPIOG_S, GPIOH_S};

use crate::rcc::{Peripheral, Rcc};

pub struct Input<M> {
    _marker: PhantomData<M>,
}

pub struct Floating;

pub struct PullDown;
pub struct PullUp;

trait PullResistor {
    const VALUE: u8;
}

impl PullResistor for Floating {
    const VALUE: u8 = 0b00;
}
impl PullResistor for PullDown {
    const VALUE: u8 = 0b10;
}
impl PullResistor for PullUp {
    const VALUE: u8 = 0b01;
}

pub struct Output<M> {
    _marker: PhantomData<M>,
}

pub struct Alternate<M, const F: u8> {
    _marker: PhantomData<M>,
}

pub struct PushPull;

macro_rules! impl_gpio {
    ($gpio:ident, $GPIO:ident, [
        $($pin:ident: $Pin:ident = ($mode:ident, $ot:ident, $pupd:ident, $id:ident, $bs:ident, $br:ident, $afr:ident, $afsel:ident),)*
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
            impl_pin!($GPIO, $Pin, $mode, $ot, $pupd, $id, $bs, $br, $afr, $afsel);
        )*
    }
}

macro_rules! impl_pin {
    ($GPIO:ident, $pin:ident, $mode:ident, $ot:ident, $pupd:ident, $id:ident, $bs:ident, $br:ident, $afr:ident, $afsel:ident) => {
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

            pub fn into_pull_up_input(self) -> $pin<Input<PullUp>> {
                // mode: 00 = general-purpose input mode
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b00) });
                // pupd: 01 = pull-up
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(0b01) });
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

            #[allow(unused)]
            fn into_alternate<R: PullResistor, const F: u8>(self) -> $pin<Alternate<R, F>> {
                const {
                    assert!(F <= 0xF);
                };
                // mode: 10 = alternate function
                self.gpio()
                    .moder()
                    .modify(|_, w| unsafe { w.$mode().bits(0b10) });
                self.gpio()
                    .pupdr()
                    .modify(|_, w| unsafe { w.$pupd().bits(R::VALUE) });
                self.gpio()
                    .$afr()
                    .modify(|_, w| unsafe { w.$afsel().bits(F) });

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

impl_gpio!(GpioA, GPIOA_S, [
    a0: PinA0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    a4: PinA4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
]);
impl_gpio!(GpioB, GPIOB_S, [
    b4: PinB4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
    b5: PinB5 = (mode5, ot5, pupd5, id5, bs5, br5, afrl,afsel5),
    b8: PinB8 = (mode8, ot8, pupd8, id8, bs8, br8, afrh,afsel8),
    b9: PinB9 = (mode9, ot9, pupd9, id9, bs9, br9, afrh,afsel9),
    b13: PinB13 = (mode13, ot13, pupd13, id13, bs13, br13, afrh,afsel13),
]);
impl_gpio!(GpioC, GPIOC_S, [
    c0: PinC0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    c1: PinC1 = (mode1, ot1, pupd1, id1, bs1, br1, afrl,afsel1),
    c2: PinC2 = (mode2, ot2, pupd2, id2, bs2, br2, afrl,afsel2),
    c3: PinC3 = (mode3, ot3, pupd3, id3, bs3, br3, afrl,afsel3),
    c4: PinC4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
    c5: PinC5 = (mode5, ot5, pupd5, id5, bs5, br5, afrl,afsel5),
    c6: PinC6 = (mode6, ot6, pupd6, id6, bs6, br6, afrl,afsel6),
    c7: PinC7 = (mode7, ot7, pupd7, id7, bs7, br7, afrl,afsel7),
    c8: PinC8 = (mode8, ot8, pupd8, id8, bs8, br8, afrh,afsel8),
    c9: PinC9 = (mode9, ot9, pupd9, id9, bs9, br9, afrh,afsel9),
    c10: PinC10 = (mode10, ot10, pupd10, id10, bs10, br10, afrh,afsel10),
    c11: PinC11 = (mode11, ot11, pupd11, id11, bs11, br11, afrh,afsel11),
    c12: PinC12 = (mode12, ot12, pupd12, id12, bs12, br12, afrh,afsel12),
    c13: PinC13 = (mode13, ot13, pupd13, id13, bs13, br13, afrh,afsel13),
]);
impl_gpio!(GpioD, GPIOD_S, [
    d0: PinD0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    d2: PinD2 = (mode2, ot2, pupd2, id2, bs2, br2, afrl,afsel2),
    d4: PinD4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
    d5: PinD5 = (mode5, ot5, pupd5, id5, bs5, br5, afrl,afsel5),
    d11: PinD11 = (mode11, ot11, pupd11, id11, bs11, br11, afrh,afsel11),
    d15: PinD15 = (mode15, ot15, pupd15, id15, bs15, br15, afrh,afsel15),
]);
impl_gpio!(GpioE, GPIOE_S, [
    e0: PinE0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    e4: PinE4 = (mode4, ot4, pupd4, id4, bs4, br4, afrl,afsel4),
    e15: PinE15 = (mode15, ot15, pupd15, id15, bs15, br15, afrh,afsel15),
]);
impl_gpio!(GpioG, GPIOG_S, [
    g0: PinG0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    g1: PinG1 = (mode1, ot1, pupd1, id1, bs1, br1, afrl,afsel1),
    g7: PinG7 = (mode7, ot7, pupd7, id7, bs7, br7, afrl,afsel7),
    g8: PinG8 = (mode8, ot8, pupd8, id8, bs8, br8, afrh,afsel8),
    g10: PinG10 = (mode10, ot10, pupd10, id10, bs10, br10, afrh,afsel10),
]);
impl_gpio!(GpioH, GPIOH_S, [
    h0: PinH0 = (mode0, ot0, pupd0, id0, bs0, br0, afrl,afsel0),
    h2: PinH2 = (mode2, ot2, pupd2, id2, bs2, br2, afrl,afsel2),
    h8: PinH8 = (mode8, ot8, pupd8, id8, bs8, br8, afrh,afsel8),
    h9: PinH9 = (mode9, ot9, pupd9, id9, bs9, br9, afrh,afsel9),
]);

pub const ALTERNATE_FUNCTION_0: u8 = 0x0000000;
pub const ALTERNATE_FUNCTION_1: u8 = 0x0000001;
pub const ALTERNATE_FUNCTION_2: u8 = 0x0000002;
pub const ALTERNATE_FUNCTION_3: u8 = 0x0000003;
pub const ALTERNATE_FUNCTION_4: u8 = 0x0000004;
pub const ALTERNATE_FUNCTION_5: u8 = 0x0000005;
pub const ALTERNATE_FUNCTION_6: u8 = 0x0000006;
pub const ALTERNATE_FUNCTION_7: u8 = 0x0000007;
pub const ALTERNATE_FUNCTION_8: u8 = 0x0000008;
pub const ALTERNATE_FUNCTION_9: u8 = 0x0000009;
pub const ALTERNATE_FUNCTION_10: u8 = 0x000000A;
pub const ALTERNATE_FUNCTION_11: u8 = 0x000000B;
pub const ALTERNATE_FUNCTION_12: u8 = 0x000000C;
pub const ALTERNATE_FUNCTION_13: u8 = 0x000000D;
pub const ALTERNATE_FUNCTION_14: u8 = 0x000000E;
pub const ALTERNATE_FUNCTION_15: u8 = 0x000000F;

macro_rules! alternate_functions {
    ($($pin:ident: $function:ident($resistor:ident, $alternate:ident)),* $(,)?) => {
        $(
            impl<M> $pin<M> {
                pub fn $function(self) -> $pin<Alternate<$resistor, $alternate>> {
                    self.into_alternate()
                }
            }
        )*
    };
}

alternate_functions!(
    PinA0: into_sdmmc2_cmd(PullUp, ALTERNATE_FUNCTION_11),
    PinB4: into_sdmmc2_d3(PullUp, ALTERNATE_FUNCTION_11),
    PinB8: into_sdmmc2_d0(PullUp, ALTERNATE_FUNCTION_11),
    PinB9: into_sdmmc2_d2(PullUp, ALTERNATE_FUNCTION_11),
    PinB13: into_sdmmc2_d6(PullUp, ALTERNATE_FUNCTION_11),
    PinC0: into_sdmmc2_d2(PullUp, ALTERNATE_FUNCTION_11),
    PinC1: into_sdmmc2_d5(PullUp, ALTERNATE_FUNCTION_11),
    PinC2: into_sdmmc2_ck(PullUp, ALTERNATE_FUNCTION_11),
    PinC3: into_sdmmc2_cmd(PullUp, ALTERNATE_FUNCTION_11),
    PinC4: into_sdmmc2_d0(PullUp, ALTERNATE_FUNCTION_11),
    PinC5: into_sdmmc2_d1(PullUp, ALTERNATE_FUNCTION_11),
    PinC6: into_sdmmc1_d6(PullUp, ALTERNATE_FUNCTION_10),
    PinC6: into_sdmmc2_d6(PullUp, ALTERNATE_FUNCTION_11),
    PinC6: into_sdmmc1_d0dir(PullUp, ALTERNATE_FUNCTION_12),
    PinC7: into_sdmmc1_d7(PullUp, ALTERNATE_FUNCTION_10),
    PinC7: into_sdmmc2_d7(PullUp, ALTERNATE_FUNCTION_11),
    PinC7: into_sdmmc1_d123dir(PullUp, ALTERNATE_FUNCTION_12),
    PinC8: into_sdmmc1_d0(PullUp, ALTERNATE_FUNCTION_10),
    PinC9: into_sdmmc1_d1(PullUp, ALTERNATE_FUNCTION_10),
    PinC10: into_sdmmc1_d2(PullUp, ALTERNATE_FUNCTION_10),
    PinC11: into_sdmmc1_d3(PullUp, ALTERNATE_FUNCTION_10),
    PinC12: into_sdmmc1_ck(PullUp, ALTERNATE_FUNCTION_10),
    PinD2: into_sdmmc2_ck(PullUp, ALTERNATE_FUNCTION_11),
    PinD5: into_sdmmc2_d7(PullUp, ALTERNATE_FUNCTION_11),
    PinD11: into_sdmmc1_d0(PullUp, ALTERNATE_FUNCTION_10),
    PinD15: into_sdmmc1_d0(PullUp, ALTERNATE_FUNCTION_10),
    PinE4: into_sdmmc2_d3(PullUp, ALTERNATE_FUNCTION_11),
    PinE15: into_sdmmc1_d0(PullUp, ALTERNATE_FUNCTION_11),
    PinG8: into_sdmmc2_d1(PullUp, ALTERNATE_FUNCTION_11),
    PinH2: into_sdmmc1_cmd(PullUp, ALTERNATE_FUNCTION_10),
    PinH8: into_sdmmc2_d1(PullUp, ALTERNATE_FUNCTION_11),
    PinH9: into_sdmmc1_d4(PullUp, ALTERNATE_FUNCTION_10),
    PinH9: into_sdmmc2_d4(PullUp, ALTERNATE_FUNCTION_11),
    PinH9: into_sdmmc1_ckin(PullUp, ALTERNATE_FUNCTION_12),
);
