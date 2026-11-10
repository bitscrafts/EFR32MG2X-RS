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

use crate::bus::{BitBangI2c, Wiring};

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

    /// The IMU's own bus (PB2 = SDA, PB3 = SCL).
    ///
    /// The on-board LSM6DS3TR-C is **not** on the Expansion Base bus: it has its
    /// own pair, and the lines are the other way round from what the Arduino
    /// variant declares. Both facts were measured — see
    /// `docs/04-reference/IMU-INVESTIGATION.md` — and both are why the sensor
    /// answered nothing until this wiring was found.
    pub fn imu_i2c(&self) -> BitBangI2c<'_> {
        BitBangI2c::on(&self.dp.gpio_s, Wiring::ImuBus)
    }

    /// Power gate for the on-board sensors (PD5).
    ///
    /// The LSM6DS3TR-C on the Sense module does not answer on the I2C bus until
    /// this pin is driven high — an unpowered IMU looks exactly like an absent
    /// one, and a bus scan will simply not show 0x6A. Seeed's own example does
    /// `pinMode(PD5, OUTPUT); digitalWrite(PD5, HIGH);` before touching the
    /// sensor. That is wiring, so it lives here and not in the driver.
    pub fn sensors_power(&self) -> SensorPower<'_> {
        SensorPower::new(&self.dp.gpio_s)
    }

    /// The Expansion Base user button on PC01, wired to ground with a pull-up.
    pub fn button(&self) -> Button<'_> {
        Button::new(&self.dp.gpio_s)
    }

    /// IADC0 clocked from HFRCOEM23, ready for the microphone driver.
    ///
    /// HFRCOEM23 rather than FSRCO: measured, with IADCCLK from FSRCO the
    /// converter's synchroniser never finished disabling and the following
    /// configuration write faulted. HFRCOEM23 is the oscillator already proven
    /// to clock TIMER0 here, so it is forced on and waited for before the IADC
    /// clock is pointed at it.
    pub fn microphone_adc(&self) -> &pac::iadc0_s::RegisterBlock {
        self.dp
            .cmu_s
            .clken0()
            .modify(|_r, w| w.iadc0().set_bit().hfrcoem23().set_bit());
        self.dp
            .hfrcoem23_s
            .ctrl()
            .modify(|_r, w| w.forceen().set_bit());
        let mut spin = 0u32;
        while !self.dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                break;
            }
        }
        self.dp
            .cmu_s
            .iadcclkctrl()
            .modify(|_r, w| w.clksel().hfrcoem23());
        &self.dp.iadc0_s
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

/// The sensor power enable on PD5.
pub struct SensorPower<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl<'a> SensorPower<'a> {
    const PIN: u32 = 1 << 5;

    fn new(gpio: &'a pac::gpio_s::RegisterBlock) -> Self {
        Self { gpio }
    }

    /// Drive PC8 high, powering the microphone. The IMU's gate is PD5; the
    /// microphone's is PC8 (`MIC_ENPIN` in Seeed's MG24 backend), so they are
    /// separate and both belong to the board, not to the drivers.
    pub fn enable_microphone(&self) {
        self.gpio.portc_modeh().modify(|_r, w| w.mode0().pushpull());
        self.gpio
            .portc_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 8)) });
        // PC9 is the analog input: leave it in disabled mode so the ADC owns the
        // pad, and route the odd analog bus to the converter.
        self.gpio.portc_modeh().modify(|_r, w| w.mode1().disabled());
        self.gpio
            .cdbusalloc()
            .modify(|_r, w| unsafe { w.cdodd0().bits(1) });
    }

    /// Drive PD5 high, powering the IMU.
    pub fn enable(&self) {
        self.gpio.portd_model().modify(|_r, w| w.mode5().pushpull());
        self.gpio
            .portd_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | Self::PIN) });
    }

    /// Drop PD5 low again.
    pub fn disable(&self) {
        self.gpio
            .portd_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() & !Self::PIN) });
    }
}

/// The user button on PC01 (XIAO pin D1).
///
/// The base board pulls the pin up and the button shorts it to ground, so the
/// pin reads high at rest and low while pressed. `MODE 2` (input with pull)
/// plus `DOUT = 1` selects the internal pull-up, which is what makes the pin
/// read a defined level when the board is used without the base.
pub struct Button<'a> {
    gpio: &'a pac::gpio_s::RegisterBlock,
}

impl<'a> Button<'a> {
    const PIN: u32 = 1 << 1;

    fn new(gpio: &'a pac::gpio_s::RegisterBlock) -> Self {
        gpio.portc_model().modify(|_r, w| w.mode1().inputpull());
        gpio.portc_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() | Self::PIN) });
        Self { gpio }
    }

    /// True while the button is held down.
    pub fn is_pressed(&self) -> bool {
        self.gpio.portc_din().read().bits() & Self::PIN == 0
    }
}
