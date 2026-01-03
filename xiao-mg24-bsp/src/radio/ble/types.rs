//! BLE type definitions and utilities
//!
//! This module provides type-safe wrappers and utilities for BLE operations.

use super::super::BleError;

/// GATT characteristic handle
///
/// Opaque wrapper around BGAPI GATT handle (u16).
///
/// # Usage
///
/// GATT handles are obtained from the GATT Configurator tool when designing
/// the GATT database. They identify specific characteristics.
///
/// ```no_run
/// use xiao_mg24_bsp::radio::ble::CharHandle;
///
/// // From GATT Configurator output
/// const GATTDB_HEART_RATE_MEASUREMENT: CharHandle = CharHandle(3);
/// const GATTDB_BATTERY_LEVEL: CharHandle = CharHandle(5);
/// ```
///
/// # Future Work
///
/// In future phases, a GATT database builder will automatically assign handles.
/// For now, users must hardcode values from GATT Configurator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharHandle(pub u16);

impl CharHandle {
    /// Get raw handle value (for FFI calls)
    pub fn value(&self) -> u16 {
        self.0
    }
}

/// GAP Advertising Data format builder
///
/// Automatically encodes advertising data in GAP AD format (Length-Type-Value).
///
/// # GAP AD Format
///
/// BLE advertising data uses a specific format:
/// ```text
/// [length, type, data...]
/// ```
///
/// Where:
/// - `length`: Number of bytes in type + data (excludes length byte itself)
/// - `type`: AD type (flags, name, UUID, manufacturer data, etc.)
/// - `data`: Type-specific payload
///
/// # Size Limit
///
/// Total advertising data must not exceed 31 bytes (BLE 4.x limit).
/// BLE 5.0 supports extended advertising (up to 254 bytes), but this is not
/// yet implemented.
///
/// # Example
///
/// ```no_run
/// use xiao_mg24_bsp::radio::ble::GapAdBuilder;
///
/// let mut builder = GapAdBuilder::new();
/// builder.add_name("XIAO Device")?;
/// builder.add_service_uuid_16(0x180D)?; // Heart Rate Service
///
/// let adv_data = builder.as_slice();
/// // adv_data is now ready to pass to sl_bt_advertiser_set_data()
/// ```
#[derive(Debug, Clone)]
pub struct GapAdBuilder {
    /// Internal buffer (max 31 bytes)
    buffer: [u8; 31],

    /// Current offset in buffer
    offset: usize,
}

impl GapAdBuilder {
    /// Create new GAP AD builder with default flags
    ///
    /// Automatically adds flags field (General Discoverable, BR/EDR Not Supported).
    ///
    /// Initial buffer content:
    /// ```text
    /// [2, 0x01, 0x06]
    /// ```
    ///
    /// This means:
    /// - Length: 2 bytes (type + value)
    /// - Type: 0x01 (Flags)
    /// - Value: 0x06 (General Discoverable + BR/EDR Not Supported)
    pub fn new() -> Self {
        let mut buffer = [0u8; 31];
        let mut offset = 0;

        // Add flags field (required for all advertising)
        buffer[offset] = 2;      // Length
        buffer[offset + 1] = 0x01; // AD Type: Flags
        buffer[offset + 2] = 0x06; // Value: General Discoverable + BR/EDR Not Supported
        offset += 3;

        Self { buffer, offset }
    }

    /// Add device name to advertising data
    ///
    /// Encodes as Complete Local Name (AD type 0x09).
    ///
    /// # Arguments
    ///
    /// - `name` - Device name (max 29 characters to fit in 31 byte limit)
    ///
    /// # Errors
    ///
    /// - `BleError::InvalidParameter` if name exceeds 29 characters
    /// - `BleError::BufferOverflow` if adding name would exceed 31 bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// builder.add_name("XIAO")?;
    /// ```
    pub fn add_name(&mut self, name: &str) -> Result<(), BleError> {
        // Validate name length
        if name.len() > 29 {
            return Err(BleError::InvalidParameter);
        }

        // Check if there's room
        // Need: 1 (length) + 1 (type) + name.len()
        let needed = 2 + name.len();
        if self.offset + needed > 31 {
            return Err(BleError::BufferOverflow);
        }

        // Add Complete Local Name field
        self.buffer[self.offset] = (name.len() + 1) as u8; // Length (type + data)
        self.buffer[self.offset + 1] = 0x09; // AD Type: Complete Local Name
        self.buffer[self.offset + 2..self.offset + 2 + name.len()].copy_from_slice(name.as_bytes());
        self.offset += needed;

        Ok(())
    }

    /// Add 16-bit service UUID to advertising data
    ///
    /// Encodes as Complete List of 16-bit Service UUIDs (AD type 0x03).
    ///
    /// # Arguments
    ///
    /// - `uuid` - 16-bit service UUID (e.g., 0x180D for Heart Rate Service)
    ///
    /// # Errors
    ///
    /// - `BleError::BufferOverflow` if adding UUID would exceed 31 bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// builder.add_service_uuid_16(0x180D)?; // Heart Rate Service
    /// builder.add_service_uuid_16(0x180F)?; // Battery Service
    /// ```
    ///
    /// # Note
    ///
    /// Multiple calls to this method will append UUIDs to the same field
    /// (more efficient than creating separate fields).
    pub fn add_service_uuid_16(&mut self, uuid: u16) -> Result<(), BleError> {
        // Check if there's room
        // Need: 1 (length) + 1 (type) + 2 (UUID)
        let needed = 4;
        if self.offset + needed > 31 {
            return Err(BleError::BufferOverflow);
        }

        // Add Complete List of 16-bit Service UUIDs field
        self.buffer[self.offset] = 3; // Length (type + 2 bytes UUID)
        self.buffer[self.offset + 1] = 0x03; // AD Type: Complete List of 16-bit Service UUIDs
        self.buffer[self.offset + 2] = (uuid & 0xFF) as u8; // UUID LSB
        self.buffer[self.offset + 3] = ((uuid >> 8) & 0xFF) as u8; // UUID MSB
        self.offset += needed;

        Ok(())
    }

    /// Add manufacturer-specific data to advertising data
    ///
    /// Encodes as Manufacturer Specific Data (AD type 0xFF).
    ///
    /// # Arguments
    ///
    /// - `company_id` - 16-bit company identifier (assigned by Bluetooth SIG)
    /// - `data` - Manufacturer-specific payload (max ~25 bytes depending on other fields)
    ///
    /// # Errors
    ///
    /// - `BleError::BufferOverflow` if adding data would exceed 31 bytes
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Company ID 0xFFFF is reserved for testing
    /// builder.add_manufacturer_data(0xFFFF, &[0x42, 0x00])?;
    /// ```
    pub fn add_manufacturer_data(&mut self, company_id: u16, data: &[u8]) -> Result<(), BleError> {
        // Check if there's room
        // Need: 1 (length) + 1 (type) + 2 (company_id) + data.len()
        let needed = 4 + data.len();
        if self.offset + needed > 31 {
            return Err(BleError::BufferOverflow);
        }

        // Add Manufacturer Specific Data field
        self.buffer[self.offset] = (3 + data.len()) as u8; // Length (type + company_id + data)
        self.buffer[self.offset + 1] = 0xFF; // AD Type: Manufacturer Specific Data
        self.buffer[self.offset + 2] = (company_id & 0xFF) as u8; // Company ID LSB
        self.buffer[self.offset + 3] = ((company_id >> 8) & 0xFF) as u8; // Company ID MSB
        self.buffer[self.offset + 4..self.offset + 4 + data.len()].copy_from_slice(data);
        self.offset += needed;

        Ok(())
    }

    /// Get advertising data as byte slice
    ///
    /// Returns only the filled portion of the buffer (not all 31 bytes).
    ///
    /// # Example
    ///
    /// ```no_run
    /// let adv_data = builder.as_slice();
    /// unsafe {
    ///     bt::sl_bt_advertiser_set_data(
    ///         advertising_set,
    ///         0,
    ///         adv_data.len(),
    ///         adv_data.as_ptr(),
    ///     );
    /// }
    /// ```
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.offset]
    }

    /// Get current buffer usage in bytes
    pub fn len(&self) -> usize {
        self.offset
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.offset == 0
    }

    /// Get remaining buffer space in bytes
    pub fn remaining(&self) -> usize {
        31 - self.offset
    }

    /// Clear all advertising data and reset to default flags
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

impl Default for GapAdBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gap_ad_builder_initializes_with_flags() {
        let builder = GapAdBuilder::new();
        let slice = builder.as_slice();

        // Should have flags field: [2, 0x01, 0x06]
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 2);      // Length
        assert_eq!(slice[1], 0x01);   // AD Type: Flags
        assert_eq!(slice[2], 0x06);   // Value: General Discoverable + BR/EDR Not Supported
    }

    #[test]
    fn gap_ad_builder_adds_name() {
        let mut builder = GapAdBuilder::new();
        builder.add_name("Test").unwrap();

        let slice = builder.as_slice();

        // Should have flags + name: [2, 0x01, 0x06] + [5, 0x09, 'T', 'e', 's', 't']
        assert_eq!(slice.len(), 9); // 3 (flags) + 6 (name)
        assert_eq!(slice[3], 5);       // Name length (type + 4 chars)
        assert_eq!(slice[4], 0x09);    // AD Type: Complete Local Name
        assert_eq!(&slice[5..9], b"Test");
    }

    #[test]
    fn gap_ad_builder_rejects_long_name() {
        let mut builder = GapAdBuilder::new();
        let long_name = "This name is way too long to fit in 31 bytes";

        let result = builder.add_name(long_name);
        assert_eq!(result, Err(BleError::InvalidParameter));
    }

    #[test]
    fn gap_ad_builder_adds_service_uuid() {
        let mut builder = GapAdBuilder::new();
        builder.add_service_uuid_16(0x180D).unwrap(); // Heart Rate Service

        let slice = builder.as_slice();

        // Should have flags + UUID: [2, 0x01, 0x06] + [3, 0x03, 0x0D, 0x18]
        assert_eq!(slice.len(), 7); // 3 (flags) + 4 (UUID)
        assert_eq!(slice[3], 3);       // UUID field length
        assert_eq!(slice[4], 0x03);    // AD Type: 16-bit Service UUIDs
        assert_eq!(slice[5], 0x0D);    // UUID LSB
        assert_eq!(slice[6], 0x18);    // UUID MSB
    }

    #[test]
    fn gap_ad_builder_detects_buffer_overflow() {
        let mut builder = GapAdBuilder::new();
        // Max name length that fits: 31 - 3 (flags) - 2 (header) = 26 chars
        let max_name = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"; // 26 chars
        builder.add_name(max_name).unwrap();

        // Try to add UUID (should overflow)
        let result = builder.add_service_uuid_16(0x180D);
        assert_eq!(result, Err(BleError::BufferOverflow));
    }

    #[test]
    fn gap_ad_builder_clear_resets_to_flags() {
        let mut builder = GapAdBuilder::new();
        builder.add_name("Test").unwrap();
        assert_eq!(builder.len(), 9);

        builder.clear();
        assert_eq!(builder.len(), 3); // Back to just flags
    }
}
