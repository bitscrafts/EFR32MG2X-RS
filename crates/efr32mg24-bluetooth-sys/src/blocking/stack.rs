//! Bluetooth Stack Implementation
//!
//! Safe wrapper around Silicon Labs BGAPI for Bluetooth Low Energy operations.

use super::error::BleError;
use super::events::BleEvent;
use super::types::{CharHandle, GapAdBuilder};
use crate::ffi as bt;
use core::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
use embedded_hal::delay::DelayNs;

/// The header of the most recent event popped from the stack's queue, for diagnosis.
///
/// The event id is `header & 0xffff00f8` (`SL_BGAPI_MSG_ID`), which is the number to compare against the `*_id`
/// constants in `sl_bt_api.h`. Reading it is how "the device heard nothing" was separated from "the device heard
/// everything and this parser could not name it": measured on the board, two seconds of scanning produced 39 events
/// and zero recognised advertisements, and the header said which event they were.
pub static LAST_EVENT_HEADER: AtomicU32 = AtomicU32::new(0);

/// Read [`LAST_EVENT_HEADER`].
pub fn last_event_header() -> u32 {
    LAST_EVENT_HEADER.load(AtomicOrdering::Relaxed)
}

/// Bluetooth Low Energy stack
///
/// Safe wrapper around Silicon Labs BGAPI providing:
/// - Advertising (connectable and non-connectable)
/// - GATT server operations
/// - Event-driven connection management
///
/// # Example
///
/// ```no_run
/// use efr32mg24_bluetooth_sys::blocking::{Bluetooth, BleEvent};
///
/// let mut ble = Bluetooth::new(&mut delay)?;
/// ble.set_device_name("My Device")?;
/// ble.start_connectable(100, 0)?;
///
/// loop {
///     if let Some(event) = ble.poll_event() {
///         match event {
///             BleEvent::Connected { .. } => { /* handle connection */ }
///             BleEvent::GattWrite { attribute, value, .. } => { /* handle write */ }
///             _ => {}
///         }
///     }
///     delay.delay_ms(10);
/// }
/// ```
pub struct Bluetooth {
    /// Advertising set handle (from BGAPI)
    advertising_set: u8,

    /// Connection state tracking
    connected: bool,

    /// Whether this device started a scan and has not stopped it.
    scanning: bool,

    /// Active connection handle (if connected)
    connection_handle: u8,

    /// Current advertising data
    ///
    /// Cached to allow incremental updates (add name, add UUID, etc.)
    adv_data: GapAdBuilder,
}

impl Bluetooth {
    /// Initialize Bluetooth stack and create Bluetooth instance
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
    /// use efr32mg24_bluetooth_sys::blocking::Bluetooth;
    ///
    /// let mut ble = Bluetooth::new(&mut delay)?;
    /// ```
    pub fn new<D: DelayNs>(delay: &mut D) -> Result<Self, BleError> {
        // Step 1: start the host through the entry points this build actually has.
        //
        // `sl_bt_system_start_bluetooth()` is the obvious call and the wrong one here: the
        // "on-demand start" component is not in this build, so it returns
        // SL_STATUS_NOT_AVAILABLE (0x000E). Measured on the board; see `stack_init.rs`.
        let status =
            unsafe { crate::stack_init::start(crate::stack_init::default_configuration()) };
        if status != 0 {
            return Err(BleError::from(status));
        }

        // Step 2: the stack emits `system_boot` when it is ready; give it time and then poll.
        delay.delay_ms(100);
        let mut waited_ms = 0u32;
        loop {
            // **The host has to be run, or this loop can never end.** `system_boot` is queued during the start but
            // only delivered once the host processes its own events; polling `sl_bt_pop_event` without `sl_bt_run`
            // waits out the full two seconds and reports "boot event never arrived", which reads like a radio problem
            // and is not one. Measured on the board.
            crate::stack_init::pump();
            let mut evt = core::mem::MaybeUninit::<bt::sl_bt_msg_t>::uninit();
            let popped = unsafe { bt::sl_bt_pop_event(evt.as_mut_ptr()) };
            if popped == 0 {
                let evt = unsafe { evt.assume_init() };
                // `SL_BGAPI_MSG_ID` masks with 0xffff00f8; the id is not the whole header.
                if evt.header & 0xffff_00f8 == bt::sl_bt_evt_system_boot_id {
                    break;
                }
            }
            if waited_ms > 2_000 {
                return Err(BleError::BgapiError(0xB007)); // boot event never arrived
            }
            delay.delay_ms(10);
            waited_ms += 10;
        }

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
            scanning: false,
            connection_handle: 0,
            adv_data,
        })
    }

    /// Create a simple beacon with device name (convenience method)
    ///
    /// This is a convenience wrapper around `new()` that:
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
    /// use efr32mg24_bluetooth_sys::blocking::Bluetooth;
    ///
    /// let ble = Bluetooth::beacon(&mut delay, "My Beacon")?;
    ///
    /// // Beacon is now advertising - done!
    /// loop {
    ///     cortex_m::asm::wfi();
    /// }
    /// ```
    pub fn beacon<D: DelayNs>(delay: &mut D, name: &str) -> Result<Self, BleError> {
        let mut ble = Self::new(delay)?;

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
    pub fn start_advertising(
        &mut self,
        interval_ms: u16,
        tx_power_dbm: i16,
    ) -> Result<(), BleError> {
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
                interval_units, // interval_min
                interval_units, // interval_max
                0,              // duration (0 = continuous)
                0,              // max_events (0 = unlimited)
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
                // sl_bt_advertiser_user_data (0x4): send the data set above as it is. In general_discoverable (0x2) the stack
                // builds the advertising PDU itself -- flags only -- and the set_data payload, name included, never reaches
                // the air (measured: the first FIFO write was 3 bytes of data, flags, with the name set). The builder
                // already carries the flags field, so nothing is lost.
                4,
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
    /// ble.set_device_name("My Peripheral")?;
    /// ble.start_connectable(100, 0)?; // 100ms interval, 0 dBm
    /// ```
    pub fn start_connectable(
        &mut self,
        interval_ms: u16,
        tx_power_dbm: i16,
    ) -> Result<(), BleError> {
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
                interval_units, // interval_min
                interval_units, // interval_max
                0,              // duration (0 = continuous)
                0,              // max_events (0 = unlimited)
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
                // sl_bt_advertiser_user_data (0x4): send the data set above as it is. In general_discoverable (0x2) the stack
                // builds the advertising PDU itself -- flags only -- and the set_data payload, name included, never reaches
                // the air (measured: the first FIFO write was 3 bytes of data, flags, with the name set). The builder
                // already carries the flags field, so nothing is lost.
                4,
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

    /// Start scanning for advertisers (central role).
    ///
    /// # Arguments
    ///
    /// - `interval_ms` — how often a scan starts; the radio cycles the three advertising channels
    /// - `window_ms` — how long each scan listens; must not exceed `interval_ms`, or the stack
    ///   refuses the command and returns `BleError::InvalidParameter`
    /// - `active` — active scanning also sends a scan request and receives a scan response, which is
    ///   how a name carried in the scan response (rather than the advertisement) becomes visible
    ///
    /// # The units are not milliseconds
    ///
    /// BGAPI takes 0.625 ms units, so both values are multiplied here. That conversion is the whole
    /// reason this wrapper exists: passing 100 for "100 ms" asks the radio for 62.5 ms, and the
    /// scanner works — it just scans less than you asked for, which is invisible until you compare
    /// against a sniffer.
    ///
    /// # Errors
    ///
    /// `BleError::InvalidParameter` when `window_ms > interval_ms`, or when either is out of range.
    pub fn start_scan(
        &mut self,
        interval_ms: u16,
        window_ms: u16,
        active: bool,
    ) -> Result<(), BleError> {
        if window_ms > interval_ms || interval_ms < 3 || window_ms < 3 {
            return Err(BleError::InvalidParameter);
        }
        // 0.625 ms units. 20 ms of interval is 32 units; rounding down by one unit is 0.6 ms and
        // does not matter, so integer division is honest here.
        let interval = interval_ms * 8 / 5;
        let window = window_ms * 8 / 5;

        let mode: u8 = if active {
            bt::sl_bt_scanner_scan_mode_t_sl_bt_scanner_scan_mode_active as u8
        } else {
            bt::sl_bt_scanner_scan_mode_t_sl_bt_scanner_scan_mode_passive as u8
        };

        let status = unsafe {
            bt::sl_bt_scanner_set_mode(
                bt::sl_bt_scanner_scan_phy_t_sl_bt_scanner_scan_phy_1m as u8,
                mode,
            )
        };
        if status != 0 {
            return Err(BleError::from(status));
        }
        let status = unsafe {
            bt::sl_bt_scanner_set_timing(
                bt::sl_bt_scanner_scan_phy_t_sl_bt_scanner_scan_phy_1m as u8,
                interval,
                window,
            )
        };
        if status != 0 {
            return Err(BleError::from(status));
        }
        // Discover-observation mode reports everything, including beacons that are not discoverable —
        // which is exactly what a demo scanner should see, since the board's own beacon is one.
        let status = unsafe {
            bt::sl_bt_scanner_start(
                bt::sl_bt_scanner_scan_phy_t_sl_bt_scanner_scan_phy_1m as u8,
                bt::sl_bt_scanner_discover_mode_t_sl_bt_scanner_discover_observation as u8,
            )
        };
        if status != 0 {
            return Err(BleError::from(status));
        }
        self.scanning = true;
        Ok(())
    }

    /// Stop scanning. Stopping a scanner that is not running is not an error.
    pub fn stop_scan(&mut self) -> Result<(), BleError> {
        let status = unsafe { bt::sl_bt_scanner_stop() };
        self.scanning = false;
        if status != 0 {
            return Err(BleError::from(status));
        }
        Ok(())
    }

    /// Whether this device is currently scanning.
    #[must_use]
    pub fn is_scanning(&self) -> bool {
        self.scanning
    }

    /// Add manufacturer-specific data to the next advertising payload.
    ///
    /// The AD structure is `[len][0xFF][company_id_lo][company_id_hi][data...]`, and it shares the
    /// same 31 bytes as everything else — so adding it may make the payload too large, which the
    /// builder refuses instead of truncating.
    ///
    /// # Errors
    ///
    /// `BleError::InvalidParameter` when the company id is the reserved `0xFFFF`, or when the data
    /// does not fit in the remaining advertising space.
    pub fn set_manufacturer_data(&mut self, company_id: u16, data: &[u8]) -> Result<(), BleError> {
        if company_id == 0xFFFF {
            return Err(BleError::InvalidParameter);
        }
        self.adv_data.replace_manufacturer_data(company_id, data)
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
    /// use efr32mg24_bluetooth_sys::blocking::CharHandle;
    ///
    /// const GATTDB_HEART_RATE: CharHandle = CharHandle(3);
    /// let heart_rate_data = [0x06, 75]; // 75 bpm
    ///
    /// // Assumes connection was established
    /// ble.notify(GATTDB_HEART_RATE, &heart_rate_data)?;
    /// ```
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
    /// use efr32mg24_bluetooth_sys::blocking::CharHandle;
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

    /// Make this device pairable and bondable, with LE Secure Connections only.
    ///
    /// The board has no display and no keyboard, so the pairing method is Just Works: the client is authenticated by
    /// the ECDH exchange and not by a passkey. `SC_ONLY` refuses legacy pairing, which is the one that can be broken
    /// with a sniffer; a product that cannot do Secure Connections should fail loudly rather than fall back.
    ///
    /// `BONDING_REQUIRED` makes the link key persist, and the builtin bonding database stores it in NVM3, so a paired
    /// peer is remembered across a reboot.
    pub fn enable_pairing(&mut self) -> Result<(), BleError> {
        /// `SL_BT_SM_CONFIGURATION_SC_ONLY`: refuse legacy pairing, accept what the peer asks for otherwise.
        ///
        /// `BONDING_REQUIRED` (0x02) is deliberately **not** set. With it, a peer that pairs without requesting a bond
        /// -- which is what BlueZ sends from `pair` -- gets `Pairing Failed: unspecified reason` before any key
        /// exchange. Measured on this board: flags 0x02|0x04 refused the client outright; see the doc.
        const FLAGS: u8 = 0x04;
        /// `sl_bt_sm_io_capability_noinputnooutput`
        const NO_INPUT_NO_OUTPUT: u8 = 0x03;

        let status = unsafe { bt::sl_bt_sm_configure(FLAGS, NO_INPUT_NO_OUTPUT) };
        if status != 0 {
            return Err(BleError::from(status));
        }
        // EXPERIMENT: not bondable, to separate "the security manager cannot run" from "it cannot store the bond".
        let status = unsafe { bt::sl_bt_sm_set_bondable_mode(0) };
        if status != 0 {
            return Err(BleError::from(status));
        }
        Ok(())
    }

    /// Set connection state (manual tracking)
    ///
    /// Updates the internal connection state. This is typically called
    /// automatically by the event system.
    ///
    /// # Arguments
    ///
    /// - `connected` - True if device is connected, false if disconnected
    /// - `connection_handle` - Connection handle from BGAPI (if connected)
    ///
    /// # Note
    ///
    /// This is called automatically by `poll_event()` when connection/
    /// disconnection events are received. Manual use is rarely needed.
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
    /// use efr32mg24_bluetooth_sys::blocking::{Bluetooth, BleEvent, CharHandle};
    ///
    /// const GATTDB_LED_CONTROL: CharHandle = CharHandle(3);
    ///
    /// let mut ble = Bluetooth::new(&mut delay)?;
    /// ble.set_device_name("LED Controller")?;
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
        // Run any expired timer callbacks **first**. The stack schedules its own work through
        // `sl_sleeptimer_restart_timer` (retries, connection supervision, advertising refresh), and this project
        // answers that with a polled scheduler because the SYSRTC compare interrupt does not fire yet. Putting
        // the poll here rather than in each example means no example can forget it -- and an example that forgot
        // it looks exactly like a stack that never advances, which is the most expensive bug shape in this tree.
        crate::sleeptimer::poll();
        // ...and the host itself. `sl_bt_run` is what makes BGAPI commands progress and events appear; without it
        // `poll_event` answers "no event" forever while the stack is perfectly healthy.
        crate::stack_init::pump();

        // Allocate event structure (256 bytes on stack)
        let mut evt = core::mem::MaybeUninit::<bt::sl_bt_msg_t>::uninit();

        // Pop event from Bluetooth stack queue
        let status = unsafe { bt::sl_bt_pop_event(evt.as_mut_ptr()) };

        // Status != 0 means no events available
        if status != 0 {
            return None;
        }

        // **The raw header of the last event, kept for diagnosis.** An event that this parser does not recognise is
        // dropped silently, and "the scanner heard nothing" and "the scanner received events this code cannot name"
        // look identical from outside. Recording the header -- and its id, which `SL_BGAPI_MSG_ID` extracts with
        // `0xffff00f8` -- turns that into a number a reader can compare against `sl_bt_api.h`.
        // SAFETY: `sl_bt_pop_event` returned 0, so the structure holds a valid header.
        let header =
            unsafe { core::ptr::read_volatile(core::ptr::addr_of!((*evt.as_ptr()).header)) };
        LAST_EVENT_HEADER.store(header, AtomicOrdering::Relaxed);

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
