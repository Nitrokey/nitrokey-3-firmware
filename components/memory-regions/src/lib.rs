#![no_std]

use core::ops::Range;

pub struct MemoryRegions {
    pub firmware: Range<usize>,
    pub filesystem: Range<usize>,
}

impl MemoryRegions {
    pub const LPC55: Self = Self {
        firmware: 0..0x92_000,
        filesystem: 0x93_000..0x9D_E00,
    };

    pub const NRF52: Self = Self::split(0x1_000..0xEC_000, 0xD8_000);

    pub const fn split(region: Range<usize>, boundary: usize) -> Self {
        Self {
            firmware: region.start..boundary,
            filesystem: boundary..region.end,
        }
    }
}
