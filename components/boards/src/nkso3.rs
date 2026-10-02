pub mod storage;
pub mod ui;

use core::sync::atomic::{AtomicBool, Ordering};

use littlefs2::{
    consts,
    fs::Filesystem,
    io::{Error as LfsError, Result as LfsResult},
};
use stm32n6::stm32n657::{GPIOB_S, GPIOC_S, GPIOD_S, GPIOG_S, GPIOH_S, SDMMC1_S, TIM7_S};
use stm32n657_hal::{
    gpio::{GpioB, GpioC, GpioD, GpioG, GpioH},
    rcc::{ClockConfig, Rcc},
    sdmmc::Disabled,
    timer::Tim7,
};

use crate::{
    nfc::DummyNfc,
    soc::stm32n6::{EpMemory, Stm32n6, TimerClock},
    ui::UserInterface,
    Board,
};

pub use crate::soc::stm32n6::mmc::{self, Mmc};

use ui::{DummyButton, Led};

pub use storage::{xts_bench_cycles, UsbStorage, BUFFER_LEN};

pub struct NKSO3;

impl Board for NKSO3 {
    type Soc = Stm32n6;

    type Resources = EpMemory;

    type NfcDevice = DummyNfc;
    type Buttons = DummyButton;
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

        impl littlefs2::driver::Storage for $Name {
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
    pub led: Led,
}

pub fn init_pins(
    gpiob: GPIOB_S,
    gpioc: GPIOC_S,
    gpiod: GPIOD_S,
    gpiog: GPIOG_S,
    gpioh: GPIOH_S,
    sdmmc: SDMMC1_S,
    rcc: &Rcc,
) -> (BoardGPIO, Mmc<Disabled>) {
    let gpiob = GpioB::new(gpiob, rcc);
    let gpioc = GpioC::new(gpioc, rcc);
    let gpiod = GpioD::new(gpiod, rcc);
    let gpiog = GpioG::new(gpiog, rcc);
    let gpioh = GpioH::new(gpioh, rcc);
    (
        BoardGPIO {
            led: Led::init(gpiog.g10, gpiog.g1, gpiob.b10),
        },
        Mmc::new(
            sdmmc,
            (
                gpioh.h2.into_sdmmc1_cmd(),
                gpioc.c12.into_sdmmc1_ck(),
                gpiod.d11.into_sdmmc1_d0(),
                gpioc.c9.into_sdmmc1_d1(),
                gpioc.c10.into_sdmmc1_d2(),
                gpioc.c11.into_sdmmc1_d3(),
                gpioh.h9.into_sdmmc1_d4(),
                gpioc.c1.into_sdmmc1_d5(),
                gpioc.c6.into_sdmmc1_d6(),
                gpioc.c7.into_sdmmc1_d7(),
            ),
        ),
    )
}

pub fn init_ui(
    gpio: BoardGPIO,
    tim7: TIM7_S,
    rcc: &Rcc,
    clock_config: ClockConfig,
) -> UserInterface<TimerClock, DummyButton, Led> {
    let clock = TimerClock::new(Tim7::new(tim7, rcc), clock_config);
    UserInterface::new(clock, Some(DummyButton), Some(gpio.led))
}

/// Both filesystems are volatile, formatting here keeps the init status clean.
pub fn init_storage() -> (InternalStorage, ExternalStorage) {
    let mut ifs = InternalStorage::new();
    let mut efs = ExternalStorage::new();
    Filesystem::format(&mut ifs).expect("IFS format");
    Filesystem::format(&mut efs).expect("EFS format");
    (ifs, efs)
}
