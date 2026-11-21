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
            // SAFETY: o HFXO0 nao e' exposto por nenhuma outra API deste crate, e quem chega aqui esta'
            // a construir a arvore de relogios -- ou seja, e' o dono dela por construcao. O `steal` e'
            // o mesmo padrao que o `Board` usa para o EMU e o WDOG.
            let hfxo = unsafe { crate::pac::Hfxo0Ns::steal() };

            hfxo.ctrl().modify(|_r, w| w.forceen().set_bit());

            // Espera limitada: um laco infinito num arranque e' uma placa que nunca mais responde a
            // ninguem, e num produto isso e' indistinguivel de brick. O limite e' um tecto, nao uma
            // espera -- sai no primeiro `RDY`.
            let mut voltas = 0u32;
            const TECTO: u32 = 2_000_000;
            while !hfxo.status().read().rdy().bit_is_set() {
                voltas += 1;
                if voltas > TECTO {
                    return Err(ClockError::HfxoTimeout);
                }
                core::hint::spin_loop();
            }

            cmu.sysclkctrl().modify(|_r, w| w.clksel().hfxo());
            hfxo_config.frequency
        } else {
            Hertz(Self::HFRCO_FREQ)
        };

        // ## O LFXO: recusado por falta do registo que o liga, nao por falta do selector

        // A razao escrita aqui antes -- "o selector do relogio lento nao esta' mapeado neste PAC" -- estava
        // errada, e mandava quem lesse procurar no sitio errado. Medido 2026-11-21 neste SVD e neste PAC:
        //
        //   em23grpaclkctrl    TEM a variante `Lfxo`  -> o selector existe e e' utilizavel
        //   CMU_OSCENCMD       0 ocorrencias no SVD    -> falta o registo que LIGA o oscilador
        //   LFXOEN             0 ocorrencias no SVD    -> falta o bit que o liga
        //   STATUS.LFXORDY     0 ocorrencias no PAC    -> falta a flag de "esta' pronto"
        //
        // Ou seja: nao ha como pedir ao silicio que arranque o cristal, nem como saber quando arrancou. O
        // selector sozinho nao serve de nada -- escolher LFXO num selector com o oscilador desligado da' um
        // relogio parado, e um relogio parado num contador e' pior do que o LFRCO que temos.
        //
        // O desbloqueio e' a montante, e nao e' codigo desta crate: obter um SVD com o `CMU_OSCENCMD`
        // (Silabs publica um por familia) e regenerar o PAC. O PAC esta' publicado em 0.1.1, portanto isso e'
        // uma versao nova, com o que isso implica. O HFXO acima mostra a forma que falta: ligar, esperar
        // `RDY`, seleccionar.

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
