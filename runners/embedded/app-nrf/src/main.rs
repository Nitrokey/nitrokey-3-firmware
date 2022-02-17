#![no_std]
#![no_main]

use panic_halt as _;
use nrf52840_hal;
use embedded_runner_lib as ERL;

#[macro_use]
extern crate delog;
delog::generate_macros!();

#[rtic::app(device = nrf52840_hal::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
        struct Resources {
		apdu_dispatch: ERL::types::ApduDispatch,
		ctaphid_dispatch: ERL::types::CtaphidDispatch,
		trussed: ERL::types::Trussed,
		apps: ERL::types::Apps,
		usb_classes: Option<ERL::types::UsbClasses>,
		contactless: Option<ERL::types::Iso14443>,

		/* NRF specific */
		// (display UI)
		// (fingerprint sensor)
		// (SE050)

		/* LPC55 specific */
		// perf_timer
		// clock_ctrl
		// wait_extender
	}

        #[init()]
        fn init(mut ctx: init::Context) -> init::LateResources {
		ctx.core.DCB.enable_trace();
		ctx.core.DWT.enable_cycle_counter();

		rtt_target::rtt_init_print!();
		// setup delog+flusher (grab from lpc55)

		// do common setup through (mostly) generic code in ERL::initializer
		// - flash
		// - filesystem
		// - trussed
		// - apps
		// - buttons

		// do board-specific setup
		/* bspobj: ERL::soc::types::BSPObjects = ERL::soc::init_board_specific(...); */
		/* -> idea: BSPObjects contains exactly the "specific" items of App::Resources above;
		   objects have to be individually transferred to Resources to be usable as individual
		   RTIC resources though */

		// compose LateResources
		init::LateResources { ... }
	}

};
