//! EMU Configuration Types
//!
//! This module defines types for configuring the Energy Management Unit (EMU).

/// Energy Mode Selection
///
/// Defines the five energy modes available on the EFR32MG24.
///
/// # Energy Modes
///
/// - **EM0 (Active)**: Full functionality, CPU running, all peripherals available
/// - **EM1 (Sleep)**: CPU stopped, peripherals running, wake on any interrupt
/// - **EM2 (Deep Sleep)**: Major power savings, limited peripherals, wake on specific interrupts
/// - **EM3 (Stop)**: Minimal power, very limited functionality, wake on reset or limited sources
/// - **EM4 (Hibernate/Shutoff)**: Lowest power (~100nA), requires reset to wake
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyMode {
    /// EM0: Active mode (no entry function needed, this is the default state)
    EM0,
    /// EM1: Sleep mode (CPU stopped, peripherals running)
    EM1,
    /// EM2: Deep sleep mode (major power savings)
    EM2,
    /// EM3: Stop mode (minimal power)
    EM3,
    /// EM4: Hibernate/Shutoff mode (lowest power, reset required)
    EM4,
}

/// Voltage Scaling Levels
///
/// Configures the voltage regulator output level for power optimization.
///
/// # Voltage Levels
///
/// - **Vscale0**: 0.9V (lowest power, reduced frequency capability)
/// - **Vscale1**: 1.0V (medium power)
/// - **Vscale2**: 1.1V (highest power, full performance, default)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VoltageScale {
    /// VSCALE0: 0.9V (lowest power)
    Vscale0,
    /// VSCALE1: 1.0V (medium power)
    Vscale1,
    /// VSCALE2: 1.1V (highest power, default)
    #[default]
    Vscale2,
}

/// EM2 Configuration
///
/// Configuration options for entering EM2 (Deep Sleep) mode.
#[derive(Debug, Clone, Copy)]
pub struct Em2Config {
    /// Voltage scaling level for EM2/EM3
    ///
    /// Lower voltage = lower power consumption but may limit peripheral functionality.
    pub voltage_scale: VoltageScale,

    /// Enable debugging in EM2
    ///
    /// When true, the debug interface remains active in EM2.
    /// Set to false in production for lowest power consumption.
    pub debug_enable: bool,
}

impl Default for Em2Config {
    fn default() -> Self {
        Self {
            voltage_scale: VoltageScale::Vscale2,
            debug_enable: false, // Disabled for production
        }
    }
}

/// EM4 Configuration
///
/// Configuration options for entering EM4 (Hibernate/Shutoff) mode.
#[derive(Debug, Clone, Copy)]
pub struct Em4Config {
    /// IO retention mode during EM4
    pub io_retention_mode: IoRetentionMode,

    /// Enable BOD3SENSE as EM4 wakeup source
    ///
    /// When true, BOD3SENSE can wake the device from EM4.
    pub bod3sense_wakeup: bool,
}

impl Default for Em4Config {
    fn default() -> Self {
        Self {
            io_retention_mode: IoRetentionMode::Disable,
            bod3sense_wakeup: false,
        }
    }
}

/// IO Retention Mode for EM4
///
/// Defines how GPIO pin states are retained during EM4 mode.
///
/// # Modes
///
/// - **Disable**: Pads enter reset state on EM4 entry
/// - **Em4Exit**: Pads retained through EM4, reset on EM4 exit
/// - **SwUnlatch**: Pads retained through EM4 and wakeup, software controlled unlatch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoRetentionMode {
    /// No retention: pads enter reset state on EM4 entry
    Disable,
    /// Retention through EM4: pads reset on EM4 exit
    Em4Exit,
    /// Retention through EM4 and wakeup: software controlled unlatch
    SwUnlatch,
}

/// EMU Error Types
///
/// Errors that can occur during EMU operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Voltage scaling operation failed
    ///
    /// The voltage scaling transition did not complete successfully.
    /// Check STATUS.VSCALEFAILED for details.
    VoltageScaleFailed,

    /// EMU registers are locked
    ///
    /// The EMU peripheral is locked and cannot be modified.
    /// Unlock via the LOCK register before attempting configuration.
    Locked,

    /// Timeout waiting for voltage scaling
    ///
    /// Voltage scaling did not complete within the expected time.
    Timeout,
}
