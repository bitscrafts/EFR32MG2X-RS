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

use efr32mg2x_hal::board::Board as HalBoard;
use efr32mg2x_hal::delay::Delay;
use efr32mg2x_hal::pin::{Output, Port};

#[cfg(feature = "sense")]
use efr32mg2x_hal::bus::{BitBangI2c, Bus};
#[cfg(feature = "sense")]
use efr32mg2x_hal::microphone::{MicError, Microphone};

/// The module: its peripherals, taken once.
pub struct Module {
    /// The peripheral owner. Only the `sense` half reads it (for the converter's register
    /// block); without that feature the module still needs it taken, so it stays.
    #[cfg_attr(not(feature = "sense"), allow(dead_code))]
    board: HalBoard,
}

impl Module {
    /// The chip-level board underneath this module.
    ///
    /// A carrier (the Expansion Base, for instance) has to wire its own devices onto the same
    /// buses, gates and timer this module already took. Handing out the HAL's board is what keeps
    /// the dependency arrow pointing one way: the carrier builds on the module, and the module is
    /// the only crate that touches the HAL's ownership of the peripherals.
    ///
    /// It is deliberately the HAL's board and not a re-export of the PAC: a carrier that could see
    /// registers could undo what this module configured.
    pub fn hal(&self) -> &HalBoard {
        &self.board
    }

    /// Take the peripherals. `None` if they were already taken.
    #[must_use]
    pub fn take() -> Option<Self> {
        // Before anything can touch the flash: the emlib's write and erase routines live in `.ram` and must execute
        // from RAM. `memory.x` links that section with its load image in flash; this copies it, once, here — every
        // entry path in this BSP goes through `take`, and the flash controller is used by NVM3 as soon as the
        // Bluetooth stack starts.
        unsafe { ramfunc_init() };
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
            // The clock in force, not the reset one: a delay calibrated for 19 MHz on a board running from the
            // 39 MHz crystal sleeps half of what was asked, and nothing says so. Measured on the parent board
            // 2026-11-21: five `delay_ms(1000)` calls took 2,46 s until this line read the live clock.
            self.board.clock_hz().0,
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
        self.board
            .converter()
            .expect("the converter is handed out once")
            .microphone()
    }

    /// The matrix/vector processor, enabled and ready.
    ///
    /// The driver lives in the HAL (`efr32mg2x_hal::mvp`) and this is the board's way of handing it out,
    /// for the same reason every other peripheral here is handed out: an example or an application should
    /// not have to name a PAC type, and enabling the block needs the CMU as well as the peripheral.
    ///
    /// Measured on this board (2026-11-17): 64 element `binary16` additions cost 289 cycles through the
    /// MVP against 921 in a software loop, self-check pass. That is the floor for its advantage, not a
    /// claim about a whole model.
    ///
    /// # Errors
    ///
    /// [`efr32mg2x_hal::mvp::Error::StuckDisabling`] when the block will not leave its disabled state,
    /// which is what a missing clock gate looks like from the outside, and
    /// [`efr32mg2x_hal::board::Handout::Taken`] when this board already handed the block out.
    pub fn mvp(
        &self,
    ) -> Result<efr32mg2x_hal::mvp::Mvp, efr32mg2x_hal::board::Handout<efr32mg2x_hal::mvp::Error>>
    {
        // The board refuses a second hand-out, so the refusal is part of what this returns -- a caller that asked
        // twice gets an error naming it, not a second handle to the same block.
        self.board.mvp()
    }
}

/// Copy the RAM-resident functions from their load image in flash to where they are linked to run.
///
/// `memory.x` gives `.ramfunc` a RAM VMA and a flash LMA; the reset handler from `cortex-m-rt` copies `.data` and
/// knows nothing about this section, so without this the flash routines run from the flash they are programming and
/// the write silently does nothing (measured: NVM3 reported success and no bond survived a reset).
///
/// # Safety
///
/// Call once, before any code that writes or erases flash, and never while interrupts can be using those routines.
unsafe fn ramfunc_init() {
    extern "C" {
        static mut __ramfunc_start: u32;
        static mut __ramfunc_end: u32;
        static __ramfunc_load: u32;
    }
    let start = core::ptr::addr_of_mut!(__ramfunc_start);
    let end = core::ptr::addr_of_mut!(__ramfunc_end);
    let load = core::ptr::addr_of!(__ramfunc_load);
    let words = (end as usize - start as usize) / 4;
    for i in 0..words {
        core::ptr::write_volatile(start.add(i), core::ptr::read_volatile(load.add(i)));
    }
}
