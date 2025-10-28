#[cfg(feature = "lpc55-v0.3")]
mod lpc55_03 {
    use crate::I2CError;

    use lpc55_hal::drivers::i2c::Error;

    impl I2CError for Error {
        fn is_address_nack(&self) -> bool {
            matches!(self, Error::NackAddress)
        }
        fn is_data_nack(&self) -> bool {
            matches!(self, Error::NackData)
        }
    }
}

#[cfg(feature = "lpc55-v0.4")]
mod lpc55_04 {
    use crate::I2CError;

    use lpc55_hal_04::drivers::i2c::Error;

    impl I2CError for Error {
        fn is_address_nack(&self) -> bool {
            matches!(self, Error::NackAddress)
        }
        fn is_data_nack(&self) -> bool {
            matches!(self, Error::NackData)
        }
    }
}
