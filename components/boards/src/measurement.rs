//! Boot/measurement-timer plumbing.
//!

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

static NOW_US_FN: AtomicUsize = AtomicUsize::new(0);
// offset timer
static EPOCH_US: AtomicU32 = AtomicU32::new(0);
static MEASURED_US: AtomicU32 = AtomicU32::new(0);
static FROZEN: AtomicBool = AtomicBool::new(false);

/// Register the function used by [`now_us`] to read the live timer.
/// Call once during boot; later calls overwrite the previous installer.
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
}
