use embedded_graphics::{
	// DrawTarget,
	pixelcolor::{RgbColor, Rgb565, raw::RawU16, raw::RawData},
};
use embedded_hal::blocking::delay::DelayUs;
use nrf52840_hal::{
	gpio::{Pin, Input, Output, PullUp, PushPull, Floating, Level},
	pac::SPIM0,
	prelude::{OutputPin, InputPin},
	spim::Spim,
};
use trussed::{
	platform::{consent, reboot, ui},
};

mod sprites;

type OutPin = Pin<Output<PushPull>>;
type InPin = Pin<Input<PullUp>>;
type FloatingPin = Pin<Input<Floating>>;

type LLDisplay = picolcd114::ST7789<display_interface_spi::SPIInterfaceNoCS<Spim<SPIM0>, OutPin>, OutPin>;

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
	pub fn new(disp: Option<LLDisplay>,
			disp_power: Option<OutPin>,
			disp_backlight: Option<OutPin>,
			buttons: [Option<InPin>; 8],
			leds: [Option<OutPin>; 4],
			touch: Option<OutPin>) -> Self {

		Self {
			buf: unsafe { &mut DISPLAY_BUF },
			disp, disp_power, disp_backlight,
			buttons, leds, touch,
			state: StickUIState::PoweredDown,
		}
	}

	pub fn power_on(&mut self, delay_timer: &mut impl DelayUs<u32>) {
		if let Some(ref mut d) = self.disp {
			if let Some(ref mut p) = self.disp_power {
				p.set_low().ok();
			}
			d.init(delay_timer);
		}
	}

	pub fn power_off(&mut self) {
		if let Some(ref mut p) = self.disp_power {
			p.set_low().ok();
		}
		self.state = StickUIState::PoweredDown;
	}

	fn rgb16_memset(&mut self, color: Rgb565) {
		// holy cow, Rust type inference/annotation is so sh*tty...
		let c: u16 = Into::<RawU16>::into(color).into_inner();
		let ch: u8 = (c >> 8) as u8;
		let cl: u8 = (c & 255) as u8;
		let buflen: usize = self.buf.len();

		// the code generated from this looks more complicated than necessary;
		// one day, replace all this nonsense with a tasty call to __aeabi_memset4()
		// or figure out the "proper" Rust incantation the compiler happens to grasp
		for i in (0..buflen).step_by(2) {
			self.buf[i+0] = cl;
			self.buf[i+1] = ch;
		}
	}

	pub fn refresh(&mut self, t: u32) {
		if t & 7 == 0 {
			self.set_led(0, if (t & 8) == 8 { Level::High } else { Level::Low });
		}
	}

	fn tile_bg(&mut self) {
		if let Some(ref mut d) = self.disp {
			let tile_buf: &[u8] = &self.buf[0..60*15*2];
			for i in 0..4*9 {
				let x = (i & 0b000011) * 60;
				let y = (i >> 2) * 15;
				// d.blit_at(&self.buf[0..60*15*2], x*60, y*15, 60, 15);
				d.blit_pixels(x, y, 60, 15, tile_buf);
			}
		}
	}

	pub fn set_led(&mut self, idx: usize, lvl: Level) {
		if let Some(ref mut led) = self.leds[idx] {
			match lvl {
				Level::High => { led.set_high().ok(); }
				Level::Low => { led.set_low().ok(); }
			}
		}
	}

}

impl trussed::platform::UserInterface for DisplayUI {}
