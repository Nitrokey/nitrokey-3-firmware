//! Timers, see Section 55 of RM0486.

use core::convert::Infallible;

use stm32n6::stm32n657::TIM6_S;

use crate::{
    Rate,
    rcc::{ClockConfig, Peripheral, Rcc},
};

pub trait Tim {
    fn enable_counter(&self);
    fn disable_counter(&self);
    fn reset_counter(&self);
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

pub struct Tim6(TIM6_S);

impl Tim6 {
    pub fn new(tim: TIM6_S, rcc: &Rcc) -> Self {
        rcc.enable(Peripheral::Tim6);
        Self(tim)
    }
}

impl Tim for Tim6 {
    fn enable_counter(&self) {
        self.0.cr1().write(|w| w.cen().set_bit());
    }

    fn disable_counter(&self) {
        self.0.cr1().write(|w| w.cen().clear_bit());
    }

    fn reset_counter(&self) {
        self.0.cnt().reset();
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
