use embedded_time::duration::units::Milliseconds;
use systick_monotonic::{
    fugit::{MillisDurationU64, TimerDurationU64, TimerInstantU64},
    Systick,
};

pub type SystickMonotonic = MonotonicWrapper<Systick<100>>;

pub struct MonotonicWrapper<M: rtic::Monotonic>(M);

impl<
        M: rtic::Monotonic<Instant = TimerInstantU64<FREQ_HZ>, Duration = TimerDurationU64<FREQ_HZ>>,
        const FREQ_HZ: u32,
    > rtic::Monotonic for MonotonicWrapper<M>
{
    type Instant = Milliseconds;
    type Duration = Milliseconds;

    const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = M::DISABLE_INTERRUPT_ON_EMPTY_QUEUE;

    fn now(&mut self) -> Self::Instant {
        convert(self.0.now())
    }

    fn set_compare(&mut self, instant: Self::Instant) {
        // TODO: this does not feel right
        let duration = MillisDurationU64::from_ticks(instant.0.into());
        let duration: M::Duration = duration.convert();
        let instant = M::Instant::from_ticks(duration.ticks());
        self.0.set_compare(instant);
    }

    fn clear_compare_flag(&mut self) {
        self.0.clear_compare_flag();
    }

    fn zero() -> Self::Instant {
        convert(M::zero())
    }

    unsafe fn reset(&mut self) {
        self.0.reset();
    }

    fn on_interrupt(&mut self) {
        self.0.on_interrupt();
    }

    fn enable_timer(&mut self) {
        self.0.enable_timer();
    }

    fn disable_timer(&mut self) {
        self.0.disable_timer();
    }
}

impl<M: rtic::Monotonic> From<M> for MonotonicWrapper<M> {
    fn from(monotonic: M) -> Self {
        Self(monotonic)
    }
}

fn convert<const FREQ_HZ: u32>(instant: TimerInstantU64<FREQ_HZ>) -> Milliseconds {
    let duration: MillisDurationU64 = instant.duration_since_epoch().convert();
    Milliseconds(duration.ticks().try_into().unwrap())
}
