//! XIAO MG24 Sense board initialization and configuration

use crate::pins::Pins;
use crate::{hal, pac};
use hal::clock::{ClockConfig, FrozenClocks, HfxoConfig, LfxoConfig};
use hal::gpio::GpioExt;

/// XIAO MG24 Sense board abstraction
///
/// Provides access to all board peripherals and pins.
///
/// # Singleton Pattern
///
/// Only one instance can exist at a time (enforced by PAC peripheral ownership).
pub struct Board {
    /// GPIO pins
    pub pins: Pins,

    /// Frozen clock configuration
    pub clocks: FrozenClocks,

    /// Core peripherals (SysTick, NVIC, etc.)
    ///
    /// SysTick is wrapped in Option to allow taking for delay creation.
    pub core: cortex_m::Peripherals,
}

impl Board {
    /// Take ownership of board peripherals
    ///
    /// Returns `None` if peripherals have already been taken.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::Board;
    ///
    /// let board = Board::take().unwrap();
    /// ```
    pub fn take() -> Option<Self> {
        let dp = pac::Peripherals::take()?;
        let cp = cortex_m::Peripherals::take()?;

        Some(Self::new(dp, cp))
    }

    /// Initialize board with default configuration
    ///
    /// Sets up:
    /// - HFXO @ 39 MHz (crystal oscillator)
    /// - LFXO @ 32.768 kHz (crystal oscillator)
    /// - SYSCLK @ 39 MHz (from HFXO)
    /// - All GPIO clocks enabled
    /// - SysTick configured for delays
    ///
    /// # Arguments
    ///
    /// - `dp` - PAC device peripherals
    /// - `cp` - Cortex-M core peripherals
    pub fn new(dp: pac::Peripherals, cp: cortex_m::Peripherals) -> Self {
        // Configure clocks: HFXO @ 39 MHz, LFXO @ 32.768 kHz
        let (clocks, cmu) = hal::clock::Clocks::new(
            dp.cmu_s,
            ClockConfig {
                hfxo: Some(HfxoConfig::new(39_000_000)),
                lfxo: Some(LfxoConfig::default()),
            },
        )
        .expect("Clock configuration failed");

        let frozen_clocks = clocks.freeze(cmu);

        // Split GPIO peripheral into individual pins
        let gpio_parts = dp.gpio_s.split(&frozen_clocks);

        // Initialize pins from GPIO parts
        let pins = Pins::new(gpio_parts);

        Self {
            pins,
            clocks: frozen_clocks,
            core: cp,
        }
    }

    /// Initialize board with custom clock configuration
    ///
    /// Allows fine-grained control over clock tree.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::Board;
    /// use xiao_mg24_bsp::hal::clock::{ClockConfig, HfxoConfig};
    ///
    /// let config = ClockConfig {
    ///     hfxo: Some(HfxoConfig::new(39_000_000)),
    ///     lfxo: None,  // Use internal LFRCO
    /// };
    ///
    /// let board = Board::with_config(config).unwrap();
    /// ```
    pub fn with_config(config: ClockConfig) -> Option<Self> {
        let dp = pac::Peripherals::take()?;
        let cp = cortex_m::Peripherals::take()?;

        let (clocks, cmu) =
            hal::clock::Clocks::new(dp.cmu_s, config).expect("Clock configuration failed");

        let frozen_clocks = clocks.freeze(cmu);

        let gpio_parts = dp.gpio_s.split(&frozen_clocks);
        let pins = Pins::new(gpio_parts);

        Some(Self {
            pins,
            clocks: frozen_clocks,
            core: cp,
        })
    }

    /// Create a delay provider using SysTick
    ///
    /// This method consumes the Board and returns both the Delay provider
    /// and the Board with SysTick removed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::Board;
    /// use embedded_hal::delay::DelayNs;
    ///
    /// let board = Board::take().unwrap();
    /// let mut delay = board.into_delay();
    ///
    /// delay.delay_ms(100);
    /// ```
    pub fn into_delay(self) -> hal::delay::Delay {
        hal::delay::Delay::new(self.core.SYST, &self.clocks)
    }

    /// Get a reference to the clocks
    pub fn clocks(&self) -> &FrozenClocks {
        &self.clocks
    }

    /// Split the board into its components
    ///
    /// This is useful when you need to use individual pins while also creating a delay provider.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::Board;
    /// use embedded_hal::digital::OutputPin;
    /// use embedded_hal::delay::DelayNs;
    ///
    /// let board = Board::take().unwrap();
    /// let (pins, clocks, core) = board.split();
    ///
    /// let mut led = pins.d10.into_push_pull_output();
    /// let mut delay = hal::delay::Delay::new(core.SYST, &clocks);
    /// ```
    pub fn split(self) -> (Pins, FrozenClocks, cortex_m::Peripherals) {
        (self.pins, self.clocks, self.core)
    }
}
