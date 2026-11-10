//! BME680 / BME688 driver: temperature, pressure and humidity.
//!
//! ## The calibration layout, which is not what the register order suggests
//!
//! Bosch stores the trim in three blocks — 0x8A (23 bytes), 0xE1 (14 bytes) and
//! 0x00 (5 bytes) — and the fields are **not** in register order. The block at
//! 0x8A begins with `par_t2` and `par_t3`; `par_t1` lives at coefficient index
//! 31/32, i.e. inside the second block. Reading 0x8A as `par_t1` gave -1.01 C on
//! a 32 C bench, and that mistake is why this comment exists.
//!
//! `par_h1` and `par_h2` also share a byte: index 24 holds `par_h2`'s low
//! nibble in its high half and `par_h1`'s lsb in its low half. Treating it as a
//! plain lsb byte made humidity saturate at 100 %.
//!
//! ## Mode changes
//!
//! Bosch's `set_op_mode` contract: write **sleep** first, then the wanted mode.
//! Skipping that makes the sensor ignore the write silently — the bus ACKs, and
//! `ctrl_meas` still reads back as sleep.
//!
//! ## Provenance
//!
//! The compensation formulas and the coefficient indices were cross-checked
//! against the working implementation in `/projects/t-dongle-c5/crates/bme688`
//! (same sensor family, same silicon, verified on hardware there) and then
//! re-verified here against a room at ~32 C and ~1015 hPa.

use crate::bus::{BusError, I2cBus};

/// Default address when the sensor's SDO pin is pulled low.
pub const ADDR_PRIMARY: u8 = 0x76;
/// Alternate address when SDO is pulled high.
pub const ADDR_SECONDARY: u8 = 0x77;

const REG_STATUS: u8 = 0x1D; // FIELD0: status + raw data block (17 bytes)
const REG_CTRL_HUM: u8 = 0x72;
const REG_CTRL_MEAS: u8 = 0x74;
const REG_CHIP_ID: u8 = 0xD0;
const REG_VARIANT_ID: u8 = 0xF0;
const REG_COEFF1: u8 = 0x8A; // 23 bytes
const REG_COEFF2: u8 = 0xE1; // 14 bytes
const REG_COEFF3: u8 = 0x00; // 5 bytes
const NEW_DATA: u8 = 0x80;
const CHIP_ID_BME68X: u8 = 0x61;

/// One reading. Temperature and humidity in hundredths, pressure in hPa.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// One compensated measurement.
pub struct Reading {
    /// Temperature in hundredths of a degree Celsius.
    pub temperature_centi_c: i32,
    /// Pressure in whole hectopascals.
    pub pressure_hpa: i32,
    /// Relative humidity in hundredths of a percent.
    pub humidity_centi_pct: i32,
    /// Variant byte from register 0xF0: 0x00 = BME680, 0x01 = BME688.
    /// Variant byte (0x00 = BME680, 0x01 = BME688).
    pub variant: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// What can go wrong while talking to the sensor.
pub enum SensorError {
    /// The transport failed.
    Bus(BusError),
    /// Chip id was not 0x61; carries what was read instead.
    NotABme68x(u8),
    /// The conversion never signalled new data.
    NoData,
}

impl From<BusError> for SensorError {
    fn from(e: BusError) -> Self {
        SensorError::Bus(e)
    }
}

struct Calib {
    par_t1: u16,
    par_t2: i16,
    par_t3: i8,
    par_p1: u16,
    par_p2: i16,
    par_p3: i8,
    par_p4: i16,
    par_p5: i16,
    par_p6: i8,
    par_p7: i8,
    par_p8: i16,
    par_p9: i16,
    par_p10: i8,
    par_h1: u16,
    par_h2: u16,
    par_h3: i8,
    par_h4: i8,
    par_h5: i8,
    par_h6: u8,
    par_h7: i8,
}

impl Calib {
    fn from_bytes(c: &[u8; 42]) -> Self {
        let be = |msb: u8, lsb: u8| ((msb as u16) << 8) | lsb as u16;
        Self {
            par_t2: be(c[1], c[0]) as i16,
            par_t3: c[2] as i8,
            par_p1: be(c[5], c[4]),
            par_p2: be(c[7], c[6]) as i16,
            par_p3: c[8] as i8,
            par_p4: be(c[11], c[10]) as i16,
            par_p5: be(c[13], c[12]) as i16,
            par_p6: c[15] as i8,
            par_p7: c[14] as i8,
            par_p8: be(c[19], c[18]) as i16,
            par_p9: be(c[21], c[20]) as i16,
            par_p10: c[22] as i8,
            // H1/H2 share byte 24: H2 takes its high nibble, H1 its low nibble.
            par_h2: ((c[23] as u16) << 4) | ((c[24] as u16) >> 4),
            par_h1: ((c[25] as u16) << 4) | ((c[24] as u16) & 0x0F),
            par_h3: c[26] as i8,
            par_h4: c[27] as i8,
            par_h5: c[28] as i8,
            par_h6: c[29],
            par_h7: c[30] as i8,
            // par_t1 is index 31/32, inside the block read from 0xE1.
            par_t1: be(c[32], c[31]),
        }
    }
}

/// A detected BME68x with its calibration already loaded.
pub struct Bme68x<B: I2cBus> {
    bus: B,
    addr: u8,
    calib: Calib,
    /// Variant byte from register 0xF0: 0x00 = BME680, 0x01 = BME688.
    /// Variant byte (0x00 = BME680, 0x01 = BME688).
    pub variant: u8,
}

impl<B: I2cBus> Bme68x<B> {
    /// Detect the sensor at 0x76 or 0x77, check the id and load the trim.
    pub fn new(bus: B) -> Result<Self, SensorError> {
        let mut bus = bus;
        let addr = if bus.probe(ADDR_PRIMARY) {
            ADDR_PRIMARY
        } else if bus.probe(ADDR_SECONDARY) {
            ADDR_SECONDARY
        } else {
            return Err(SensorError::Bus(BusError::NoAck));
        };

        let id = bus.read_reg(addr, REG_CHIP_ID)?;
        if id != CHIP_ID_BME68X {
            return Err(SensorError::NotABme68x(id));
        }
        let variant = bus.read_reg(addr, REG_VARIANT_ID).unwrap_or(0xFF);

        let mut coeff = [0u8; 42];
        bus.read_regs(addr, REG_COEFF1, &mut coeff[..23])?;
        bus.read_regs(addr, REG_COEFF2, &mut coeff[23..37])?;
        bus.read_regs(addr, REG_COEFF3, &mut coeff[37..42])?;

        Ok(Self {
            bus,
            addr,
            calib: Calib::from_bytes(&coeff),
            variant,
        })
    }

    /// Trigger one forced measurement and wait for it by polling `new_data`.
    ///
    /// `pump` is called between polls so a caller on a bare-metal target can
    /// drive its own time base; the driver does not own a timer.
    pub fn read_with<F: FnMut()>(&mut self, mut pump: F) -> Result<Reading, SensorError> {
        // Sleep first: a mode change from any other mode is ignored.
        let ctrl = self.bus.read_reg(self.addr, REG_CTRL_MEAS)?;
        self.bus.write_reg(self.addr, REG_CTRL_MEAS, ctrl & !0x03)?;
        // Humidity oversampling must be set before ctrl_meas.
        self.bus.write_reg(self.addr, REG_CTRL_HUM, 0b001)?;
        // osrs_t x2, osrs_p x2, forced mode.
        self.bus.write_reg(self.addr, REG_CTRL_MEAS, 0b010_010_01)?;

        let mut ready = false;
        for _ in 0..40 {
            if self.bus.read_reg(self.addr, REG_STATUS)? & NEW_DATA != 0 {
                ready = true;
                break;
            }
            pump();
        }
        if !ready {
            return Err(SensorError::NoData);
        }

        // Field block: [0] status, [2..5] pressure, [5..8] temperature,
        // [8..10] humidity.
        let mut f = [0u8; 17];
        self.bus.read_regs(self.addr, REG_STATUS, &mut f)?;
        let press_adc = ((f[2] as u32) << 12) | ((f[3] as u32) << 4) | ((f[4] as u32) >> 4);
        let temp_adc = ((f[5] as u32) << 12) | ((f[6] as u32) << 4) | ((f[7] as u32) >> 4);
        let hum_adc = ((f[8] as u16) << 8) | f[9] as u16;

        let c = &self.calib;
        let t1 = c.par_t1 as f32;
        let adc = temp_adc as f32;
        let var1 = ((adc / 16384.0) - (t1 / 1024.0)) * c.par_t2 as f32;
        let dt = (adc / 131072.0) - (t1 / 8192.0);
        let var2 = dt * dt * (c.par_t3 as f32 * 16.0);
        let t_fine = var1 + var2;
        let temp_c = t_fine / 5120.0;

        let (p1, p2, p3, p4, p5, p6, p7, p8, p9, p10) = (
            c.par_p1 as f32,
            c.par_p2 as f32,
            c.par_p3 as f32,
            c.par_p4 as f32,
            c.par_p5 as f32,
            c.par_p6 as f32,
            c.par_p7 as f32,
            c.par_p8 as f32,
            c.par_p9 as f32,
            c.par_p10 as f32,
        );
        let mut pv1 = (t_fine / 2.0) - 64000.0;
        let mut pv2 = pv1 * pv1 * (p6 / 131072.0);
        pv2 += pv1 * p5 * 2.0;
        pv2 = (pv2 / 4.0) + (p4 * 65536.0);
        pv1 = (((p3 * pv1 * pv1) / 16384.0) + (p2 * pv1)) / 524288.0;
        pv1 = (1.0 + (pv1 / 32768.0)) * p1;
        let press_pa = if pv1 == 0.0 {
            0.0
        } else {
            let mut calc = 1048576.0 - press_adc as f32;
            calc = ((calc - (pv2 / 4096.0)) * 6250.0) / pv1;
            let p9t = (p9 * calc * calc) / 2147483648.0;
            let p8t = calc * (p8 / 32768.0);
            let c3 = calc / 256.0;
            let p10t = (c3 * c3 * c3) * (p10 / 131072.0);
            calc + (p9t + p8t + p10t + (p7 * 128.0)) / 16.0
        };

        let temp_comp = t_fine / 5120.0;
        let hadc = hum_adc as f32;
        let hv1 = hadc - ((c.par_h1 as f32 * 16.0) + ((c.par_h3 as f32 / 2.0) * temp_comp));
        let hv2 = hv1
            * (c.par_h2 as f32 / 262144.0
                * (1.0
                    + ((c.par_h4 as f32 / 16384.0) * temp_comp)
                    + ((c.par_h5 as f32 / 1048576.0) * temp_comp * temp_comp)));
        let hv3 = c.par_h6 as f32 / 16384.0;
        let hv4 = c.par_h7 as f32 / 2097152.0;
        let hum = (hv2 + ((hv3 + (hv4 * temp_comp)) * hv2 * hv2)).clamp(0.0, 100.0);

        Ok(Reading {
            temperature_centi_c: (temp_c * 100.0) as i32,
            pressure_hpa: (press_pa / 100.0) as i32,
            humidity_centi_pct: (hum * 100.0) as i32,
            variant: self.variant,
        })
    }

    /// Give the bus back.
    pub fn free(self) -> B {
        self.bus
    }
}
