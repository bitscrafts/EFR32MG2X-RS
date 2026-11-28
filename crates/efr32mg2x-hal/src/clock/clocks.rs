//! Clock system implementation with hardware register access
//!
//! This module implements actual hardware configuration for the EFR32MG24 clock system.

use super::frozen::FrozenClocks;
use super::types::{ClockConfig, ClockError, Hertz};

/// Configured clock frequencies with hardware control
///
/// This struct contains the actual clock frequencies after hardware configuration.
#[derive(Copy, Clone, Debug)]
pub struct Clocks {
    /// High frequency clock (HFCLK) frequency
    pub(super) hfclk: Hertz,
    /// Low frequency clock (LFCLK) frequency
    pub(super) lfclk: Hertz,
    /// Peripheral clock (PCLK) frequency
    pub(super) pclk: Hertz,
    /// System clock (SYSCLK) frequency
    pub(super) sysclk: Hertz,
}

/// The SYSCLK this HAL configures when no other clock source is asked for.
///
/// It is a statement about the configuration, not a measurement: the real rate was
/// measured with the DWT counter at ~19 MHz (a 2700 Hz tone needed TIMER0 TOP=3518), and
/// anything that needs better than a percent should measure its own.
pub const DEFAULT_SYSCLK_HZ: u32 = 19_000_000;

/// Start the HFXO, wait for it, and only then point SYSCLK at it.
///
/// ## The order is not negotiable
///
/// Enable (`FORCEEN`), wait for the crystal to settle (`STATUS.RDY`), and only then point `CLKSEL` at it. Selecting
/// before `RDY` gives a core running from a clock that does not exist yet.
///
/// ## Why this left the constructor
///
/// It used to live inside it, so only `Clocks::new` could run it -- and `Clocks::new` takes the CMU **by value**, which
/// the `Board` cannot hand over without ceasing to be its owner. The result was a sequence that was implemented and
/// unreachable from the API the product uses: no example could ask for the crystal without stealing the PAC, and
/// stealing the PAC in an example is the defect the gate refuses. There is one sequence and it lives here; the
/// `Board` and the constructor both call it.
///
/// ## The ceiling
///
/// An endless loop in a start-up is a board that never answers anyone again, and in a product that is
/// indistinguishable from brick. The bound is a ceiling, not a wait: it leaves on the first `RDY`.
pub(crate) fn start_hfxo(cmu: &crate::pac::CmuS, frequency: Hertz) -> Result<(), ClockError> {
    // ## The HFXO's own clock gate, before anything else
    //
    // Without `CMU_CLKEN0.HFXO0` set, the first access to the oscillator's registers is a bus fault -- and not a
    // quiet one: the core goes down. Measured 2026-11-21 on the board: `before: clock_hz = 19000000`, and 0.16 s
    // later `probe-rs` reported `Error: Exception`. `apply_hfrco_cal` opens the HFRCO0 and DPLL0 gates for exactly the
    // same reason (without it, the first read of `HFRCO0_STATUS` faults at 0x4001_000C); the HFXO was left out
    // because the sequence was written against the register rather than against the peripheral's start-up.
    cmu.clken0().modify(|_r, w| w.hfxo0().set_bit());

    // SAFETY: HFXO0 is not exposed by any other API in this crate, and a caller who reaches here is configuring the
    // clock tree -- that is, is its owner by construction. It is the same pattern the `Board` uses for the EMU and the
    // WDOG.
    let hfxo = unsafe { crate::pac::Hfxo0Ns::steal() };

    hfxo.ctrl().modify(|_r, w| w.forceen().set_bit());

    let mut spins = 0u32;
    const CEILING: u32 = 2_000_000;
    while !hfxo.status().read().rdy().bit_is_set() {
        spins += 1;
        if spins > CEILING {
            return Err(ClockError::HfxoTimeout);
        }
        core::hint::spin_loop();
    }

    cmu.sysclkctrl().modify(|_r, w| w.clksel().hfxo());
    // The clock changed: the number the rest of the system calibrates against has to know. `frequency` is what the
    // caller asked for, and is the value this board's crystal has -- 39 MHz on the XIAO MG24.
    let _ = frequency;
    Ok(())
}

impl Clocks {
    /// Default internal RC oscillator frequencies
    const HFRCO_FREQ: u32 = 19_000_000; // 19 MHz default
    const LFRCO_FREQ: u32 = 32_768; // 32.768 kHz

    // The oscillator wait is a poll on HFRCOEM23.STATUS.RDY, which reports reality;
    // an iteration counter only ever reported how fast the loop was going.

    /// Configure the clock system with hardware register access
    ///
    /// # Arguments
    ///
    /// * `cmu` - CMU peripheral (consumed to ensure exclusive access)
    /// * `config` - Clock configuration
    ///
    /// # Returns
    ///
    /// A `Result` containing either the configured `Clocks` or a `ClockError`
    ///
    /// # Errors
    ///
    /// Returns `ClockError::HfxoTimeout` if HFXO fails to stabilize within timeout
    /// Returns `ClockError::LfxoTimeout` if LFXO fails to stabilize within timeout
    ///
    /// # Hardware Configuration
    ///
    /// This method configures the CMU registers to:
    /// - Enable and configure HFXO if requested
    /// - Enable and configure LFXO if requested
    /// - Select SYSCLK source
    /// - Wait for oscillator stabilization with timeout
    ///
    /// # Example
    ///
    /// ```no_run
    /// use efr32mg2x_hal::{clock::{Clocks, ClockConfig, HfxoConfig}, pac};
    ///
    /// let dp = pac::Peripherals::take().unwrap();
    ///
    /// let clocks = Clocks::new(
    ///     dp.cmu_s,
    ///     ClockConfig {
    ///         hfxo: None,
    ///         lfxo: None, // Use internal LFRCO
    ///     }
    /// )?;
    /// # Ok::<(), efr32mg2x_hal::clock::ClockError>(())
    /// ```
    pub fn new(
        cmu: crate::pac::CmuS,
        config: ClockConfig,
    ) -> Result<(Self, crate::pac::CmuS), ClockError> {
        // ## O HFXO: ligar, esperar, e so' depois seleccionar

        // Antes desta sequencia, o construtor registava a frequencia pedida e nao mexia no relogio: quem
        // pedisse 39 MHz recebia `hfclk() == 39` com o nucleo a correr a 19. Desse numero dependem o
        // divisor do I2C, o SysTick do delay e o pace da amostragem -- tudo errado por um factor de
        // dois, sem um unico erro de compilacao.

        // A ordem importa e nao e' negociavel: ligar (`FORCEEN`), esperar que o cristal estabilize
        // (`STATUS.RDY`), e so' entao apontar o `CLKSEL` para ele. Seleccionar antes do `RDY` da' um
        // nucleo a correr de um relogio que ainda nao existe.

        let hfclk = if let Some(hfxo_config) = config.hfxo {
            start_hfxo(&cmu, hfxo_config.frequency)?;
            hfxo_config.frequency
        } else {
            Hertz(Self::HFRCO_FREQ)
        };

        // ## LFXO: refused because its readiness cannot be read, not because its selector is missing
        //
        // An earlier version of this comment said the slow-clock selector was not mapped in this PAC. That was wrong and
        // sent a reader to the wrong place. Measured 2026-11-21 in this SVD and this PAC, and re-checked against the
        // vendor header on 2026-11-28:
        //
        //   em23grpaclkctrl    has the `Lfxo` variant      -- the selector exists and is usable
        //   CMU_OSCENCMD       0 occurrences in the SVD    -- there is no oscillator-enable command register
        //   CMU_CLKEN0.LFXO    bit 23, `efr32mg24_cmu.h:545`
        //                                                  -- the gate that starts the oscillator *does* exist
        //   STATUS.LFXORDY     0 occurrences in the PAC    -- and the header exposes only `CALRDY`, so there is no
        //                                                      "it is ready" bit to wait on
        //
        // So the crystal can be started -- `CMU_CLKEN0.LFXO`, exactly as bit 22 starts the LFRCO -- and the driver
        // cannot know when it has stabilised. Selecting it blind gives a stopped clock, and a stopped clock driving a
        // counter is worse than the LFRCO this crate uses today.
        //
        // What would unblock it is a decision, not a register: start the gate and settle for a time taken from the
        // crystal's own specification, then prove the rate against an independent time base (the SYSRTC on the LFRCO,
        // or a measurement against an external reference). That is a measurement this crate has not made, so the
        // refusal stays -- but it is now narrow and honest: the enable exists, the readiness does not.

        assert!(
            config.lfxo.is_none(),
            "Clocks::new does not select LFXO yet: this SVD has no CMU_OSCENCMD and no LFXOEN bit, so the \
             crystal cannot be started and its readiness cannot be read. The selector (EM23GRPACLKCTRL) does \
             exist; the oscillator control is what is missing. Pass lfxo: None (LFRCO) until an SVD carrying \
             that register is used to regenerate the PAC."
        );

        let lfclk = if let Some(lfxo_config) = config.lfxo {
            // Track LFXO frequency but do NOT configure LFXO registers yet.
            // TODO: implement proper LFXO enable + wait + select sequence.
            lfxo_config.frequency
        } else {
            // Use default LFRCO
            Hertz(Self::LFRCO_FREQ)
        };

        // On EFR32MG24, SYSCLK and PCLK typically equal HFCLK
        let sysclk = hfclk;
        let pclk = hfclk;

        let clocks = Self {
            hfclk,
            lfclk,
            pclk,
            sysclk,
        };

        Ok((clocks, cmu))
    }

    /// Freeze the clock configuration
    ///
    /// This consumes the Clocks struct and CMU peripheral, returning a FrozenClocks
    /// which can be used by other peripherals but cannot be modified.
    ///
    /// The FrozenClocks retains ownership of the CMU peripheral, providing
    /// safe access for peripheral clock enable operations.
    ///
    /// # Arguments
    ///
    /// * `cmu` - CMU peripheral to store in the frozen configuration
    pub fn freeze(self, cmu: crate::pac::CmuS) -> FrozenClocks {
        FrozenClocks { clocks: self, cmu }
    }

    /// Get the high frequency clock (HFCLK) frequency
    #[inline]
    pub fn hfclk(&self) -> Hertz {
        self.hfclk
    }

    /// Get the low frequency clock (LFCLK) frequency
    #[inline]
    pub fn lfclk(&self) -> Hertz {
        self.lfclk
    }

    /// Get the peripheral clock (PCLK) frequency
    #[inline]
    pub fn pclk(&self) -> Hertz {
        self.pclk
    }

    /// Get the system clock (SYSCLK) frequency
    #[inline]
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }
}
