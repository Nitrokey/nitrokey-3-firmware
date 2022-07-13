use nrf52840_hal::{
    gpio::{p0, p1, Level, Output, Pin, PushPull},
    gpiote::Gpiote,
    pac,
    prelude::InputPin,
    pwm,
    pwm::Pwm,
    spim,
};

pub const BOARD_NAME: &'static str = "NK3AM";

// @todo: remove these, actually only needed for physical, literal, mechanical keep-alive
pub const KEEPALIVE_PINS: &'static [u8] = &[0x0b, 0x0c, 0x18, 0x19, 0x25, 0x26, 0x27, 0x28];

use crate::traits::buttons::{Button, Press};
use crate::traits::rgb_led;
use crate::traits::rgb_led::Color;

pub type OutPin = Pin<Output<PushPull>>;

use crate::soc::types::BoardGPIO;

use crate::soc::trussed_ui::UserInterface;

pub type TrussedUI = UserInterface<HardwareButtons, RgbLed>;

pub struct RgbLed {
    pwm_red: Pwm<pac::PWM0>,
    pwm_green: Pwm<pac::PWM1>,
    pwm_blue: Pwm<pac::PWM2>,
}

pub struct HardwareButtons {
    pub touch_button: Option<OutPin>,
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

impl RgbLed {
    pub fn init_led<T: pwm::Instance>(led: OutPin, raw_pwm: T, channel: pwm::Channel) -> Pwm<T> {
        let pwm = Pwm::new(raw_pwm);
        pwm.set_output_pin(channel, led);

        //pwm.set_period(500u32.hz());
        //debug!("max duty: {:?}", pwm.max_duty());
        pwm.set_max_duty(255);
        pwm
    }

    pub fn set_led(&mut self, color: Color, channel: pwm::Channel, intensity: u8) {
        match color {
            Color::Red => {
                let duty: u16 =
                    ((intensity as f32 / 255.0) * self.pwm_red.max_duty() as f32) as u16;
                self.pwm_red.set_duty_on(channel, duty as u16);
            }
            Color::Green => {
                let duty: u16 =
                    ((intensity as f32 / 255.0) * self.pwm_green.max_duty() as f32) as u16;
                self.pwm_green.set_duty_on(channel, duty as u16);
            }
            Color::Blue => {
                let duty: u16 =
                    ((intensity as f32 / 255.0) * self.pwm_blue.max_duty() as f32) as u16;
                self.pwm_blue.set_duty_on(channel, duty as u16);
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
) -> TrussedUI {
    let rgb = RgbLed::new(leds, pwm_red, pwm_green, pwm_blue);

    let buttons = HardwareButtons {
        touch_button: Some(touch),
    };

    #[cfg(feature = "provisioner")]
    let ui = TrussedUI::new(Some(buttons), Some(rgb), true);

    #[cfg(not(feature = "provisioner"))]
    let ui = TrussedUI::new(Some(buttons), Some(rgb), false);

    ui
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

pub fn gpio_irq_sources(dir: &[u32]) -> u32 {
    let mut src: u32 = 0;
    fn bit_set(x: u32, y: u32) -> bool {
        (x & (1u32 << y)) != 0
    }

    if !bit_set(dir[0], 11) {
        src |= 0b0000_0001;
    }
    if !bit_set(dir[0], 12) {
        src |= 0b0000_0010;
    }
    if !bit_set(dir[0], 24) {
        src |= 0b0000_0100;
    }
    if !bit_set(dir[0], 25) {
        src |= 0b0000_1000;
    }
    if !bit_set(dir[1], 8) {
        src |= 0b0001_0000;
    }
    if !bit_set(dir[1], 7) {
        src |= 0b0010_0000;
    }
    if !bit_set(dir[1], 6) {
        src |= 0b0100_0000;
    }
    if !bit_set(dir[1], 5) {
        src |= 0b1000_0000;
    }
    src
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
