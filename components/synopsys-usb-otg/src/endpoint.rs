use crate::endpoint_memory::{EndpointBuffer, EndpointBufferState};
use crate::ral::{endpoint0_out, endpoint_in, endpoint_out, modify_reg, read_reg, write_reg};
use crate::target::interrupt::{self, CriticalSection, Mutex};
use crate::target::{fifo_write, UsbRegisters};
use crate::transition::EndpointDescriptor;
use crate::UsbPeripheral;
use core::cell::RefCell;
use core::ops::{Deref, DerefMut};
use usb_device::endpoint::EndpointAddress;
use usb_device::{Result, UsbDirection, UsbError};

pub fn set_stalled(usb: UsbRegisters, address: EndpointAddress, stalled: bool) {
    interrupt::free(|_| match address.direction() {
        UsbDirection::Out => {
            let ep = usb.endpoint_out(address.index() as usize);
            modify_reg!(endpoint_out, ep, DOEPCTL, STALL: stalled as u32);
        }
        UsbDirection::In => {
            let ep = usb.endpoint_in(address.index() as usize);
            modify_reg!(endpoint_in, ep, DIEPCTL, STALL: stalled as u32);
        }
    })
}

pub fn is_stalled(usb: UsbRegisters, address: EndpointAddress) -> bool {
    let stall = match address.direction() {
        UsbDirection::Out => {
            let ep = usb.endpoint_out(address.index());
            read_reg!(endpoint_out, ep, DOEPCTL, STALL)
        }
        UsbDirection::In => {
            let ep = usb.endpoint_in(address.index());
            read_reg!(endpoint_in, ep, DIEPCTL, STALL)
        }
    };
    stall != 0
}

/// Arbitrates access to the endpoint-specific registers and packet buffer memory.
pub struct Endpoint {
    descriptor: EndpointDescriptor,
    usb: UsbRegisters,
}

impl Endpoint {
    pub fn new<USB: UsbPeripheral>(descriptor: EndpointDescriptor) -> Endpoint {
        Endpoint {
            descriptor,
            usb: UsbRegisters::new::<USB>(),
        }
    }

    pub fn address(&self) -> EndpointAddress {
        self.descriptor.address
    }

    #[inline(always)]
    fn index(&self) -> u8 {
        self.descriptor.address.index() as u8
    }
}

pub struct EndpointIn {
    common: Endpoint,
}

impl EndpointIn {
    pub fn new<USB: UsbPeripheral>(descriptor: EndpointDescriptor) -> EndpointIn {
        EndpointIn {
            common: Endpoint::new::<USB>(descriptor),
        }
    }

    pub fn configure(&self, _cs: &CriticalSection) {
        if self.index() == 0 {
            let mpsiz = match self.descriptor.max_packet_size {
                8 => 0b11,
                16 => 0b10,
                32 => 0b01,
                64 => 0b00,
                other => panic!("Unsupported EP0 size: {}", other),
            };

            let regs = self.usb.endpoint_in(self.index() as usize);
            write_reg!(endpoint_in, regs, DIEPCTL, MPSIZ: mpsiz as u32, SNAK: 1);
            write_reg!(endpoint_in, regs, DIEPTSIZ, PKTCNT: 0, XFRSIZ: self.descriptor.max_packet_size as u32);
        } else {
            let regs = self.usb.endpoint_in(self.index() as usize);
            write_reg!(endpoint_in, regs, DIEPCTL,
                SNAK: 1,
                USBAEP: 1,
                EPTYP: self.descriptor.ep_type as u32,
                SD0PID_SEVNFRM: 1,
                TXFNUM: self.index() as u32,
                MPSIZ: self.descriptor.max_packet_size as u32
            );
        }
    }

    pub fn deconfigure(&self, _cs: &CriticalSection) {
        let regs = self.usb.endpoint_in(self.index() as usize);

        // deactivating endpoint
        modify_reg!(endpoint_in, regs, DIEPCTL, USBAEP: 0);

        // TODO: flushing FIFO

        // disabling endpoint
        if read_reg!(endpoint_in, regs, DIEPCTL, EPENA) != 0 && self.index() != 0 {
            modify_reg!(endpoint_in, regs, DIEPCTL, EPDIS: 1)
        }

        // clean EP interrupts
        write_reg!(endpoint_in, regs, DIEPINT, 0xff);

        // TODO: deconfiguring TX FIFO
    }

    pub fn write(&self, buf: &[u8]) -> Result<()> {
        let ep = self.usb.endpoint_in(self.index() as usize);
        if self.index() != 0 && read_reg!(endpoint_in, ep, DIEPCTL, EPENA) != 0 {
            return Err(UsbError::WouldBlock);
        }

        if buf.len() > self.descriptor.max_packet_size as usize {
            return Err(UsbError::BufferOverflow);
        }

        if !buf.is_empty() {
            // Check for FIFO free space
            let size_words = (buf.len() + 3) / 4;
            if size_words > read_reg!(endpoint_in, ep, DTXFSTS, INEPTFSAV) as usize {
                return Err(UsbError::WouldBlock);
            }
        }

        #[cfg(feature = "fs")]
        write_reg!(endpoint_in, ep, DIEPTSIZ, PKTCNT: 1, XFRSIZ: buf.len() as u32);
        #[cfg(feature = "hs")]
        write_reg!(endpoint_in, ep, DIEPTSIZ, MCNT: 1, PKTCNT: 1, XFRSIZ: buf.len() as u32);

        modify_reg!(endpoint_in, ep, DIEPCTL, CNAK: 1, EPENA: 1);

        fifo_write(self.usb, self.index(), buf);

        Ok(())
    }
}

pub struct EndpointOut {
    common: Endpoint,
    pub(crate) buffer: Mutex<RefCell<EndpointBuffer>>,
}

impl EndpointOut {
    pub fn new<USB: UsbPeripheral>(
        descriptor: EndpointDescriptor,
        buffer: EndpointBuffer,
    ) -> EndpointOut {
        EndpointOut {
            common: Endpoint::new::<USB>(descriptor),
            buffer: Mutex::new(RefCell::new(buffer)),
        }
    }

    pub fn configure(&self, _cs: &CriticalSection) {
        if self.index() == 0 {
            let mpsiz = match self.descriptor.max_packet_size {
                8 => 0b11,
                16 => 0b10,
                32 => 0b01,
                64 => 0b00,
                other => panic!("Unsupported EP0 size: {}", other),
            };

            let regs = self.usb.endpoint0_out();
            write_reg!(endpoint0_out, regs, DOEPTSIZ0, STUPCNT: 3, PKTCNT: 1, XFRSIZ: self.descriptor.max_packet_size as u32);
            modify_reg!(endpoint0_out, regs, DOEPCTL0, MPSIZ: mpsiz as u32, EPENA: 1, CNAK: 1);
        } else {
            let regs = self.usb.endpoint_out(self.index() as usize);
            write_reg!(endpoint_out, regs, DOEPCTL,
                SD0PID_SEVNFRM: 1,
                CNAK: 1,
                EPENA: 1,
                USBAEP: 1,
                EPTYP: self.descriptor.ep_type as u32,
                MPSIZ: self.descriptor.max_packet_size as u32
            );
        }
    }

    /// Re-enables a disabled OUT endpoint for one packet, cores >= 0x5000 disable it after every transfer.
    pub fn rearm(&self) {
        if self.index() == 0 {
            let regs = self.usb.endpoint0_out();
            if read_reg!(endpoint0_out, regs, DOEPCTL0, EPENA) != 0 {
                return;
            }
            modify_reg!(endpoint0_out, regs, DOEPTSIZ0, STUPCNT: 3, PKTCNT: 1, XFRSIZ: self.descriptor.max_packet_size as u32);
            modify_reg!(endpoint0_out, regs, DOEPCTL0, CNAK: 1, EPENA: 1);
        } else {
            let regs = self.usb.endpoint_out(self.index() as usize);
            if read_reg!(endpoint_out, regs, DOEPCTL, EPENA) != 0 {
                return;
            }
            modify_reg!(endpoint_out, regs, DOEPTSIZ, PKTCNT: 1, XFRSIZ: self.descriptor.max_packet_size as u32);
            modify_reg!(endpoint_out, regs, DOEPCTL, CNAK: 1, EPENA: 1);
        }
    }

    pub fn deconfigure(&self, _cs: &CriticalSection) {
        let regs = self.usb.endpoint_out(self.index() as usize);

        // deactivating endpoint
        modify_reg!(endpoint_out, regs, DOEPCTL, USBAEP: 0);

        // disabling endpoint
        if read_reg!(endpoint_out, regs, DOEPCTL, EPENA) != 0 && self.index() != 0 {
            modify_reg!(endpoint_out, regs, DOEPCTL, EPDIS: 1)
        }

        // clean EP interrupts
        write_reg!(endpoint_out, regs, DOEPINT, 0xff);
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        interrupt::free(|cs| self.buffer.borrow(cs).borrow_mut().read_packet(buf))
    }

    pub fn buffer_state(&self) -> EndpointBufferState {
        interrupt::free(|cs| self.buffer.borrow(cs).borrow().state())
    }
}

impl Deref for EndpointIn {
    type Target = Endpoint;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl DerefMut for EndpointIn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}

impl Deref for EndpointOut {
    type Target = Endpoint;

    fn deref(&self) -> &Self::Target {
        &self.common
    }
}

impl DerefMut for EndpointOut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.common
    }
}
