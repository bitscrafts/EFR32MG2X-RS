//! XIAO MG24 BLE Convenience Wrapper
//!
//! Thin wrapper around `efr32mg24_bluetooth_sys::blocking::Bluetooth` with
//! XIAO-specific defaults.
//!
//! # Architecture
//!
//! This module provides a board-specific convenience layer:
//! - Generic BLE code → `efr32mg24_bluetooth_sys` (reusable across all EFR32MG24)
//! - Board-specific defaults → This module (XIAO MG24 Sense only)
//!
//! # Example
//!
//! ```no_run
//! use xiao_mg24_bsp::Board;
//! use xiao_mg24_bsp::radio::ble::XiaoBle;
//!
//! let board = Board::take().unwrap();
//! let mut delay = board.into_delay();
//!
//! // XIAO-specific wrapper with defaults
//! let mut ble = XiaoBle::init(&mut delay)?;
//! // or use bluetooth-sys directly:
//! // let mut ble = Bluetooth::new(&mut delay)?;
//! ```

// Re-export all bluetooth-sys types for convenience
pub use efr32mg24_bluetooth_sys::blocking::*;

use efr32mg24_bluetooth_sys::blocking::Bluetooth;
use embedded_hal::delay::DelayNs;

// Re-export submodules (if any board-specific BLE utilities are added in future)
// (Currently none - all generic code moved to bluetooth-sys)

/// XIAO MG24 BLE convenience wrapper
///
/// Thin wrapper around `Bluetooth` with XIAO-specific defaults.
///
/// For most applications, you can use `Bluetooth` directly from
/// `efr32mg24_bluetooth_sys::blocking::Bluetooth` instead.
pub struct XiaoBle {
    bluetooth: Bluetooth,
}

impl XiaoBle {
    /// Initialize BLE stack with XIAO defaults
    ///
    /// Convenience wrapper that initializes Bluetooth and sets XIAO-specific
    /// device name ("XIAO MG24").
    ///
    /// # Arguments
    ///
    /// - `delay` - Delay provider for initialization (100ms delay required)
    ///
    /// # Errors
    ///
    /// Returns `BleError` if Bluetooth stack fails to initialize.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::Board;
    /// use xiao_mg24_bsp::radio::ble::XiaoBle;
    ///
    /// let board = Board::take().unwrap();
    /// let mut delay = board.into_delay();
    /// let mut ble = XiaoBle::init(&mut delay)?;
    /// ```
    pub fn init(delay: &mut impl DelayNs) -> Result<Self, BleError> {
        let mut bluetooth = Bluetooth::new(delay)?;

        // XIAO-specific default device name
        bluetooth.set_device_name("XIAO MG24")?;

        Ok(Self { bluetooth })
    }

    /// Create BLE beacon with XIAO defaults
    ///
    /// Convenience wrapper around `Bluetooth::beacon()`.
    ///
    /// # Arguments
    ///
    /// - `delay` - Delay provider for initialization
    /// - `name` - Device name (max 29 characters)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::Board;
    /// use xiao_mg24_bsp::radio::ble::XiaoBle;
    ///
    /// let board = Board::take().unwrap();
    /// let mut delay = board.into_delay();
    /// let ble = XiaoBle::beacon(&mut delay, "XIAO Beacon")?;
    ///
    /// loop {
    ///     cortex_m::asm::wfi();
    /// }
    /// ```
    pub fn beacon(delay: &mut impl DelayNs, name: &str) -> Result<Self, BleError> {
        let bluetooth = Bluetooth::beacon(delay, name)?;
        Ok(Self { bluetooth })
    }

    // ========== Forward all methods to inner Bluetooth ==========

    /// Set the device name in advertising data
    pub fn set_device_name(&mut self, name: &str) -> Result<(), BleError> {
        self.bluetooth.set_device_name(name)
    }

    /// Add 16-bit service UUID to advertising data
    pub fn add_service_uuid(&mut self, uuid: u16) -> Result<(), BleError> {
        self.bluetooth.add_service_uuid(uuid)
    }

    /// Start non-connectable advertising (beacon mode)
    pub fn start_advertising(&mut self, interval_ms: u16, tx_power_dbm: i16) -> Result<(), BleError> {
        self.bluetooth.start_advertising(interval_ms, tx_power_dbm)
    }

    /// Start connectable advertising (peripheral mode)
    pub fn start_connectable(&mut self, interval_ms: u16, tx_power_dbm: i16) -> Result<(), BleError> {
        self.bluetooth.start_connectable(interval_ms, tx_power_dbm)
    }

    /// Stop advertising
    pub fn stop_advertising(&mut self) -> Result<(), BleError> {
        self.bluetooth.stop_advertising()
    }

    /// Check if device is currently advertising
    pub fn is_advertising(&self) -> bool {
        self.bluetooth.is_advertising()
    }

    /// Get advertising set handle (for advanced use)
    pub fn advertising_set(&self) -> u8 {
        self.bluetooth.advertising_set()
    }

    /// Check if device is currently connected
    pub fn is_connected(&self) -> bool {
        self.bluetooth.is_connected()
    }

    /// Get current connection handle (if connected)
    pub fn connection_handle(&self) -> Option<u8> {
        self.bluetooth.connection_handle()
    }

    /// Send notification to connected client
    pub fn notify(&mut self, characteristic: CharHandle, data: &[u8]) -> Result<(), BleError> {
        self.bluetooth.notify(characteristic, data)
    }

    /// Write to GATT characteristic attribute
    pub fn write_attribute(&mut self, attribute: CharHandle, value: &[u8]) -> Result<(), BleError> {
        self.bluetooth.write_attribute(attribute, value)
    }

    /// Set connection state (manual tracking)
    pub fn set_connected(&mut self, connected: bool, connection_handle: u8) {
        self.bluetooth.set_connected(connected, connection_handle)
    }

    /// Poll for Bluetooth events (non-blocking)
    pub fn poll_event(&mut self) -> Option<BleEvent> {
        self.bluetooth.poll_event()
    }
}
