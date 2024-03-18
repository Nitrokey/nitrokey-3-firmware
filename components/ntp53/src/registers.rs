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
    ($name: ident) => {
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

        impl Default for $name {
            fn default() -> Self {
                Self(0)
            }
        }
    };
}

macro_rules! custom_bits {
    ($name: ident) => {
        impl bitfield::Bit for $name {
            fn bit(&self, bit: usize) -> bool {
                self.0.bit(bit)
            }
        }

        impl bitfield::BitMut for $name {
            fn set_bit(&mut self, bit: usize, value: bool) {
                self.0.set_bit(bit, value);
            }
        }

        impl bitfield::BitRange<U8Wrapper> for $name {
            fn bit_range(&self, msb: usize, lsb: usize) -> U8Wrapper {
                U8Wrapper(self.0.bit_range(msb, lsb))
            }
        }

        impl bitfield::BitRangeMut<U8Wrapper> for $name {
            fn set_bit_range(&mut self, msb: usize, lsb: usize, U8Wrapper(value): U8Wrapper) {
                self.0.set_bit_range(msb, lsb, value);
            }
        }
    };
}

bitfield! {
    pub struct StatusRegister0(u8);
    impl Debug;

    // EEPROM is busy
    pub eeprom_wr_busy, _: 7;
    // 1: EEPROM write error happened
    pub mask EEPROM_WR_ERROR_MASK(u8), eeprom_wr_error, set_eeprom_wr_error: 6;
    // 1: data is ready, used in pass-through mode
    pub sram_data_ready, _: 5;
    // 1: data has been written to SYNCH_BLOCK
    pub mask SYNCH_BLOCK_WRITE_MASK(u8), synch_block_write, set_synch_block_write: 4;
    // 1: data has been read from SYNCH_BLOCK
    pub mask SYNCH_BLOCK_READ_MASK(u8), synch_block_read, set_synch_block_read: 3;
    // 1: I2C to NFC passthrough direction
    // 0: NFC to I2C passthrough direction
    pub pt_transfer_dir, _: 2;
    pub vcc_supply_ok, _: 1;
    pub nfc_field_ok, _: 0;
}

bitfield! {
    pub struct StatusRegister1(u8);
    impl Debug;

    pub vcc_boot_ok, _: 7;
    pub nfc_boot_ok, _: 6;
    pub rfu1, _: 5;
    /// 1: GPIO_1 input is HIGH
    pub gpio1_in_status, _: 4;
    /// 1: GPIO_0 input is HIGH
    pub gpio0_in_status, _: 3;
    pub rfu2, _: 2;
    /// 1: arbitrer locked to I2C
    pub mask I2C_IF_LOCKED_MASK(u8), i2c_if_locked, set_i2c_if_locked: 1;
    /// 1: arbitrer locked to NFC
    pub nfc_if_locked, _: 0;
}

u8newtype!(StatusRegister0);
u8newtype!(StatusRegister1);

#[derive(Debug)]
pub struct StatusRegister {
    pub status0: StatusRegister0,
    pub status1: StatusRegister1,
}

enum_u8!(
    #[derive(Debug)]
    pub enum UseCaseConf {
        I2cSlave = 0b00,
        I2cMaster = 0b01,
        GpioPwm = 0b10,
        /// All host interface functionality disabled and pads are in 3-state mode
        Disabled3StateMode = 0b11,
    }
);

enum_u8!(
    #[derive(Debug)]
    pub enum ArbiterMode {
        Normal = 0b00,
        SramMirror = 0b01,
        SramPassThrough = 0b10,
        SramPhdc = 0b11,
    }
);

bitfield! {
    pub struct ConfigRegister0(u8);
    impl Debug;

    /// SRAM copy on POR is enabled
    pub sram_copy_en, _: 7;
    rfu1, _: 6;
    /// NFC is disabled
    pub mask DISABLE_NFC(u8), disable_nfc, set_disable_nfc: 5;
    rfu2, _: 4;
    rfu3, _: 3;
    rfu4, _: 2;
    rfu5, _: 1;
    /// true: IC enters standby mode after boot if there is no RF present automatically
    pub mask AUTO_STANDBY_MODE_EN(u8), auto_standby_mode_en, set_auto_standby_mode_en: 0;

}

bitfield! {
    pub struct ConfigRegister1(u8);
    no default BitRange;
    impl Debug;

    rfu1, _: 7;
    rfu2, _: 6;
    pub U8Wrapper, from into UseCaseConf, use_case_conf, _: 5, 4;
    pub U8Wrapper, mask ARBITER_MODE(u8), from into ArbiterMode, arbiter_mode, set_arbiter_mode: 3, 2;
    pub sram_enabled, _: 1;
    pub mask PT_TRANSFER_DIR(u8), pt_transfer_dir, set_pt_transfer_dir: 0;
}

custom_bits!(ConfigRegister1);

enum_u8!(
    #[derive(Debug)]
    pub enum GpioInConf {
        ReceiverDisabled = 0b00,
        PlainInputWithWeakPullUp = 0b01,
        PlainInput = 0b10,
        PlainInputWithWeakPullDown = 0b11,
    }
);

bitfield! {
    pub struct ConfigRegister2(u8);
    no default BitRange;
    impl Debug;

    pub U8Wrapper, from into GpioInConf, gpio1_in, _: 7, 6;
    pub U8Wrapper, from into GpioInConf, gpio0_in, _: 5, 4;
    pub extended_commands_supported, _: 3;
    pub lock_block_command_supported, _: 2;
    /// false: low-speed
    /// true: high-speed
    pub gpio1_slew_rate, _: 1;
    /// false: low-speed
    /// true: high-speed
    pub gpio0_slew_rate, _: 0;

}

custom_bits!(ConfigRegister2);

u8newtype!(ConfigRegister0);
u8newtype!(ConfigRegister1);
u8newtype!(ConfigRegister2);

#[non_exhaustive]
#[derive(Debug)]
pub struct ConfigRegister {
    pub config0: ConfigRegister0,
    pub config1: ConfigRegister1,
    pub config2: ConfigRegister2,
}

bitfield! {
    pub struct PwmGpioRegister0(u8);
    impl Debug;

    /// true: ouput is HIGH
    pub sda_gpio1_out_status, _: 7;
    /// true: ouput is HIGH
    pub scl_gpio0_out_status, _: 6;
    /// true: input is HIGH
    pub sda_gpio1_in_status, _: 5;
    /// true: input is HIGH
    pub scl_gpio0_in_status, _: 4;
    /// false: gpio1 is output
    /// true: gpio1 is input
    pub sda_gpio1, _: 3;
    /// false: gpio0 is output
    /// true: gpio0 is input
    pub scl_gpio1, _: 2;
    /// false: gpio1 is gpio
    /// true: gpio1 is pwm
    pub sda_gpio1_pwm1, _: 1;
    /// false: gpio0 is gpio
    /// true: gpio0 is pwm
    pub scl_gpio1_pwm0, _: 0;
}

enum_u8!(
    #[derive(Debug)]
    pub enum Resolution {
        /// 6 bit resolution
        Bit6 = 0b00,
        /// 8 bit resolution
        Bit8 = 0b01,
        /// 10 bit resolution
        Bit10 = 0b10,
        /// 12 bit resolution
        Bit12 = 0b11,
    }
);

/// Get the PWM frequency for a given configuration in Hz
pub fn pwm_frequency(resolution: Resolution, pre_scale: PreScale) -> u32 {
    #[rustfmt::skip]
    const TABLE: &[&[u32]] = &[
        //                  pre-scalar
        // Resolution    | One    | Two    | Three | Four  |
        /* 6 bit     */ &[ 26_400 , 13_200 , 6_600 , 3_300 ],
        /* 8 bit     */ &[ 6_600  , 3_300  , 1_700 , 825   ],
        /* 10 bit    */ &[ 1_700  , 825    , 413   , 206   ],
        /* 12 bit    */ &[ 413    , 206    , 103   , 52    ],
    ];

    TABLE[resolution as usize][pre_scale as usize]
}

enum_u8!(
    #[derive(Debug)]
    pub enum PreScale {
        One = 0b00,
        Two = 0b01,
        Three = 0b10,
        Four = 0b11,
    }
);

bitfield! {
    pub struct PwmGpioRegister1(u8);
    no default BitRange;
    impl Debug;

    pub U8Wrapper, from into PreScale, pwm1_prescale, _: 7,6;
    pub U8Wrapper, from into PreScale, pwm0_prescale, _: 5,4;
    pub U8Wrapper, from into Resolution, pwm1_resolution, _: 3,2;
    pub U8Wrapper, from into Resolution, pwm0_resolution, _: 1,0;
}

custom_bits!(PwmGpioRegister1);

u8newtype!(PwmGpioRegister0);
u8newtype!(PwmGpioRegister1);

#[non_exhaustive]
#[derive(Debug)]
pub struct PwmGpioRegister {
    pub pwm_gpio_0: PwmGpioRegister0,
    pub pwm_gpio_1: PwmGpioRegister1,
}

pub struct PwmXOnRegister {
    /// Actually a u12
    pub on_duration: u16,
    /// Actually a u12
    pub off_duration: u16,
}

impl PwmXOnRegister {
    pub fn from_block([on_lsb, on_msb, off_lsb, off_msb]: [u8; 4]) -> Self {
        const MSB_MASK: u8 = 0b00001111;
        const MSB_OFFSET: u8 = 8;
        let on_duration = on_lsb as u16 + (((on_msb & MSB_MASK) as u16) << MSB_OFFSET);
        let off_duration = off_lsb as u16 + (((off_msb & MSB_MASK) as u16) << MSB_OFFSET);
        Self {
            on_duration,
            off_duration,
        }
    }
}

bitfield! {
    pub struct WdtEnableRegister(u8);
    impl Debug;

    /// true: watchdog timer is enabled
    pub mask WDT_ENABLE_MASK(u8), wdt_enable, set_wdt_enable: 0;
}

/// Watch Dog timer register
#[derive(Debug)]
pub struct WdtRegister {
    /// up to 618ms
    pub duration: u16,
    pub enable: WdtEnableRegister,
}
u8newtype!(WdtEnableRegister);

enum_u8!(
    #[derive(Debug)]
    pub enum EhVoutISel {
        /// 0.4 mA (Default)
        I04 = 0b000,
        /// 0.6 mA
        I06 = 0b001,
        /// 1.4 mA
        I14 = 0b010,
        /// 2.7 mA
        I27 = 0b011,
        /// 4.0 mA
        I40 = 0b100,
        /// 6.5 mA
        I65 = 0b101,
        /// 9.0 mA
        I90 = 0b110,
        /// 12.5 mA
        I125 = 0b111,
    }
);

enum_u8!(
    #[derive(Debug)]
    pub enum EhVoutVSel {
        /// 1.8 V (Default)
        V18 = 0b00,
        /// 2.4 V
        V24 = 0b01,
        /// 3.0 V
        V30 = 0b10,
    }
);

bitfield! {
    pub struct EhConfigRegister(u8);
    no default BitRange;
    impl Debug;

    pub U8Wrapper, from into EhVoutISel, eh_vout_i_sel, _: 6, 4;
    /// false: Only if sufficient power can be harvested, VOUT will be enabled (default)
    /// true: Power level will not be checked, VOUT will be enabled immediately after startup
    pub disable_power_check, _: 3;
    pub U8Wrapper, from into EhVoutVSel, eh_vout_v_sel, _: 2, 1;
    /// true: Energy harvesting enabled
    pub eh_enable, _: 0;
}
custom_bits!(EhConfigRegister);
u8newtype!(EhConfigRegister);

enum_u8!(
    #[derive(Debug)]
    pub enum EdConfig {
        /// Ed is always off.
        Disabled = 0b0000,
        /// Ed is **ON** if and only if NFC an field is present.
        NfcFieldDetect = 0b0001,
        /// Ed is **ON** during PWM off period.
        Pwm = 0b0010,
        /// Ed is **ON**: last byte of SRAM has been read via NFC; host can access SRAM again.
        /// Ed is **OFF**: Last byte written by I2C or NFC is off or Vcc is off.
        I2cToNfcPassThrough = 0b0011,
        /// Ed is **ON**: last byte of SRAM has been written via NFC; host can read SRAM.
        /// Ed is **OFF**: Last byte read by I2C or NFC is off or Vcc is off.
        NfcToI2cPassThrough = 0b0100,
        /// Ed is **ON**: Arbiter locked to NFC
        /// Ed is **OFF**: Lock to NFC released
        ArbiterLock = 0b0101,
        /// Ed is **ON**: Length byte (block 1, byte 1) is not 0
        /// Ed is **OFF**: Length byte (block 1, byte 1) is 0
        NdefMessageTlvLen = 0b0110,
        /// Ed is **ON**: IC is in stand-by mode
        /// Ed is **OFF**: IC is not in stand-by mode
        StandByMode = 0b0111,
        /// Ed is **ON**: start of programming cycle during WRITE command
        /// Ed is **OFF**: start of response to WRITE command or NFC off
        WriteCommandIndication = 0b1000,
        /// Ed is **ON**: start of read cycle during READ command
        /// Ed is **OFF**: end of read access or NFC off
        ReadCommandIndication = 0b1001,
        /// Ed is **ON**: start of any command
        /// Ed is **OFF**: end of response or NFC off
        StartOfCommandIndication = 0b1010,
        /// Ed is **ON**: data read from SYNCH_BLOCK
        /// Ed is **OFF**:i Needs to be cleared by setting EdResetReg to 0b1
        ReadFromSynchBlock = 0b1011,
        /// Ed is **ON**: data written from SYNCH_BLOCK
        /// Ed is **OFF**:i Needs to be cleared by setting EdResetReg to 0b1
        ReadToSynchBlock = 0b1100,
        /// Ed is **ON**: 0b1101 written to ED_CONFIG
        /// Ed is **OFF**:i Needs to be cleared by setting EdResetReg to 0b1
        SoftwareInterrup = 0b1101,
        Rfu1 = 0b1110,
        Rfu2 = 0b1111,
    }
);

bitfield! {
    pub struct EdConfigRegister(u8);
    no default BitRange;
    impl Debug;

    U8Wrapper, from into EdConfig, ed_config, _: 3, 0;
}

custom_bits!(EdConfigRegister);
u8newtype!(EdConfigRegister);

#[derive(Debug)]
pub struct I2cSlaveConfiguration {
    /// Actually a u7
    pub addr: u8,
    pub config: I2cSlaveConfigReg,
}

impl I2cSlaveConfiguration {
    pub fn from_data([addr, config]: [u8; 2]) -> Self {
        Self {
            addr: addr & 0b0111111,
            config: config.into(),
        }
    }
}

bitfield!(
    pub struct I2cSlaveConfigReg(u8);
    impl Debug;

    /// Watch Dog expired
    pub i2c_wdt_expired, _: 4;
    pub mask I2C_SOFT_RESET(u8), i2c_soft_reset, set_i2c_soft_read: 3;
    pub mask I2C_S_REPEATED_START(u8), i2c_s_repeated_start, set_i2c_s_repeated_start: 2;
    pub disable_i2c, _: 0;
);

u8newtype!(I2cSlaveConfigReg);

bitfield! {
    pub struct EdIntrClear(u8);
    impl Debug;

    pub mask ED_INTR_CLEAR(u8), ed_intr_clear, set_ed_intr_clear: 0;
}

#[derive(Clone, Copy)]
#[repr(u16)]
pub enum Register {
    Status = 0x10A0,
    Config = 0x10A1,
    SyncDataBlock = 0x10A2,
    PwmGpioConfig = 0x10A3,
    Pwm0OnOff = 0x10A4,
    Pwm1OnOff = 0x10A5,
    WdtConfig = 0x10A6,
    EhConfig = 0x10A7,
    EdConfig = 0x10A8,
    I2cSlaveConfig = 0x10A9,
    ResetGen = 0x10AA,
    EdIntrClear = 0x10AB,
    I2cMConfig = 0x10AC,
    I2cMStatus = 0x10AD,
}
