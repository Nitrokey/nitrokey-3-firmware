use embedded_hal::blocking::delay::DelayUs;
use nrf52840_hal::{
	gpio::{Pin, Input, Output, PullDown, PullUp, PushPull, Level},
	pac::SPIM0,
	prelude::{InputPin, OutputPin},
	spim::Spim,
};
use trussed::types::{GUIControlCommand, GUIControlResponse};

// mod sprites;
mod sprites2;

type OutPin = Pin<Output<PushPull>>;
pub enum ButtonPin {
	LowTriggerPin(Pin<Input<PullUp>>),
	HighTriggerPin(Pin<Input<PullDown>>),
	NoPin
}

type LLDisplay = picolcd114::ST7789<display_interface_spi::SPIInterfaceNoCS<Spim<SPIM0>, OutPin>, OutPin>;

enum StickUIState {
	PreInitGarbled,
	Logo,
	Idle,
	// ShowRequest
	PoweredDown,
}

/* sufficient (rgb565) room for a 32x32 sprite, a 60x15 sprite or six 9x18 characters */
static mut DISPLAY_BUF: [u16; 1024] = [0; 1024];

//////////////////////////////////////////////////////////////////////////////
// UI
//////////////////////////////////////////////////////////////////////////////

pub struct DisplayUI {
	buf: &'static mut [u16; 1024],
	disp: Option<LLDisplay>,
	disp_power: Option<OutPin>,
	disp_backlight: Option<OutPin>,
	buttons: [ButtonPin; 8],
	button_state: [u8; 8],
	leds: [Option<OutPin>; 4],
	touch: Option<OutPin>,
	state: StickUIState,
}

impl DisplayUI {
	pub fn new(disp: Option<LLDisplay>,
			disp_power: Option<OutPin>,
			disp_backlight: Option<OutPin>,
			buttons: [ButtonPin; 8],
			leds: [Option<OutPin>; 4],
			touch: Option<OutPin>) -> Self {

		Self {
			buf: unsafe { &mut DISPLAY_BUF },
			disp, disp_power, disp_backlight,
			buttons, button_state: [0u8; 8],
			leds, touch,
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
				Level::High => { led.set_high().ok(); }
				Level::Low => { led.set_low().ok(); }
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
				sprites2::FONT_MAP.blit_single(c, self.buf, d, xi, y).ok();
				xi += sprites2::FONT_MAP.width;
			}
		}
	}
}

fn rgb565_memset(buf: &mut [u16], col: u16) {
	buf.fill(col);
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
			let (xs, ys): (u16, u16) = if h > w { (16, 64) } else { (64, 16) };
			for xi in (x..x+w).step_by(xs as usize) {
				for yi in (y..y+h).step_by(ys as usize) {
					let xd = core::cmp::min(xs, x+w-xi);
					let yd = core::cmp::min(ys, y+h-yi);
					let buf = &self.buf[0..(xd*yd) as usize];
					let buf8 = bytemuck::cast_slice::<u16, u8>(buf);
					d.blit_pixels(xi, yi, xd, yd, buf8).ok();
				}
			}
		}
	}

	fn draw_text(&mut self, x: u16, y: u16, text: &[u8]) {
		let mut yi: u16 = y;

		for line in text.split(|c| *c == 0x0a_u8) {
			self.draw_text_line(x, yi, line);
			yi += sprites2::FONT_MAP.height;
		}
	}

	fn draw_sprite(&mut self, x: u16, y: u16, sprite_map: u16, index: u16) {
		let smap = match sprite_map {
			1 => { sprites2::ICONS_MAP }
			_ => { return; }
		};
		if let Some(ref mut d) = self.disp {
			smap.blit_single(index, self.buf, d, x, y).ok();
		}
	}

	fn get_gui_dimension(&self) -> Option<(u16, u16)> { Some((240, 135)) }

	fn gui_control(&mut self, cmd: GUIControlCommand) -> Option<GUIControlResponse> {
		if let Some(ref mut d) = self.disp {
			match cmd {
				GUIControlCommand::Rotate(2) => {
					d.flip_view();
					Some(GUIControlResponse::Orientation(d.get_orientation()))
				}
				GUIControlCommand::SetOrientation(o) => {
					let o0 = d.get_orientation();
					if (o0 ^ o) == 2 {
						d.flip_view();
						Some(GUIControlResponse::Orientation(o))
					} else if o0 == o {
						Some(GUIControlResponse::Orientation(o))
					} else {
						None
					}
				}
				_ => { None }
			}
		} else {
			None
		}
	}

	fn update_button_state(&mut self) {
		for i in 0..self.buttons.len() {
			self.button_state[i] = match &self.buttons[i] {
				ButtonPin::NoPin => { 0 }
				ButtonPin::LowTriggerPin(p) => { if p.is_low().unwrap_or(false) { 1 } else { 0 } }
				ButtonPin::HighTriggerPin(p) => { if p.is_low().unwrap_or(true) { 0 } else { 1 } }
			}
		}
	}

	fn get_button_state(&mut self, bitmap: u32) -> Option<[u8; 8]> {
		Some(self.button_state.clone())
	}
}
