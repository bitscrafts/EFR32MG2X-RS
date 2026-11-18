//! Analog-to-Digital Converter (IADC) Driver
//!
//! This module provides ADC functionality using the IADC0 peripheral
//! for the EFR32MG24 microcontroller.
//!
//! # Hardware Registers
//!
//! - **EN**: Enable register
//! - **CTRL**: Control register
//! - **CMD**: Command register (start conversion)
//! - **STATUS**: Status flags
//! - **CFG0**: Configuration register for single conversion
//! - **SINGLE**: Single conversion configuration
//! - **SINGLEFIFODATA**: Single conversion result
//! - **IF/IEN**: Interrupt flags and enable
//!
//! # Features
//!
//! - Single-shot ADC conversion
//! - Configurable resolution (12-bit)
//! - Multiple reference voltage sources (VBGR, VDD)
//! - Timeout-protected conversion
//! - Critical section protection for RTOS safety
//!
//! # Example
//!
//! ```no_run
//! use efr32mg2x_hal::{
//!     clock::{Clocks, ClockConfig, HfxoConfig},
//!     adc::{Adc, Config, Channel, Reference},
//!     pac,
//! };
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Configure clocks
//! let clocks = Clocks::new(dp.cmu_s, ClockConfig {
//!     hfxo: Some(HfxoConfig::new(39_000_000)),
//!     lfxo: Some(Default::default()),
//! }).freeze();
//!
//! // Create ADC with default configuration (12-bit, VBGR reference)
//! let mut adc = Adc::new(dp.iadc0_s, Config::default(), &dp.cmu_s);
//!
//! // Read from channel 0
//! let value = adc.read(Channel::Ch0).unwrap();
//! ```

mod traits;
mod types;

pub use types::{Channel, Config, Error, Reference, Resolution};

use crate::pac;

/// ADC instance
pub struct Adc {
    adc: pac::Iadc0S,
    reference: Reference,
}

/// Wait for the converter's synchroniser to settle: `SYNCBUSY` clear and `EN.DISABLING` clear.
fn wait_ready(adc: &pac::Iadc0S) -> bool {
    let mut spin = 0u32;
    while adc.status().read().syncbusy().bit_is_set() || adc.en().read().disabling().bit_is_set() {
        spin += 1;
        if spin > 2_000_000 {
            return false;
        }
    }
    true
}

impl Adc {
    /// Creates a new ADC instance
    ///
    /// # Arguments
    ///
    /// * `adc` - IADC peripheral instance
    /// * `config` - ADC configuration
    /// * `clocks` - Frozen clock configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use efr32mg2x_hal::{adc::{Adc, Config}, clock::{Clocks, ClockConfig, HfxoConfig}, pac};
    /// # let dp = pac::Peripherals::take().unwrap();
    /// # let clocks = Clocks::new(dp.cmu_s, ClockConfig {
    /// #     hfxo: Some(HfxoConfig::new(39_000_000)),
    /// #     lfxo: Some(Default::default()),
    /// # }).freeze();
    /// let adc = Adc::new(dp.iadc0_s, Config::default(), &dp.cmu_s);
    /// ```
    pub fn new(adc: pac::Iadc0S, config: Config, cmu: &pac::CmuS) -> Self {
        // The driver opens its own gate, the way `Wdog` does: an example should not have to
        // freeze the whole clock tree to read a channel.
        cmu.clken0()
            .modify(|_, w| w.iadc0().set_bit().hfrcoem23().set_bit());
        cmu.iadcclkctrl().modify(|_, w| w.clksel().hfrcoem23());

        // Measured 2026-11-15, in this order, and the order is the whole point:
        //
        // 1. Take the converter down and wait for the synchroniser, because configuration is only
        //    accepted while it is disabled.
        // 2. Write CTRL, CFG0, SCHED0 and SINGLEFIFOCFG while disabled.
        // 3. Bring it back up and wait again before anything writes a WSYNC register.
        //
        // The previous version enabled first and then wrote CTRL, which faults with
        // `Precise data access error at 0x49004008` and leaves the converter in DISABLING -- a
        // state that survives a core reset, so the next boot faults too. `microphone::Converter`
        // has always done it in this order; this driver had not, and no example reached it.
        adc.en().write(|w| w.en().clear_bit());
        if !wait_ready(&adc) {
            // Still stuck: leave it disabled and let the caller see a converter that reads zero
            // rather than a fault inside the driver.
            return Self {
                adc,
                reference: config.reference,
            };
        }

        adc.ctrl().write(|w| {
            w.warmupmode().normal();
            w.adcclksuspend0().prswudis();
            w.adcclksuspend1().prswudis();
            unsafe {
                // ceil(fsrc / 1 MHz) - 1; a larger value only lengthens the internal timing
                // windows, so 19 is safe for any MHz-scale clock.
                w.timebase().bits(19);
                w.hsclkrate().bits(0);
            }
            w
        });

        adc.cfg0().write(|w| match config.reference {
            Reference::Vbgr => w
                .adcmode()
                .normal()
                .osrhs()
                .hispd2()
                .analoggain()
                .anagain1()
                .refsel()
                .vbgr(),
            Reference::Vdd => w
                .adcmode()
                .normal()
                .osrhs()
                .hispd2()
                .analoggain()
                .anagain1()
                .refsel()
                .vddx(),
        });

        adc.sched0().write(|w| unsafe { w.prescale().bits(1) });
        adc.singlefifocfg().write(|w| {
            w.alignment().right12();
            w.showid().clear_bit();
            w.dvl().valid1()
        });

        adc.en().write(|w| w.en().set_bit());
        let _ = wait_ready(&adc);

        Self {
            adc,
            reference: config.reference,
        }
    }

    /// Perform a single ADC conversion on the specified channel
    ///
    /// This function configures the channel, starts the conversion, waits for
    /// completion (with timeout), and returns the result.
    ///
    /// # Arguments
    ///
    /// * `channel` - ADC channel to read from
    ///
    /// # Returns
    ///
    /// `Ok(u16)` with the conversion result (12-bit value 0-4095)
    /// `Err(Error)` if the conversion times out
    ///
    /// # Thread Safety
    ///
    /// This function uses critical sections for atomic register access,
    /// making it safe to call from interrupt contexts or RTOS tasks.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use efr32mg2x_hal::{adc::{Adc, Config, Channel}, clock::{Clocks, ClockConfig, HfxoConfig}, pac};
    /// # let dp = pac::Peripherals::take().unwrap();
    /// # let clocks = Clocks::new(dp.cmu_s, ClockConfig {
    /// #     hfxo: Some(HfxoConfig::new(39_000_000)),
    /// #     lfxo: Some(Default::default()),
    /// # }).freeze();
    /// # let mut adc = Adc::new(dp.iadc0_s, Config::default(), &dp.cmu_s);
    /// let value = adc.read(Channel::Ch0).unwrap();
    /// ```
    /// Read one single-ended channel on a **named port**, instead of always Port A.
    ///
    /// `read` hard-codes `portpos = 8`, which is Port A, so `Channel::Ch5` always meant PA5 no matter what the caller
    /// wanted; the board's own analogue inputs are elsewhere (the microphone is on PC9). The port encoding is the
    /// PAC's: `8` = Port A, `9` = B, `10` = C, `11` = D (`iadc0_s/single.rs`, "Port A - Select pin number using
    /// PINNEG"), and the negative input stays ground.
    ///
    /// **The analogue bus is not allocated here, and that is deliberate.** Routing a pin to the ADC needs
    /// `GPIO->ABUSALLOC`, whose fields select *analogue buses* (`adc0`, `aodd0`, `aeven0`, ...), not pins: which bus a
    /// given pin lands on is a table in the reference manual, and this project does not have that table. Getting it
    /// wrong does not fail loudly — it reads a different pin's voltage and returns a plausible number — so the honest
    /// state is a function that fixes the port and says the other half is missing, rather than one that guesses an
    /// offset. `microphone.rs` shows the shape of the missing piece: it configures its own input end to end, including
    /// the bus, which is why the microphone works and this driver does not.
    pub fn read_on(&mut self, port: crate::pin::Port, pin: u8) -> Result<u16, Error> {
        // The word comes from `efr32mg2x-adc-core`, where the four fields and their positions are measured on the host
        // (64 inputs, 64 distinct words, every one parseable back). This block only writes it.
        let port_enum = match port {
            crate::pin::Port::A => crate::cores::adc::Port::A,
            crate::pin::Port::B => crate::cores::adc::Port::B,
            crate::pin::Port::C => crate::cores::adc::Port::C,
            crate::pin::Port::D => crate::cores::adc::Port::D,
        };
        let input = match crate::cores::adc::Input::new(port_enum, pin) {
            Ok(input) => input,
            Err(_) => return Err(Error::Timeout), // no such pin: refused before anything is written
        };
        let word = input.single_word();
        critical_section::with(|_cs| {
            // SAFETY: the word is built by the core crate from the fields the PAC documents.
            self.adc.single().write(|w| unsafe { w.bits(word) });
        });
        self.adc.cmd().write(|w| w.singlestart().set_bit());
        const TIMEOUT_CYCLES: u32 = 100_000;
        let mut timeout = 0;
        while self.adc.status().read().singlefifodv().bit_is_clear() {
            timeout += 1;
            if timeout >= TIMEOUT_CYCLES {
                return Err(Error::Timeout);
            }
        }
        Ok(self.adc.singledata().read().data().bits() as u16)
    }

    /// Read one single-ended channel, always on **Port A**.
    ///
    /// Kept as it was for callers that rely on it, with the limitation named: `Channel` carries a pin number and the
    /// port is fixed to Port A by the code below. Use [`Adc::read_on`] to name the port.
    pub fn read(&mut self, channel: Channel) -> Result<u16, Error> {
        // Configure single conversion for the selected channel
        // Use critical section for atomic configuration
        critical_section::with(|_cs| {
            // SAFETY: SINGLE register configuration is type-safe. The portpos/pinpos and
            // portneg/pinneg fields are validated by the Channel enum which only contains
            // hardware-valid values. For single-ended mode, positive = channel, negative = GND.
            self.adc.single().write(|w| unsafe {
                w.portpos()
                    .bits(8) // Port A (value 8)
                    .pinpos()
                    .bits(channel as u8) // Positive pin = channel
                    .portneg()
                    .bits(0) // Ground (single-ended)
                    .pinneg()
                    .bits(0) // Pin 0 (ignored for GND)
            });
        });

        // Start single conversion
        // SAFETY: Writing to CMD register's singlestart bit is a write-only operation
        // that triggers the ADC conversion. This is the standard hardware operation.
        self.adc.cmd().write(|w| w.singlestart().set_bit());

        // Wait for conversion to complete with timeout
        // Timeout prevents infinite loop if hardware fails
        const TIMEOUT_CYCLES: u32 = 100_000; // ~1ms at 78 MHz
        let mut timeout = 0;

        while self.adc.status().read().singlefifodv().bit_is_clear() {
            timeout += 1;
            if timeout >= TIMEOUT_CYCLES {
                return Err(Error::Timeout);
            }
        }

        // Read conversion result
        // SAFETY: Reading from SINGLEFIFODATA is always safe. The hardware ensures
        // valid data is present when SINGLEFIFODV flag is set (checked above).
        let result = self.adc.singlefifodata().read().data().bits();

        // Return 12-bit result (mask to ensure 12-bit range, convert u32 to u16)
        Ok((result & 0x0FFF) as u16)
    }

    /// Get the configured reference voltage source
    pub fn reference(&self) -> Reference {
        self.reference
    }

    /// Disable the ADC peripheral
    ///
    /// This disables the IADC hardware module to save power.
    pub fn disable(self) {
        // SAFETY: Writing to EN register to disable the peripheral is always safe.
        // This is a write-only operation that disables the IADC hardware module.
        self.adc.en().write(|w| w.en().clear_bit());
    }
}
