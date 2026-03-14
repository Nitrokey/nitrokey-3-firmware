//! Timers, see Section 55 of RM0486.

use core::convert::Infallible;

use stm32n6::stm32n657::{RCC, TIM6_S};

pub trait Tim {
    fn enable_counter(&self);
    fn disable_counter(&self);
    fn reset_counter(&self);
    fn set_prescaler(&self, psc: u16);
    fn set_auto_reload(&self, arr: u16);
    fn trigger_update(&self);
    fn interrupt_flag(&self) -> bool;
    fn clear_interrupt_flag(&self);
}

pub struct Timer<T>(T);

impl<T: Tim> Timer<T> {
    pub fn new(tim: T) -> Self {
        Self(tim)
    }

    pub fn start(&mut self, prescaler: u16, auto_reload: u16) {
        self.0.disable_counter();
        self.0.reset_counter();

        self.0.set_prescaler(prescaler);
        self.0.set_auto_reload(auto_reload);
        self.0.trigger_update();

        self.0.enable_counter();
    }

    pub fn wait(&mut self) -> nb::Result<(), Infallible> {
        if self.0.interrupt_flag() {
            self.0.clear_interrupt_flag();
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

pub struct Tim6(TIM6_S);

impl Tim6 {
    pub fn new(tim: TIM6_S, rcc: &RCC) -> Self {
        rcc.apb1lensr().write(|w| w.tim6ens().set_bit());
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
