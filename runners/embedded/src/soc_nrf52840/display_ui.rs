use embedded_hal::blocking::delay::DelayUs;
use nrf52840_hal::{
    gpio::{Input, Level, Output, Pin, PullUp, PushPull},
    pac::SPIM0,
    prelude::OutputPin,
    spim::Spim,
};

mod sprites;

type OutPin = Pin<Output<PushPull>>;
type InPin = Pin<Input<PullUp>>;

type LLDisplay =
    picolcd114::ST7789<display_interface_spi::SPIInterfaceNoCS<Spim<SPIM0>, OutPin>, OutPin>;

enum StickUIState {
    PreInitGarbled,
    Logo,
    Idle,
    // ShowRequest
    PoweredDown,
}

/* sufficient (rgb565) room for a 32x32 sprite, a 60x15 sprite or six 9x18 characters */
static mut DISPLAY_BUF: [u8; 2048] = [0; 2048];

//////////////////////////////////////////////////////////////////////////////
// UI
//////////////////////////////////////////////////////////////////////////////

pub struct DisplayUI {
    buf: &'static mut [u8; 2048],
    disp: Option<LLDisplay>,
    disp_power: Option<OutPin>,
    disp_backlight: Option<OutPin>,
    buttons: [Option<InPin>; 8],
    leds: [Option<OutPin>; 4],
    touch: Option<OutPin>,
    state: StickUIState,
}

impl DisplayUI {
    pub fn new(
        disp: Option<LLDisplay>,
        disp_power: Option<OutPin>,
        disp_backlight: Option<OutPin>,
        buttons: [Option<InPin>; 8],
        leds: [Option<OutPin>; 4],
        touch: Option<OutPin>,
    ) -> Self {
        Self {
            buf: unsafe { &mut DISPLAY_BUF },
            disp,
            disp_power,
            disp_backlight,
            buttons,
            leds,
            touch,
            state: StickUIState::PoweredDown,
        }
    }

    pub fn power_on(&mut self, delay_timer: &mut impl DelayUs<u32>) {
        if let Some(ref mut d) = self.disp {
            if let Some(ref mut p) = self.disp_power {
                p.set_low().ok();
                delay_timer.delay_us(1000u32);
            }
            d.init(delay_timer).ok();
        }
    }

    pub fn power_off(&mut self) {
        if let Some(ref mut p) = self.disp_power {
            p.set_high().ok();
        }
        self.state = StickUIState::PoweredDown;
    }

    pub fn set_led(&mut self, idx: usize, lvl: Level) {
        if let Some(ref mut led) = self.leds[idx] {
            match lvl {
                Level::High => {
                    led.set_high().ok();
                }
                Level::Low => {
                    led.set_low().ok();
                }
            }
        }
    }

    fn draw_text_line(&mut self, x: u16, y: u16, line: &[u8]) {
        let mut xi: u16 = x;

        if let Some(ref mut d) = self.disp {
            for c0 in line {
                let mut c: u16 = *c0 as u16;
                if c == 0x0d {
                    continue;
                } else if c >= 0x20 && c < 0x80 {
                    c = c - 0x20;
                } else if c >= 0xa0 {
                    c = c - 0x40;
                } else {
                    c = 0x7f - 0x20;
                }
                sprites::FONT_MAP.blit_single(c, self.buf, d, xi, y).ok();
                xi += sprites::FONT_MAP.width;
            }
        }
    }
}

fn rgb565_memset(buf: &mut [u8], col: u16) {
    let ch: u8 = (col >> 8) as u8;
    let cl: u8 = (col & 255) as u8;
    let buflen: usize = buf.len();

    // the code generated from this looks more complicated than necessary;
    // one day, replace all this nonsense with a tasty call to __aeabi_memset4()
    // or figure out the "proper" Rust incantation the compiler happens to grasp
    for i in (0..buflen).step_by(2) {
        buf[i + 0] = cl;
        buf[i + 1] = ch;
    }
}

impl trussed::platform::UserInterface for DisplayUI {
    // fn check_user_presence(&mut self) -> consent::Level
    // fn set_status(&mut self, status: ui::Status)
    // fn status(&self) -> ui::Status
    // fn refresh(&mut self)
    // fn uptime(&mut self) -> core::time::Duration
    // fn reboot(&mut self, to: reboot::To) -> !
    // fn wink(&mut self, duration: core::time::Duration)

    fn draw_filled_rect(&mut self, x: u16, y: u16, w: u16, h: u16, col: u16) {
        if let Some(ref mut d) = self.disp {
            rgb565_memset(self.buf, col);
            let (xs, ys): (u16, u16) = if h > w { (15, 60) } else { (60, 15) };
            for xi in (x..x + w - 1).step_by(xs as usize) {
                for yi in (y..y + h - 1).step_by(ys as usize) {
                    let xd = core::cmp::min(xs, x + w - xi);
                    let yd = core::cmp::min(ys, y + h - yi);
                    d.blit_pixels(xi, yi, xd, yd, &self.buf[0..(xd * yd * 2) as usize])
                        .ok();
                }
            }
        }
    }

    fn draw_text(&mut self, x: u16, y: u16, text: &[u8]) {
        let mut yi: u16 = y;

        for line in text.split(|c| *c == 0x0a_u8) {
            self.draw_text_line(x, yi, line);
            yi += sprites::FONT_MAP.height;
        }
    }

    fn draw_sprite(&mut self, x: u16, y: u16, sprite_map: u16, index: u16) {
        let smap = match sprite_map {
            1 => sprites::ICONS_MAP,
            _ => {
                return;
            }
        };
        if let Some(ref mut d) = self.disp {
            smap.blit_single(index, self.buf, d, x, y).ok();
        }
    }

    fn get_gui_dimension(&self) -> Option<(u16, u16)> {
        Some((240, 135))
    }
}
