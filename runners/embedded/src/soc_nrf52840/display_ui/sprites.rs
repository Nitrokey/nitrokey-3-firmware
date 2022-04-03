pub struct SpriteMap {
	count_x: u16,
	count_y: u16,
	width: u16,
	height: u16,
	/* assume: bitsperpixel = 16 */
	buf: &'static [u8],
}

const FONT: &[u8; (9*18*2)*192] = include_bytes!("../../../data/font_9x18.raw");
const FONT_MAP: SpriteMap = SpriteMap { count_x: 1, count_y: 192, width: 9, height: 18, buf: FONT };

const BATTERY: &[u8; (25*10*2)*6] = include_bytes!("../../../data/texmap.raw");
const BATTERY_MAP: SpriteMap = SpriteMap { count_x: 1, count_y: 6, width: 25, height: 10, buf: BATTERY };

pub enum SpriteErr {
	UnknownError
}

impl SpriteMap {
	pub fn draw(&self, index: u16, dbuf: &mut [u8], dstride: u16) -> Result<(), SpriteErr> {
		use core::convert::TryInto;

		if index > self.count_x*self.count_y {
			return Err(SpriteErr::UnknownError);
		}

		let mut src_offset: isize = (index*self.width*self.height*2) as isize;
		let mut dst_offset: isize = 0;
		if (self.count_x == 1) && ((dstride == 0) || (dstride == self.width*2)) { unsafe {
			let dst = (dbuf.as_mut_ptr()).offset(dst_offset);
			let src = (self.buf.as_ptr()).offset(src_offset);
			__aeabi_memcpy(dst, src, (self.width*self.height*2) as usize);
			}
			return Ok(());
		}
		for _y in 0..self.height { unsafe {
			let dst = (dbuf.as_mut_ptr()).offset(dst_offset);
			let src = (self.buf.as_ptr()).offset(src_offset);
			__aeabi_memcpy(dst, src, (self.width*2) as usize);
			dst_offset += dstride as isize;
			src_offset += (self.width*self.count_x*2) as isize;
		}}
		Ok(())
	}

	pub fn blit_single(&self, index: u16, tmpbuf: &mut [u8], disp: &mut super::LLDisplay, px: u16, py: u16) -> Result<(), SpriteErr> {
		let bufsz_needed: usize = (self.width * self.height * 2) as usize;
		self.draw(index, &mut tmpbuf[0..bufsz_needed], 0)?;
		disp.blit_pixels(px, py, self.width, self.height, &tmpbuf[0..bufsz_needed]).map_err(|_| SpriteErr::UnknownError)
	}
}

macro_rules! draw_sprite {
($dsp:expr, $map:ident, $idx:expr, $px:expr, $py:expr) => {
	$map.draw($idx, $dsp.buf, 0).ok();
	$dsp.dsp.as_mut().unwrap().blit_at(&$dsp.buf[0..($map.width*$map.height*2) as usize], $px, $py, $map.width, $map.height);
}}

////////////////////////////////////////////////////////////////////////////////

extern "C" {
	fn __aeabi_memcpy(dst: *mut u8, src: *const u8, len: usize);
}
