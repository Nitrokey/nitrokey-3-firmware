use core::time::Duration;

use littlefs2::{
    fs::{Allocation, Filesystem},
    io::Result as LfsResult,
};
use nrf52840_hal::{
    gpio::{p0, p1, Level, Output, Pin, PushPull},
    gpiote::Gpiote,
    pac,
    prelude::InputPin,
    pwm,
    pwm::Pwm,
    spim, Spim,
};
use nrf52840_pac::SPIM3;
use trussed::platform::consent;

use super::{
    migrations::ftl_journal::{self, ifs_flash_old::FlashStorage as OldFlashStorage},
    types::Soc,
};
use crate::{
    flash::ExtFlashStorage,
    soc::{rtic_monotonic::RtcMonotonic, types::BoardGPIO},
    types::Board,
    ui::{
        buttons::{Button, Press, UserPresence},
        rgb_led::{self, Color},
        Clock, UserInterface,
    },
};

type OutPin = Pin<Output<PushPull>>;

pub struct NK3AM;

impl Board for NK3AM {
    type Soc = Soc;

    type NfcDevice = DummyNfc;
    type Buttons = HardwareButtons;
    type Led = RgbLed;

    #[cfg(feature = "se050")]
    type Twi = nrf52840_hal::twim::Twim<nrf52840_pac::TWIM1>;
    #[cfg(feature = "se050")]
    type Se050Timer = nrf52840_hal::Timer<nrf52840_pac::TIMER1>;
    #[cfg(not(feature = "se050"))]
    type Twi = ();
    #[cfg(not(feature = "se050"))]
    type Se050Timer = ();

    const BOARD_NAME: &'static str = "NK3AM";

    fn prepare_ifs(ifs: &mut Self::InternalStorage) {
        ifs.format_journal_blocks();
    }

    fn recover_ifs(
        ifs_storage: &mut Self::InternalStorage,
        ifs_alloc: &mut Allocation<Self::InternalStorage>,
        efs_storage: &mut Self::ExternalStorage,
    ) -> LfsResult<()> {
        error_now!("IFS (nrf42) mount-fail");

        // regular mount failed, try mounting "old" (pre-journaling) IFS
        let pac = unsafe { nrf52840_pac::Peripherals::steal() };
        let mut old_ifs_storage = OldFlashStorage::new(pac.NVMC);
        let mut old_ifs_alloc: Allocation<OldFlashStorage> = Filesystem::allocate();
        let old_mountable = Filesystem::is_mountable(&mut old_ifs_storage);

        // we can mount the old ifs filesystem, thus we need to migrate
        if old_mountable {
            let mounted_ifs = ftl_journal::migrate(
                &mut old_ifs_storage,
                &mut old_ifs_alloc,
                ifs_alloc,
                ifs_storage,
                efs_storage,
            );
            // migration went fine => use its resulting IFS
            if let Ok(()) = mounted_ifs {
                info_now!("migration ok, mounting IFS");
                Ok(())
            // migration failed => format IFS
            } else {
                error_now!("failed migration, formatting IFS");
                Filesystem::format(ifs_storage)
            }
        } else {
            info_now!("recovering from journal");
            // IFS and old-IFS cannot be mounted, try to recover from journal
            ifs_storage.recover_from_journal();
            Ok(())
        }
    }
}

pub type InternalFlashStorage = super::flash::FlashStorage;
pub type ExternalFlashStorage = ExtFlashStorage<Spim<SPIM3>, OutPin>;

impl_storage_pointers!(
    NK3AM,
    Internal = InternalFlashStorage,
    External = ExternalFlashStorage,
);

pub struct DummyNfc;

impl nfc_device::traits::nfc::Device for DummyNfc {
    fn read(
        &mut self,
        _buf: &mut [u8],
    ) -> Result<nfc_device::traits::nfc::State, nfc_device::traits::nfc::Error> {
        Err(nfc_device::traits::nfc::Error::NoActivity)
    }
    fn send(&mut self, _buf: &[u8]) -> Result<(), nfc_device::traits::nfc::Error> {
        Err(nfc_device::traits::nfc::Error::NoActivity)
    }
    fn frame_size(&self) -> usize {
        0
    }
}

pub struct HardwareButtons {
    pub touch_button: Option<OutPin>,
}

impl UserPresence for HardwareButtons {
    fn check_user_presence(&mut self, clock: &mut dyn Clock) -> consent::Level {
        // essentially a blocking call for up to ~30secs
        // this outer loop accumulates *presses* from the
        // inner loop & maintains (loading) delays.

        let mut counter: u8 = 0;
        let threshold: u8 = 1;

        let start_time = clock.uptime();
        let timeout_at = start_time + Duration::from_millis(1_000);
        let mut next_check = start_time + Duration::from_millis(25);

        loop {
            let cur_time = clock.uptime();

            // timeout reached
            if cur_time > timeout_at {
                break;
            }
            // loop until next check shall be done
            if cur_time < next_check {
                continue;
            }

            if self.is_pressed(Button::A) {
                counter += 1;
                // with press -> delay 25ms
                next_check = cur_time + Duration::from_millis(25);
            } else {
                // w/o press -> delay 100ms
                next_check = cur_time + Duration::from_millis(100);
            }

            if counter >= threshold {
                break;
            }
        }

        // consent, if we've counted 3 "presses"
        if counter >= threshold {
            consent::Level::Normal
        } else {
            consent::Level::None
        }
    }
}

impl Press for HardwareButtons {
    fn is_pressed(&mut self, but: Button) -> bool {
        // As we do not have other buttons,
        // we simply ignore requests for them.
        // Like this they also don't block our time!
        if but == Button::B || but == Button::Middle {
            return false;
        }
        // @TODO: to be discussed how this is intended

        let mut ticks = 0;
        let need_ticks = 100;
        if let Some(touch) = self.touch_button.take() {
            let floating = touch.into_floating_input();

            for idx in 0..need_ticks + 1 {
                match floating.is_low() {
                    Err(_e) => {
                        trace!("is_pressed: err!");
                    }
                    Ok(v) => match v {
                        true => {
                            ticks = idx;
                            break;
                        }
                        false => {
                            if idx >= need_ticks {
                                ticks = idx;
                                break;
                            }
                        }
                    },
                }
            }
            self.touch_button = Some(floating.into_push_pull_output(Level::High));
        }
        ticks >= need_ticks
    }
}

pub struct RgbLed {
    pwm_red: Pwm<pac::PWM0>,
    pwm_green: Pwm<pac::PWM1>,
    pwm_blue: Pwm<pac::PWM2>,
}

impl RgbLed {
    pub fn init_led<T: pwm::Instance>(led: OutPin, raw_pwm: T, channel: pwm::Channel) -> Pwm<T> {
        let pwm = Pwm::new(raw_pwm);
        pwm.set_output_pin(channel, led);
        pwm.set_max_duty(u8::MAX as u16);
        pwm
    }

    pub fn set_led(&mut self, color: Color, channel: pwm::Channel, intensity: u8) {
        let intensity: f32 = intensity as f32;
        match color {
            Color::Red => {
                let duty: u16 = (intensity / 4f32) as u16;
                self.pwm_red.set_duty_on(channel, duty);
            }
            Color::Green => {
                let duty: u16 = (intensity / 2f32) as u16;
                self.pwm_green.set_duty_on(channel, duty);
            }
            Color::Blue => {
                let duty: u16 = (intensity) as u16;
                self.pwm_blue.set_duty_on(channel, duty);
            }
        }
    }
}

impl RgbLed {
    pub fn new(
        leds: [Option<OutPin>; 3],
        pwm_red: pac::PWM0,
        pwm_green: pac::PWM1,
        pwm_blue: pac::PWM2,
    ) -> RgbLed {
        let [red, green, blue] = leds;

        let red_pwm_obj = RgbLed::init_led(red.unwrap(), pwm_red, pwm::Channel::C0);
        let green_pwm_obj = RgbLed::init_led(green.unwrap(), pwm_green, pwm::Channel::C1);
        let blue_pwm_obj = RgbLed::init_led(blue.unwrap(), pwm_blue, pwm::Channel::C2);

        Self {
            pwm_red: red_pwm_obj,
            pwm_green: green_pwm_obj,
            pwm_blue: blue_pwm_obj,
        }
    }
}

impl rgb_led::RgbLed for RgbLed {
    fn red(&mut self, intensity: u8) {
        self.set_led(Color::Red, pwm::Channel::C0, intensity);
    }

    fn green(&mut self, intensity: u8) {
        self.set_led(Color::Green, pwm::Channel::C1, intensity);
    }

    fn blue(&mut self, intensity: u8) {
        self.set_led(Color::Blue, pwm::Channel::C2, intensity);
    }
}

pub fn init_ui(
    leds: [Option<OutPin>; 3],
    pwm_red: pac::PWM0,
    pwm_green: pac::PWM1,
    pwm_blue: pac::PWM2,
    touch: OutPin,
) -> UserInterface<RtcMonotonic, HardwareButtons, RgbLed> {
    // TODO: safely share the RTC
    let pac = unsafe { nrf52840_pac::Peripherals::steal() };
    let rtc_mono = RtcMonotonic::new(pac.RTC0);

    let rgb = RgbLed::new(leds, pwm_red, pwm_green, pwm_blue);

    let buttons = HardwareButtons {
        touch_button: Some(touch),
    };

    UserInterface::new(rtc_mono, Some(buttons), Some(rgb))
}

pub fn init_pins(_gpiote: &Gpiote, gpio_p0: p0::Parts, gpio_p1: p1::Parts) -> BoardGPIO {
    /* touch sensor */
    let touch = gpio_p0.p0_04.into_push_pull_output(Level::High).degrade();
    // not used, just ensure output + low
    gpio_p0.p0_06.into_push_pull_output(Level::Low).degrade();

    /* irq configuration */

    // gpiote.port().input_pin(&btn3).low();
    // gpiote.port().input_pin(&btn4).low();
    // gpiote.port().input_pin(&btn5).low();
    // gpiote.port().input_pin(&btn6).low();
    // gpiote.port().input_pin(&btn7).low();
    // gpiote.port().input_pin(&btn8).low();

    /* RGB LED */
    let led_r = gpio_p0.p0_09.into_push_pull_output(Level::Low).degrade();
    let led_g = gpio_p0.p0_10.into_push_pull_output(Level::Low).degrade();
    let led_b = gpio_p1.p1_02.into_push_pull_output(Level::Low).degrade();

    /* SE050 */
    let se_pwr = gpio_p1.p1_10.into_push_pull_output(Level::Low).degrade();
    let se_scl = gpio_p1.p1_15.into_floating_input().degrade();
    let se_sda = gpio_p0.p0_02.into_floating_input().degrade();

    let se_pins = nrf52840_hal::twim::Pins {
        scl: se_scl,
        sda: se_sda,
    };

    /* Ext. Flash SPI */
    // Flash WP# gpio_p0.p0_22
    // Flash HOLD# gpio_p0.p0_23
    let flash_spi_cs = gpio_p0.p0_24.into_push_pull_output(Level::High).degrade();
    let flash_spi_clk = gpio_p1.p1_06.into_push_pull_output(Level::Low).degrade();
    let flash_spi_mosi = gpio_p1.p1_04.into_push_pull_output(Level::Low).degrade();
    let flash_spi_miso = gpio_p1.p1_00.into_floating_input().degrade();
    //let _flash_wp = gpio_p0.p0_22.into_push_pull_output(Level::Low).degrade();
    //let _flash_hold = gpio_p0.p0_23.into_push_pull_output(Level::High).degrade();

    let flash_spi = spim::Pins {
        sck: flash_spi_clk,
        miso: Some(flash_spi_miso),
        mosi: Some(flash_spi_mosi),
    };

    BoardGPIO {
        buttons: [None, None, None, None, None, None, None, None],
        leds: [None, None, None, None],
        rgb_led: [Some(led_r), Some(led_g), Some(led_b)],
        touch: Some(touch),
        uart_pins: None,
        fpr_detect: None,
        fpr_power: None,
        display_spi: None,
        display_cs: None,
        display_reset: None,
        display_dc: None,
        display_backlight: None,
        display_power: None,
        se_pins: Some(se_pins),
        se_power: Some(se_pwr),
        flashnfc_spi: Some(flash_spi),
        flash_cs: Some(flash_spi_cs),
        flash_power: None,
        nfc_cs: None,
        nfc_irq: None,
    }
}

pub fn set_panic_led() {
    unsafe {
        let pac = nrf52840_pac::Peripherals::steal();
        let p0 = nrf52840_hal::gpio::p0::Parts::new(pac.P0);
        let p1 = nrf52840_hal::gpio::p1::Parts::new(pac.P1);

        // red
        p0.p0_09.into_push_pull_output(Level::Low).degrade();
        // green
        p0.p0_10.into_push_pull_output(Level::High).degrade();
        // blue
        p1.p1_02.into_push_pull_output(Level::High).degrade();
    }
}
