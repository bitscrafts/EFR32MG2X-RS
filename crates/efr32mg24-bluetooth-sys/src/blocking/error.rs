//! Bluetooth Low Energy error types
//!
//! Represents all possible errors that can occur when interacting with the
//! Bluetooth stack through the BGAPI interface.

use core::fmt;

/// Bluetooth Low Energy error types
///
/// Represents all possible errors that can occur when interacting with the
/// Bluetooth stack through the BGAPI interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BleError {
    /// Raw BGAPI error status code
    ///
    /// When a BGAPI function returns a non-zero status code that doesn't
    /// map to a specific error variant, this variant preserves the raw code.
    ///
    /// Common BGAPI status codes:
    /// - `0x0000`: Success (should not appear in error)
    /// - `0x0002`: Invalid parameter
    /// - `0x0003`: Device in wrong state
    /// - `0x0181`: Out of memory
    /// - `0x0402`: Invalid connection handle
    BgapiError(u16),

    /// Invalid parameter provided to BLE API
    ///
    /// Common causes:
    /// - Device name exceeds 29 characters (GAP AD format limit)
    /// - TX power out of valid range (-127 to +20 dBm)
    /// - Invalid advertising interval (must be >= 20ms)
    /// - Invalid UUID format
    InvalidParameter,

    /// Operation requires active connection but device is not connected
    ///
    /// Occurs when attempting to:
    /// - Send GATT notifications without connection
    /// - Read/write GATT characteristics when disconnected
    NotConnected,

    /// Bluetooth stack is in incorrect state for this operation
    ///
    /// Common causes:
    /// - Attempting to advertise when already advertising
    /// - Attempting to stop advertising when not advertising
    /// - Attempting to connect when already connected
    InvalidState,

    /// Advertising data buffer overflow
    ///
    /// BLE advertising data is limited to 31 bytes (GAP AD format).
    /// This error occurs when attempting to add more data than fits.
    ///
    /// Calculation: flags (3 bytes) + name length + 2 bytes per UUID + manufacturer data
    BufferOverflow,
}

impl BleError {
    /// Convert BGAPI status code to BleError
    ///
    /// Maps common BGAPI error codes to typed error variants.
    /// Unknown codes are wrapped in `BgapiError`.
    pub fn from_bgapi_status(status: u16) -> Self {
        match status {
            0x0002 => BleError::InvalidParameter,
            0x0003 => BleError::InvalidState,
            0x0402 => BleError::NotConnected,
            0x0181 => BleError::BufferOverflow, // Out of memory -> buffer overflow
            _ => BleError::BgapiError(status),
        }
    }
}

impl From<u16> for BleError {
    /// Convert BGAPI status code (u16) to BleError
    ///
    /// This implementation allows using `?` operator with BGAPI FFI calls.
    fn from(status: u16) -> Self {
        BleError::from_bgapi_status(status)
    }
}

impl From<u32> for BleError {
    /// Convert BGAPI status code (u32) to BleError
    ///
    /// BGAPI functions return sl_status_t which is u32. We take the lower 16 bits
    /// which contain the actual error code.
    ///
    /// This implementation allows using `?` operator with BGAPI FFI calls:
    ///
    /// ```no_run
    /// unsafe {
    ///     let status = bt::sl_bt_advertiser_start(...);
    ///     if status != 0 {
    ///         return Err(BleError::from(status));
    ///     }
    /// }
    /// ```
    fn from(status: u32) -> Self {
        BleError::from_bgapi_status((status & 0xFFFF) as u16)
    }
}

impl fmt::Display for BleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BleError::BgapiError(code) => write!(f, "BGAPI error: 0x{:04X}", code),
            BleError::InvalidParameter => write!(f, "Invalid parameter"),
            BleError::NotConnected => write!(f, "Device is not connected"),
            BleError::InvalidState => write!(f, "Invalid Bluetooth state"),
            BleError::BufferOverflow => {
                write!(f, "Advertising data buffer overflow (max 31 bytes)")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BleError {}
