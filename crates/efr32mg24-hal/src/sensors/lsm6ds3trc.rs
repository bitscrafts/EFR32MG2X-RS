//! LSM6DS3TR-C 6-axis IMU driver (accelerometer + gyroscope).
//!
//! ## Power first
//!
//! On the XIAO MG24 **Sense** the IMU is power-gated: it does not answer on the
//! bus at all until PD5 is driven high. That is a board fact, so the pin lives in
//! the BSP layer, not here — this driver assumes the part is already powered and
//! answers at [`ADDR_DEFAULT`].
//!
//! Consequence worth stating: an I2C scan of a powered-down Sense board shows
//! 0x3C, 0x51 and 0x76 and **no 0x6A**. That is not a fault in the bus.
//!
//! ## Registers used
//!
//! - `0x0F` WHO_AM_I (must read [`WHO_AM_I_LSM6DS3`]),
//! - `0x10` CTRL1_XL — accelerometer ODR and full scale,
//! - `0x11` CTRL2_G — gyroscope ODR and full scale,
//! - `0x12` CTRL3_C — block data update, register auto-increment,
//! - `0x22..0x2D` the six output words (gyro X/Y/Z then accel X/Y/Z).
//!
//! Sensitivities are the datasheet constants for the ranges configured below:
//! 0.061 mg/LSB at ±2 g, 8.75 mdps/LSB at ±250 dps.

use crate::bus::{BusError, I2cBus};

/// Default 7-bit address (SDO/SA0 low).
pub const ADDR_DEFAULT: u8 = 0x6A;

const REG_WHO_AM_I: u8 = 0x0F;
const REG_CTRL1_XL: u8 = 0x10;
const REG_CTRL2_G: u8 = 0x11;
const REG_CTRL3_C: u8 = 0x12;
const REG_OUT_G: u8 = 0x22;
const REG_OUT_XL: u8 = 0x28;

/// What WHO_AM_I must read on an LSM6DS3 / LSM6DS3TR-C.
pub const WHO_AM_I_LSM6DS3: u8 = 0x6A;

/// Accelerometer range in thousandths of a g.
const ACCEL_MG_PER_LSB: f32 = 0.061;
/// Gyroscope range in thousandths of a degree per second.
const GYRO_MDPS_PER_LSB: f32 = 8.75;

/// One 6-axis sample.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Motion {
    /// Acceleration on X/Y/Z in thousandths of a g.
    pub accel_mg: [i32; 3],
    /// Angular rate on X/Y/Z in thousandths of a degree per second.
    pub gyro_mdps: [i32; 3],
}

/// Errors from the IMU.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ImuError {
    /// The transport failed.
    Bus(BusError),
    /// WHO_AM_I was not 0x6A; carries what was read. A powered-down part reads
    /// as a bus NACK instead, so this usually means "wrong chip".
    NotAnLsm6ds3(u8),
}

impl From<BusError> for ImuError {
    fn from(e: BusError) -> Self {
        ImuError::Bus(e)
    }
}

/// A configured LSM6DS3TR-C.
pub struct Lsm6ds3trc<B: I2cBus> {
    bus: B,
    addr: u8,
}

impl<B: I2cBus> Lsm6ds3trc<B> {
    /// Verify the identity and configure ±2 g / ±250 dps at 104 Hz.
    pub fn new(bus: B) -> Result<Self, ImuError> {
        Self::with_address(bus, ADDR_DEFAULT)
    }

    /// Same, for a part strapped to the alternate address.
    pub fn with_address(mut bus: B, addr: u8) -> Result<Self, ImuError> {
        let id = bus.read_reg(addr, REG_WHO_AM_I)?;
        if id != WHO_AM_I_LSM6DS3 {
            return Err(ImuError::NotAnLsm6ds3(id));
        }
        // Block data update on (so a read cannot straddle two samples) and
        // register auto-increment on (so one 12-byte read gets all six axes).
        bus.write_reg(addr, REG_CTRL3_C, 0b0100_0100)?;
        // ODR 104 Hz (0100), ±2 g (00).
        bus.write_reg(addr, REG_CTRL1_XL, 0b0100_0000)?;
        // ODR 104 Hz (0100), ±250 dps (00).
        bus.write_reg(addr, REG_CTRL2_G, 0b0100_0000)?;
        Ok(Self { bus, addr })
    }

    /// Read one sample. Twelve bytes in one transaction: gyro X/Y/Z then accel
    /// X/Y/Z, little-endian, as the auto-incrementing pointer delivers them.
    pub fn read(&mut self) -> Result<Motion, ImuError> {
        // One 12-byte burst from the gyro block: 0x22..0x27 is the gyroscope and
        // 0x28..0x2D the accelerometer, so a single auto-incrementing read gets
        // both. Measured, this matters: with the accelerometer read as a
        // *separate* transaction the driver reported 9/0/1 mg on a board that the
        // PAC example — one burst — measured at 985/81/139 mg (i.e. one g). The
        // two-transaction version was wrong; a single burst reproduces the
        // working path exactly.
        let mut raw = [0u8; 12];
        self.bus.read_regs(self.addr, REG_OUT_G, &mut raw)?;
        let word = |lo: u8, hi: u8| i16::from_le_bytes([lo, hi]) as f32;
        let gyro_mdps = [
            (word(raw[0], raw[1]) * GYRO_MDPS_PER_LSB) as i32,
            (word(raw[2], raw[3]) * GYRO_MDPS_PER_LSB) as i32,
            (word(raw[4], raw[5]) * GYRO_MDPS_PER_LSB) as i32,
        ];
        let accel_mg = [
            (word(raw[6], raw[7]) * ACCEL_MG_PER_LSB) as i32,
            (word(raw[8], raw[9]) * ACCEL_MG_PER_LSB) as i32,
            (word(raw[10], raw[11]) * ACCEL_MG_PER_LSB) as i32,
        ];
        Ok(Motion {
            accel_mg,
            gyro_mdps,
        })
    }

    /// Give the bus back.
    pub fn free(self) -> B {
        self.bus
    }
}
