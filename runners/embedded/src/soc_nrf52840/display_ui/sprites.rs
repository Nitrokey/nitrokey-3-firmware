pub struct SpriteMap {
    count_x: u16,
    count_y: u16,
    pub width: u16,
    pub height: u16,
    /* assume: bitsperpixel = 16 */
    buf: &'static [u8],
}

macro_rules! sprite_map {
    ($name:ident, $w:expr, $h:expr, $n:expr, $rawfile:expr) => {
        pub const $name: SpriteMap = {
            let data: &[u8; ($w) * ($h) * 2 * ($n)] = include_bytes!($rawfile);
            SpriteMap {
                count_x: 1,
                count_y: ($n),
                width: ($w),
                height: ($h),
                buf: data,
            }
        };
    };
}

/*const FONT: &[u8; (9*18*2)*192] = include_bytes!("../../../data/font_9x18.raw");
pub const FONT_MAP: SpriteMap = SpriteMap { count_x: 1, count_y: 192, width: 9, height: 18, buf: FONT };

const ICONS: &[u8; (25*10*2)*8] = include_bytes!("../../../data/texmap.raw");
pub const ICONS_MAP: SpriteMap = SpriteMap { count_x: 1, count_y: 8, width: 25, height: 10, buf: ICONS };*/
sprite_map!(FONT_MAP, 9, 18, 192, "../../../data/font_9x18.raw");
sprite_map!(ICONS_MAP, 25, 10, 8, "../../../data/texmap.raw");

pub enum SpriteErr {
    UnknownError,
}

impl SpriteMap {
    pub fn draw(&self, index: u16, dbuf: &mut [u8], dstride: u16) -> Result<(), SpriteErr> {
        if index > self.count_x * self.count_y {
            return Err(SpriteErr::UnknownError);
        }

        let mut src_offset: isize = (index * self.width * self.height * 2) as isize;
        let mut dst_offset: isize = 0;
        if (self.count_x == 1) && ((dstride == 0) || (dstride == self.width * 2)) {
            unsafe {
                let dst = (dbuf.as_mut_ptr()).offset(dst_offset);
                let src = (self.buf.as_ptr()).offset(src_offset);
                __aeabi_memcpy(dst, src, (self.width * self.height * 2) as usize);
            }
            return Ok(());
        }
        for _y in 0..self.height {
            unsafe {
                let dst = (dbuf.as_mut_ptr()).offset(dst_offset);
                let src = (self.buf.as_ptr()).offset(src_offset);
                __aeabi_memcpy(dst, src, (self.width * 2) as usize);
                dst_offset += dstride as isize;
                src_offset += (self.width * self.count_x * 2) as isize;
            }
        }
        Ok(())
    }

    pub fn blit_single(
        &self,
        index: u16,
        tmpbuf: &mut [u8],
        disp: &mut super::LLDisplay,
        px: u16,
        py: u16,
    ) -> Result<(), SpriteErr> {
        let bufsz_needed: usize = (self.width * self.height * 2) as usize;
        self.draw(index, &mut tmpbuf[0..bufsz_needed], 0)?;
        disp.blit_pixels(px, py, self.width, self.height, &tmpbuf[0..bufsz_needed])
            .map_err(|_| SpriteErr::UnknownError)
    }
}

////////////////////////////////////////////////////////////////////////////////

extern "C" {
    fn __aeabi_memcpy(dst: *mut u8, src: *const u8, len: usize);
}
