#![no_std]
#![no_main]

use core::panic::PanicInfo;

use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::{OutputPin as _, PinState};
use stm32n6::stm32n657::Peripherals;
use stm32n657_hal::{
    bsec::Bsec,
    gpio::{GpioG, OutputMode, PinG0, PinG10, PinG8},
    rcc::{ClockConfig, Rcc},
    timer::{Tim6, Timer},
    Rate,
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
    let p = Peripherals::take().unwrap();

    let bsec = Bsec::new(p.BSEC);
    let uid = bsec.uid();

    hprintln!("nkso3 firmware is running on {:x?}", uid).ok();

    let rcc = Rcc::new(p.RCC);
    let clock_config = rcc.clock_config();
    assert_eq!(clock_config, ClockConfig::DEFAULT);

    let gpiog = GpioG::new(p.GPIOG_S, &rcc);
    let mut leds = Leds::new(&gpiog);

    let tim6 = Tim6::new(p.TIM6_S, &rcc);
    let mut timer = Timer::new(tim6, clock_config);
    timer.start(Rate::Hz(1));

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
