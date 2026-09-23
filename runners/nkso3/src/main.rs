#![no_std]
#![no_main]

mod mmc;
mod nucleo;

use core::panic::PanicInfo;

use boards::ui::rgb_led::RgbLed as _;
use cortex_m_rt::{exception, ExceptionFrame};

use self::nucleo::Led;

delog::generate_macros!();

#[rtic::app(device = stm32n6::stm32n657)]
mod app {
    use boards::{
        init::Delogger,
        ui::{
            buttons::UserPresence as _,
            rgb_led::{Intensities, RgbLed as _},
        },
    };
    use stm32n657_hal::{
        bsec::Bsec,
        gpio::{GpioA, GpioC, GpioE, GpioG},
        mmc::MmcMaster,
        pwr::Pwr,
        rcc::{ClockConfig, Rcc},
        timer::{MillisecondsCounter, Tim6, Tim7, Timer},
        Rate,
    };
    use systick_monotonic::Systick;
    use trussed::platform::consent;

    use crate::mmc::Mmc;
    use crate::nucleo::{Button, Led};

    #[monotonic(binds = SysTick, default = true)]
    type Monotonic = Systick<100>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
        button: Button,
        timer: Timer<Tim6>,
        counter: MillisecondsCounter<Tim7>,
        #[allow(unused)]
        mmc: Mmc,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let bsec = Bsec::new(cx.device.BSEC);
        let uid = bsec.uid();
        Delogger::init_default(delog::LevelFilter::Debug, &boards::init::DELOG_FLUSHER).ok();

        info!("nkso3 firmware is running on {:x?}", uid);

        let rcc = Rcc::new(cx.device.RCC);
        let clock_config = rcc.clock_config();
        assert_eq!(clock_config, ClockConfig::DEFAULT);

        let monotonic = Systick::new(cx.core.SYST, clock_config.sys_bus_ck().to_Hz());

        let pwr = Pwr::new(cx.device.PWR_S);
        pwr.enable_mmc_vddio();

        let gpiog = GpioG::new(cx.device.GPIOG_S, &rcc);
        let gpioc = GpioC::new(cx.device.GPIOC_S, &rcc);
        let led = Led::init(gpiog.g10, gpiog.g0, gpiog.g8);

        let button = Button::init(gpioc.c13);

        let _gpioa = GpioA::new(cx.device.GPIOA_S, &rcc);
        let gpioe = GpioE::new(cx.device.GPIOE_S, &rcc);
        info_now!("Before pins");
        let pins = (
            gpioc.c3.into_sdmmc2_cmd(),
            gpioc.c2.into_sdmmc2_ck(),
            gpioc.c4.into_sdmmc2_d0(),
            gpioc.c5.into_sdmmc2_d1(),
            gpioc.c0.into_sdmmc2_d2(),
            gpioe.e4.into_sdmmc2_d3(),
        );
        info_now!("after pins");
        let mmc = MmcMaster::new(cx.device.SDMMC2_S, pins);

        info_now!("before enable");
        let mmc = mmc.enable(&rcc).expect("Enabling mmc");
        info_now!("after enable");

        let tim7 = Tim7::new(cx.device.TIM7_S, &rcc);
        let counter = MillisecondsCounter::new(tim7, clock_config);

        let tim6 = Tim6::new(cx.device.TIM6_S, &rcc);
        let mut timer = Timer::new(tim6, clock_config);
        timer.start(Rate::Hz(100));

        (
            Shared {},
            Local {
                counter,
                led,
                button,
                timer,
                mmc,
            },
            init::Monotonics(monotonic),
        )
    }

    #[idle(local = [led, button, timer, counter])]
    fn idle(cx: idle::Context) -> ! {
        let idle::LocalResources {
            led,
            button,
            timer,
            counter,
        } = cx.local;

        let start = counter.now();
        let mut cycle_start = start;
        loop {
            info_now!("idle");
            let user_presence = button.check_user_presence();
            let is_user_present = user_presence != consent::Level::None;

            let now = counter.now();
            info_now!("got now");
            let elapsed = now.checked_duration_since(cycle_start).unwrap().to_millis();
            info_now!("got elapsed");
            if elapsed >= 1_000 {
                info_now!("Restart");
                cycle_start = now;

                let _total_elapsed = now.checked_duration_since(start).unwrap();
                info_now!("{}", _total_elapsed);
            }

            let mut intensities = Intensities::from(0);
            if elapsed < 500 {
                intensities.green = u8::MAX;
            }
            info_now!("After set");
            if is_user_present {
                intensities.blue = u8::MAX;
            }
            led.set(intensities);

            nb::block!(timer.wait()).ok();
        }
    }
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    Led::set_panic_led();
    error_now!("{}", info);
    loop {
        cortex_m::asm::wfi();
    }
}

#[exception]
unsafe fn HardFault(ef: &ExceptionFrame) -> ! {
    error_now!("HardFault: {:?}", ef);
    loop {
        cortex_m::asm::wfi();
    }
}
