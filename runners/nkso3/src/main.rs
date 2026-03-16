#![no_std]
#![no_main]

mod nucleo;

use core::panic::PanicInfo;

use boards::ui::rgb_led::RgbLed as _;
use cortex_m_rt::{exception, ExceptionFrame};
use cortex_m_semihosting::hprintln;

use self::nucleo::Led;

#[rtic::app(device = stm32n6::stm32n657)]
mod app {
    use boards::ui::rgb_led::{Intensities, RgbLed as _};
    use cortex_m_semihosting::hprintln;
    use stm32n657_hal::{
        bsec::Bsec,
        gpio::GpioG,
        rcc::{ClockConfig, Rcc},
        timer::{MillisecondsCounter, Tim6, Tim7, Timer},
        Rate,
    };
    use systick_monotonic::Systick;

    use crate::nucleo::Led;

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<100>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
        timer: Timer<Tim6>,
        counter: MillisecondsCounter<Tim7>,
    }

    #[init(local = [gpiog: Option<GpioG> = None])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let bsec = Bsec::new(cx.device.BSEC);
        let uid = bsec.uid();

        hprintln!("nkso3 firmware is running on {:x?}", uid).ok();

        let rcc = Rcc::new(cx.device.RCC);
        let clock_config = rcc.clock_config();
        assert_eq!(clock_config, ClockConfig::DEFAULT);

        let monotonic = Systick::new(cx.core.SYST, clock_config.sys_bus_ck().to_Hz());

        let gpiog = GpioG::new(cx.device.GPIOG_S, &rcc);
        let gpiog = cx.local.gpiog.insert(gpiog);
        let led = Led::new(gpiog);

        let tim7 = Tim7::new(cx.device.TIM7_S, &rcc);
        let counter = MillisecondsCounter::new(tim7, clock_config);

        let tim6 = Tim6::new(cx.device.TIM6_S, &rcc);
        let mut timer = Timer::new(tim6, clock_config);
        timer.start(Rate::Hz(1));

        (
            Shared {},
            Local {
                counter,
                led,
                timer,
            },
            init::Monotonics(monotonic),
        )
    }

    #[idle(local = [led, timer, counter])]
    fn idle(cx: idle::Context) -> ! {
        let idle::LocalResources {
            led,
            timer,
            counter,
        } = cx.local;

        let mut led_cycle = true;
        loop {
            let mut intensities = Intensities::from(0);
            if led_cycle {
                intensities.blue = u8::MAX;
            } else {
                intensities.green = u8::MAX;
            }
            led.set(intensities);
            led_cycle = !led_cycle;

            nb::block!(timer.wait()).ok();

            let now = counter.now();
            hprintln!("{}", now.duration_since_epoch()).ok();
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    Led::set_panic_led();
    hprintln!("{}", info).ok();
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    hprintln!("HardFault: {:?}", ef).ok();
    loop {
        cortex_m::asm::wfi();
    }
}
