// Adapter for the generic `boards::measurement` module read

use core::cell::RefCell;

use cortex_m::interrupt::{self, Mutex};
use lpc55_hal::{
    drivers::{timer::Elapsed as _, Timer},
    peripherals::ctimer::Ctimer1,
    typestates::init_state::Enabled,
};

pub type BootTimer = Timer<Ctimer1<Enabled>>;

static BOOT_TIMER: Mutex<RefCell<Option<BootTimer>>> = Mutex::new(RefCell::new(None));

/// Take ownership of the boot timer and install the reader so that
/// `boards::measurement::now_us()` returns useful values from now on.
pub fn install(timer: BootTimer) {
    interrupt::free(|cs| {
        *BOOT_TIMER.borrow(cs).borrow_mut() = Some(timer);
    });
    crate::measurement::install_now_us(read_us);
}

fn read_us() -> u32 {
    interrupt::free(|cs| {
        BOOT_TIMER
            .borrow(cs)
            .borrow()
            .as_ref()
            .map(|t| t.elapsed().0)
            .unwrap_or(0)
    })
}
