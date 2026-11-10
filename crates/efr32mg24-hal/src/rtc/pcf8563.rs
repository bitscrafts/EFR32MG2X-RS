//! PCF8563 real-time clock — the calendar chip on the XIAO Expansion Base.
//!
//! This is the board's *external* clock at I2C address 0x51, not the on-chip
//! SYSRTC in [`super`]: the PCF8563 keeps a full calendar (year, month, day,
//! weekday, hours, minutes, seconds) and a coin cell behind it keeps counting when
//! the MG24 is unpowered.
//!
//! ## Registers used
//!
//! - `0x00` control 1, `0x01` control 2 — left at their reset values; the clock
//!   runs in its default 32.768 kHz mode,
//! - `0x02` seconds — bit 7 is **VL**, "clock integrity not guaranteed": it is set
//!   when the chip lost power, so a reading with VL set is not a time, it is a
//!   question mark. The driver surfaces it instead of hiding it,
//! - `0x03` minutes, `0x04` hours, `0x05` days, `0x06` weekday, `0x07` months (bit
//!   7 = century), `0x08` years.
//!
//! All fields are BCD. One seven-byte read gets the whole calendar, so a reading
//! cannot straddle a rollover.
//!
//! ## What this driver does not do
//!
//! Alarms, the timer, and the CLKOUT pin. They are in the chip, and none of them
//! are needed to answer "what time is it" — adding them without a use would be
//! untested code pretending to be a feature.

use crate::bus::{BusError, I2cBus};

/// Default 7-bit address of the PCF8563.
pub const ADDR_DEFAULT: u8 = 0x51;

/// First register of the calendar block.
const REG_SECONDS: u8 = 0x02;
/// Bytes from seconds through years.
const CALENDAR_LEN: usize = 7;

/// Bit 7 of the seconds register: clock integrity not guaranteed.
const VL_FLAG: u8 = 0x80;
/// Bit 7 of the months register: century.
const CENTURY_FLAG: u8 = 0x80;

/// A calendar reading.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DateTime {
    /// Year, 2000..=2099 (the chip only carries two digits plus a century bit).
    pub year: u16,
    /// Month, 1..=12.
    pub month: u8,
    /// Day of month, 1..=31.
    pub day: u8,
    /// Weekday, 0..=6 (the chip's own numbering; 0 is Sunday on power-up).
    pub weekday: u8,
    /// Hours, 0..=23.
    pub hours: u8,
    /// Minutes, 0..=59.
    pub minutes: u8,
    /// Seconds, 0..=59.
    pub seconds: u8,
}

/// Errors from the clock chip.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RtcError {
    /// The transport failed.
    Bus(BusError),
    /// The chip answered, but the values are not a valid date or time.
    Corrupt,
    /// The clock lost power: `VL` was set, so the time cannot be trusted.
    ClockInvalid,
}

impl From<BusError> for RtcError {
    fn from(error: BusError) -> Self {
        RtcError::Bus(error)
    }
}

/// A PCF8563 on some I2C bus.
pub struct Pcf8563<B: I2cBus> {
    bus: B,
    addr: u8,
}

impl<B: I2cBus> Pcf8563<B> {
    /// Bind the driver to a bus.
    pub fn new(bus: B) -> Self {
        Self::with_address(bus, ADDR_DEFAULT)
    }

    /// Bind the driver to a chip strapped at another address.
    pub fn with_address(bus: B, addr: u8) -> Self {
        Self { bus, addr }
    }

    /// Read the whole calendar in one transaction.
    ///
    /// Returns [`RtcError::ClockInvalid`] when `VL` is set: that is the chip saying
    /// its contents survived a power loss, and reporting a plausible-looking time
    /// from it would be worse than reporting nothing.
    pub fn read(&mut self) -> Result<DateTime, RtcError> {
        let mut raw = [0u8; CALENDAR_LEN];
        self.bus
            .read_regs(self.addr, REG_SECONDS, &mut raw)
            .map_err(RtcError::Bus)?;

        if raw[0] & VL_FLAG != 0 {
            return Err(RtcError::ClockInvalid);
        }

        let time = DateTime {
            seconds: bcd(raw[0] & 0x7F)?,
            minutes: bcd(raw[1] & 0x7F)?,
            hours: bcd(raw[2] & 0x3F)?,
            day: bcd(raw[3] & 0x3F)?,
            weekday: bcd(raw[4] & 0x07)?,
            month: bcd(raw[5] & 0x1F)?,
            year: 2000 + u16::from(bcd(raw[6])?),
        };

        // Range checks, so a bus glitch cannot produce month 13.
        let valid = (1..=12).contains(&time.month)
            && (1..=31).contains(&time.day)
            && time.hours < 24
            && time.minutes < 60
            && time.seconds < 60
            && time.weekday <= 6
            && time.year <= 2099;
        if valid {
            Ok(time)
        } else {
            Err(RtcError::Corrupt)
        }
    }

    /// Set the calendar. Clears `VL`, because writing a time is exactly the claim
    /// that the clock is valid again.
    pub fn set(&mut self, when: &DateTime) -> Result<(), RtcError> {
        if !(1..=12).contains(&when.month)
            || !(1..=31).contains(&when.day)
            || when.hours >= 24
            || when.minutes >= 60
            || when.seconds >= 60
            || when.weekday > 6
            || !(2000..=2099).contains(&when.year)
        {
            return Err(RtcError::Corrupt);
        }
        let payload = [
            REG_SECONDS,
            to_bcd(when.seconds) & 0x7F, // VL cleared
            to_bcd(when.minutes),
            to_bcd(when.hours),
            to_bcd(when.day),
            to_bcd(when.weekday),
            to_bcd(when.month) | if when.year >= 2100 { CENTURY_FLAG } else { 0 },
            to_bcd((when.year % 100) as u8),
        ];
        self.bus.write(self.addr, &payload)?;
        Ok(())
    }

    /// True when the chip says its contents survived a power loss.
    pub fn clock_valid(&mut self) -> Result<bool, RtcError> {
        let seconds = self.bus.read_reg(self.addr, REG_SECONDS)?;
        Ok(seconds & VL_FLAG == 0)
    }

    /// Give the bus back.
    pub fn free(self) -> B {
        self.bus
    }
}

/// Decode one BCD byte.
fn bcd(byte: u8) -> Result<u8, RtcError> {
    let tens = byte >> 4;
    let units = byte & 0x0F;
    if tens > 9 || units > 9 {
        return Err(RtcError::Corrupt);
    }
    Ok(tens * 10 + units)
}

/// Encode as BCD.
fn to_bcd(value: u8) -> u8 {
    ((value / 10) << 4) | (value % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bcd_round_trip() {
        for value in 0u8..=99 {
            assert_eq!(bcd(to_bcd(value)), Ok(value), "value {value}");
        }
    }

    #[test]
    fn invalid_bcd_is_rejected() {
        // 0x0A is not a decimal digit; a bus glitch must not become a time.
        assert_eq!(bcd(0x0A), Err(RtcError::Corrupt));
        assert_eq!(bcd(0xA0), Err(RtcError::Corrupt));
    }
}
