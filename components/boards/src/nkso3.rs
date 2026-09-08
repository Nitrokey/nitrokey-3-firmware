use core::sync::atomic::{AtomicBool, Ordering};

use littlefs2::{
    consts,
    driver::Storage,
    fs::Filesystem,
    io::{Error as LfsError, Result as LfsResult},
};
use stm32n6::stm32n657::{GPIOC_S, GPIOG_S, TIM7_S};
use stm32n657_hal::{
    gpio::{GpioC, GpioG},
    rcc::{ClockConfig, Rcc},
    timer::Tim7,
};

use crate::{
    nfc::DummyNfc,
    soc::stm32n6::{EpMemory, Stm32n6, TimerClock},
    ui::UserInterface,
    Board,
};

use ui::{Button, Led};

pub mod ui;

pub struct NKSO3;

impl Board for NKSO3 {
    type Soc = Stm32n6;

    type Resources = EpMemory;

    type NfcDevice = DummyNfc;
    type Buttons = Button;
    type Led = Led;

    type InternalStorage = InternalStorage;
    type ExternalStorage = ExternalStorage;

    type Twi = ();
    type Se050Timer = ();

    const BOARD_NAME: &'static str = "NKSO3";
    const HAS_NFC: bool = false;
}

/// RAM-backed littlefs storage over a static buffer, so the value itself stays small.
macro_rules! ram_storage {
    (
        $Name:ident,
        read_size = $read_size:expr,
        write_size = $write_size:expr,
        block_size = $block_size:expr,
        block_count = $block_count:expr,
        cache_size_ty = $cache_size:ty,
    ) => {
        pub struct $Name(());

        impl $Name {
            const SIZE: usize = $block_size * $block_count;

            /// Panics when called a second time, the buffer has a single owner.
            pub fn new() -> Self {
                static TAKEN: AtomicBool = AtomicBool::new(false);
                assert!(!TAKEN.swap(true, Ordering::AcqRel));
                Self(())
            }

            fn buf(&mut self) -> &mut [u8; Self::SIZE] {
                static mut BUF: [u8; $block_size * $block_count] = [0; $block_size * $block_count];
                // SAFETY: `new` hands out one instance and every access goes through `&mut self`.
                unsafe { &mut *(&raw mut BUF) }
            }
        }

        impl Default for $Name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl Storage for $Name {
            const READ_SIZE: usize = $read_size;
            const WRITE_SIZE: usize = $write_size;
            const BLOCK_SIZE: usize = $block_size;
            const BLOCK_COUNT: usize = $block_count;

            type CACHE_SIZE = $cache_size;
            type LOOKAHEAD_SIZE = consts::U1;

            fn read(&mut self, off: usize, data: &mut [u8]) -> LfsResult<usize> {
                let buf = self.buf();
                let end = off.checked_add(data.len()).filter(|end| *end <= buf.len());
                let Some(end) = end else {
                    return Err(LfsError::INVALID);
                };
                data.copy_from_slice(&buf[off..end]);
                Ok(data.len())
            }

            fn write(&mut self, off: usize, data: &[u8]) -> LfsResult<usize> {
                let buf = self.buf();
                let end = off.checked_add(data.len()).filter(|end| *end <= buf.len());
                let Some(end) = end else {
                    return Err(LfsError::NO_SPACE);
                };
                buf[off..end].copy_from_slice(data);
                Ok(data.len())
            }

            fn erase(&mut self, off: usize, len: usize) -> LfsResult<usize> {
                let buf = self.buf();
                let end = off.checked_add(len).filter(|end| *end <= buf.len());
                let Some(end) = end else {
                    return Err(LfsError::INVALID);
                };
                buf[off..end].fill(0xff);
                Ok(len)
            }
        }
    };
}

ram_storage!(
    InternalStorage,
    read_size = 4,
    write_size = 4,
    block_size = 512,
    block_count = 128 * 1024 / 512,
    cache_size_ty = consts::U512,
);

ram_storage!(
    ExternalStorage,
    read_size = 4,
    write_size = 256,
    block_size = 4096,
    block_count = 256 * 1024 / 4096,
    cache_size_ty = consts::U256,
);

pub struct BoardGPIO {
    pub button: Button,
    pub led: Led,
}

pub fn init_pins(gpioc: GPIOC_S, gpiog: GPIOG_S, rcc: &Rcc) -> BoardGPIO {
    let gpioc = GpioC::new(gpioc, rcc);
    let gpiog = GpioG::new(gpiog, rcc);
    BoardGPIO {
        button: Button::init(gpioc.c13),
        led: Led::init(gpiog.g10, gpiog.g0, gpiog.g8),
    }
}

pub fn init_ui(
    gpio: BoardGPIO,
    tim7: TIM7_S,
    rcc: &Rcc,
    clock_config: ClockConfig,
) -> UserInterface<TimerClock, Button, Led> {
    let clock = TimerClock::new(Tim7::new(tim7, rcc), clock_config);
    UserInterface::new(clock, Some(gpio.button), Some(gpio.led))
}

/// Both filesystems are volatile, formatting here keeps the init status clean.
pub fn init_storage() -> (InternalStorage, ExternalStorage) {
    let mut ifs = InternalStorage::new();
    let mut efs = ExternalStorage::new();
    Filesystem::format(&mut ifs).expect("IFS format");
    Filesystem::format(&mut efs).expect("EFS format");
    (ifs, efs)
}
