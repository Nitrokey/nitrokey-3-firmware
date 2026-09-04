#![allow(non_snake_case, non_upper_case_globals)]
#![allow(non_camel_case_types)]
//! USB on the go high speed
//!
//! Used by: stm32f405, stm32f407, stm32f427, stm32f429, stm32f446

use super::super::register::{RORegister, RWRegister, UnsafeRWRegister};
#[cfg(not(feature = "nosync"))]
use core::marker::PhantomData;

/// OTG_HS device configuration register
pub mod DCFG {

    /// Device speed
    pub mod DSPD {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (2 bits: 0b11 << 0)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Nonzero-length status OUT handshake
    pub mod NZLSOHSK {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
        /// Mask (1 bit: 1 << 2)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Device address
    pub mod DAD {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (7 bits: 0x7f << 4)
        pub const mask: u32 = 0x7f << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Periodic (micro)frame interval
    pub mod PFIVL {
        /// Offset (11 bits)
        pub const offset: u32 = 11;
        /// Mask (2 bits: 0b11 << 11)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    #[cfg(feature = "xcvrdly")]
    /// Transceiver Delay
    pub mod XCVRDLY {
        /// Offset (14 bits)
        pub const offset: u32 = 14;
        /// Mask (1 bit: 1 << 14)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Periodic scheduling interval
    pub mod PERSCHIVL {
        /// Offset (24 bits)
        pub const offset: u32 = 24;
        /// Mask (2 bits: 0b11 << 24)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device control register
pub mod DCTL {

    /// Remote wakeup signaling
    pub mod RWUSIG {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Soft disconnect
    pub mod SDIS {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Global IN NAK status
    pub mod GINSTS {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
        /// Mask (1 bit: 1 << 2)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Global OUT NAK status
    pub mod GONSTS {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Test control
    pub mod TCTL {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (3 bits: 0b111 << 4)
        pub const mask: u32 = 0b111 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set global IN NAK
    pub mod SGINAK {
        /// Offset (7 bits)
        pub const offset: u32 = 7;
        /// Mask (1 bit: 1 << 7)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Clear global IN NAK
    pub mod CGINAK {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set global OUT NAK
    pub mod SGONAK {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Clear global OUT NAK
    pub mod CGONAK {
        /// Offset (10 bits)
        pub const offset: u32 = 10;
        /// Mask (1 bit: 1 << 10)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Power-on programming done
    pub mod POPRGDNE {
        /// Offset (11 bits)
        pub const offset: u32 = 11;
        /// Mask (1 bit: 1 << 11)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device status register
pub mod DSTS {

    /// Suspend status
    pub mod SUSPSTS {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Enumerated speed
    pub mod ENUMSPD {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (2 bits: 0b11 << 1)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Erratic error
    pub mod EERR {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Frame number of the received SOF
    pub mod FNSOF {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (14 bits: 0x3fff << 8)
        pub const mask: u32 = 0x3fff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device IN endpoint common interrupt mask register
pub mod DIEPMSK {

    /// Transfer completed interrupt mask
    pub mod XFRCM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disabled interrupt mask
    pub mod EPDM {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Timeout condition mask (nonisochronous endpoints)
    pub mod TOM {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN token received when TxFIFO empty mask
    pub mod ITTXFEMSK {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN token received with EP mismatch mask
    pub mod INEPNMM {
        /// Offset (5 bits)
        pub const offset: u32 = 5;
        /// Mask (1 bit: 1 << 5)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN endpoint NAK effective mask
    pub mod INEPNEM {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// FIFO underrun mask
    pub mod TXFURM {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// BNA interrupt mask
    pub mod BIM {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device OUT endpoint common interrupt mask register
pub mod DOEPMSK {

    /// Transfer completed interrupt mask
    pub mod XFRCM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disabled interrupt mask
    pub mod EPDM {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// SETUP phase done mask
    pub mod STUPM {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT token received when endpoint disabled mask
    pub mod OTEPDM {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Back-to-back SETUP packets received mask
    pub mod B2BSTUP {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT packet error mask
    pub mod OPEM {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// BNA interrupt mask
    pub mod BOIM {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device all endpoints interrupt register
pub mod DAINT {

    /// IN endpoint interrupt bits
    pub mod IEPINT {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT endpoint interrupt bits
    pub mod OEPINT {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (16 bits: 0xffff << 16)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS all endpoints interrupt mask register
pub mod DAINTMSK {

    /// IN EP interrupt mask bits
    pub mod IEPM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT EP interrupt mask bits
    pub mod OEPM {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (16 bits: 0xffff << 16)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device VBUS discharge time register
pub mod DVBUSDIS {

    /// Device VBUS discharge time
    pub mod VBUSDT {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device VBUS pulsing time register
pub mod DVBUSPULSE {

    /// Device VBUS pulsing time
    pub mod DVBUSP {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (12 bits: 0xfff << 0)
        pub const mask: u32 = 0xfff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS Device threshold control register
pub mod DTHRCTL {

    /// Nonisochronous IN endpoints threshold enable
    pub mod NONISOTHREN {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// ISO IN endpoint threshold enable
    pub mod ISOTHREN {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Transmit threshold length
    pub mod TXTHRLEN {
        /// Offset (2 bits)
        pub const offset: u32 = 2;
        /// Mask (9 bits: 0x1ff << 2)
        pub const mask: u32 = 0x1ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Receive threshold enable
    pub mod RXTHREN {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (1 bit: 1 << 16)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Receive threshold length
    pub mod RXTHRLEN {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (9 bits: 0x1ff << 17)
        pub const mask: u32 = 0x1ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Arbiter parking enable
    pub mod ARPEN {
        /// Offset (27 bits)
        pub const offset: u32 = 27;
        /// Mask (1 bit: 1 << 27)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device IN endpoint FIFO empty interrupt mask register
pub mod DIEPEMPMSK {

    /// IN EP Tx FIFO empty interrupt mask bits
    pub mod INEPTXFEM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device each endpoint interrupt register
pub mod DEACHINT {

    /// IN endpoint 1interrupt bit
    pub mod IEP1INT {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT endpoint 1 interrupt bit
    pub mod OEP1INT {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (1 bit: 1 << 17)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device each endpoint interrupt register mask
pub mod DEACHINTMSK {

    /// IN Endpoint 1 interrupt mask bit
    pub mod IEP1INTM {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT Endpoint 1 interrupt mask bit
    pub mod OEP1INTM {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (1 bit: 1 << 17)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device each in endpoint-1 interrupt register
pub mod DIEPEACHMSK1 {

    /// Transfer completed interrupt mask
    pub mod XFRCM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disabled interrupt mask
    pub mod EPDM {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Timeout condition mask (nonisochronous endpoints)
    pub mod TOM {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN token received when TxFIFO empty mask
    pub mod ITTXFEMSK {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN token received with EP mismatch mask
    pub mod INEPNMM {
        /// Offset (5 bits)
        pub const offset: u32 = 5;
        /// Mask (1 bit: 1 << 5)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN endpoint NAK effective mask
    pub mod INEPNEM {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// FIFO underrun mask
    pub mod TXFURM {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// BNA interrupt mask
    pub mod BIM {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK interrupt mask
    pub mod NAKM {
        /// Offset (13 bits)
        pub const offset: u32 = 13;
        /// Mask (1 bit: 1 << 13)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device each OUT endpoint-1 interrupt register
pub mod DOEPEACHMSK1 {

    /// Transfer completed interrupt mask
    pub mod XFRCM {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disabled interrupt mask
    pub mod EPDM {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Timeout condition mask
    pub mod TOM {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN token received when TxFIFO empty mask
    pub mod ITTXFEMSK {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN token received with EP mismatch mask
    pub mod INEPNMM {
        /// Offset (5 bits)
        pub const offset: u32 = 5;
        /// Mask (1 bit: 1 << 5)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN endpoint NAK effective mask
    pub mod INEPNEM {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT packet error mask
    pub mod TXFURM {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// BNA interrupt mask
    pub mod BIM {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Bubble error interrupt mask
    pub mod BERRM {
        /// Offset (12 bits)
        pub const offset: u32 = 12;
        /// Mask (1 bit: 1 << 12)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK interrupt mask
    pub mod NAKM {
        /// Offset (13 bits)
        pub const offset: u32 = 13;
        /// Mask (1 bit: 1 << 13)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NYET interrupt mask
    pub mod NYETM {
        /// Offset (14 bits)
        pub const offset: u32 = 14;
        /// Mask (1 bit: 1 << 14)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG device endpoint-0 control register
pub mod DIEPCTL0 {

    /// Maximum packet size
    pub mod MPSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (11 bits: 0x7ff << 0)
        pub const mask: u32 = 0x7ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// USB active endpoint
    pub mod USBAEP {
        /// Offset (15 bits)
        pub const offset: u32 = 15;
        /// Mask (1 bit: 1 << 15)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Even/odd frame
    pub mod EONUM_DPID {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (1 bit: 1 << 16)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK status
    pub mod NAKSTS {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (1 bit: 1 << 17)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint type
    pub mod EPTYP {
        /// Offset (18 bits)
        pub const offset: u32 = 18;
        /// Mask (2 bits: 0b11 << 18)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// STALL handshake
    pub mod STALL {
        /// Offset (21 bits)
        pub const offset: u32 = 21;
        /// Mask (1 bit: 1 << 21)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// TxFIFO number
    pub mod TXFNUM {
        /// Offset (22 bits)
        pub const offset: u32 = 22;
        /// Mask (4 bits: 0b1111 << 22)
        pub const mask: u32 = 0b1111 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Clear NAK
    pub mod CNAK {
        /// Offset (26 bits)
        pub const offset: u32 = 26;
        /// Mask (1 bit: 1 << 26)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set NAK
    pub mod SNAK {
        /// Offset (27 bits)
        pub const offset: u32 = 27;
        /// Mask (1 bit: 1 << 27)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set DATA0 PID
    pub mod SD0PID_SEVNFRM {
        /// Offset (28 bits)
        pub const offset: u32 = 28;
        /// Mask (1 bit: 1 << 28)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set odd frame
    pub mod SODDFRM {
        /// Offset (29 bits)
        pub const offset: u32 = 29;
        /// Mask (1 bit: 1 << 29)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disable
    pub mod EPDIS {
        /// Offset (30 bits)
        pub const offset: u32 = 30;
        /// Mask (1 bit: 1 << 30)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint enable
    pub mod EPENA {
        /// Offset (31 bits)
        pub const offset: u32 = 31;
        /// Mask (1 bit: 1 << 31)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG device endpoint-1 control register
pub mod DIEPCTL1 {
    pub use super::DIEPCTL0::CNAK;
    pub use super::DIEPCTL0::EONUM_DPID;
    pub use super::DIEPCTL0::EPDIS;
    pub use super::DIEPCTL0::EPENA;
    pub use super::DIEPCTL0::EPTYP;
    pub use super::DIEPCTL0::MPSIZ;
    pub use super::DIEPCTL0::NAKSTS;
    pub use super::DIEPCTL0::SD0PID_SEVNFRM;
    pub use super::DIEPCTL0::SNAK;
    pub use super::DIEPCTL0::SODDFRM;
    pub use super::DIEPCTL0::STALL;
    pub use super::DIEPCTL0::TXFNUM;
    pub use super::DIEPCTL0::USBAEP;
}

/// OTG device endpoint-1 control register
pub mod DIEPCTL2 {
    pub use super::DIEPCTL0::CNAK;
    pub use super::DIEPCTL0::EONUM_DPID;
    pub use super::DIEPCTL0::EPDIS;
    pub use super::DIEPCTL0::EPENA;
    pub use super::DIEPCTL0::EPTYP;
    pub use super::DIEPCTL0::MPSIZ;
    pub use super::DIEPCTL0::NAKSTS;
    pub use super::DIEPCTL0::SD0PID_SEVNFRM;
    pub use super::DIEPCTL0::SNAK;
    pub use super::DIEPCTL0::SODDFRM;
    pub use super::DIEPCTL0::STALL;
    pub use super::DIEPCTL0::TXFNUM;
    pub use super::DIEPCTL0::USBAEP;
}

/// OTG device endpoint-1 control register
pub mod DIEPCTL3 {
    pub use super::DIEPCTL0::CNAK;
    pub use super::DIEPCTL0::EONUM_DPID;
    pub use super::DIEPCTL0::EPDIS;
    pub use super::DIEPCTL0::EPENA;
    pub use super::DIEPCTL0::EPTYP;
    pub use super::DIEPCTL0::MPSIZ;
    pub use super::DIEPCTL0::NAKSTS;
    pub use super::DIEPCTL0::SD0PID_SEVNFRM;
    pub use super::DIEPCTL0::SNAK;
    pub use super::DIEPCTL0::SODDFRM;
    pub use super::DIEPCTL0::STALL;
    pub use super::DIEPCTL0::TXFNUM;
    pub use super::DIEPCTL0::USBAEP;
}

/// OTG device endpoint-1 control register
pub mod DIEPCTL4 {
    pub use super::DIEPCTL0::CNAK;
    pub use super::DIEPCTL0::EONUM_DPID;
    pub use super::DIEPCTL0::EPDIS;
    pub use super::DIEPCTL0::EPENA;
    pub use super::DIEPCTL0::EPTYP;
    pub use super::DIEPCTL0::MPSIZ;
    pub use super::DIEPCTL0::NAKSTS;
    pub use super::DIEPCTL0::SD0PID_SEVNFRM;
    pub use super::DIEPCTL0::SNAK;
    pub use super::DIEPCTL0::SODDFRM;
    pub use super::DIEPCTL0::STALL;
    pub use super::DIEPCTL0::TXFNUM;
    pub use super::DIEPCTL0::USBAEP;
}

/// OTG device endpoint-1 control register
pub mod DIEPCTL5 {
    pub use super::DIEPCTL0::CNAK;
    pub use super::DIEPCTL0::EONUM_DPID;
    pub use super::DIEPCTL0::EPDIS;
    pub use super::DIEPCTL0::EPENA;
    pub use super::DIEPCTL0::EPTYP;
    pub use super::DIEPCTL0::MPSIZ;
    pub use super::DIEPCTL0::NAKSTS;
    pub use super::DIEPCTL0::SD0PID_SEVNFRM;
    pub use super::DIEPCTL0::SNAK;
    pub use super::DIEPCTL0::SODDFRM;
    pub use super::DIEPCTL0::STALL;
    pub use super::DIEPCTL0::TXFNUM;
    pub use super::DIEPCTL0::USBAEP;
}

/// OTG device endpoint-0 interrupt register
pub mod DIEPINT0 {

    /// Transfer completed interrupt
    pub mod XFRC {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disabled interrupt
    pub mod EPDISD {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Timeout condition
    pub mod TOC {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN token received when TxFIFO is empty
    pub mod ITTXFE {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// IN endpoint NAK effective
    pub mod INEPNE {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Transmit FIFO empty
    pub mod TXFE {
        /// Offset (7 bits)
        pub const offset: u32 = 7;
        /// Mask (1 bit: 1 << 7)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Transmit Fifo Underrun
    pub mod TXFIFOUDRN {
        /// Offset (8 bits)
        pub const offset: u32 = 8;
        /// Mask (1 bit: 1 << 8)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Buffer not available interrupt
    pub mod BNA {
        /// Offset (9 bits)
        pub const offset: u32 = 9;
        /// Mask (1 bit: 1 << 9)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Packet dropped status
    pub mod PKTDRPSTS {
        /// Offset (11 bits)
        pub const offset: u32 = 11;
        /// Mask (1 bit: 1 << 11)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Babble error interrupt
    pub mod BERR {
        /// Offset (12 bits)
        pub const offset: u32 = 12;
        /// Mask (1 bit: 1 << 12)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK interrupt
    pub mod NAK {
        /// Offset (13 bits)
        pub const offset: u32 = 13;
        /// Mask (1 bit: 1 << 13)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG device endpoint-0 interrupt register
pub mod DIEPINT1 {
    pub use super::DIEPINT0::BERR;
    pub use super::DIEPINT0::BNA;
    pub use super::DIEPINT0::EPDISD;
    pub use super::DIEPINT0::INEPNE;
    pub use super::DIEPINT0::ITTXFE;
    pub use super::DIEPINT0::NAK;
    pub use super::DIEPINT0::PKTDRPSTS;
    pub use super::DIEPINT0::TOC;
    pub use super::DIEPINT0::TXFE;
    pub use super::DIEPINT0::TXFIFOUDRN;
    pub use super::DIEPINT0::XFRC;
}

/// OTG device endpoint-0 interrupt register
pub mod DIEPINT2 {
    pub use super::DIEPINT0::BERR;
    pub use super::DIEPINT0::BNA;
    pub use super::DIEPINT0::EPDISD;
    pub use super::DIEPINT0::INEPNE;
    pub use super::DIEPINT0::ITTXFE;
    pub use super::DIEPINT0::NAK;
    pub use super::DIEPINT0::PKTDRPSTS;
    pub use super::DIEPINT0::TOC;
    pub use super::DIEPINT0::TXFE;
    pub use super::DIEPINT0::TXFIFOUDRN;
    pub use super::DIEPINT0::XFRC;
}

/// OTG device endpoint-0 interrupt register
pub mod DIEPINT3 {
    pub use super::DIEPINT0::BERR;
    pub use super::DIEPINT0::BNA;
    pub use super::DIEPINT0::EPDISD;
    pub use super::DIEPINT0::INEPNE;
    pub use super::DIEPINT0::ITTXFE;
    pub use super::DIEPINT0::NAK;
    pub use super::DIEPINT0::PKTDRPSTS;
    pub use super::DIEPINT0::TOC;
    pub use super::DIEPINT0::TXFE;
    pub use super::DIEPINT0::TXFIFOUDRN;
    pub use super::DIEPINT0::XFRC;
}

/// OTG device endpoint-0 interrupt register
pub mod DIEPINT4 {
    pub use super::DIEPINT0::BERR;
    pub use super::DIEPINT0::BNA;
    pub use super::DIEPINT0::EPDISD;
    pub use super::DIEPINT0::INEPNE;
    pub use super::DIEPINT0::ITTXFE;
    pub use super::DIEPINT0::NAK;
    pub use super::DIEPINT0::PKTDRPSTS;
    pub use super::DIEPINT0::TOC;
    pub use super::DIEPINT0::TXFE;
    pub use super::DIEPINT0::TXFIFOUDRN;
    pub use super::DIEPINT0::XFRC;
}

/// OTG device endpoint-0 interrupt register
pub mod DIEPINT5 {
    pub use super::DIEPINT0::BERR;
    pub use super::DIEPINT0::BNA;
    pub use super::DIEPINT0::EPDISD;
    pub use super::DIEPINT0::INEPNE;
    pub use super::DIEPINT0::ITTXFE;
    pub use super::DIEPINT0::NAK;
    pub use super::DIEPINT0::PKTDRPSTS;
    pub use super::DIEPINT0::TOC;
    pub use super::DIEPINT0::TXFE;
    pub use super::DIEPINT0::TXFIFOUDRN;
    pub use super::DIEPINT0::XFRC;
}

/// OTG_HS device IN endpoint 0 transfer size register
pub mod DIEPTSIZ0 {

    /// Transfer size
    pub mod XFRSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (7 bits: 0x7f << 0)
        pub const mask: u32 = 0x7f << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Packet count
    pub mod PKTCNT {
        /// Offset (19 bits)
        pub const offset: u32 = 19;
        /// Mask (2 bits: 0b11 << 19)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device endpoint-1 DMA address register
pub mod DIEPDMA1 {

    /// DMA address
    pub mod DMAADDR {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (32 bits: 0xffffffff << 0)
        pub const mask: u32 = 0xffffffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device endpoint-2 DMA address register
pub mod DIEPDMA2 {
    pub use super::DIEPDMA1::DMAADDR;
}

/// OTG_HS device endpoint-3 DMA address register
pub mod DIEPDMA3 {
    pub use super::DIEPDMA1::DMAADDR;
}

/// OTG_HS device endpoint-4 DMA address register
pub mod DIEPDMA4 {
    pub use super::DIEPDMA1::DMAADDR;
}

/// OTG_HS device endpoint-5 DMA address register
pub mod DIEPDMA5 {
    pub use super::DIEPDMA1::DMAADDR;
}

/// OTG_HS device IN endpoint transmit FIFO status register
pub mod DTXFSTS0 {

    /// IN endpoint TxFIFO space avail
    pub mod INEPTFSAV {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (16 bits: 0xffff << 0)
        pub const mask: u32 = 0xffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device IN endpoint transmit FIFO status register
pub mod DTXFSTS1 {
    pub use super::DTXFSTS0::INEPTFSAV;
}

/// OTG_HS device IN endpoint transmit FIFO status register
pub mod DTXFSTS2 {
    pub use super::DTXFSTS0::INEPTFSAV;
}

/// OTG_HS device IN endpoint transmit FIFO status register
pub mod DTXFSTS3 {
    pub use super::DTXFSTS0::INEPTFSAV;
}

/// OTG_HS device IN endpoint transmit FIFO status register
pub mod DTXFSTS4 {
    pub use super::DTXFSTS0::INEPTFSAV;
}

/// OTG_HS device IN endpoint transmit FIFO status register
pub mod DTXFSTS5 {
    pub use super::DTXFSTS0::INEPTFSAV;
}

/// OTG_HS device endpoint transfer size register
pub mod DIEPTSIZ1 {

    /// Transfer size
    pub mod XFRSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (19 bits: 0x7ffff << 0)
        pub const mask: u32 = 0x7ffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Packet count
    pub mod PKTCNT {
        /// Offset (19 bits)
        pub const offset: u32 = 19;
        /// Mask (10 bits: 0x3ff << 19)
        pub const mask: u32 = 0x3ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Multi count
    pub mod MCNT {
        /// Offset (29 bits)
        pub const offset: u32 = 29;
        /// Mask (2 bits: 0b11 << 29)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device endpoint transfer size register
pub mod DIEPTSIZ2 {
    pub use super::DIEPTSIZ1::MCNT;
    pub use super::DIEPTSIZ1::PKTCNT;
    pub use super::DIEPTSIZ1::XFRSIZ;
}

/// OTG_HS device endpoint transfer size register
pub mod DIEPTSIZ3 {
    pub use super::DIEPTSIZ1::MCNT;
    pub use super::DIEPTSIZ1::PKTCNT;
    pub use super::DIEPTSIZ1::XFRSIZ;
}

/// OTG_HS device endpoint transfer size register
pub mod DIEPTSIZ4 {
    pub use super::DIEPTSIZ1::MCNT;
    pub use super::DIEPTSIZ1::PKTCNT;
    pub use super::DIEPTSIZ1::XFRSIZ;
}

/// OTG_HS device endpoint transfer size register
pub mod DIEPTSIZ5 {
    pub use super::DIEPTSIZ1::MCNT;
    pub use super::DIEPTSIZ1::PKTCNT;
    pub use super::DIEPTSIZ1::XFRSIZ;
}

/// OTG_HS device control OUT endpoint 0 control register
pub mod DOEPCTL0 {

    /// Maximum packet size
    pub mod MPSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (2 bits: 0b11 << 0)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// USB active endpoint
    pub mod USBAEP {
        /// Offset (15 bits)
        pub const offset: u32 = 15;
        /// Mask (1 bit: 1 << 15)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK status
    pub mod NAKSTS {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (1 bit: 1 << 17)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint type
    pub mod EPTYP {
        /// Offset (18 bits)
        pub const offset: u32 = 18;
        /// Mask (2 bits: 0b11 << 18)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Snoop mode
    pub mod SNPM {
        /// Offset (20 bits)
        pub const offset: u32 = 20;
        /// Mask (1 bit: 1 << 20)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// STALL handshake
    pub mod STALL {
        /// Offset (21 bits)
        pub const offset: u32 = 21;
        /// Mask (1 bit: 1 << 21)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Clear NAK
    pub mod CNAK {
        /// Offset (26 bits)
        pub const offset: u32 = 26;
        /// Mask (1 bit: 1 << 26)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set NAK
    pub mod SNAK {
        /// Offset (27 bits)
        pub const offset: u32 = 27;
        /// Mask (1 bit: 1 << 27)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disable
    pub mod EPDIS {
        /// Offset (30 bits)
        pub const offset: u32 = 30;
        /// Mask (1 bit: 1 << 30)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint enable
    pub mod EPENA {
        /// Offset (31 bits)
        pub const offset: u32 = 31;
        /// Mask (1 bit: 1 << 31)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG device endpoint-1 control register
pub mod DOEPCTL1 {

    /// Maximum packet size
    pub mod MPSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (11 bits: 0x7ff << 0)
        pub const mask: u32 = 0x7ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// USB active endpoint
    pub mod USBAEP {
        /// Offset (15 bits)
        pub const offset: u32 = 15;
        /// Mask (1 bit: 1 << 15)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Even odd frame/Endpoint data PID
    pub mod EONUM_DPID {
        /// Offset (16 bits)
        pub const offset: u32 = 16;
        /// Mask (1 bit: 1 << 16)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NAK status
    pub mod NAKSTS {
        /// Offset (17 bits)
        pub const offset: u32 = 17;
        /// Mask (1 bit: 1 << 17)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint type
    pub mod EPTYP {
        /// Offset (18 bits)
        pub const offset: u32 = 18;
        /// Mask (2 bits: 0b11 << 18)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Snoop mode
    pub mod SNPM {
        /// Offset (20 bits)
        pub const offset: u32 = 20;
        /// Mask (1 bit: 1 << 20)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// STALL handshake
    pub mod STALL {
        /// Offset (21 bits)
        pub const offset: u32 = 21;
        /// Mask (1 bit: 1 << 21)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Clear NAK
    pub mod CNAK {
        /// Offset (26 bits)
        pub const offset: u32 = 26;
        /// Mask (1 bit: 1 << 26)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set NAK
    pub mod SNAK {
        /// Offset (27 bits)
        pub const offset: u32 = 27;
        /// Mask (1 bit: 1 << 27)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set DATA0 PID/Set even frame
    pub mod SD0PID_SEVNFRM {
        /// Offset (28 bits)
        pub const offset: u32 = 28;
        /// Mask (1 bit: 1 << 28)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Set odd frame
    pub mod SODDFRM {
        /// Offset (29 bits)
        pub const offset: u32 = 29;
        /// Mask (1 bit: 1 << 29)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disable
    pub mod EPDIS {
        /// Offset (30 bits)
        pub const offset: u32 = 30;
        /// Mask (1 bit: 1 << 30)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint enable
    pub mod EPENA {
        /// Offset (31 bits)
        pub const offset: u32 = 31;
        /// Mask (1 bit: 1 << 31)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG device endpoint-1 control register
pub mod DOEPCTL2 {
    pub use super::DOEPCTL1::CNAK;
    pub use super::DOEPCTL1::EONUM_DPID;
    pub use super::DOEPCTL1::EPDIS;
    pub use super::DOEPCTL1::EPENA;
    pub use super::DOEPCTL1::EPTYP;
    pub use super::DOEPCTL1::MPSIZ;
    pub use super::DOEPCTL1::NAKSTS;
    pub use super::DOEPCTL1::SD0PID_SEVNFRM;
    pub use super::DOEPCTL1::SNAK;
    pub use super::DOEPCTL1::SNPM;
    pub use super::DOEPCTL1::SODDFRM;
    pub use super::DOEPCTL1::STALL;
    pub use super::DOEPCTL1::USBAEP;
}

/// OTG device endpoint-1 control register
pub mod DOEPCTL3 {
    pub use super::DOEPCTL1::CNAK;
    pub use super::DOEPCTL1::EONUM_DPID;
    pub use super::DOEPCTL1::EPDIS;
    pub use super::DOEPCTL1::EPENA;
    pub use super::DOEPCTL1::EPTYP;
    pub use super::DOEPCTL1::MPSIZ;
    pub use super::DOEPCTL1::NAKSTS;
    pub use super::DOEPCTL1::SD0PID_SEVNFRM;
    pub use super::DOEPCTL1::SNAK;
    pub use super::DOEPCTL1::SNPM;
    pub use super::DOEPCTL1::SODDFRM;
    pub use super::DOEPCTL1::STALL;
    pub use super::DOEPCTL1::USBAEP;
}

/// OTG device endpoint-1 control register
pub mod DOEPCTL4 {
    pub use super::DOEPCTL1::CNAK;
    pub use super::DOEPCTL1::EONUM_DPID;
    pub use super::DOEPCTL1::EPDIS;
    pub use super::DOEPCTL1::EPENA;
    pub use super::DOEPCTL1::EPTYP;
    pub use super::DOEPCTL1::MPSIZ;
    pub use super::DOEPCTL1::NAKSTS;
    pub use super::DOEPCTL1::SD0PID_SEVNFRM;
    pub use super::DOEPCTL1::SNAK;
    pub use super::DOEPCTL1::SNPM;
    pub use super::DOEPCTL1::SODDFRM;
    pub use super::DOEPCTL1::STALL;
    pub use super::DOEPCTL1::USBAEP;
}

/// OTG device endpoint-1 control register
pub mod DOEPCTL5 {
    pub use super::DOEPCTL1::CNAK;
    pub use super::DOEPCTL1::EONUM_DPID;
    pub use super::DOEPCTL1::EPDIS;
    pub use super::DOEPCTL1::EPENA;
    pub use super::DOEPCTL1::EPTYP;
    pub use super::DOEPCTL1::MPSIZ;
    pub use super::DOEPCTL1::NAKSTS;
    pub use super::DOEPCTL1::SD0PID_SEVNFRM;
    pub use super::DOEPCTL1::SNAK;
    pub use super::DOEPCTL1::SNPM;
    pub use super::DOEPCTL1::SODDFRM;
    pub use super::DOEPCTL1::STALL;
    pub use super::DOEPCTL1::USBAEP;
}

/// OTG_HS device endpoint-0 interrupt register
pub mod DOEPINT0 {

    /// Transfer completed interrupt
    pub mod XFRC {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (1 bit: 1 << 0)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Endpoint disabled interrupt
    pub mod EPDISD {
        /// Offset (1 bits)
        pub const offset: u32 = 1;
        /// Mask (1 bit: 1 << 1)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// SETUP phase done
    pub mod STUP {
        /// Offset (3 bits)
        pub const offset: u32 = 3;
        /// Mask (1 bit: 1 << 3)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// OUT token received when endpoint disabled
    pub mod OTEPDIS {
        /// Offset (4 bits)
        pub const offset: u32 = 4;
        /// Mask (1 bit: 1 << 4)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Back-to-back SETUP packets received
    pub mod B2BSTUP {
        /// Offset (6 bits)
        pub const offset: u32 = 6;
        /// Mask (1 bit: 1 << 6)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// NYET interrupt
    pub mod NYET {
        /// Offset (14 bits)
        pub const offset: u32 = 14;
        /// Mask (1 bit: 1 << 14)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device endpoint-0 interrupt register
pub mod DOEPINT1 {
    pub use super::DOEPINT0::B2BSTUP;
    pub use super::DOEPINT0::EPDISD;
    pub use super::DOEPINT0::NYET;
    pub use super::DOEPINT0::OTEPDIS;
    pub use super::DOEPINT0::STUP;
    pub use super::DOEPINT0::XFRC;
}

/// OTG_HS device endpoint-0 interrupt register
pub mod DOEPINT2 {
    pub use super::DOEPINT0::B2BSTUP;
    pub use super::DOEPINT0::EPDISD;
    pub use super::DOEPINT0::NYET;
    pub use super::DOEPINT0::OTEPDIS;
    pub use super::DOEPINT0::STUP;
    pub use super::DOEPINT0::XFRC;
}

/// OTG_HS device endpoint-0 interrupt register
pub mod DOEPINT3 {
    pub use super::DOEPINT0::B2BSTUP;
    pub use super::DOEPINT0::EPDISD;
    pub use super::DOEPINT0::NYET;
    pub use super::DOEPINT0::OTEPDIS;
    pub use super::DOEPINT0::STUP;
    pub use super::DOEPINT0::XFRC;
}

/// OTG_HS device endpoint-0 interrupt register
pub mod DOEPINT4 {
    pub use super::DOEPINT0::B2BSTUP;
    pub use super::DOEPINT0::EPDISD;
    pub use super::DOEPINT0::NYET;
    pub use super::DOEPINT0::OTEPDIS;
    pub use super::DOEPINT0::STUP;
    pub use super::DOEPINT0::XFRC;
}

/// OTG_HS device endpoint-0 interrupt register
pub mod DOEPINT5 {
    pub use super::DOEPINT0::B2BSTUP;
    pub use super::DOEPINT0::EPDISD;
    pub use super::DOEPINT0::NYET;
    pub use super::DOEPINT0::OTEPDIS;
    pub use super::DOEPINT0::STUP;
    pub use super::DOEPINT0::XFRC;
}

/// OTG_HS device endpoint-1 transfer size register
pub mod DOEPTSIZ0 {

    /// Transfer size
    pub mod XFRSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (7 bits: 0x7f << 0)
        pub const mask: u32 = 0x7f << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Packet count
    pub mod PKTCNT {
        /// Offset (19 bits)
        pub const offset: u32 = 19;
        /// Mask (1 bit: 1 << 19)
        pub const mask: u32 = 1 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// SETUP packet count
    pub mod STUPCNT {
        /// Offset (29 bits)
        pub const offset: u32 = 29;
        /// Mask (2 bits: 0b11 << 29)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device endpoint-2 transfer size register
pub mod DOEPTSIZ1 {

    /// Transfer size
    pub mod XFRSIZ {
        /// Offset (0 bits)
        pub const offset: u32 = 0;
        /// Mask (19 bits: 0x7ffff << 0)
        pub const mask: u32 = 0x7ffff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Packet count
    pub mod PKTCNT {
        /// Offset (19 bits)
        pub const offset: u32 = 19;
        /// Mask (10 bits: 0x3ff << 19)
        pub const mask: u32 = 0x3ff << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }

    /// Received data PID/SETUP packet count
    pub mod RXDPID_STUPCNT {
        /// Offset (29 bits)
        pub const offset: u32 = 29;
        /// Mask (2 bits: 0b11 << 29)
        pub const mask: u32 = 0b11 << offset;
        /// Read-only values (empty)
        pub mod R {}
        /// Write-only values (empty)
        pub mod W {}
        /// Read-write values (empty)
        pub mod RW {}
    }
}

/// OTG_HS device endpoint-2 transfer size register
pub mod DOEPTSIZ2 {
    pub use super::DOEPTSIZ1::PKTCNT;
    pub use super::DOEPTSIZ1::RXDPID_STUPCNT;
    pub use super::DOEPTSIZ1::XFRSIZ;
}

/// OTG_HS device endpoint-2 transfer size register
pub mod DOEPTSIZ3 {
    pub use super::DOEPTSIZ1::PKTCNT;
    pub use super::DOEPTSIZ1::RXDPID_STUPCNT;
    pub use super::DOEPTSIZ1::XFRSIZ;
}

/// OTG_HS device endpoint-2 transfer size register
pub mod DOEPTSIZ4 {
    pub use super::DOEPTSIZ1::PKTCNT;
    pub use super::DOEPTSIZ1::RXDPID_STUPCNT;
    pub use super::DOEPTSIZ1::XFRSIZ;
}

/// OTG_HS device endpoint-2 transfer size register
pub mod DOEPTSIZ5 {
    pub use super::DOEPTSIZ1::PKTCNT;
    pub use super::DOEPTSIZ1::RXDPID_STUPCNT;
    pub use super::DOEPTSIZ1::XFRSIZ;
}
#[repr(C)]
pub struct RegisterBlock {
    /// OTG_HS device configuration register
    pub DCFG: RWRegister<u32>,

    /// OTG_HS device control register
    pub DCTL: RWRegister<u32>,

    /// OTG_HS device status register
    pub DSTS: RORegister<u32>,

    _reserved1: [u32; 1],

    /// OTG_HS device IN endpoint common interrupt mask register
    pub DIEPMSK: RWRegister<u32>,

    /// OTG_HS device OUT endpoint common interrupt mask register
    pub DOEPMSK: RWRegister<u32>,

    /// OTG_HS device all endpoints interrupt register
    pub DAINT: RORegister<u32>,

    /// OTG_HS all endpoints interrupt mask register
    pub DAINTMSK: RWRegister<u32>,

    _reserved2: [u32; 2],

    /// OTG_HS device VBUS discharge time register
    pub DVBUSDIS: RWRegister<u32>,

    /// OTG_HS device VBUS pulsing time register
    pub DVBUSPULSE: RWRegister<u32>,

    /// OTG_HS Device threshold control register
    pub DTHRCTL: RWRegister<u32>,

    /// OTG_HS device IN endpoint FIFO empty interrupt mask register
    pub DIEPEMPMSK: RWRegister<u32>,

    /// OTG_HS device each endpoint interrupt register
    pub DEACHINT: RWRegister<u32>,

    /// OTG_HS device each endpoint interrupt register mask
    pub DEACHINTMSK: RWRegister<u32>,

    /// OTG_HS device each in endpoint-1 interrupt register
    pub DIEPEACHMSK1: RWRegister<u32>,

    _reserved3: [u32; 15],

    /// OTG_HS device each OUT endpoint-1 interrupt register
    pub DOEPEACHMSK1: RWRegister<u32>,

    _reserved4: [u32; 31],

    /// OTG device endpoint-0 control register
    pub DIEPCTL0: RWRegister<u32>,

    _reserved5: [u32; 1],

    /// OTG device endpoint-0 interrupt register
    pub DIEPINT0: RWRegister<u32>,

    _reserved6: [u32; 1],

    /// OTG_HS device IN endpoint 0 transfer size register
    pub DIEPTSIZ0: RWRegister<u32>,

    /// OTG_HS device endpoint-1 DMA address register
    pub DIEPDMA1: UnsafeRWRegister<u32>,

    /// OTG_HS device IN endpoint transmit FIFO status register
    pub DTXFSTS0: RORegister<u32>,

    _reserved7: [u32; 1],

    /// OTG device endpoint-1 control register
    pub DIEPCTL1: RWRegister<u32>,

    _reserved8: [u32; 1],

    /// OTG device endpoint-0 interrupt register
    pub DIEPINT1: RWRegister<u32>,

    _reserved9: [u32; 1],

    /// OTG_HS device endpoint transfer size register
    pub DIEPTSIZ1: RWRegister<u32>,

    /// OTG_HS device endpoint-2 DMA address register
    pub DIEPDMA2: UnsafeRWRegister<u32>,

    /// OTG_HS device IN endpoint transmit FIFO status register
    pub DTXFSTS1: RORegister<u32>,

    _reserved10: [u32; 1],

    /// OTG device endpoint-1 control register
    pub DIEPCTL2: RWRegister<u32>,

    _reserved11: [u32; 1],

    /// OTG device endpoint-0 interrupt register
    pub DIEPINT2: RWRegister<u32>,

    _reserved12: [u32; 1],

    /// OTG_HS device endpoint transfer size register
    pub DIEPTSIZ2: RWRegister<u32>,

    /// OTG_HS device endpoint-3 DMA address register
    pub DIEPDMA3: UnsafeRWRegister<u32>,

    /// OTG_HS device IN endpoint transmit FIFO status register
    pub DTXFSTS2: RORegister<u32>,

    _reserved13: [u32; 1],

    /// OTG device endpoint-1 control register
    pub DIEPCTL3: RWRegister<u32>,

    _reserved14: [u32; 1],

    /// OTG device endpoint-0 interrupt register
    pub DIEPINT3: RWRegister<u32>,

    _reserved15: [u32; 1],

    /// OTG_HS device endpoint transfer size register
    pub DIEPTSIZ3: RWRegister<u32>,

    /// OTG_HS device endpoint-4 DMA address register
    pub DIEPDMA4: UnsafeRWRegister<u32>,

    /// OTG_HS device IN endpoint transmit FIFO status register
    pub DTXFSTS3: RORegister<u32>,

    _reserved16: [u32; 1],

    /// OTG device endpoint-1 control register
    pub DIEPCTL4: RWRegister<u32>,

    _reserved17: [u32; 1],

    /// OTG device endpoint-0 interrupt register
    pub DIEPINT4: RWRegister<u32>,

    _reserved18: [u32; 1],

    /// OTG_HS device endpoint transfer size register
    pub DIEPTSIZ4: RWRegister<u32>,

    /// OTG_HS device endpoint-5 DMA address register
    pub DIEPDMA5: UnsafeRWRegister<u32>,

    /// OTG_HS device IN endpoint transmit FIFO status register
    pub DTXFSTS4: RORegister<u32>,

    _reserved19: [u32; 1],

    /// OTG device endpoint-1 control register
    pub DIEPCTL5: RWRegister<u32>,

    _reserved20: [u32; 1],

    /// OTG device endpoint-0 interrupt register
    pub DIEPINT5: RWRegister<u32>,

    _reserved21: [u32; 1],

    /// OTG_HS device endpoint transfer size register
    pub DIEPTSIZ5: RWRegister<u32>,

    _reserved22: [u32; 1],

    /// OTG_HS device IN endpoint transmit FIFO status register
    pub DTXFSTS5: RORegister<u32>,

    _reserved23: [u32; 81],

    /// OTG_HS device control OUT endpoint 0 control register
    pub DOEPCTL0: RWRegister<u32>,

    _reserved24: [u32; 1],

    /// OTG_HS device endpoint-0 interrupt register
    pub DOEPINT0: RWRegister<u32>,

    _reserved25: [u32; 1],

    /// OTG_HS device endpoint-1 transfer size register
    pub DOEPTSIZ0: RWRegister<u32>,

    _reserved26: [u32; 3],

    /// OTG device endpoint-1 control register
    pub DOEPCTL1: RWRegister<u32>,

    _reserved27: [u32; 1],

    /// OTG_HS device endpoint-0 interrupt register
    pub DOEPINT1: RWRegister<u32>,

    _reserved28: [u32; 1],

    /// OTG_HS device endpoint-2 transfer size register
    pub DOEPTSIZ1: RWRegister<u32>,

    _reserved29: [u32; 3],

    /// OTG device endpoint-1 control register
    pub DOEPCTL2: RWRegister<u32>,

    _reserved30: [u32; 1],

    /// OTG_HS device endpoint-0 interrupt register
    pub DOEPINT2: RWRegister<u32>,

    _reserved31: [u32; 1],

    /// OTG_HS device endpoint-2 transfer size register
    pub DOEPTSIZ2: RWRegister<u32>,

    _reserved32: [u32; 3],

    /// OTG device endpoint-1 control register
    pub DOEPCTL3: RWRegister<u32>,

    _reserved33: [u32; 1],

    /// OTG_HS device endpoint-0 interrupt register
    pub DOEPINT3: RWRegister<u32>,

    _reserved34: [u32; 1],

    /// OTG_HS device endpoint-2 transfer size register
    pub DOEPTSIZ3: RWRegister<u32>,

    _reserved35: [u32; 3],

    /// OTG device endpoint-1 control register
    pub DOEPCTL4: RWRegister<u32>,

    _reserved36: [u32; 1],

    /// OTG_HS device endpoint-0 interrupt register
    pub DOEPINT4: RWRegister<u32>,

    _reserved37: [u32; 1],

    /// OTG_HS device endpoint-2 transfer size register
    pub DOEPTSIZ4: RWRegister<u32>,

    _reserved38: [u32; 3],

    /// OTG device endpoint-1 control register
    pub DOEPCTL5: RWRegister<u32>,

    _reserved39: [u32; 1],

    /// OTG_HS device endpoint-0 interrupt register
    pub DOEPINT5: RWRegister<u32>,

    _reserved40: [u32; 1],

    /// OTG_HS device endpoint-2 transfer size register
    pub DOEPTSIZ5: RWRegister<u32>,
}
pub struct ResetValues {
    pub DCFG: u32,
    pub DCTL: u32,
    pub DSTS: u32,
    pub DIEPMSK: u32,
    pub DOEPMSK: u32,
    pub DAINT: u32,
    pub DAINTMSK: u32,
    pub DVBUSDIS: u32,
    pub DVBUSPULSE: u32,
    pub DTHRCTL: u32,
    pub DIEPEMPMSK: u32,
    pub DEACHINT: u32,
    pub DEACHINTMSK: u32,
    pub DIEPEACHMSK1: u32,
    pub DOEPEACHMSK1: u32,
    pub DIEPCTL0: u32,
    pub DIEPINT0: u32,
    pub DIEPTSIZ0: u32,
    pub DIEPDMA1: u32,
    pub DTXFSTS0: u32,
    pub DIEPCTL1: u32,
    pub DIEPINT1: u32,
    pub DIEPTSIZ1: u32,
    pub DIEPDMA2: u32,
    pub DTXFSTS1: u32,
    pub DIEPCTL2: u32,
    pub DIEPINT2: u32,
    pub DIEPTSIZ2: u32,
    pub DIEPDMA3: u32,
    pub DTXFSTS2: u32,
    pub DIEPCTL3: u32,
    pub DIEPINT3: u32,
    pub DIEPTSIZ3: u32,
    pub DIEPDMA4: u32,
    pub DTXFSTS3: u32,
    pub DIEPCTL4: u32,
    pub DIEPINT4: u32,
    pub DIEPTSIZ4: u32,
    pub DIEPDMA5: u32,
    pub DTXFSTS4: u32,
    pub DIEPCTL5: u32,
    pub DIEPINT5: u32,
    pub DIEPTSIZ5: u32,
    pub DTXFSTS5: u32,
    pub DOEPCTL0: u32,
    pub DOEPINT0: u32,
    pub DOEPTSIZ0: u32,
    pub DOEPCTL1: u32,
    pub DOEPINT1: u32,
    pub DOEPTSIZ1: u32,
    pub DOEPCTL2: u32,
    pub DOEPINT2: u32,
    pub DOEPTSIZ2: u32,
    pub DOEPCTL3: u32,
    pub DOEPINT3: u32,
    pub DOEPTSIZ3: u32,
    pub DOEPCTL4: u32,
    pub DOEPINT4: u32,
    pub DOEPTSIZ4: u32,
    pub DOEPCTL5: u32,
    pub DOEPINT5: u32,
    pub DOEPTSIZ5: u32,
}
#[cfg(not(feature = "nosync"))]
pub struct Instance {
    pub(crate) addr: u32,
    pub(crate) _marker: PhantomData<*const RegisterBlock>,
}
#[cfg(not(feature = "nosync"))]
impl ::core::ops::Deref for Instance {
    type Target = RegisterBlock;
    #[inline(always)]
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*(self.addr as *const _) }
    }
}
#[cfg(feature = "rtfm")]
unsafe impl Send for Instance {}
