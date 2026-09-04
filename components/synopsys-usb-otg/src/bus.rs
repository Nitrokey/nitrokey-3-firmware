use crate::ral::{
    modify_reg, otg_device, otg_global, otg_global_dieptxfx, otg_pwrclk, read_reg, write_reg,
};
use crate::transition::{EndpointConfig, EndpointDescriptor};
use core::marker::PhantomData;
use embedded_hal::blocking::delay::DelayMs;
use usb_device::bus::{PollResult, UsbBusAllocator};
use usb_device::endpoint::{EndpointAddress, EndpointType};
use usb_device::{Result, UsbDirection, UsbError};

use crate::endpoint::{EndpointIn, EndpointOut};
use crate::endpoint_memory::{EndpointBufferState, EndpointMemoryAllocator};
use crate::target::interrupt::{self, CriticalSection, Mutex};
use crate::target::UsbRegisters;
use crate::{PhyType, UsbPeripheral};

/// USB peripheral driver for STM32 microcontrollers.
pub struct UsbBus<USB> {
    peripheral: USB,
    regs: Mutex<UsbRegisters>,
    allocator: EndpointAllocator<USB>,
}

impl<USB: UsbPeripheral> UsbBus<USB> {
    /// Constructs a new USB peripheral driver.
    pub fn new(peripheral: USB, ep_memory: &'static mut [u32]) -> UsbBusAllocator<Self> {
        let bus = UsbBus {
            peripheral,
            regs: Mutex::new(UsbRegisters::new::<USB>()),
            allocator: EndpointAllocator::new(ep_memory),
        };

        UsbBusAllocator::new(bus)
    }

    pub fn free(self) -> USB {
        self.peripheral
    }

    fn configure_all(&self, cs: &CriticalSection) {
        let regs = self.regs.borrow(cs);

        // Rx FIFO
        // This calculation doesn't correspond to one in a Reference Manual.
        // In fact, the required number of words is higher than indicated in RM.
        // The following numbers are pessimistic and were figured out empirically.
        let rx_fifo_size = if USB::HIGH_SPEED {
            self.allocator.memory_allocator.total_rx_buffer_size_words() + 30
        } else {
            // F429 requires 35+ words for the (EP0[8] + EP2[64]) setup
            // F446 requires 39+ words for the same setup
            self.allocator.memory_allocator.total_rx_buffer_size_words() + 30
        };
        write_reg!(otg_global, regs.global(), GRXFSIZ, rx_fifo_size as u32);
        let mut fifo_top = rx_fifo_size;

        // Tx FIFO #0
        let fifo_size = self.allocator.memory_allocator.tx_fifo_size_words(0);

        #[cfg(feature = "fs")]
        write_reg!(otg_global, regs.global(), DIEPTXF0,
            TX0FD: fifo_size as u32,
            TX0FSA: fifo_top as u32
        );
        #[cfg(feature = "hs")]
        write_reg!(otg_global, regs.global(), GNPTXFSIZ,
            TX0FD: fifo_size as u32,
            TX0FSA: fifo_top as u32
        );

        fifo_top += fifo_size;

        // Tx FIFOs
        for i in 1..USB::ENDPOINT_COUNT {
            let fifo_size = self.allocator.memory_allocator.tx_fifo_size_words(i);

            let dieptxfx = regs.dieptxfx(i);
            write_reg!(otg_global_dieptxfx, dieptxfx, DIEPTXFx,
                INEPTXFD: fifo_size as u32,
                INEPTXSA: fifo_top as u32
            );

            fifo_top += fifo_size;
        }

        assert!(fifo_top as usize <= USB::FIFO_DEPTH_WORDS);

        // Flush Rx & Tx FIFOs
        modify_reg!(otg_global, regs.global(), GRSTCTL, RXFFLSH: 1, TXFFLSH: 1, TXFNUM: 0x10);
        while read_reg!(otg_global, regs.global(), GRSTCTL, RXFFLSH, TXFFLSH) != (0, 0) {}

        for ep in &self.allocator.endpoints_in {
            if let Some(ep) = ep {
                // enabling EP TX interrupt
                modify_reg!(otg_device, regs.device(), DAINTMSK, |v| v
                    | (0x0001 << ep.address().index()));

                ep.configure(cs);
            }
        }

        for ep in &self.allocator.endpoints_out {
            if let Some(ep) = ep {
                if ep.address().index() == 0 {
                    // enabling RX interrupt from EP0
                    modify_reg!(otg_device, regs.device(), DAINTMSK, |v| v | 0x00010000);
                }

                ep.configure(cs);
            }
        }
    }

    fn deconfigure_all(&self, cs: &CriticalSection) {
        let regs = self.regs.borrow(cs);

        // disable interrupts
        modify_reg!(otg_device, regs.device(), DAINTMSK, IEPM: 0, OEPM: 0);

        for ep in &self.allocator.endpoints_in {
            if let Some(ep) = ep {
                ep.deconfigure(cs);
            }
        }

        for ep in &self.allocator.endpoints_out {
            if let Some(ep) = ep {
                ep.deconfigure(cs);
            }
        }
    }

    pub fn force_reset(&self, delay: &mut impl DelayMs<u32>) -> Result<()> {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);
            write_reg!(otg_device, regs.device(), DCTL, SDIS: 1); // Soft disconnect
            delay.delay_ms(3);
            write_reg!(otg_device, regs.device(), DCTL, SDIS: 0); // Soft connect
            delay.delay_ms(3);
        });
        Ok(())
    }

    #[cfg(feature = "hs")]
    /// Reads from a ULPI register in an external ULPI PHY.
    ///
    /// Interrupts are disabled for the duration of the function call.
    ///
    /// **Panics:** if `phy_type` is not `PhyType::ExternalHighSpeed`.
    ///
    /// # Example
    ///
    /// ```
    /// # use synopsys_usb_otg::{UsbPeripheral, UsbBus};
    /// fn read_usb_vid_pid<USB: UsbPeripheral>(bus: &UsbBus<USB>) -> (u16, u16) {
    ///     let mut vid: u16 = bus.ulpi_read(0x00) as u16;
    ///     vid |= (bus.ulpi_read(0x01) as u16) << 8;
    ///     let mut pid: u16 = bus.ulpi_read(0x02) as u16;
    ///     pid |= (bus.ulpi_read(0x03) as u16) << 8;
    ///     (vid, pid)
    /// }
    /// ```
    pub fn ulpi_read(&self, addr: u8) -> core::result::Result<u8, UlpiError> {
        if self.peripheral.phy_type() != PhyType::ExternalHighSpeed {
            panic!("ulpi_read is only supported with external ULPI PHYs");
        }

        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            // Begin ULPI register read transaction
            modify_reg!(otg_global, regs.global(), PHYCR,
                NEW: 1,
                RW: 0,
                ADDR: addr as u32
            );

            // ULPI transactions should take less than 1us. 1000 iterations should be enough even for fast MCUs.
            let mut timeout = 1000;
            while timeout > 0 {
                // Wait for transaction to complete
                if read_reg!(otg_global, regs.global(), PHYCR, DONE) != 0 {
                    // Read transaction data
                    return Ok((read_reg!(otg_global, regs.global(), PHYCR, DATA) & 0xFF) as u8);
                }
                timeout -= 1;
            }
            Err(UlpiError::Timeout)
        })
    }

    #[cfg(feature = "hs")]
    /// Writes to a ULPI register in an external ULPI PHY.
    ///
    /// Interrupts are disabled for the duration of the function call.
    ///
    /// **Panics:** if `phy_type` is not `PhyType::ExternalHighSpeed`.
    pub fn ulpi_write(&self, addr: u8, data: u8) -> core::result::Result<(), UlpiError> {
        if self.peripheral.phy_type() != PhyType::ExternalHighSpeed {
            panic!("ulpi_write is only supported with external ULPI PHYs");
        }

        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            // Begin ULPI register write transaction
            modify_reg!(otg_global, regs.global(), PHYCR,
                NEW: 1,
                RW: 1,
                ADDR: addr as u32,
                DATA: data as u32
            );

            // ULPI transactions should take less than 1us. 1000 iterations should be enough even for fast MCUs.
            let mut timeout = 1000;
            while timeout > 0 {
                // Wait for transaction to complete
                if read_reg!(otg_global, regs.global(), PHYCR, DONE) != 0 {
                    return Ok(());
                }
                timeout -= 1;
            }
            Err(UlpiError::Timeout)
        })
    }
}

#[cfg(feature = "hs")]
#[derive(Debug)]
/// Errors that can occur while interfacing with a ULPI PHY.
pub enum UlpiError {
    /// The action has timed out.
    Timeout,
}

pub(crate) struct EndpointAllocator<USB> {
    bitmap_in: u8,
    bitmap_out: u8,
    endpoints_in: [Option<EndpointIn>; 9],
    endpoints_out: [Option<EndpointOut>; 9],
    memory_allocator: EndpointMemoryAllocator<USB>,
    _marker: PhantomData<USB>,
}

impl<USB: UsbPeripheral> EndpointAllocator<USB> {
    fn new(memory: &'static mut [u32]) -> Self {
        assert!(USB::ENDPOINT_COUNT <= 9);
        Self {
            bitmap_in: 0,
            bitmap_out: 0,
            // [None; 9] requires Copy
            endpoints_in: [None, None, None, None, None, None, None, None, None],
            endpoints_out: [None, None, None, None, None, None, None, None, None],
            memory_allocator: EndpointMemoryAllocator::new(memory),
            _marker: PhantomData,
        }
    }

    fn alloc_number(bitmap: &mut u8, number: Option<u8>) -> Result<u8> {
        if let Some(number) = number {
            if number as usize >= USB::ENDPOINT_COUNT {
                return Err(UsbError::InvalidEndpoint);
            }
            if *bitmap & (1 << number) == 0 {
                *bitmap |= 1 << number;
                Ok(number)
            } else {
                Err(UsbError::InvalidEndpoint)
            }
        } else {
            // Skip EP0
            for number in 1..USB::ENDPOINT_COUNT {
                if *bitmap & (1 << number) == 0 {
                    *bitmap |= 1 << number;
                    return Ok(number as u8);
                }
            }
            Err(UsbError::EndpointOverflow)
        }
    }

    fn alloc(
        bitmap: &mut u8,
        config: &EndpointConfig,
        direction: UsbDirection,
    ) -> Result<EndpointDescriptor> {
        let number = Self::alloc_number(bitmap, config.number)?;
        let address = EndpointAddress::from_parts(number as usize, direction);
        Ok(EndpointDescriptor {
            address,
            ep_type: config.ep_type,
            max_packet_size: config.max_packet_size,
            interval: config.interval,
        })
    }

    fn alloc_in(&mut self, config: &EndpointConfig) -> Result<EndpointIn> {
        let descr = Self::alloc(&mut self.bitmap_in, config, UsbDirection::In)?;

        self.memory_allocator
            .allocate_tx_buffer(descr.address.index() as u8, descr.max_packet_size as usize)?;
        let ep = EndpointIn::new::<USB>(descr);

        Ok(ep)
    }

    fn alloc_out(&mut self, config: &EndpointConfig) -> Result<EndpointOut> {
        let descr = Self::alloc(&mut self.bitmap_out, config, UsbDirection::Out)?;

        let buffer = self
            .memory_allocator
            .allocate_rx_buffer(descr.max_packet_size as usize)?;
        let ep = EndpointOut::new::<USB>(descr, buffer);

        Ok(ep)
    }

    fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<EndpointAddress> {
        let ep_type = unsafe { core::mem::transmute(ep_type) };
        let number = ep_addr.map(|a| a.index() as u8);

        let config = EndpointConfig {
            ep_type,
            max_packet_size,
            interval,
            number,
            pair_of: None,
        };
        match ep_dir {
            UsbDirection::Out => {
                let ep = self.alloc_out(&config)?;
                let address = ep.address();
                self.endpoints_out[address.index()] = Some(ep);
                Ok(address)
            }
            UsbDirection::In => {
                let ep = self.alloc_in(&config)?;
                let address = ep.address();
                self.endpoints_in[address.index()] = Some(ep);
                Ok(address)
            }
        }
    }
}

impl<USB: UsbPeripheral> usb_device::bus::UsbBus for UsbBus<USB> {
    fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<EndpointAddress> {
        self.allocator
            .alloc_ep(ep_dir, ep_addr, ep_type, max_packet_size, interval)
    }

    fn enable(&mut self) {
        // Enable USB_OTG in RCC
        USB::enable();

        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            let core_id = read_reg!(otg_global, regs.global(), CID);

            // Wait for AHB ready
            while read_reg!(otg_global, regs.global(), GRSTCTL, AHBIDL) == 0 {}

            // Configure OTG as device
            #[cfg(feature = "fs")]
            modify_reg!(otg_global, regs.global(), GUSBCFG,
                SRPCAP: 0, // SRP capability is not enabled
                FDMOD: 1 // Force device mode
            );
            #[cfg(feature = "hs")]
            modify_reg!(otg_global, regs.global(), GUSBCFG,
                SRPCAP: 0, // SRP capability is not enabled
                TOCAL: 0x1,
                FDMOD: 1 // Force device mode
            );

            // Configure USB PHY
            #[cfg(feature = "hs")]
            match self.peripheral.phy_type() {
                PhyType::InternalFullSpeed => {
                    // Select FS Embedded PHY
                    modify_reg!(otg_global, regs.global(), GUSBCFG, PHYSEL: 1);
                }
                PhyType::InternalHighSpeed => {
                    // Turn off PHY
                    modify_reg!(otg_global, regs.global(), GCCFG, PWRDWN: 0);

                    // Init The UTMI Interface
                    modify_reg!(otg_global, regs.global(), GUSBCFG,
                        TSDPS: 0,
                        ULPIFSLS: 0,
                        PHYSEL: 0 // ULPI or UTMI
                    );

                    // Select VBUS source
                    modify_reg!(otg_global, regs.global(), GUSBCFG,
                        ULPIEVBUSD: 0,
                        ULPIEVBUSI: 0
                    );

                    // Select UTMI Interace
                    //modify_reg!(otg_global, regs.global(), GUSBCFG, ULPISEL: 0);
                    modify_reg!(otg_global, regs.global(), GUSBCFG, |r| r & !(1 << 4));

                    // This is a secret bit from ST that is not mentioned anywhere except
                    // the driver code shipped with STM32CubeIDE.
                    //modify_reg!(otg_global, regs.global(), GCCFG, PHYHSEN: 1);
                    modify_reg!(otg_global, regs.global(), GCCFG, |r| r | (1 << 23));

                    self.peripheral.setup_internal_hs_phy();
                }
                PhyType::ExternalHighSpeed => {
                    // Turn off embedded PHY
                    modify_reg!(otg_global, regs.global(), GCCFG, PWRDWN: 0);

                    // Init The ULPI Interface
                    modify_reg!(otg_global, regs.global(), GUSBCFG,
                        TSDPS: 0,
                        ULPIFSLS: 0,
                        PHYSEL: 0 // ULPI or UTMI
                    );

                    // Select VBUS source
                    modify_reg!(otg_global, regs.global(), GUSBCFG,
                        ULPIEVBUSD: 0,
                        ULPIEVBUSI: 0
                    );
                }
            }

            // Perform core soft-reset
            while read_reg!(otg_global, regs.global(), GRSTCTL, AHBIDL) == 0 {}
            modify_reg!(otg_global, regs.global(), GRSTCTL, CSRST: 1);
            while read_reg!(otg_global, regs.global(), GRSTCTL, CSRST) == 1 {}

            if self.peripheral.phy_type() == PhyType::InternalFullSpeed {
                // Activate the USB Transceiver
                modify_reg!(otg_global, regs.global(), GCCFG, PWRDWN: 1);
            }

            // Configuring Vbus sense and SOF output
            match core_id {
                0x0000_1200 | 0x0000_1100 => {
                    // F429-like chips have the GCCFG.NOVBUSSENS bit

                    //modify_reg!(otg_global, regs.global, GCCFG, NOVBUSSENS: 1);
                    modify_reg!(otg_global, regs.global(), GCCFG, |r| r | (1 << 21));

                    modify_reg!(otg_global, regs.global(), GCCFG, VBUSASEN: 0, VBUSBSEN: 0, SOFOUTEN: 0);
                }
                0x0000_2000 | 0x0000_2100 | 0x0000_2300 | 0x0000_3000 | 0x0000_3100 => {
                    // F446-like chips have the GCCFG.VBDEN bit with the opposite meaning

                    //modify_reg!(otg_global, regs.global, GCCFG, VBDEN: 0);
                    modify_reg!(otg_global, regs.global(), GCCFG, |r| r & !(1 << 21));

                    // Force B-peripheral session
                    //modify_reg!(otg_global, regs.global, GOTGCTL, BVALOEN: 1, BVALOVAL: 1);
                    modify_reg!(otg_global, regs.global(), GOTGCTL, |r| r | (0b11 << 6));
                }
                _ => {}
            }

            // Enable PHY clock
            write_reg!(otg_pwrclk, regs.pwrclk(), PCGCCTL, 0);

            // Soft disconnect device
            modify_reg!(otg_device, regs.device(), DCTL, SDIS: 1);

            // Setup USB speed and frame interval
            let speed = match (USB::HIGH_SPEED, self.peripheral.phy_type()) {
                (false, _) => 0b11,
                (true, PhyType::InternalFullSpeed) => 0b11,
                (true, PhyType::InternalHighSpeed) => 0b00,
                (true, PhyType::ExternalHighSpeed) => 0b00,
            };
            modify_reg!(otg_device, regs.device(), DCFG,
                PFIVL: 0b00,
                DSPD: speed
            );
            #[cfg(feature = "xcvrdly")]
            modify_reg!(otg_device, regs.device(), DCFG, XCVRDLY: 1);

            // unmask EP interrupts, cores >= 0x5000 signal the SETUP phase end via DOEPINT.STUP
            write_reg!(otg_device, regs.device(), DIEPMSK, XFRCM: 1);
            let oepint = (core_id == 0x0000_5000) as u32;
            write_reg!(otg_device, regs.device(), DOEPMSK, STUPM: oepint);

            // unmask core interrupts
            write_reg!(otg_global, regs.global(), GINTMSK,
                USBRST: 1, ENUMDNEM: 1,
                USBSUSPM: 1, WUIM: 1,
                IEPINT: 1, OEPINT: oepint, RXFLVLM: 1
            );

            // clear pending interrupts
            write_reg!(otg_global, regs.global(), GINTSTS, 0xffffffff);

            // unmask global interrupt
            modify_reg!(otg_global, regs.global(), GAHBCFG, GINT: 1);

            // connect(true)
            modify_reg!(otg_device, regs.device(), DCTL, SDIS: 0);
        });
    }

    fn reset(&self) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            self.configure_all(cs);

            modify_reg!(otg_device, regs.device(), DCFG, DAD: 0);
        });
    }

    fn set_device_address(&self, addr: u8) {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            modify_reg!(otg_device, regs.device(), DCFG, DAD: addr as u32);
        });
    }

    fn write(&self, ep_addr: EndpointAddress, buf: &[u8]) -> Result<usize> {
        if !ep_addr.is_in() || ep_addr.index() >= USB::ENDPOINT_COUNT {
            return Err(UsbError::InvalidEndpoint);
        }
        if let Some(ep) = &self.allocator.endpoints_in[ep_addr.index()] {
            ep.write(buf).map(|_| buf.len())
        } else {
            Err(UsbError::InvalidEndpoint)
        }
    }

    fn read(&self, ep_addr: EndpointAddress, buf: &mut [u8]) -> Result<usize> {
        if !ep_addr.is_out() || ep_addr.index() >= USB::ENDPOINT_COUNT {
            return Err(UsbError::InvalidEndpoint);
        }

        if let Some(ep) = &self.allocator.endpoints_out[ep_addr.index()] {
            ep.read(buf)
        } else {
            Err(UsbError::InvalidEndpoint)
        }
    }

    fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        if ep_addr.index() >= USB::ENDPOINT_COUNT {
            return;
        }

        let regs = UsbRegisters::new::<USB>();
        crate::endpoint::set_stalled(regs, ep_addr, stalled)
    }

    fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        if ep_addr.index() >= USB::ENDPOINT_COUNT {
            return true;
        }

        let regs = UsbRegisters::new::<USB>();
        crate::endpoint::is_stalled(regs, ep_addr)
    }

    fn suspend(&self) {
        // Nothing to do here?
    }

    fn resume(&self) {
        // Nothing to do here?
    }

    fn poll(&self) -> PollResult {
        interrupt::free(|cs| {
            let regs = self.regs.borrow(cs);

            let core_id = read_reg!(otg_global, regs.global(), CID);

            let (wakeup, suspend, enum_done, reset, iep, oep, rxflvl) = read_reg!(
                otg_global,
                regs.global(),
                GINTSTS,
                WKUPINT,
                USBSUSP,
                ENUMDNE,
                USBRST,
                IEPINT,
                OEPINT,
                RXFLVL
            );

            if reset != 0 {
                write_reg!(otg_global, regs.global(), GINTSTS, USBRST: 1);

                self.deconfigure_all(cs);

                // Flush RX
                modify_reg!(otg_global, regs.global(), GRSTCTL, RXFFLSH: 1);
                while read_reg!(otg_global, regs.global(), GRSTCTL, RXFFLSH) == 1 {}
            }

            if enum_done != 0 {
                write_reg!(otg_global, regs.global(), GINTSTS, ENUMDNE: 1);

                let speed = read_reg!(otg_device, regs.device(), DSTS, ENUMSPD);

                // Compute and update TRDT
                let trdt;
                match speed {
                    0b00 => {
                        // High speed

                        // From RM0431 (F72xx), RM0090 (F429), RM0390 (F446)
                        if self.peripheral.ahb_frequency_hz() >= 30_000_000 {
                            trdt = 0x9;
                        } else {
                            panic!("AHB frequency is too low")
                        }
                    }
                    0b01 | 0b11 => {
                        // Full speed

                        // From RM0431 (F72xx), RM0090 (F429)
                        trdt = match self.peripheral.ahb_frequency_hz() {
                            0..=14_199_999 => panic!("AHB frequency is too low"),
                            14_200_000..=14_999_999 => 0xF,
                            15_000_000..=15_999_999 => 0xE,
                            16_000_000..=17_199_999 => 0xD,
                            17_200_000..=18_499_999 => 0xC,
                            18_500_000..=19_999_999 => 0xB,
                            20_000_000..=21_799_999 => 0xA,
                            21_800_000..=23_999_999 => 0x9,
                            24_000_000..=27_499_999 => 0x8,
                            27_500_000..=31_999_999 => 0x7, // 27.7..32 in code from CubeIDE
                            32_000_000..=u32::MAX => 0x6,
                        };
                    }
                    _ => unimplemented!(),
                }
                modify_reg!(otg_global, regs.global(), GUSBCFG, TRDT: trdt);

                PollResult::Reset
            } else if wakeup != 0 {
                // Clear the interrupt
                write_reg!(otg_global, regs.global(), GINTSTS, WKUPINT: 1);

                PollResult::Resume
            } else if suspend != 0 {
                write_reg!(otg_global, regs.global(), GINTSTS, USBSUSP: 1);

                PollResult::Suspend
            } else {
                let mut ep_out = 0;
                let mut ep_in_complete = 0;
                let mut ep_setup = 0;

                use crate::ral::{endpoint_in, endpoint_out};

                // RXFLVL & IEPINT flags are read-only, there is no need to clear them
                if rxflvl != 0 {
                    let (epnum, data_size, status) =
                        read_reg!(otg_global, regs.global(), GRXSTSR, EPNUM, BCNT, PKTSTS);
                    match status {
                        0x02 => {
                            // OUT received
                            ep_out |= 1 << epnum;
                        }
                        0x06 => {
                            // SETUP received
                            // flushing TX if something stuck in control endpoint
                            let ep = regs.endpoint_in(epnum as usize);
                            if read_reg!(endpoint_in, ep, DIEPTSIZ, PKTCNT) != 0 {
                                modify_reg!(otg_global, regs.global(), GRSTCTL, TXFNUM: epnum, TXFFLSH: 1);
                                while read_reg!(otg_global, regs.global(), GRSTCTL, TXFFLSH) == 1 {}
                            }
                            ep_setup |= 1 << epnum;
                        }
                        0x03 | 0x04 => {
                            // OUT completed | SETUP completed
                            // Re-enable the endpoint, F429-like chips only
                            if core_id == 0x0000_1200 || core_id == 0x0000_1100 {
                                let ep = regs.endpoint_out(epnum as usize);
                                modify_reg!(endpoint_out, ep, DOEPCTL, CNAK: 1, EPENA: 1);
                            }
                            // STM32N6-like cores (CID 0x5000) clear PKTCNT/STUPCNT, re-arm them
                            if core_id == 0x0000_5000 {
                                if let Some(ep) = &self.allocator.endpoints_out[epnum as usize] {
                                    ep.rearm();
                                }
                            }
                            read_reg!(otg_global, regs.global(), GRXSTSP); // pop GRXSTSP
                        }
                        _ => {
                            read_reg!(otg_global, regs.global(), GRXSTSP); // pop GRXSTSP
                        }
                    }

                    if status == 0x02 || status == 0x06 {
                        if let Some(ep) = &self.allocator.endpoints_out[epnum as usize] {
                            let mut buffer = ep.buffer.borrow(cs).borrow_mut();
                            if buffer.state() == EndpointBufferState::Empty {
                                read_reg!(otg_global, regs.global(), GRXSTSP); // pop GRXSTSP

                                let is_setup = status == 0x06;
                                buffer
                                    .fill_from_fifo(*regs, data_size as u16, is_setup)
                                    .ok();

                                // Re-enable the endpoint, F446-like chips only
                                if core_id == 0x0000_2000
                                    || core_id == 0x0000_2100
                                    || core_id == 0x0000_2300
                                    || core_id == 0x0000_3000
                                    || core_id == 0x0000_3100
                                {
                                    let ep = regs.endpoint_out(epnum as usize);
                                    modify_reg!(endpoint_out, ep, DOEPCTL, CNAK: 1, EPENA: 1);
                                }
                                if core_id == 0x0000_5000 && !is_setup {
                                    ep.rearm();
                                }
                            }
                        }
                    }
                }

                if oep != 0 {
                    // SETUP phase done: the core disabled the endpoint, re-arm it for the OUT stage
                    for ep in &self.allocator.endpoints_out {
                        if let Some(ep) = ep {
                            let ep_regs = regs.endpoint_out(ep.address().index());
                            if read_reg!(endpoint_out, ep_regs, DOEPINT, STUP) != 0 {
                                write_reg!(endpoint_out, ep_regs, DOEPINT, STUP: 1);
                                ep.rearm();
                            }
                        }
                    }
                }

                if iep != 0 {
                    for ep in &self.allocator.endpoints_in {
                        if let Some(ep) = ep {
                            let ep_regs = regs.endpoint_in(ep.address().index());
                            if read_reg!(endpoint_in, ep_regs, DIEPINT, XFRC) != 0 {
                                write_reg!(endpoint_in, ep_regs, DIEPINT, XFRC: 1);
                                ep_in_complete |= 1 << ep.address().index();
                            }
                        }
                    }
                }

                for ep in &self.allocator.endpoints_out {
                    if let Some(ep) = ep {
                        match ep.buffer_state() {
                            EndpointBufferState::DataOut => {
                                ep_out |= 1 << ep.address().index();
                            }
                            EndpointBufferState::DataSetup => {
                                ep_setup |= 1 << ep.address().index();
                            }
                            EndpointBufferState::Empty => {}
                        }
                    }
                }

                if (ep_in_complete | ep_out | ep_setup) != 0 {
                    PollResult::Data {
                        ep_out,
                        ep_in_complete,
                        ep_setup,
                    }
                } else {
                    PollResult::None
                }
            }
        })
    }

    const QUIRK_SET_ADDRESS_BEFORE_STATUS: bool = true;
}
