//! Boot/measurement-timer plumbing.
//!

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

static NOW_US_FN: AtomicUsize = AtomicUsize::new(0);
// offset timer
static EPOCH_US: AtomicU32 = AtomicU32::new(0);
static MEASURED_US: AtomicU32 = AtomicU32::new(0);
static FROZEN: AtomicBool = AtomicBool::new(false);

// save 2 timers
static RX_FIRST_US: AtomicU32 = AtomicU32::new(0);
static TX_FIRST_US: AtomicU32 = AtomicU32::new(0);

// Per-event counters. Useful to tell apart "single late receive" from
// "many retries before first response" when looking at rx/tx timing.
static RX_COUNT: AtomicU32 = AtomicU32::new(0);
static TX_COUNT: AtomicU32 = AtomicU32::new(0);

/// Register the function used by [`now_us`] to read the live timer.
pub fn install_now_us(f: fn() -> u32) {
    NOW_US_FN.store(f as usize, Ordering::Release);
}

/// Read the live boot timer. Returns 0 if no timer has been installed yet.
pub fn now_us() -> u32 {
    let raw = NOW_US_FN.load(Ordering::Acquire);
    if raw == 0 {
        return 0;
    }
    // SAFETY: only `install_now_us` writes this slot, and it only writes
    // valid `fn() -> u32` pointers.
    let f: fn() -> u32 = unsafe { core::mem::transmute(raw) };
    f()
}

/// Snapshot `now_us()`
pub fn freeze_us() {
    if FROZEN
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        let epoch = EPOCH_US.load(Ordering::Acquire);
        let delta = now_us().saturating_sub(epoch);
        MEASURED_US.store(delta, Ordering::Release);
    }
}

/// Frozen snapshot, or 0 if [`freeze_us`] hasn't run yet.
pub fn measured_us() -> u32 {
    MEASURED_US.load(Ordering::Acquire)
}

/// Reset the displayed timer - µs *since this point*.
pub fn reset() {
    FROZEN.store(false, Ordering::Release);
    EPOCH_US.store(now_us(), Ordering::Release);
    MEASURED_US.store(0, Ordering::Release);
    RX_FIRST_US.store(0, Ordering::Release);
    TX_FIRST_US.store(0, Ordering::Release);
    RX_COUNT.store(0, Ordering::Release);
    TX_COUNT.store(0, Ordering::Release);
}

/// Record a receive event: bump the counter, and on the first call after
/// a [`reset`], also latch the timestamp. Safe to call from interrupt
/// context.
pub fn record_rx_us() {
    let v = now_us()
        .saturating_sub(EPOCH_US.load(Ordering::Acquire))
        .max(1);
    let _ = RX_FIRST_US.compare_exchange(0, v, Ordering::AcqRel, Ordering::Acquire);
    RX_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Record a send event: bump the counter, and on the first call after a
/// [`reset`], also latch the timestamp.
pub fn record_tx_us() {
    let v = now_us()
        .saturating_sub(EPOCH_US.load(Ordering::Acquire))
        .max(1);
    let _ = TX_FIRST_US.compare_exchange(0, v, Ordering::AcqRel, Ordering::Acquire);
    TX_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// First-receive timestamp, or 0 if not yet recorded.
pub fn rx_first_us() -> u32 {
    RX_FIRST_US.load(Ordering::Acquire)
}

/// First-send timestamp, or 0 if not yet recorded.
pub fn tx_first_us() -> u32 {
    TX_FIRST_US.load(Ordering::Acquire)
}

/// Total number of receive events recorded since [`reset`].
pub fn rx_count() -> u32 {
    RX_COUNT.load(Ordering::Relaxed)
}

/// Total number of send events recorded since [`reset`].
pub fn tx_count() -> u32 {
    TX_COUNT.load(Ordering::Relaxed)
}
