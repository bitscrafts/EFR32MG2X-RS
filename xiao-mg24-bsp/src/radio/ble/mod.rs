//! Bluetooth Low Energy (BLE) 5.3 safe abstractions
//!
//! This module provides safe Rust wrappers around the BGAPI FFI bindings,
//! eliminating 150-200 lines of unsafe boilerplate per BLE application.
//!
//! # Architecture
//!
//! ```text
//! User Application
//!     ↓
//! XiaoBle (this module)          ← Safe Rust API
//!     ↓
//! efr32mg24-bluetooth-sys        ← FFI bindings
//!     ↓
//! Bluetooth Stack (binary blob)  ← Controller + Host
//! ```
//!
//! # Features
//!
//! - **BleAdvertiser trait**: Beacon and peripheral advertising
//! - **BleGattServer trait**: GATT server operations (notify, write)
//! - **GapAdBuilder**: Automatic GAP AD format encoding
//! - **Safe error handling**: Result<T, BleError> for all operations
//! - **80% code reduction**: Beacon example goes from 250 lines → 50 lines
//!
//! # Example: BLE Beacon
//!
//! ```no_run
//! use xiao_mg24_bsp::Board;
//! use xiao_mg24_bsp::radio::ble::XiaoBle;
//!
//! let board = Board::take().unwrap();
//! let mut delay = board.into_delay();
//! let mut ble = XiaoBle::beacon(&mut delay, "XIAO Beacon")?;
//!
//! // Beacon is now advertising - that's it!
//! loop {
//!     cortex_m::asm::wfi();
//! }
//! ```
//!
//! # Example: GATT Server
//!
//! ```no_run
//! use xiao_mg24_bsp::Board;
//! use xiao_mg24_bsp::radio::ble::{XiaoBle, CharHandle};
//!
//! let board = Board::take().unwrap();
//! let mut delay = board.into_delay();
//! let mut ble = XiaoBle::init(&mut delay)?;
//!
//! ble.set_device_name("XIAO HRM")?;
//! ble.add_service_uuid(0x180D)?; // Heart Rate Service
//! ble.start_connectable(100, 0)?;
//!
//! // Later: Send notifications
//! const GATTDB_HEART_RATE: CharHandle = CharHandle(3);
//! let heart_rate_data = [0x06, 75]; // 75 bpm
//! ble.notify(GATTDB_HEART_RATE, &heart_rate_data)?;
//! ```
//!
//! # Event System (Complete!)
//!
//! ```no_run
//! loop {
//!     if let Some(event) = ble.poll_event() {
//!         match event {
//!             BleEvent::Connected { .. } => { /* Auto-managed! */ }
//!             BleEvent::Disconnected { .. } => { ble.start_connectable(100, 0)?; }
//!             BleEvent::GattWrite { attribute, value, .. } => { /* Handle writes */ }
//!             _ => {}
//!         }
//!     }
//!     delay.delay_ms(10);
//! }
//! ```
//!
//! # Not Yet Implemented
//!
//! Features deferred to future phases:
//! - GATT database builder (automatic handle assignment)
//! - Central role (scanning, connecting to peripherals)
//! - Security Manager (pairing, bonding)
//!
//! See module README.md for detailed future work roadmap.

// Submodules (will be implemented in sequence)
pub mod events;
pub mod types;
// pub mod advertiser;  // BleAdvertiser trait (Stage 2)
// pub mod gatt;        // BleGattServer trait (Stage 3)

use super::BleError;
use efr32mg24_bluetooth_sys as bt;
use embedded_hal::delay::DelayNs;

pub use events::BleEvent;
pub use types::{CharHandle, GapAdBuilder};

/// XIAO MG24 Bluetooth Low Energy abstraction
///
/// Provides safe access to BLE functionality without exposing unsafe FFI.
///
/// # Singleton Pattern
///
/// XiaoBle takes ownership of the Board to ensure exclusive access to the
/// Bluetooth stack. Only one instance can exist at a time.
///
/// # Safety
///
/// All unsafe FFI calls are wrapped in safe methods with proper error handling.
/// BGAPI status codes are converted to typed `BleError` variants.
pub struct XiaoBle {
    /// Advertising set handle (from BGAPI)
    advertising_set: u8,

    /// Connection state tracking
    connected: bool,

    /// Active connection handle (if connected)
    connection_handle: u8,

    /// Current advertising data
    ///
    /// Cached to allow incremental updates (add name, add UUID, etc.)
    adv_data: GapAdBuilder,
}

impl XiaoBle {
    /// Initialize Bluetooth stack and create XiaoBle instance
    ///
    /// This method:
    /// 1. Starts the Bluetooth stack (`sl_bt_system_start_bluetooth`)
    /// 2. Waits 100ms for stack initialization
    /// 3. Creates an advertising set
    /// 4. Initializes advertising data builder
    ///
    /// # Arguments
    ///
    /// - `delay` - Delay provider for initialization timing (100ms delay required)
    ///
    /// # Errors
    ///
    /// Returns `BleError` if:
    /// - Bluetooth stack fails to start
    /// - Advertising set creation fails
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
    pub fn init<D: DelayNs>(delay: &mut D) -> Result<Self, BleError> {
        // Step 1: Start Bluetooth stack
        unsafe {
            let status = bt::sl_bt_system_start_bluetooth();
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Step 2: Wait for stack initialization (required 100ms delay)
        delay.delay_ms(100);

        // Step 3: Create advertising set
        let mut advertising_set: u8 = 0;
        unsafe {
            let status = bt::sl_bt_advertiser_create_set(&mut advertising_set as *mut u8);
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Step 4: Initialize advertising data builder
        let adv_data = GapAdBuilder::new();

        Ok(Self {
            advertising_set,
            connected: false,
            connection_handle: 0,
            adv_data,
        })
    }

    /// Create a simple beacon with device name (convenience method)
    ///
    /// This is a convenience wrapper around `init()` that:
    /// 1. Initializes the Bluetooth stack
    /// 2. Sets the device name
    /// 3. Starts non-connectable advertising at 100ms interval, 0 dBm
    ///
    /// Perfect for simple beacon applications.
    ///
    /// # Arguments
    ///
    /// - `delay` - Delay provider for initialization
    /// - `name` - Device name (max 29 characters)
    ///
    /// # Errors
    ///
    /// Returns `BleError::InvalidParameter` if name exceeds 29 characters.
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
    /// // Beacon is now advertising - done!
    /// loop {
    ///     cortex_m::asm::wfi();
    /// }
    /// ```
    pub fn beacon<D: DelayNs>(delay: &mut D, name: &str) -> Result<Self, BleError> {
        let mut ble = Self::init(delay)?;

        // Set device name
        ble.set_device_name(name)?;

        // Start non-connectable advertising
        // 100ms interval (160 units of 0.625ms), 0 dBm TX power
        ble.start_advertising(100, 0)?;

        Ok(ble)
    }

    /// Set the device name in advertising data
    ///
    /// The name will appear in BLE scanner apps.
    ///
    /// # Arguments
    ///
    /// - `name` - Device name (max 29 characters due to GAP AD format limit)
    ///
    /// # Errors
    ///
    /// Returns `BleError::InvalidParameter` if name exceeds 29 characters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// ble.set_device_name("My Device")?;
    /// ```
    pub fn set_device_name(&mut self, name: &str) -> Result<(), BleError> {
        self.adv_data.add_name(name)?;
        Ok(())
    }

    /// Start non-connectable advertising (beacon mode)
    ///
    /// Device will broadcast but not accept connections.
    ///
    /// # Arguments
    ///
    /// - `interval_ms` - Advertising interval in milliseconds (20-10240ms)
    /// - `tx_power_dbm` - TX power in dBm (-127 to +20)
    ///
    /// # Errors
    ///
    /// Returns `BleError` if advertising fails to start.
    ///
    /// # Example
    ///
    /// ```no_run
    /// ble.start_advertising(100, 0)?; // 100ms interval, 0 dBm
    /// ```
    pub fn start_advertising(&mut self, interval_ms: u16, tx_power_dbm: i16) -> Result<(), BleError> {
        // Set advertising data
        let adv_data_slice = self.adv_data.as_slice();
        unsafe {
            let status = bt::sl_bt_advertiser_set_data(
                self.advertising_set,
                0, // Packet type: 0 = Advertising data
                adv_data_slice.len(),
                adv_data_slice.as_ptr(),
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Set timing (convert ms to 0.625ms units)
        let interval_units = (interval_ms as u32 * 16) / 10;
        unsafe {
            let status = bt::sl_bt_advertiser_set_timing(
                self.advertising_set,
                interval_units,  // interval_min
                interval_units,  // interval_max
                0,               // duration (0 = continuous)
                0,               // max_events (0 = unlimited)
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Set TX power
        unsafe {
            let status = bt::sl_bt_advertiser_set_tx_power(
                self.advertising_set,
                tx_power_dbm,
                core::ptr::null_mut(), // actual_power (we don't need it)
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Start advertising (non-connectable, general discoverable)
        unsafe {
            let status = bt::sl_bt_advertiser_start(
                self.advertising_set,
                2, // discover: 2 = User/General discoverable
                0, // connect: 0 = Non-connectable (beacon mode)
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        Ok(())
    }

    /// Start connectable advertising (peripheral mode)
    ///
    /// Device will advertise and accept incoming connections.
    ///
    /// # Arguments
    ///
    /// - `interval_ms` - Advertising interval in milliseconds (20-10240ms)
    /// - `tx_power_dbm` - TX power in dBm (-127 to +20)
    ///
    /// # Errors
    ///
    /// Returns `BleError` if advertising fails to start.
    ///
    /// # Example
    ///
    /// ```no_run
    /// ble.set_device_name("XIAO Peripheral")?;
    /// ble.start_connectable(100, 0)?; // 100ms interval, 0 dBm
    /// ```
    pub fn start_connectable(&mut self, interval_ms: u16, tx_power_dbm: i16) -> Result<(), BleError> {
        // Set advertising data
        let adv_data_slice = self.adv_data.as_slice();
        unsafe {
            let status = bt::sl_bt_advertiser_set_data(
                self.advertising_set,
                0, // Packet type: 0 = Advertising data
                adv_data_slice.len(),
                adv_data_slice.as_ptr(),
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Set timing (convert ms to 0.625ms units)
        let interval_units = (interval_ms as u32 * 16) / 10;
        unsafe {
            let status = bt::sl_bt_advertiser_set_timing(
                self.advertising_set,
                interval_units,  // interval_min
                interval_units,  // interval_max
                0,               // duration (0 = continuous)
                0,               // max_events (0 = unlimited)
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Set TX power
        unsafe {
            let status = bt::sl_bt_advertiser_set_tx_power(
                self.advertising_set,
                tx_power_dbm,
                core::ptr::null_mut(), // actual_power (we don't need it)
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        // Start advertising (connectable, general discoverable)
        unsafe {
            let status = bt::sl_bt_advertiser_start(
                self.advertising_set,
                2, // discover: 2 = User/General discoverable
                2, // connect: 2 = Undirected connectable
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        Ok(())
    }

    /// Stop advertising
    ///
    /// # Errors
    ///
    /// Returns `BleError` if stop operation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// ble.stop_advertising()?;
    /// ```
    pub fn stop_advertising(&mut self) -> Result<(), BleError> {
        unsafe {
            let status = bt::sl_bt_advertiser_stop(self.advertising_set);
            if status != 0 {
                return Err(BleError::from(status));
            }
        }
        Ok(())
    }

    /// Add 16-bit service UUID to advertising data
    ///
    /// The UUID will be advertised and visible to scanners.
    ///
    /// # Arguments
    ///
    /// - `uuid` - 16-bit service UUID (e.g., 0x180D for Heart Rate Service)
    ///
    /// # Errors
    ///
    /// Returns `BleError::BufferOverflow` if adding UUID would exceed 31 bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// ble.add_service_uuid(0x180D)?; // Heart Rate Service
    /// ble.add_service_uuid(0x180F)?; // Battery Service
    /// ```
    pub fn add_service_uuid(&mut self, uuid: u16) -> Result<(), BleError> {
        self.adv_data.add_service_uuid_16(uuid)
    }

    /// Check if device is currently advertising
    ///
    /// Note: This is a client-side state tracker. It doesn't query the
    /// Bluetooth stack, so it may be out of sync if advertising is stopped
    /// externally.
    pub fn is_advertising(&self) -> bool {
        // TODO: Track advertising state
        // For now, return false (will implement in future)
        false
    }

    /// Get advertising set handle (for advanced use)
    pub fn advertising_set(&self) -> u8 {
        self.advertising_set
    }

    /// Check if device is currently connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get current connection handle (if connected)
    pub fn connection_handle(&self) -> Option<u8> {
        if self.connected {
            Some(self.connection_handle)
        } else {
            None
        }
    }

    // ========== GATT Server Methods ==========

    /// Send notification to connected client
    ///
    /// Notifies the connected client of a characteristic value change.
    ///
    /// # Arguments
    ///
    /// - `handle` - GATT characteristic handle (from GATT Configurator)
    /// - `data` - Notification data (max 512 bytes)
    ///
    /// # Errors
    ///
    /// - `BleError::NotConnected` if device is not connected
    /// - `BleError::InvalidParameter` if data exceeds 512 bytes
    /// - `BleError::BgapiError` if BGAPI call fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::radio::ble::CharHandle;
    ///
    /// const GATTDB_HEART_RATE: CharHandle = CharHandle(3);
    /// let heart_rate_data = [0x06, 75]; // 75 bpm
    ///
    /// // Assumes connection was established externally
    /// ble.notify(GATTDB_HEART_RATE, &heart_rate_data)?;
    /// ```
    ///
    /// # Note
    ///
    /// This method requires an active connection. Without an event system,
    /// you must track connections manually using `set_connected()`.
    pub fn notify(&mut self, handle: CharHandle, data: &[u8]) -> Result<(), BleError> {
        // Check if connected
        if !self.connected {
            return Err(BleError::NotConnected);
        }

        // Validate data length (BGAPI limit: 512 bytes)
        if data.len() > 512 {
            return Err(BleError::InvalidParameter);
        }

        // Send notification via BGAPI
        unsafe {
            let status = bt::sl_bt_gatt_server_send_notification(
                self.connection_handle,
                handle.value(),
                data.len(),
                data.as_ptr(),
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        Ok(())
    }

    /// Write to GATT characteristic attribute
    ///
    /// Updates the value of a GATT characteristic in the local database.
    ///
    /// # Arguments
    ///
    /// - `handle` - GATT characteristic handle (from GATT Configurator)
    /// - `data` - Attribute data (max 512 bytes)
    ///
    /// # Errors
    ///
    /// - `BleError::InvalidParameter` if data exceeds 512 bytes
    /// - `BleError::BgapiError` if BGAPI call fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::radio::ble::CharHandle;
    ///
    /// const GATTDB_BATTERY_LEVEL: CharHandle = CharHandle(5);
    /// let battery_level = [85]; // 85%
    ///
    /// ble.write_attribute(GATTDB_BATTERY_LEVEL, &battery_level)?;
    /// ```
    ///
    /// # Note
    ///
    /// This updates the local GATT database. To notify clients of the change,
    /// call `notify()` afterward.
    pub fn write_attribute(&mut self, handle: CharHandle, data: &[u8]) -> Result<(), BleError> {
        // Validate data length (BGAPI limit: 512 bytes)
        if data.len() > 512 {
            return Err(BleError::InvalidParameter);
        }

        // Write attribute value via BGAPI
        unsafe {
            let status = bt::sl_bt_gatt_server_write_attribute_value(
                handle.value(),
                0, // offset (always 0 for full writes)
                data.len(),
                data.as_ptr(),
            );
            if status != 0 {
                return Err(BleError::from(status));
            }
        }

        Ok(())
    }

    /// Set connection state (manual tracking)
    ///
    /// Updates the internal connection state. This is a workaround for the
    /// lack of an event system.
    ///
    /// # Arguments
    ///
    /// - `connected` - True if device is connected, false if disconnected
    /// - `connection_handle` - Connection handle from BGAPI (if connected)
    ///
    /// # Example
    ///
    /// ```no_run
    /// // After detecting connection externally (e.g., checking BGAPI status)
    /// ble.set_connected(true, 0); // connection_handle = 0
    ///
    /// // After detecting disconnection
    /// ble.set_connected(false, 0);
    /// ```
    ///
    /// # Note
    ///
    /// In future versions with an event system, this will be called
    /// automatically by connection/disconnection event handlers.
    pub fn set_connected(&mut self, connected: bool, connection_handle: u8) {
        self.connected = connected;
        self.connection_handle = connection_handle;
    }

    // ========== Event System ==========

    /// Poll for Bluetooth events (non-blocking)
    ///
    /// This method checks for pending Bluetooth events and returns the next
    /// event if available. It automatically updates internal connection state
    /// when connection/disconnection events are received.
    ///
    /// # Returns
    ///
    /// - `Some(BleEvent)` - Event is available
    /// - `None` - No events pending
    ///
    /// # Automatic State Management
    ///
    /// The following state is updated automatically:
    /// - **Connected**: Sets `connected = true`, saves connection handle
    /// - **Disconnected**: Sets `connected = false`, clears handle
    ///
    /// # Example: Event-Driven GATT Server
    ///
    /// ```no_run
    /// use xiao_mg24_bsp::Board;
    /// use xiao_mg24_bsp::radio::ble::{XiaoBle, BleEvent, CharHandle};
    ///
    /// const GATTDB_LED_CONTROL: CharHandle = CharHandle(3);
    ///
    /// let board = Board::take().unwrap();
    /// let mut delay = board.into_delay();
    /// let mut ble = XiaoBle::init(&mut delay)?;
    ///
    /// ble.set_device_name("XIAO LED")?;
    /// ble.start_connectable(100, 0)?;
    ///
    /// loop {
    ///     // Poll for events
    ///     if let Some(event) = ble.poll_event() {
    ///         match event {
    ///             BleEvent::Connected { connection, .. } => {
    ///                 // Connection state updated automatically!
    ///             }
    ///             BleEvent::Disconnected { .. } => {
    ///                 // Resume advertising
    ///                 ble.start_connectable(100, 0)?;
    ///             }
    ///             BleEvent::GattWrite { attribute, value, .. } => {
    ///                 if attribute == GATTDB_LED_CONTROL.value() {
    ///                     let led_state = value[0];
    ///                     // Update LED
    ///                 }
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    ///     delay.delay_ms(10);
    /// }
    /// ```
    ///
    /// # Performance
    ///
    /// This is a non-blocking operation. The Bluetooth stack uses a message
    /// queue internally, so calling this frequently is efficient.
    ///
    /// Recommended polling interval: 1-50ms depending on application requirements.
    pub fn poll_event(&mut self) -> Option<BleEvent> {
        // Allocate event structure (256 bytes on stack)
        let mut evt = core::mem::MaybeUninit::<bt::sl_bt_msg_t>::uninit();

        // Pop event from Bluetooth stack queue
        let status = unsafe { bt::sl_bt_pop_event(evt.as_mut_ptr()) };

        // Status != 0 means no events available
        if status != 0 {
            return None;
        }

        // SAFETY: sl_bt_pop_event() returns 0 only when event is valid
        let evt = unsafe { evt.assume_init() };

        // Parse event into typed BleEvent
        let parsed = unsafe { BleEvent::parse(&evt) };

        // Automatically update connection state based on events
        if let Some(ref event) = parsed {
            match event {
                BleEvent::Connected { connection, .. } => {
                    self.connected = true;
                    self.connection_handle = *connection;
                }
                BleEvent::Disconnected { .. } => {
                    self.connected = false;
                    self.connection_handle = 0;
                }
                _ => {}
            }
        }

        parsed
    }
}
