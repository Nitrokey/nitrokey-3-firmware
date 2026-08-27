#[cfg(feature = "lpc55-v0.6")]
mod lpc55_06 {
    use crate::I2CError;

    use lpc55_hal_06::drivers::i2c::Error;

    impl I2CError for Error {
        fn is_address_nack(&self) -> bool {
            matches!(self, Error::NackAddress)
        }
        fn is_data_nack(&self) -> bool {
            matches!(self, Error::NackData)
        }
    }
}
