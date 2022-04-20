pub struct SpritePalette {
	bpp: usize,
	colors: &'static [u16]
}

pub struct Sprite {
	data: &'static [u16]
}

pub struct SpriteMap {
	pub width: u16,
	pub height: u16,
	pub count_x: u16,
	pub count_y: u16,
	palette: &'static SpritePalette,
	sprites: &'static [&'static Sprite]
}

pub enum SpriteErr {
	OutOfBoundsError,
	UnknownError
}

struct DataUnpacker {
	bpp: usize,
	next_index: usize,
	next_unpacked_index: usize,
	unpacked: [u8; 16]
}

impl DataUnpacker {
	pub fn init(bpp: usize) -> Self {
		Self { bpp, next_index: 0, next_unpacked_index: 0, unpacked: [0; 16] }
	}

	pub fn next(&mut self, data: &'static [u16]) -> u8 {
		if self.next_unpacked_index == 0 {
			self.unpack_word(data);
		}
		let v = self.unpacked[self.next_unpacked_index];
		self.next_unpacked_index += 1;
		if self.next_unpacked_index == 16/self.bpp {
			self.next_unpacked_index = 0;
		}
		v
	}

	fn unpack_word(&mut self, data: &'static [u16]) {
		let word: u16 = data[self.next_index];
		self.next_index += 1;
		match self.bpp {
		1 => { for i in 0..16 {
			self.unpacked[i] = ((word >> i) & 1) as u8;
		}}
		2 => { for i in (0..16).step_by(2) {
			self.unpacked[i] = ((word >> (2*i)) & 3) as u8;
		}}
		3 => { for i in (0..15).step_by(3) {
			self.unpacked[i] = ((word >> (3*i)) & 7) as u8;
		}}
		4 => { for i in (0..16).step_by(4) {
			self.unpacked[i] = ((word >> (4*i)) & 15) as u8;
		}}
		_ => {}
		}
	}
}

impl SpriteMap {
	pub fn draw(&self, index: u16, dbuf: &mut [u16], dstride: u16) -> Result<(), SpriteErr> {
		if index > self.count_x*self.count_y {
			return Err(SpriteErr::OutOfBoundsError);
		}

		let sp: &Sprite = self.sprites[index as usize];
		let mut du = DataUnpacker::init(self.palette.bpp);
		for y in 0..self.height {
			for x in 0..self.width {
				let d = du.next(sp.data);
				dbuf[(y*dstride+x) as usize] = self.palette.colors[d as usize];
			}
		}
		Ok(())
	}

	pub fn blit_single(&self, index: u16, tmpbuf: &mut [u16], disp: &mut super::LLDisplay, px: u16, py: u16) -> Result<(), SpriteErr> {
		let bufsz_needed: usize = (self.width * self.height) as usize;
		self.draw(index, &mut tmpbuf[0..bufsz_needed], 0)?;
		let buf = &tmpbuf[0..bufsz_needed];
		let buf8 = bytemuck::cast_slice::<u16, u8>(buf);
		disp.blit_pixels(px, py, self.width, self.height, buf8).map_err(|_| SpriteErr::UnknownError)
	}
}

include!(concat!(env!("OUT_DIR"), "/sprite_data.rs"));
