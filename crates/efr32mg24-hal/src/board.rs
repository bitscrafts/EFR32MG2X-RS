//! Board init: the single place where the HAL touches the PAC singleton.
//!
//! The point of this module is that no **example** ends up naming `pac`. An
//! application asks the HAL for a board, and the board hands out the pieces
//! (I2C bus, LED) with their bus clocks already enabled — because a missing
//! `CMU_CLKEN0` bit is a precise bus fault on the first register access, and
//! that failure belongs in one place, not in every example.
//!
//! The PAC is still underneath: this is a PAC-based HAL, and it is the PAC that
//! defines every register this board pokes. What the HAL owns is the sequence
//! and the defaults that the hardware demands (gate first, then configure).

use efr32mg24_pac as pac;

use crate::bus::BitBangI2c;

/// Everything an application needs from the XIAO MG24 Sense + Expansion Base.
pub struct Board {
    dp: pac::Peripherals,
}

impl Board {
    /// Take the peripherals and enable the bus clocks this HAL drives.
    ///
    /// Returns `None` if called twice, which is what the PAC singleton is for.
    pub fn take() -> Option<Self> {
        let dp = pac::Peripherals::take()?;
        // GPIO is needed by the I2C bit-banger and the LED; without this gate
        // the first DOUT/MODEL access faults (CFSR=0x8200, BFAR = the address).
        dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
        Some(Self { dp })
    }

    /// The Expansion Base I2C bus (PC04 = SDA, PC05 = SCL).
    ///
    /// Bit-banged on purpose: the on-chip I2C0 master on this die cannot
    /// complete a read that follows a write. See
    /// `docs/04-reference/I2C0-READ-DEFECT.md`. When that is fixed this method
    /// returns the hardware backend and nothing above it changes.
    pub fn i2c(&self) -> BitBangI2c<'_> {
        BitBangI2c::new(&self.dp.gpio_s)
    }

    /// The on-board user LED (PA7, active low).
    pub fn led(&self) -> Led<'_> {
        Led::new(&self.dp.gpio_s)
    }
}

/// The yellow user LED on PA7, wired active low.
pub struct Led<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl<'a> Led<'a> {
    const PIN: u32 = 1 << 7;

    /// Configure PA7 as a push-pull output, starting off (which is high).
    pub fn new(gpio: &'a pac::gpio_s::RegisterBlock) -> Self {
        gpio.porta_model().modify(|_r, w| w.mode7().pushpull());
        gpio.porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | Self::PIN) });
        Self { gpio }
    }

    /// On (the pin is driven low).
    pub fn on(&self) {
        self.gpio
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() & !Self::PIN) });
    }

    /// Off (the pin goes high).
    pub fn off(&self) {
        self.gpio
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | Self::PIN) });
    }

    /// Invert.
    pub fn toggle(&self) {
        self.gpio
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ Self::PIN) });
    }
}
