//! WDOG Type Definitions
//!
//! This module defines types used by the WDOG (Watchdog Timer) module.

/// WDOG Error Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Configuration is locked, cannot modify
    Locked,
    /// Invalid timeout period value
    InvalidPeriod,
    /// Watchdog already enabled
    AlreadyEnabled,
}

/// A snapshot of the watchdog's registers.
///
/// It exists because the gate refuses a PAC type named outside the PAC crate, and an example that has to say *what the
/// block was doing* when it refused a write has no other door. The layout is the RM's: `IPVERSION`, `EN`, `CFG`,
/// `STATUS` (bit 31 is `LOCK`), `SYNCBUSY` (bit 0 is `CMD`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    /// `WDOG_IPVERSION`, 1 on this part.
    pub ipversion: u32,
    /// `WDOG_EN`: bit 0 enables, bit 1 reports a disable in progress.
    pub en: u32,
    /// `WDOG_CFG`: `PERSEL` in bits 19:16, `DEBUGRUN` in bit 5.
    pub cfg: u32,
    /// `WDOG_STATUS`: bit 31 is the configuration lock.
    pub status: u32,
    /// `WDOG_SYNCBUSY`: bit 0 is the `CMD` synchroniser.
    pub syncbusy: u32,
}

impl State {
    /// Whether the configuration is locked, which is the state a configured watchdog must be in.
    pub fn is_locked(&self) -> bool {
        self.status & (1 << 31) != 0
    }

    /// Whether a `CMD` write is still crossing into the watchdog's clock domain.
    pub fn cmd_sync_busy(&self) -> bool {
        self.syncbusy & 1 != 0
    }
}

/// Watchdog Timeout Period
///
/// Defines the timeout period in watchdog clock cycles (typically 1 kHz from LFRCO/LFXO).
/// With 1 kHz clock:
/// - Sel0: 9ms
/// - Sel15: 256 seconds (4.3 minutes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeoutPeriod {
    /// 9 cycles (~9ms at 1kHz)
    Cycles9 = 0,
    /// 17 cycles (~17ms at 1kHz)
    Cycles17 = 1,
    /// 33 cycles (~33ms at 1kHz)
    Cycles33 = 2,
    /// 65 cycles (~65ms at 1kHz)
    Cycles65 = 3,
    /// 129 cycles (~129ms at 1kHz)
    Cycles129 = 4,
    /// 257 cycles (~257ms at 1kHz)
    Cycles257 = 5,
    /// 513 cycles (~513ms at 1kHz)
    Cycles513 = 6,
    /// 1024 cycles (~1s at 1kHz)
    Cycles1k = 7,
    /// 2048 cycles (~2s at 1kHz)
    Cycles2k = 8,
    /// 4096 cycles (~4s at 1kHz)
    Cycles4k = 9,
    /// 8192 cycles (~8s at 1kHz)
    Cycles8k = 10,
    /// 16384 cycles (~16s at 1kHz)
    Cycles16k = 11,
    /// 32768 cycles (~32s at 1kHz)
    Cycles32k = 12,
    /// 65536 cycles (~65s at 1kHz)
    Cycles64k = 13,
    /// 131072 cycles (~131s at 1kHz)
    Cycles128k = 14,
    /// 262144 cycles (~262s / 4.3min at 1kHz)
    Cycles256k = 15,
}

impl TimeoutPeriod {
    /// Convert to hardware register value
    pub(crate) fn to_bits(self) -> u8 {
        self as u8
    }

    /// Get timeout in milliseconds (assuming 1 kHz watchdog clock)
    pub fn timeout_ms(&self) -> u32 {
        match self {
            Self::Cycles9 => 9,
            Self::Cycles17 => 17,
            Self::Cycles33 => 33,
            Self::Cycles65 => 65,
            Self::Cycles129 => 129,
            Self::Cycles257 => 257,
            Self::Cycles513 => 513,
            Self::Cycles1k => 1024,
            Self::Cycles2k => 2048,
            Self::Cycles4k => 4096,
            Self::Cycles8k => 8192,
            Self::Cycles16k => 16384,
            Self::Cycles32k => 32768,
            Self::Cycles64k => 65536,
            Self::Cycles128k => 131072,
            Self::Cycles256k => 262144,
        }
    }
}

/// Watchdog Configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WdogConfig {
    /// Timeout period
    pub period: TimeoutPeriod,

    /// Enable watchdog in EM1 (sleep mode)
    pub em1_run: bool,

    /// Enable watchdog in EM2 (deep sleep mode)
    pub em2_run: bool,

    /// Enable watchdog in EM3 (stop mode)
    pub em3_run: bool,

    /// Enable watchdog in debug mode (useful for development)
    pub debug_run: bool,

    /// Disable reset on watchdog timeout (warning only mode)
    pub reset_disable: bool,
}

impl Default for WdogConfig {
    /// Default configuration: 2 second timeout, runs in EM0 only, resets on timeout
    fn default() -> Self {
        Self {
            period: TimeoutPeriod::Cycles2k, // ~2 seconds
            em1_run: false,                  // Stop in EM1 (safe default)
            em2_run: false,                  // Stop in EM2 (safe default)
            em3_run: false,                  // Stop in EM3 (safe default)
            debug_run: false,                // Freeze in debug (development-friendly)
            reset_disable: false,            // Enable reset (watchdog behavior)
        }
    }
}

impl WdogConfig {
    /// Create a new configuration with specified timeout period
    pub fn new(period: TimeoutPeriod) -> Self {
        Self {
            period,
            ..Default::default()
        }
    }

    /// Enable watchdog to run in all energy modes
    pub fn run_in_all_modes(mut self) -> Self {
        self.em1_run = true;
        self.em2_run = true;
        self.em3_run = true;
        self
    }

    /// Enable debug mode (watchdog runs even when debugger is attached)
    pub fn debug_run(mut self, enable: bool) -> Self {
        self.debug_run = enable;
        self
    }

    /// Disable reset on timeout (warning-only mode for testing)
    pub fn warning_only(mut self) -> Self {
        self.reset_disable = true;
        self
    }
}
