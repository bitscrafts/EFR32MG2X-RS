//! embedded-hal v1.0 I2C trait implementations

use super::{Error, I2c0, I2c1};
use embedded_hal::i2c::{ErrorType, I2c, Operation};

// Implement embedded-hal ErrorType for I2c0
impl ErrorType for I2c0 {
    type Error = Error;
}

// Implement embedded-hal I2C trait for I2c0
impl I2c for I2c0 {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                Operation::Write(bytes) => {
                    self.write(address, bytes)?;
                }
                Operation::Read(buffer) => {
                    self.read(address, buffer)?;
                }
            }
        }
        Ok(())
    }
}

// Implement embedded-hal ErrorType for I2c1
impl ErrorType for I2c1 {
    type Error = Error;
}

// Implement embedded-hal I2C trait for I2c1
impl I2c for I2c1 {
    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                Operation::Write(bytes) => {
                    self.write(address, bytes)?;
                }
                Operation::Read(buffer) => {
                    self.read(address, buffer)?;
                }
            }
        }
        Ok(())
    }
}
