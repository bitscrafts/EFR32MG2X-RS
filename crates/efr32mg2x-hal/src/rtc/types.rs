//! RTC Configuration Types
//!
//! This module defines types for configuring the Real-Time Clock (SYSRTC0).

/// RTC Error Types
///
/// Errors that can occur during RTC operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Counter value would overflow
    Overflow,

    /// RTC registers are locked
    Locked,

    /// Timeout waiting for synchronization
    Timeout,
}

/// RTC Clock Source
///
/// Selects the clock source for the RTC counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockSource {
    /// Low Frequency Crystal Oscillator (32.768 kHz typical)
    Lfxo,

    /// Low Frequency RC Oscillator (~32 kHz, less accurate)
    Lfrco,
}

/// RTC Time in seconds
///
/// Represents time as a simple seconds counter.
/// For more complex time/date handling, use external crates or implement
/// conversion functions in your application.
pub type Seconds = u32;
