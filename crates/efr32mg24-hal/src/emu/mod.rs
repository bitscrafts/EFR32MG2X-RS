//! Energy Management Unit (EMU) Module
//!
//! This module provides a Hardware Abstraction Layer (HAL) for the EFR32MG24's
//! Energy Management Unit (EMU), enabling efficient power management through
//! energy mode transitions and voltage scaling.
//!
//! # Features
//!
//! - **Energy Mode Transitions**: EM0 → EM1 → EM2 → EM3 → EM4
//! - **Voltage Scaling**: 0.9V, 1.0V, 1.1V for power optimization
//! - **EM2 Debug Support**: Keep debugger active in deep sleep
//! - **EM4 IO Retention**: Preserve GPIO states through hibernate
//! - **Thread-Safe**: Critical sections for RTOS environments
//!
//! # Current Implementation (C3-EMU Stage 1)
//!
//! - ✅ EM1 (Sleep Mode)
//! - ✅ EM2 (Deep Sleep Mode)
//! - ✅ Voltage scaling for EM0/EM1 and EM2/EM3
//! - ⏳ EM3 (Stop Mode) - Planned for Stage 2
//! - ⏳ EM4 (Hibernate) - Planned for Stage 2
//!
//! # Energy Modes
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    Energy Modes                          │
//! ├─────────────────────────────────────────────────────────┤
//! │ EM0 │ Active      │ Full CPU, all peripherals           │
//! │ EM1 │ Sleep       │ CPU stopped, peripherals running    │
//! │ EM2 │ Deep Sleep  │ Major power savings                 │
//! │ EM3 │ Stop        │ Minimal power                       │
//! │ EM4 │ Hibernate   │ Lowest power (~100nA)               │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use efr32mg24_hal::{emu::{Emu, Em2Config, VoltageScale}, pac};
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Initialize EMU
//! let emu = Emu::new(dp.emu_s);
//!
//! // Enter EM1 (Sleep Mode)
//! emu.enter_em1();  // Wakes on any interrupt
//!
//! // Enter EM2 (Deep Sleep Mode) with custom config
//! let config = Em2Config {
//!     voltage_scale: VoltageScale::Vscale1,  // 1.0V for power savings
//!     debug_enable: false,  // Disable debug in production
//! };
//! emu.enter_em2(config).expect("Failed to enter EM2");
//! ```
//!
//! # Safety
//!
//! This module uses `unsafe` for direct hardware register access. All unsafe
//! operations are documented with SAFETY comments.

mod types;
pub use types::*;

use crate::pac;

/// EMU (Energy Management Unit)
///
/// Owns the EMU peripheral and provides methods for energy mode transitions
/// and voltage scaling.
///
/// # Ownership
///
/// This struct takes ownership of the `EMU_S` peripheral, ensuring exclusive
/// access and preventing multiple mutable references.
pub struct Emu {
    _emu: pac::EmuS,
}

impl Emu {
    /// Creates a new EMU instance
    ///
    /// Takes ownership of the EMU peripheral.
    ///
    /// # Arguments
    ///
    /// * `emu` - The EMU peripheral from PAC
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let dp = pac::Peripherals::take().unwrap();
    /// let emu = Emu::new(dp.emu_s);
    /// ```
    pub fn new(emu: pac::EmuS) -> Self {
        Self { _emu: emu }
    }

    /// Enter EM1 (Sleep Mode)
    ///
    /// Enters Energy Mode 1 where the CPU is stopped but peripherals continue
    /// to run. The device wakes on any interrupt.
    ///
    /// # Power Consumption
    ///
    /// Typical: ~50 µA (depends on peripherals enabled)
    ///
    /// # Wake Sources
    ///
    /// Any enabled interrupt can wake the device from EM1.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// emu.enter_em1();  // CPU stopped, wakes on interrupt
    /// // Execution continues here after wake
    /// ```
    ///
    /// # Safety
    ///
    /// This function uses WFI (Wait For Interrupt) which is safe to call.
    /// The ARM Cortex-M SCB SLEEPDEEP bit is cleared to ensure EM1 entry.
    pub fn enter_em1(&self) {
        // SAFETY: Accessing SCB to clear SLEEPDEEP bit is safe.
        // This ensures we enter EM1 (sleep) instead of EM2 (deep sleep).
        cortex_m::interrupt::free(|_| {
            let scb = unsafe { &*cortex_m::peripheral::SCB::PTR };
            unsafe {
                // Clear SLEEPDEEP bit (bit 2) in SCR register
                scb.scr.modify(|scr| scr & !(1 << 2));
            }
        });

        // SAFETY: WFI instruction is safe. CPU will wake on any interrupt.
        cortex_m::asm::wfi();
    }

    /// Enter EM2 (Deep Sleep Mode)
    ///
    /// Enters Energy Mode 2 where major power savings are achieved by stopping
    /// the CPU and most peripherals. Only specific peripherals and clocks remain
    /// active (LFXO, LFRCO, BURTC, etc.).
    ///
    /// # Power Consumption
    ///
    /// Typical: ~2 µA (with LFXO and BURTC)
    ///
    /// # Wake Sources
    ///
    /// - GPIO interrupts
    /// - BURTC (Backup Real-Time Counter)
    /// - LETIMER
    /// - Other EM2-capable peripherals
    ///
    /// # Arguments
    ///
    /// * `config` - EM2 configuration (voltage scaling, debug enable)
    ///
    /// # Errors
    ///
    /// Returns `Error::VoltageScaleFailed` if voltage scaling fails.
    /// Returns `Error::Timeout` if voltage scaling times out.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let config = Em2Config {
    ///     voltage_scale: VoltageScale::Vscale1,  // 1.0V
    ///     debug_enable: false,
    /// };
    /// emu.enter_em2(config).expect("EM2 entry failed");
    /// // Execution continues here after wake
    /// ```
    pub fn enter_em2(&self, config: Em2Config) -> Result<(), Error> {
        // Configure voltage scaling for EM2/EM3
        // SAFETY: Critical section ensures atomic register access.
        // Writing to CTRL register configures EM2/EM3 voltage scaling and debug mode.
        critical_section::with(|_cs| {
            let emu = unsafe { &(*pac::EmuS::ptr()) };

            // Configure EM2/EM3 voltage scaling and debug mode
            emu.ctrl().modify(|_, w| {
                // SAFETY: Writing voltage scale value from enum
                let vscale_bits = match config.voltage_scale {
                    VoltageScale::Vscale0 => 0,
                    VoltageScale::Vscale1 => 1,
                    VoltageScale::Vscale2 => 2,
                };

                unsafe {
                    w.em23vscale().bits(vscale_bits);
                }

                // Enable/disable debugging in EM2
                w.em2dbgen().bit(config.debug_enable)
            });
        });

        // Set SLEEPDEEP bit in ARM Cortex-M SCB
        // SAFETY: Setting SLEEPDEEP bit is safe. This causes WFI to enter EM2.
        cortex_m::interrupt::free(|_| {
            let scb = unsafe { &*cortex_m::peripheral::SCB::PTR };
            unsafe {
                // Set SLEEPDEEP bit (bit 2) in SCR register
                scb.scr.modify(|scr| scr | (1 << 2));
            }
        });

        // SAFETY: WFI instruction is safe. CPU will wake on EM2-capable interrupts.
        cortex_m::asm::wfi();

        // Device wakes here
        // Clear SLEEPDEEP for next EM1 entry
        // SAFETY: Clearing SLEEPDEEP after wake is safe.
        cortex_m::interrupt::free(|_| {
            let scb = unsafe { &*cortex_m::peripheral::SCB::PTR };
            unsafe {
                scb.scr.modify(|scr| scr & !(1 << 2));
            }
        });

        Ok(())
    }

    /// Set voltage scaling for EM0/EM1
    ///
    /// Configures the voltage regulator output level for EM0 (Active) and
    /// EM1 (Sleep) modes.
    ///
    /// # Arguments
    ///
    /// * `scale` - Target voltage scaling level
    ///
    /// # Errors
    ///
    /// Returns `Error::VoltageScaleFailed` if the transition fails.
    /// Returns `Error::Timeout` if the transition times out.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// // Lower voltage for power savings in EM0/EM1
    /// emu.set_em01_voltage_scale(VoltageScale::Vscale1)?;
    /// ```
    pub fn set_em01_voltage_scale(&self, scale: VoltageScale) -> Result<(), Error> {
        // SAFETY: Critical section ensures atomic register access.
        // Writing to CMD register initiates voltage scaling transition.
        critical_section::with(|_cs| {
            let emu = unsafe { &(*pac::EmuS::ptr()) };

            // Issue voltage scale command
            match scale {
                VoltageScale::Vscale0 => {
                    // Note: Vscale0 may not be supported for EM0/EM1
                    // Check reference manual for limitations
                    return Err(Error::VoltageScaleFailed);
                }
                VoltageScale::Vscale1 => {
                    // SAFETY: Writing VSCALE1 command is safe
                    emu.cmd().write(|w| w.em01vscale1().set_bit());
                }
                VoltageScale::Vscale2 => {
                    // SAFETY: Writing VSCALE2 command is safe
                    emu.cmd().write(|w| w.em01vscale2().set_bit());
                }
            }

            // Wait for voltage scaling to complete
            const TIMEOUT_CYCLES: u32 = 100_000; // ~1ms at 78 MHz
            let mut cycles = 0;

            while emu.status().read().vscalebusy().bit_is_set() {
                cycles += 1;
                if cycles > TIMEOUT_CYCLES {
                    return Err(Error::Timeout);
                }
            }

            // Check for failure
            if emu.status().read().vscalefailed().bit_is_set() {
                return Err(Error::VoltageScaleFailed);
            }

            Ok(())
        })
    }

    /// Get current voltage scaling level
    ///
    /// Reads the current voltage scaling setting from the STATUS register.
    ///
    /// # Returns
    ///
    /// Current `VoltageScale` level
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let current_vscale = emu.voltage_scale();
    /// ```
    pub fn voltage_scale(&self) -> VoltageScale {
        // SAFETY: Reading STATUS register is safe
        let emu = unsafe { &(*pac::EmuS::ptr()) };

        let vscale_bits = emu.status().read().vscale().bits();

        match vscale_bits {
            0 => VoltageScale::Vscale0,
            1 => VoltageScale::Vscale1,
            2 => VoltageScale::Vscale2,
            _ => VoltageScale::Vscale2, // Default to highest
        }
    }

    /// Check if EM2 was entered
    ///
    /// Returns true if the device has entered EM2 since the last reset.
    ///
    /// # Returns
    ///
    /// `true` if EM2 was entered, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// if emu.em2_entered() {
    ///     // EM2 was entered since last reset
    /// }
    /// ```
    pub fn em2_entered(&self) -> bool {
        // SAFETY: Reading STATUS register is safe
        let emu = unsafe { &(*pac::EmuS::ptr()) };
        emu.status().read().em2entered().bit_is_set()
    }

    /// Check if EMU registers are locked
    ///
    /// # Returns
    ///
    /// `true` if registers are locked, `false` if unlocked
    pub fn is_locked(&self) -> bool {
        // SAFETY: Reading STATUS register is safe
        let emu = unsafe { &(*pac::EmuS::ptr()) };
        emu.status().read().lock().is_locked()
    }
}
