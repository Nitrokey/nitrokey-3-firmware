use crate::hal::drivers::timer::Elapsed;
use embedded_hal::timer::{Cancel, CountDown};
use void::Void;

pub struct Timer<TIMER> where TIMER: Cancel + CountDown + Elapsed {
	pub timer: TIMER
}

impl<TIMER> Timer<TIMER> where TIMER: Cancel + CountDown + Elapsed {
	pub fn new(timer: TIMER) -> Self {
		Self { timer }
	}

	// TODO: actually implement this in terms of lpc55-hal
	pub fn elapsed(&self) -> <TIMER as CountDown>::Time { return self.timer.elapsed(); }
	pub fn start(&mut self, t: <TIMER as CountDown>::Time) { self.timer.start(t); }
	pub fn cancel(&mut self) -> Result<(), <TIMER as Cancel>::Error> { self.timer.cancel() }
	pub fn wait(&mut self) -> Result<(), nb::Error<Void>> { self.timer.wait() }
}
