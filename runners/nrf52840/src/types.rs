use nrf52840_hal::{
	gpio::{Pin, Input, Output, PushPull, PullDown, PullUp, Floating},
	spim,
	twim,
	uarte,
};


pub struct BoardGPIO {
	/* interactive elements */
	pub buttons: [Option<Pin<Input<PullUp>>>; 8],
	pub leds: [Option<Pin<Output<PushPull>>>; 4],
	pub touch: Option<Pin<Output<PushPull>>>,

	/* UARTE0 */
	pub uart_pins: Option<uarte::Pins>,

	/* Fingerprint Reader (through UARTE0) */
	pub fpr_detect: Option<Pin<Input<PullDown>>>,
	pub fpr_power: Option<Pin<Output<PushPull>>>,

	/* LCD (through SPIM0) */
	pub display_spi: Option<spim::Pins>,
	pub display_cs: Option<Pin<Output<PushPull>>>,
	pub display_reset: Option<Pin<Output<PushPull>>>,
	pub display_dc: Option<Pin<Output<PushPull>>>,
	pub display_backlight: Option<Pin<Output<PushPull>>>,
	pub display_power: Option<Pin<Output<PushPull>>>,

	/* Secure Element (through TWIM1) */
	pub se_pins: Option<twim::Pins>,
	pub se_power: Option<Pin<Output<PushPull>>>,

	/* External Flash & NFC (through SPIM3) */
	pub flash_spi: Option<spim::Pins>,
	pub flash_cs: Option<Pin<Output<PushPull>>>,
	pub flash_power: Option<Pin<Output<PushPull>>>,
	pub nfc_cs: Option<Pin<Output<PushPull>>>,
	pub nfc_irq: Option<Pin<Input<PullUp>>>,
}

pub fn is_pin_latched<T>(pin: &Pin<Input<T>>, latches: &[u32]) -> bool {
	let pinport = match pin.port() {
		nrf52840_hal::gpio::Port::Port0 => 0,
		nrf52840_hal::gpio::Port::Port1 => 1
	};
	let pinshift = pin.pin();

	((latches[pinport] >> pinshift) & 1) != 0
}
