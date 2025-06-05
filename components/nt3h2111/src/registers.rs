use bitfield::bitfield;

#[derive(Debug)]
pub struct TryFromU8Error;

// For bitfield compatibility
struct U8Wrapper(u8);

macro_rules! enum_u8 {
    (
        $(#[$outer:meta])*
        $vis:vis enum $name:ident {

            $($(#[doc = $doc:literal])* $var:ident = $num:expr),+
            $(,)*
        }
    ) => {
        $(#[$outer])*
        #[repr(u8)]
        $vis enum $name {
            $(
                $(#[doc = $doc])*
                $var = $num,
            )*
        }

        impl TryFrom<u8> for $name {
            type Error = TryFromU8Error;
            fn try_from(val: u8) -> ::core::result::Result<Self, TryFromU8Error> {
                match val {
                    $(
                        $num => Ok($name::$var),
                    )*
                    _ => Err(TryFromU8Error)
                }
            }
        }

        impl From<$name> for u8 {
            fn from(val: $name) -> u8 {
                match val {
                    $(
                        $name::$var => $num,
                    )*
                }
            }
        }

        impl From<$name> for U8Wrapper {
            fn from(val: $name) -> U8Wrapper {
                match val {
                    $(
                        $name::$var => U8Wrapper($num),
                    )*
                }
            }
        }

        impl From<U8Wrapper> for $name {
            fn from(val: U8Wrapper) -> $name {
                // We know this can't fail because the from is only used with the proper bitmasking in the bitfield macro
                val.0.try_into().unwrap()
            }
        }

    }
}

macro_rules! u8newtype {
    ($name:ident) => {
        impl From<u8> for $name {
            fn from(value: u8) -> Self {
                Self(value)
            }
        }
        impl From<$name> for u8 {
            fn from(value: $name) -> u8 {
                value.0
            }
        }
    };
}

macro_rules! session_register {
    ($name:ident, $address:expr) => {
        impl SessionRegister for $name {
            const ADDRESS: u8 = $address;
        }
    };
}

macro_rules! configuration_register {
    ($name:ident, $address:expr) => {
        impl ConfigurationRegister for $name {
            const ADDRESS: u8 = $address;
        }
    };
}

impl bitfield::BitRange<U8Wrapper> for u8 {
    fn bit_range(&self, msb: usize, lsb: usize) -> U8Wrapper {
        U8Wrapper(self.bit_range(msb, lsb))
    }
}

impl bitfield::BitRangeMut<U8Wrapper> for u8 {
    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: U8Wrapper) {
        self.set_bit_range(msb, lsb, value.0);
    }
}

// macro_rules! custom_bits {
//     ($name: ident) => {
//         impl bitfield::Bit for $name {
//             fn bit(&self, bit: usize) -> bool {
//                 self.0.bit(bit)
//             }
//         }

//         impl bitfield::BitMut for $name {
//             fn set_bit(&mut self, bit: usize, value: bool) {
//                 self.0.set_bit(bit, value);
//             }
//         }

//         impl bitfield::BitRange<U8Wrapper> for $name {
//             fn bit_range(&self, msb: usize, lsb: usize) -> U8Wrapper {
//                 U8Wrapper(self.0.bit_range(msb, lsb))
//             }
//         }

//         impl bitfield::BitRangeMut<U8Wrapper> for $name {
//             fn set_bit_range(&mut self, msb: usize, lsb: usize, U8Wrapper(value): U8Wrapper) {
//                 self.0.set_bit_range(msb, lsb, value);
//             }
//         }
//     };
// }

pub trait SessionRegister: From<u8> + Into<u8> {
    const ADDRESS: u8;
}

enum_u8!(
    /// defines the event upon which the signal output on the FD pin is released
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum FdOff {
        /// if the field is switched off
        Off = 0b00,
        /// if the field is switched off or the tag is set to the HALT state
        On = 0b01,
        /// if the field is switched off or the last page of the NDEF message has been read (defined in LAST_ NDEF_BLOCK)
        LastPage = 0b10,
        ///
        /// (if FD_ON = 11b) if the field is switched off or if last data is read by I²C
        /// (in pass-through mode NFC ---> I²C) or
        /// last data is written by I²C (in pass-through mode I²C---> NFC)
        LastData = 0b11,
    }
);

enum_u8!(
    /// defines the event upon which the signal output on the FD pin is pulled low
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum FdOn {
        /// if the field is switched on
        FieldOn = 0b00,
        /// by first valid start of communication (SoC)
        StartOfCommunication = 0b01,
        /// by selection of the ta
        TagSelection = 0b10,
        /// *in pass-through mode NFC-->I²C*: if the data is ready to be read from the I²C interface
        /// *in pass-through mode I-->I²C*: if the data is ready to be read from the I²C interface
        ReadyData = 0b11,
    }
);

bitfield! {
    pub struct NcReg(u8);
    impl Debug;
    bool;
    pub mask NFCS_I2C_RS_ON_OFF(u8), nfcs_i2c_rs_on_off,set_nfcs_i2c_rs_on_off:7;
    pub mask PTHRU_ON_OFF(u8), pthru_on_off, set_pthru_on_off: 6;
    pub U8Wrapper, mask FD_OFF(u8), from into FdOff, fd_off,set_fd_off: 5,4;
    pub U8Wrapper, mask FD_ON(u8), from into FdOn, fd_on,set_fd_on: 3,2;
    pub mask SRAM_MIROR_ON_OF(u8), sram_miror_on_of, set_sram_miror_on_of: 1;
    pub mask TRANSFER_DIR(u8), transfer_dir, set_transfer_dir: 0;
}

u8newtype!(NcReg);
session_register!(NcReg, 0);

#[derive(Debug)]
pub struct LastNdefBlock(u8);

u8newtype!(LastNdefBlock);
session_register!(LastNdefBlock, 1);

#[derive(Debug)]
pub struct SramMirrorBlock(u8);

u8newtype!(SramMirrorBlock);
session_register!(SramMirrorBlock, 2);

#[derive(Debug)]
pub struct WdtLs(pub u8);

u8newtype!(WdtLs);
session_register!(WdtLs, 3);

#[derive(Debug)]
pub struct WdtMs(pub u8);

u8newtype!(WdtMs);
session_register!(WdtMs, 4);

bitfield! {
    pub struct I2cClockStr(u8);
    impl Debug;

    bool;
    pub mask I2C_CLOCK_STR(u8), i2c_clock_str, set_i2c_clock_str: 0;
}

u8newtype!(I2cClockStr);
session_register!(I2cClockStr, 5);

bitfield! {
    pub struct NsReg(u8);
    impl Debug;

    bool;
    pub mask NDEF_DATA_READ(u8), ndef_data_read, setndef_data_read: 7;
    pub mask I2C_LOCKED(u8), i2c_locked, set_i2c_locked: 6;
    pub mask RF_LOCKED(u8), rf_locked, _: 5;
    pub mask SRAM_I2C_READY(u8), sram_i2c_ready, _: 4;
    pub mask SRAM_RF_READY(u8), sram_rf_ready, _: 3;
    pub mask EEPROM_WR_ERR(u8), eeprom_wr_err, set_eeprom_wr_err: 2;
    pub mask EEPROM_WR_BUSY(u8), eeprom_wr_busy, _: 1;
    pub mask RF_FIELD_PRESENT(u8), rf_field_present, _: 0;
}

u8newtype!(NsReg);
session_register!(NsReg, 6);

pub trait ConfigurationRegister: From<u8> + Into<u8> {
    const ADDRESS: u8;
}

configuration_register!(NsReg, 0);
configuration_register!(LastNdefBlock, 1);
configuration_register!(SramMirrorBlock, 2);
configuration_register!(WdtLs, 3);
configuration_register!(WdtMs, 4);
configuration_register!(I2cClockStr, 5);

bitfield! {
    pub struct RegLock(u8);
    impl Debug;

    bool;
    pub mask REG_LOC_I2C(u8), reg_lock_i2c, set_reg_lock_i2c: 1;
    pub mask REG_LOC_NFC(u8), reg_lock_nfc, set_reg_lock_nfc: 0;
}

u8newtype!(RegLock);
configuration_register!(RegLock, 6);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        let mut nc_reg = NcReg(0);
        assert!(!nc_reg.nfcs_i2c_rs_on_off());
        assert!(!nc_reg.pthru_on_off());
        assert!(!nc_reg.sram_miror_on_of());
        assert_eq!(nc_reg.fd_off(), FdOff::Off);
        assert_eq!(nc_reg.fd_on(), FdOn::FieldOn);
        nc_reg.set_fd_off(FdOff::LastData);
        assert_eq!(nc_reg.fd_off(), FdOff::LastData);
        assert_eq!(nc_reg.0, 0b11 << 4);
    }
}
