//! Target-specific definitions

use vcell::VolatileCell;

#[cfg(feature = "cortex-m")]
pub use cortex_m::interrupt;
#[cfg(feature = "riscv")]
pub use riscv::interrupt;

use crate::ral::register::RWRegister;
use crate::ral::{
    endpoint0_out, endpoint_in, endpoint_out, otg_device, otg_global, otg_global_dieptxfx,
    otg_pwrclk,
};
use crate::UsbPeripheral;

pub fn fifo_write(usb: UsbRegisters, channel: impl Into<usize>, mut buf: &[u8]) {
    let fifo = usb.fifo(channel.into());

    while buf.len() >= 4 {
        let mut u32_bytes = [0u8; 4];
        u32_bytes.copy_from_slice(&buf[..4]);
        buf = &buf[4..];
        fifo.write(u32::from_ne_bytes(u32_bytes));
    }
    if !buf.is_empty() {
        let mut u32_bytes = [0u8; 4];
        u32_bytes[..buf.len()].copy_from_slice(buf);
        fifo.write(u32::from_ne_bytes(u32_bytes));
    }
}

pub fn fifo_read_into(usb: UsbRegisters, buf: &[VolatileCell<u32>]) {
    let fifo = usb.fifo(0);

    for p in buf {
        let word = fifo.read();
        p.set(word);
    }
}

/// Wrapper around device-specific peripheral that provides unified register interface
#[derive(Copy, Clone)]
pub struct UsbRegisters(usize);

impl UsbRegisters {
    #[inline(always)]
    pub fn new<USB: UsbPeripheral>() -> Self {
        Self(USB::REGISTERS as usize)
    }

    #[inline(always)]
    pub fn global(&self) -> &'static otg_global::RegisterBlock {
        unsafe { &*(self.0 as *const _) }
    }

    #[inline(always)]
    pub fn device(&self) -> &'static otg_device::RegisterBlock {
        unsafe { &*((self.0 + 0x800) as *const _) }
    }

    #[inline(always)]
    pub fn pwrclk(&self) -> &'static otg_pwrclk::RegisterBlock {
        unsafe { &*((self.0 + 0xe00) as *const _) }
    }

    #[inline(always)]
    pub fn fifo(&self, channel: usize) -> &'static RWRegister<u32> {
        assert!(channel <= 15);
        let address = self.0 + 0x1000 + channel * 0x1000;
        unsafe { &*(address as *const RWRegister<u32>) }
    }

    #[inline(always)]
    pub fn dieptxfx(&self, index: usize) -> &'static otg_global_dieptxfx::RegisterBlock {
        let address = self.0 + 0x100 + 4 * index;
        unsafe { &*(address as *const _) }
    }

    #[inline(always)]
    pub fn endpoint_in(&self, index: usize) -> &'static endpoint_in::RegisterBlock {
        let address = self.0 + 0x900 + 0x20 * index;
        unsafe { &*(address as *const _) }
    }

    #[inline(always)]
    pub fn endpoint0_out(&self) -> &'static endpoint0_out::RegisterBlock {
        let address = self.0 + 0xb00;
        unsafe { &*(address as *const _) }
    }

    #[inline(always)]
    pub fn endpoint_out(&self, index: usize) -> &'static endpoint_out::RegisterBlock {
        let address = self.0 + 0xb00 + 0x20 * index;
        unsafe { &*(address as *const _) }
    }
}
