#![no_std]

use core::ops::Range;

const NRF52_MEMORY: Range<usize> = 0x1_000..0xEC_000;

pub struct MemoryRegions {
    pub firmware: Range<usize>,
    pub filesystem: Range<usize>,
}

impl MemoryRegions {
    pub const NK3XN: Self = Self {
        firmware: 0..0x92_000,
        filesystem: 0x93_000..0x9D_E00,
    };

    pub const NK3AM: Self = Self::split(NRF52_MEMORY, 0xD8_000);

    pub const NKPK: Self = Self::split(NRF52_MEMORY, 0xB8_000);

    // Code lives in AXISRAM1, loaded by the debugger; filesystems are RAM-backed.
    pub const NKSO3: Self = Self {
        firmware: 0x3406_4000..0x3410_0000,
        filesystem: 0..0,
    };

    pub const fn split(region: Range<usize>, boundary: usize) -> Self {
        Self {
            firmware: region.start..boundary,
            filesystem: boundary..region.end,
        }
    }
}
