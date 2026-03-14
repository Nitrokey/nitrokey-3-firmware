//! Timers, see Section 55 of RM0486.

use core::convert::Infallible;

use fugit::TimerInstantU32;
use stm32n6::stm32n657::{TIM6_S, TIM7_S};

use crate::{
    Rate,
    rcc::{ClockConfig, Peripheral, Rcc},
};

pub trait Tim {
    fn enable_counter(&self);
    fn disable_counter(&self);
    fn reset_counter(&self);
    fn counter(&self) -> u16;
    fn set_prescaler(&self, prescaler: u16);
    fn set_auto_reload(&self, auto_reload: u16);
    fn trigger_update(&self);
    fn interrupt_flag(&self) -> bool;
    fn clear_interrupt_flag(&self);
}

struct TimerConfig {
    prescaler: u16,
    auto_reload: u16,
}

impl TimerConfig {
    fn for_clock_and_target(clock: Rate, target: Rate) -> Self {
        let ticks = clock.to_Hz() / target.to_Hz();
        let prescaler = (ticks - 1) / (1 << 16);
        let auto_reload = ticks / (prescaler + 1) - 1;
        Self {
            prescaler: u16::try_from(prescaler).unwrap(),
            auto_reload: u16::try_from(auto_reload).unwrap(),
        }
    }
}

fn start_tim<T: Tim>(tim: &T, config: TimerConfig) {
    tim.disable_counter();
    tim.reset_counter();

    tim.set_prescaler(config.prescaler);
    tim.set_auto_reload(config.auto_reload);
    tim.trigger_update();

    tim.enable_counter();
}

/// A simple counter returning the elapsed time since start in [`Counter::now`][].
///
/// `F` determines the update rate of the counter and thus the precision of the produced values. To
/// make sure that counter overflows are handled correctly, users have to call [`Counter::now`][] or
/// [`Counter::update`][] at least once per overflow, either by handling the interrupt for the timer
/// or by just calling often.
pub struct Counter<const F: u32, T> {
    tim: T,
    overflows: u16,
}

impl<const F: u32, T: Tim> Counter<F, T> {
    pub fn new(tim: T, clock_config: ClockConfig) -> Self {
        // TODO: improve error handling
        let clock = clock_config.timg_ck();
        let prescaler = u16::try_from((clock.to_Hz() / F) - 1).unwrap();
        let config = TimerConfig {
            prescaler,
            auto_reload: u16::MAX,
        };
        start_tim(&tim, config);
        Self { tim, overflows: 0 }
    }

    pub fn now(&mut self) -> TimerInstantU32<F> {
        self.update();
        let ticks = u32::from(self.tim.counter());
        let overflows = u32::from(self.overflows);
        TimerInstantU32::from_ticks(overflows * u32::from(u16::MAX) + ticks)
    }

    pub fn update(&mut self) {
        if self.tim.interrupt_flag() {
            self.tim.clear_interrupt_flag();
            self.overflows += 1;
        }
    }
}

pub type MillisecondsCounter<T> = Counter<1_000, T>;

pub struct Timer<T> {
    tim: T,
    clock: Rate,
}

impl<T: Tim> Timer<T> {
    pub fn new(tim: T, clock_config: ClockConfig) -> Self {
        let clock = clock_config.timg_ck();
        Self { tim, clock }
    }

    pub fn start(&mut self, f: Rate) {
        start_tim(&self.tim, TimerConfig::for_clock_and_target(self.clock, f));
    }

    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        if self.tim.interrupt_flag() {
            self.tim.clear_interrupt_flag();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

macro_rules! impl_tim {
    ($Tim:ident, $TIM:ident) => {
        pub struct $Tim($TIM);

        impl $Tim {
            pub fn new(tim: $TIM, rcc: &Rcc) -> Self {
                rcc.enable(Peripheral::$Tim);
                Self(tim)
            }
        }

        impl Tim for $Tim {
            fn enable_counter(&self) {
                self.0.cr1().write(|w| w.cen().set_bit());
            }

            fn disable_counter(&self) {
                self.0.cr1().write(|w| w.cen().clear_bit());
            }

            fn reset_counter(&self) {
                self.0.cnt().reset();
            }

            fn counter(&self) -> u16 {
                self.0.cnt().read().cnt().bits()
            }

            fn set_prescaler(&self, psc: u16) {
                self.0.psc().write(|w| unsafe { w.psc().bits(psc) });
            }

            fn set_auto_reload(&self, arr: u16) {
                self.0.arr().write(|w| unsafe { w.arr().bits(arr.into()) });
            }

            fn trigger_update(&self) {
                self.0.cr1().modify(|_, w| w.urs().set_bit());
                self.0.egr().write(|w| w.ug().set_bit());
                self.0.cr1().modify(|_, w| w.urs().clear_bit());
            }

            fn interrupt_flag(&self) -> bool {
                self.0.sr().read().uif().bit()
            }

            fn clear_interrupt_flag(&self) {
                self.0.sr().modify(|_, w| w.uif().clear_bit());
            }
        }
    };
}

impl_tim!(Tim6, TIM6_S);
impl_tim!(Tim7, TIM7_S);

#[cfg(test)]
mod test {
    use super::TimerConfig;
    use crate::Rate;

    #[test]
    fn test_timer_config() {
        let clock = Rate::MHz(64);
        let f = Rate::Hz(1);

        let config = TimerConfig::for_clock_and_target(clock, f);
        let prescaler = u32::from(config.prescaler);
        let auto_reload = u32::from(config.auto_reload);
        let result = clock / (prescaler + 1) / (auto_reload + 1);
        assert_eq!(result, f);
    }
}
