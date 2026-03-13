#![no_std]
#![no_main]

mod gpio;
mod timer;

use core::panic::PanicInfo;

use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::{OutputPin as _, PinState};
use stm32n6::stm32n657::Peripherals;

use self::{
    gpio::{GpioG, OutputMode, PinG0, PinG10, PinG8},
    timer::{Tim6, Timer},
};

#[derive(Clone, Copy)]
enum Led {
    // user red LED (LD5): PG10
    Ld5,
    // user green LED (LD6): PG0
    Ld6,
    // user blue LED (LD7): PG8
    Ld7,
}

impl Led {
    fn next(self) -> Self {
        match self {
            Self::Ld5 => Self::Ld6,
            Self::Ld6 => Self::Ld7,
            Self::Ld7 => Self::Ld5,
        }
    }
}

struct Leds<'a> {
    ld5: PinG10<'a, OutputMode>,
    ld6: PinG0<'a, OutputMode>,
    ld7: PinG8<'a, OutputMode>,
}

impl<'a> Leds<'a> {
    fn new(gpiog: &'a GpioG) -> Self {
        let ld5 = PinG10::new(gpiog).into_push_pull_output();
        let ld6 = PinG0::new(gpiog).into_push_pull_output();
        let ld7 = PinG8::new(gpiog).into_push_pull_output();
        let mut leds = Self { ld5, ld6, ld7 };
        for led in [Led::Ld5, Led::Ld6, Led::Ld7] {
            leds.set(led, false);
        }
        leds
    }

    fn set(&mut self, led: Led, on: bool) {
        let state = PinState::from(!on);
        match led {
            Led::Ld5 => self.ld5.set_state(state).ok(),
            Led::Ld6 => self.ld6.set_state(state).ok(),
            Led::Ld7 => self.ld7.set_state(state).ok(),
        };
    }
}

#[entry]
fn main() -> ! {
    hprintln!("nkso3 firmware is running").ok();

    let p = Peripherals::take().unwrap();
    let gpiog = GpioG::new(p.GPIOG_S, &p.RCC);
    let mut leds = Leds::new(&gpiog);

    let tim6 = Tim6::new(p.TIM6_S, &p.RCC);
    let mut timer = Timer::new(tim6);
    // values are based on an input clock of 4 MHz, so the timer clock would be
    // 4 MHz / (9999 + 1) = 4 KHz, so update occurs with frequency 4 KHz / (3999 + 1) = 1Hz.
    // but from observation I think the actual frequency is slightly higher
    timer.start(9999, 3999);

    let mut led = Led::Ld5;
    leds.set(led, true);
    loop {
        nb::block!(timer.wait()).ok();
        leds.set(led, false);
        led = led.next();
        leds.set(led, true);
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hprintln!("{}", info).ok();
    loop {}
}

#[inline(never)]
#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    hprintln!("HardFault: {:?}", ef).ok();
    loop {}
}
