use core::marker::PhantomData;

use littlefs2::{driver::Storage, io::Error};

// Chosen so that the littlefs2 header fits.  Note that using this size will cause a `NoSpace`
// error during formatting.  The filesystem will still be mountable though.
const DEFAULT_RAM_SIZE: usize = 256;
const ERASED: u8 = 0xff;

pub struct RamStorage<S, const SIZE: usize> {
    buf: [u8; SIZE],
    _marker: PhantomData<S>,
}

impl<S: Storage, const SIZE: usize> Storage for RamStorage<S, SIZE> {
    const BLOCK_SIZE: usize = S::BLOCK_SIZE;
    const READ_SIZE: usize = S::READ_SIZE;
    const WRITE_SIZE: usize = S::WRITE_SIZE;
    const BLOCK_COUNT: usize = S::BLOCK_COUNT;

    type CACHE_SIZE = S::CACHE_SIZE;
    type LOOKAHEADWORDS_SIZE = S::LOOKAHEADWORDS_SIZE;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, Error> {
        let read_size: usize = Self::READ_SIZE;
        debug_assert!(off % read_size == 0);
        debug_assert!(buf.len() % read_size == 0);
        for (from, to) in self.buf.iter().skip(off).zip(buf.iter_mut()) {
            *to = *from;
        }
        // Data outside of the RAM part is always erased
        for to in buf.iter_mut().skip(self.buf.len().saturating_sub(off)) {
            *to = ERASED;
        }
        info!("{}: {:?}", buf.len(), buf);
        Ok(buf.len())
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, Error> {
        if off + data.len() > SIZE {
            return Err(Error::NoSpace);
        }
        let write_size: usize = Self::WRITE_SIZE;
        debug_assert!(off % write_size == 0);
        debug_assert!(data.len() % write_size == 0);
        for (from, to) in data.iter().zip(self.buf.iter_mut().skip(off)) {
            *to = *from;
        }
        info!("{}: {:?}", data.len(), data);
        Ok(data.len())
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize, Error> {
        let block_size: usize = Self::BLOCK_SIZE;
        debug_assert!(off % block_size == 0);
        debug_assert!(len % block_size == 0);
        for byte in self.buf.iter_mut().skip(off).take(len) {
            *byte = ERASED;
        }
        Ok(len)
    }
}

impl<S, const SIZE: usize> Default for RamStorage<S, SIZE> {
    fn default() -> Self {
        Self {
            buf: [0xff; SIZE],
            _marker: Default::default(),
        }
    }
}

pub enum OptionalStorage<S, const RAM_SIZE: usize = DEFAULT_RAM_SIZE> {
    Storage(S),
    Ram(RamStorage<S, RAM_SIZE>),
}

impl<S: Storage, const RAM_SIZE: usize> OptionalStorage<S, RAM_SIZE> {
    pub fn is_ram(&self) -> bool {
        matches!(self, Self::Ram(_))
    }
}

impl<S: Storage, const RAM_SIZE: usize> Storage for OptionalStorage<S, RAM_SIZE> {
    const BLOCK_SIZE: usize = S::BLOCK_SIZE;
    const READ_SIZE: usize = S::READ_SIZE;
    const WRITE_SIZE: usize = S::WRITE_SIZE;
    const BLOCK_COUNT: usize = S::BLOCK_COUNT;

    type CACHE_SIZE = S::CACHE_SIZE;
    type LOOKAHEADWORDS_SIZE = S::LOOKAHEADWORDS_SIZE;

    fn read(&mut self, off: usize, buf: &mut [u8]) -> Result<usize, Error> {
        info_now!("EFr {:x} {:x}", off, buf.len());
        match self {
            Self::Storage(s) => s.read(off, buf),
            Self::Ram(s) => s.read(off, buf),
        }
    }

    fn write(&mut self, off: usize, data: &[u8]) -> Result<usize, Error> {
        info_now!("EFw {:x} {:x}", off, data.len());
        match self {
            Self::Storage(s) => s.write(off, data),
            Self::Ram(s) => s.write(off, data),
        }
    }

    fn erase(&mut self, off: usize, len: usize) -> Result<usize, Error> {
        info_now!("EFe {:x} {:x}", off, len);
        match self {
            Self::Storage(s) => s.erase(off, len),
            Self::Ram(s) => s.erase(off, len),
        }
    }
}

impl<S, const RAM_SIZE: usize> Default for OptionalStorage<S, RAM_SIZE> {
    fn default() -> Self {
        Self::Ram(Default::default())
    }
}

impl<S, const RAM_SIZE: usize> From<S> for OptionalStorage<S, RAM_SIZE> {
    fn from(storage: S) -> Self {
        Self::Storage(storage)
    }
}

