//! The XIAO MG24 module as a board: what is on the module itself, and nothing else.
//!
//! Two things exist on every MG24 module: the user LED. Two more exist only on the **Sense**
//! variant, which is the same PCB with more components fitted — so they are behind the
//! `sense` feature rather than in a crate of their own. A crate would duplicate the board
//! init, the clock tree and the LED to add two sensors.
//!
//! ```text
//! both variants    user LED            PA7, active low
//! feature "sense"  IMU bus             PB2 = SDA, PB3 = SCL   (the opposite of the vendor's
//!                                      Arduino variant: measured, see
//!                                      docs/04-reference/IMU-INVESTIGATION.md)
//!                  IMU power gate      PD5, must be high before the bus answers
//!                  microphone enable   PC8, and its signal is PC9 on IADC0
//! ```
//!
//! Nothing here knows about the Expansion Base. If you own the base, depend on
//! `xiao-expansion-base-bsp` as well, which depends on this crate.

use efr32mg24_hal::board::Board as HalBoard;
use efr32mg24_hal::delay::Delay;
use efr32mg24_hal::pin::{Output, Port};

#[cfg(feature = "sense")]
use efr32mg24_hal::bus::{BitBangI2c, Bus};
#[cfg(feature = "sense")]
use efr32mg24_hal::microphone::{MicError, Microphone};

/// The module: its peripherals, taken once.
pub struct Module {
    /// The peripheral owner. Only the `sense` half reads it (for the converter's register
    /// block); without that feature the module still needs it taken, so it stays.
    #[cfg_attr(not(feature = "sense"), allow(dead_code))]
    board: HalBoard,
}

impl Module {
    /// Take the peripherals. `None` if they were already taken.
    #[must_use]
    pub fn take() -> Option<Self> {
        Some(Self {
            board: HalBoard::take()?,
        })
    }

    /// A busy-wait delay over SysTick.
    ///
    /// `cortex_m::asm::delay(n)` is not *n* cycles — it inflated a buzzer calibration by 3x
    /// in this project — so nothing here offers it.
    pub fn delay(&mut self) -> Delay {
        Delay::with_sysclk(
            // SAFETY: SysTick is used by nothing else in this workspace; see the note on the
            // same call in the Expansion Base facade.
            unsafe { cortex_m::Peripherals::steal().SYST },
            efr32mg24_hal::clock::DEFAULT_SYSCLK_HZ,
        )
    }

    /// The on-board user LED: PA7, active low (writing 0 lights it).
    #[must_use]
    pub fn led(&self) -> Output<'_> {
        self.board.output_on(Port::A, 7, true)
    }

    /// The IMU's own I2C bus: `SDA = PB2`, `SCL = PB3`.
    ///
    /// The order is the opposite of what the Arduino variant declares, and it cost a night:
    /// with the lines the other way round the START condition never occurs and the sensor
    /// never answers. The pins also have external pull-ups on the module.
    #[cfg(feature = "sense")]
    #[must_use]
    pub fn imu_bus(&self) -> BitBangI2c<'_> {
        self.board.i2c_on(Bus::new(Port::B, 1 << 2, 1 << 3))
    }

    /// The IMU's power gate on PD5 — an unpowered IMU looks exactly like an absent one, so
    /// this must be driven high before the first bus access.
    #[cfg(feature = "sense")]
    #[must_use]
    pub fn imu_power(&self) -> Output<'_> {
        self.board.output_on(Port::D, 5, false)
    }

    /// Power the microphone: PC8 high, PC9 left to the converter, and IADC0 clocked.
    ///
    /// The two pins are separate: PC8 is the enable (`MIC_ENPIN` in Seeed's backend) and PC9
    /// is the analog signal. The converter returns a driver already configured for this part.
    ///
    /// # Errors
    ///
    /// [`MicError`] when the converter will not leave its disabled state — the symptom of a
    /// clock source that is not running.
    #[cfg(feature = "sense")]
    pub fn microphone(&self) -> Result<Microphone<'_>, MicError> {
        // PC8: enable, active high. PC9 stays in disabled mode so the ADC owns the pad.
        self.board.output_on(Port::C, 8, false).on();
        self.board.converter().microphone()
    }
}
