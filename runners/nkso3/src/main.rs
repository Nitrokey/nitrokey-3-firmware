#![no_std]
#![no_main]

mod gpio;

use core::panic::PanicInfo;

use cortex_m_rt::{entry, exception, ExceptionFrame};
use cortex_m_semihosting::hprintln;
use embedded_hal::digital::v2::OutputPin as _;
use stm32n6::stm32n657::Peripherals;

use self::gpio::{GpioG, PinG0, PinG10, PinG8};

// user red LED (LD5): PG10
// user green LED (LD6): PG0
// user blue LED (LD7): PG8

#[entry]
fn main() -> ! {
    hprintln!("nkso3 firmrware is running").ok();

    let p = Peripherals::take().unwrap();
    let gpiog = GpioG::new(p.GPIOG_S, &p.RCC);

    let mut ld5 = PinG10::new(&gpiog).into_push_pull_output();
    let mut ld6 = PinG0::new(&gpiog).into_push_pull_output();
    let mut ld7 = PinG8::new(&gpiog).into_push_pull_output();
    ld5.set_low().ok();
    ld6.set_low().ok();
    ld7.set_low().ok();

    loop {}
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
