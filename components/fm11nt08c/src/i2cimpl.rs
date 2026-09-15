#[cfg(feature = "lpc55-v0.7")]
mod lpc55_07 {
    use crate::I2CError;

    use lpc55_hal_07::drivers::i2c::Error;

    impl I2CError for Error {
        fn is_address_nack(&self) -> bool {
            matches!(self, Error::NackAddress)
        }
        fn is_data_nack(&self) -> bool {
            matches!(self, Error::NackData)
        }
    }
}
