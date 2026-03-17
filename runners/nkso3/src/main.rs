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
    use boards::ui::{
        buttons::UserPresence as _,
        rgb_led::{Intensities, RgbLed as _},
    };
    use cortex_m_semihosting::hprintln;
    use stm32n657_hal::{
        bsec::Bsec,
        gpio::{GpioC, GpioG},
        otg_fs::{Otg1Fs, UsbBus1Fs},
        rcc::{ClockConfig, Rcc},
        timer::{MillisecondsCounter, Tim6, Tim7, Timer},
        Rate,
    };
    use systick_monotonic::Systick;
    use trussed::platform::consent;
    use usb_device::{bus::UsbBusAllocator, device::{UsbDevice, UsbDeviceBuilder, UsbVidPid}};

    use crate::nucleo::{Button, Led};

    type UsbBus = UsbBusAllocator<UsbBus1Fs>;

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
        usb_device: UsbDevice<'static, UsbBus1Fs>,
    }

    #[init(local = [
        usb_bus: Option<UsbBus> = None,
        ep_memory: [u32; 1024] = [0; 1024],
    ])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let bsec = Bsec::new(cx.device.BSEC);
        let uid = bsec.uid();

        hprintln!("nkso3 firmware is running on {:x?}", uid).ok();

        let rcc = Rcc::new(cx.device.RCC);
        let clock_config = rcc.clock_config();
        assert_eq!(clock_config, ClockConfig::DEFAULT);

        let monotonic = Systick::new(cx.core.SYST, clock_config.sys_bus_ck().to_Hz());

        let gpiog = GpioG::new(cx.device.GPIOG_S, &rcc);
        let led = Led::init(gpiog.g10, gpiog.g0, gpiog.g8);

        let gpioc = GpioC::new(cx.device.GPIOC_S, &rcc);
        let button = Button::init(gpioc.c13);

        let tim7 = Tim7::new(cx.device.TIM7_S, &rcc);
        let counter = MillisecondsCounter::new(tim7, clock_config);

        let tim6 = Tim6::new(cx.device.TIM6_S, &rcc);
        let mut timer = Timer::new(tim6, clock_config);
        timer.start(Rate::Hz(100));

        let vidpid = UsbVidPid(0x20A0, 0x42B2);
        let otg1 = Otg1Fs::new(cx.device.OTG1_S, clock_config);
        let usb_bus = UsbBus1Fs::new(otg1, cx.local.ep_memory);
        let usb_bus = cx.local.usb_bus.insert(usb_bus);
        let usb_device = UsbDeviceBuilder::new(usb_bus, vidpid)
            .product("Nitrokey Storage 3")
            .manufacturer("Nitrokey")
            .build();

        (
            Shared {},
            Local {
                counter,
                led,
                button,
                timer,
                usb_device,
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
            let user_presence = button.check_user_presence();
            let is_user_present = user_presence != consent::Level::None;

            let now = counter.now();
            let elapsed = now.checked_duration_since(cycle_start).unwrap().to_millis();
            if elapsed >= 1_000 {
                cycle_start = now;

                let total_elapsed = now.checked_duration_since(start).unwrap();
                hprintln!("{}", total_elapsed).ok();
            }

            let mut intensities = Intensities::from(0);
            if elapsed < 500 {
                intensities.green = u8::MAX;
            }
            if is_user_present {
                intensities.blue = u8::MAX;
            }
            led.set(intensities);

            nb::block!(timer.wait()).ok();
        }
    }

    #[task(binds = OTG1, local = [usb_device])]
    fn poll_usb(cx: poll_usb::Context) {
        cx.local.usb_device.poll(&mut []);
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
