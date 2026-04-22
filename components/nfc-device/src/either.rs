use crate::traits::nfc::{Device, Error, State};

pub enum Either<A, B> {
    A(A),
    B(B),
}

impl<A, B> Either<Option<A>, Option<B>> {
    pub fn transpose(self) -> Option<Either<A, B>> {
        match self {
            Either::A(Some(a)) => Some(Either::A(a)),
            Either::B(Some(b)) => Some(Either::B(b)),
            Either::A(None) | Either::B(None) => None,
        }
    }
}

impl<A: Device, B: Device> Device for Either<A, B> {
    fn read(&mut self, buf: &mut [u8]) -> Result<State, Error> {
        match self {
            Self::A(a) => a.read(buf),
            Self::B(b) => b.read(buf),
        }
    }

    fn send(&mut self, buf: &[u8]) -> Result<(), Error> {
        match self {
            Self::A(a) => a.send(buf),
            Self::B(b) => b.send(buf),
        }
    }

    fn frame_size(&self) -> usize {
        match self {
            Self::A(a) => a.frame_size(),
            Self::B(b) => b.frame_size(),
        }
    }
}
