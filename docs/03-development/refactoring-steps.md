# Bluetooth Refactoring Execution Steps

**Status**: Ready to execute
**Documentation**: See docs/BLUETOOTH_REFACTORING_DECISIONS.md (661 lines)
**Architecture**: See docs/RADIO_ARCHITECTURE.md (724 lines)

---

## Summary

Move 1,380 lines of generic BLE code from `xiao-mg24-bsp` to `crates/efr32mg24-bluetooth-sys/src/blocking/`

---

## Step 1: Move BleError (142 lines)

**Source:** `crates/xiao-mg24-bsp/src/radio/mod.rs` lines 30-142

**Create:** `crates/efr32mg24-bluetooth-sys/src/blocking/error.rs`

```bash
# Extract lines 30-142 from radio/mod.rs and create error.rs
```

**Content structure:**
- BleError enum (5 variants)
- from_bgapi_status() method
- From<u16> and From<u32> implementations
- Display implementation

---

## Step 2: Move Types (350 lines)

**Source:** `crates/xiao-mg24-bsp/src/radio/ble/types.rs` (entire file)

**Create:** `crates/efr32mg24-bluetooth-sys/src/blocking/types.rs`

**Content:**
- CharHandle struct
- GapAdBuilder struct
- All unit tests

**Update imports:**
```rust
// Change from:
use super::super::BleError;

// To:
use super::error::BleError;
```

---

## Step 3: Move BleEvent (280 lines)

**Source:** `crates/xiao-mg24-bsp/src/radio/ble/events.rs` (entire file)

**Create:** `crates/efr32mg24-bluetooth-sys/src/blocking/events.rs`

**Content:**
- BleEvent enum (7 variants)
- BleEvent::parse() method

**Update imports:**
```rust
// Change from:
use super::BleError;
use efr32mg24_bluetooth_sys as bt;

// To:
use super::error::BleError;
use crate::ffi as bt;
```

---

## Step 4: Move Bluetooth Stack (700 lines)

**Source:** `crates/xiao-mg24-bsp/src/radio/ble/mod.rs` (XiaoBle struct + impl)

**Create:** `crates/efr32mg24-bluetooth-sys/src/blocking/stack.rs`

**Changes:**
1. Rename `XiaoBle` → `Bluetooth`
2. Update imports
3. Remove board-specific references

**Extract from mod.rs:**
- XiaoBle struct (lines ~113-127)
- impl XiaoBle (lines ~129-721)

**Update imports:**
```rust
// Change from:
use super::BleError;
use efr32mg24_bluetooth_sys as bt;
pub use events::BleEvent;
pub use types::{CharHandle, GapAdBuilder};

// To:
use super::error::BleError;
use super::events::BleEvent;
use super::types::{CharHandle, GapAdBuilder};
use crate::ffi as bt;
use embedded_hal::delay::DelayNs;
use heapless::Vec;
```

**Rename:**
```rust
// Change all occurrences:
pub struct XiaoBle → pub struct Bluetooth
impl XiaoBle → impl Bluetooth
```

---

## Step 5: Create blocking/mod.rs

**Create:** `crates/efr32mg24-bluetooth-sys/src/blocking/mod.rs`

```rust
//! Blocking Bluetooth API
//!
//! Synchronous/polling-based Bluetooth stack for use without async runtime.

mod error;
mod events;
mod stack;
mod types;

pub use error::BleError;
pub use events::BleEvent;
pub use stack::Bluetooth;
pub use types::{CharHandle, GapAdBuilder};
```

---

## Step 6: Update bluetooth-sys/src/lib.rs

**Source:** `crates/efr32mg24-bluetooth-sys/src/lib.rs`

**Change from:**
```rust
#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

// Include the auto-generated bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

**Change to:**
```rust
#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

//! Silicon Labs Bluetooth Stack (BGAPI) for EFR32MG24
//!
//! This crate provides both raw FFI bindings and safe Rust wrappers.
//!
//! # Blocking API (default)
//!
//! ```no_run
//! use efr32mg24_bluetooth_sys::blocking::{Bluetooth, BleEvent};
//!
//! let mut ble = Bluetooth::new(&mut delay)?;
//! while let Some(event) = ble.poll_event() {
//!     match event {
//!         BleEvent::Connected { .. } => { /* handle */ }
//!         _ => {}
//!     }
//! }
//! ```
//!
//! # Raw FFI (advanced users)
//!
//! ```no_run
//! use efr32mg24_bluetooth_sys::ffi;
//! unsafe {
//!     ffi::sl_bt_system_start_bluetooth();
//! }
//! ```

// Raw FFI bindings (always available)
pub mod ffi {
    //! Raw BGAPI FFI bindings
    //!
    //! **Safety**: All functions are `unsafe`. Use the safe wrappers instead.
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Blocking API (always available)
pub mod blocking;

// Re-export blocking API at root for convenience
pub use blocking::*;
```

---

## Step 7: Update bluetooth-sys/Cargo.toml

**Source:** `crates/efr32mg24-bluetooth-sys/Cargo.toml`

**Add to [dependencies]:**
```toml
heapless = "0.8"
embedded-hal = "1.0"
```

**Full dependencies section:**
```toml
[dependencies]
# Core types for FFI
cty = "0.2"

# Safe wrapper dependencies
heapless = "0.8"
embedded-hal = "1.0"
```

---

## Step 8: Refactor BSP to Minimal Wrapper

**Replace:** `crates/xiao-mg24-bsp/src/radio/ble/mod.rs`

**With minimal wrapper (~50 lines):**

```rust
//! XIAO MG24 BLE Convenience Wrapper

// Re-export all bluetooth-sys types
pub use efr32mg24_bluetooth_sys::blocking::*;

use efr32mg24_bluetooth_sys::blocking::Bluetooth;
use embedded_hal::delay::DelayNs;

/// XIAO MG24 BLE convenience wrapper
///
/// Thin wrapper around `Bluetooth` with XIAO-specific defaults.
pub struct XiaoBle {
    bluetooth: Bluetooth,
}

impl XiaoBle {
    /// Initialize BLE stack with XIAO defaults
    pub fn init(delay: &mut impl DelayNs) -> Result<Self, BleError> {
        let mut bluetooth = Bluetooth::new(delay)?;

        // XIAO-specific default
        bluetooth.set_device_name("XIAO MG24")?;

        Ok(Self { bluetooth })
    }

    /// Create BLE beacon with XIAO defaults
    pub fn beacon(delay: &mut impl DelayNs, name: &str) -> Result<Self, BleError> {
        let mut bluetooth = Bluetooth::beacon(delay, name)?;
        Ok(Self { bluetooth })
    }

    // Forward all methods to inner Bluetooth
    pub fn set_device_name(&mut self, name: &str) -> Result<(), BleError> {
        self.bluetooth.set_device_name(name)
    }

    pub fn add_service_uuid(&mut self, uuid: u16) -> Result<(), BleError> {
        self.bluetooth.add_service_uuid(uuid)
    }

    pub fn start_connectable(&mut self, interval_min: u16, interval_max: u16) -> Result<(), BleError> {
        self.bluetooth.start_connectable(interval_min, interval_max)
    }

    pub fn poll_event(&mut self) -> Option<BleEvent> {
        self.bluetooth.poll_event()
    }

    pub fn notify(&mut self, characteristic: CharHandle, data: &[u8]) -> Result<(), BleError> {
        self.bluetooth.notify(characteristic, data)
    }

    pub fn write_attribute(&mut self, attribute: CharHandle, value: &[u8]) -> Result<(), BleError> {
        self.bluetooth.write_attribute(attribute, value)
    }

    pub fn is_connected(&self) -> bool {
        self.bluetooth.is_connected()
    }
}
```

**Delete these files:**
- `crates/xiao-mg24-bsp/src/radio/ble/types.rs` (moved to bluetooth-sys)
- `crates/xiao-mg24-bsp/src/radio/ble/events.rs` (moved to bluetooth-sys)

**Update:** `crates/xiao-mg24-bsp/src/radio/mod.rs`

Remove BleError definition (lines 30-142), replace with:
```rust
// Re-export from bluetooth-sys
pub use efr32mg24_bluetooth_sys::blocking::BleError;
```

---

## Step 9: Update crates/xiao-mg24-bsp/Cargo.toml

**Update dependencies:**
```toml
[dependencies]
# ... existing deps ...

# Bluetooth support
efr32mg24-bluetooth-sys = { path = "../efr32mg24-bluetooth-sys" }
```

---

## Step 10: Update Examples

**Update all 6 BLE examples:**

Files to update:
- `crates/xiao-mg24-bsp/examples/00_blink.rs` (no changes needed)
- `crates/xiao-mg24-bsp/examples/01_ble_beacon_safe.rs`
- `crates/xiao-mg24-bsp/examples/02_ble_led_control.rs`
- `crates/xiao-mg24-bsp/examples/03_ble_sensor_template.rs`
- `crates/xiao-mg24-bsp/examples/04_ble_led_events.rs`
- `crates/xiao-mg24-bsp/examples/05_ble_sensor_events.rs`
- `crates/xiao-mg24-bsp/examples/06_microphone_streaming.rs` (no changes needed)

**Change imports from:**
```rust
use xiao_mg24_bsp::radio::ble::{XiaoBle, BleEvent, BleError, CharHandle};
```

**To (option A - use BSP wrapper):**
```rust
use xiao_mg24_bsp::radio::ble::{XiaoBle, BleEvent, BleError, CharHandle};
// No change - still works!
```

**Or (option B - use bluetooth-sys directly):**
```rust
use efr32mg24_bluetooth_sys::blocking::{Bluetooth, BleEvent, BleError, CharHandle};
// Replace XiaoBle with Bluetooth
```

---

## Step 11: Test Compilation

```bash
# Build bluetooth-sys
cd efr32mg24-bluetooth-sys
cargo build --release

# Build BSP
cd ../xiao-mg24-bsp
cargo build --release

# Build all examples
cargo build --examples --release

# Test specific examples
cargo build --example 01_ble_beacon_safe --release
cargo build --example 04_ble_led_events --release
```

---

## Verification Checklist

- [ ] bluetooth-sys/src/blocking/error.rs created (~142 lines)
- [ ] bluetooth-sys/src/blocking/types.rs created (~350 lines)
- [ ] bluetooth-sys/src/blocking/events.rs created (~280 lines)
- [ ] bluetooth-sys/src/blocking/stack.rs created (~700 lines)
- [ ] bluetooth-sys/src/blocking/mod.rs created (~10 lines)
- [ ] bluetooth-sys/src/lib.rs updated (~100 lines)
- [ ] bluetooth-sys/Cargo.toml updated (added heapless, embedded-hal)
- [ ] BSP radio/mod.rs updated (removed BleError, re-exported from bluetooth-sys)
- [ ] BSP radio/ble/mod.rs replaced with minimal wrapper (~50 lines)
- [ ] BSP radio/ble/types.rs deleted
- [ ] BSP radio/ble/events.rs deleted
- [ ] BSP Cargo.toml updated (added bluetooth-sys dependency)
- [ ] All 6 BLE examples still compile
- [ ] Zero warnings or errors

---

## Git Commit Message

```
[REFACTOR] Move generic BLE code from BSP to bluetooth-sys

Architectural fix: 1,380 lines of generic Bluetooth code was mistakenly
placed in board-specific BSP layer.

Code Movement (BSP → bluetooth-sys/src/blocking/):
- error.rs: BleError enum (142 lines)
- types.rs: CharHandle, GapAdBuilder (350 lines)
- events.rs: BleEvent enum (280 lines)
- stack.rs: Bluetooth struct (700 lines, renamed from XiaoBle)

BSP Changes:
- Replaced 1,348-line XiaoBle with 50-line thin wrapper
- Re-exports types from bluetooth-sys
- Adds XIAO-specific defaults only

Structure:
crates/efr32mg24-bluetooth-sys/
├── src/
│   ├── ffi.rs              ← Raw FFI (unchanged)
│   ├── blocking/           ← Safe wrappers (NEW)
│   │   ├── error.rs
│   │   ├── types.rs
│   │   ├── events.rs
│   │   └── stack.rs
│   └── lib.rs              ← Module organization

Benefits:
- Correct layering: Generic code in proper crate
- BSP simplified: 1,348 lines → 50 lines
- Reusable: Other boards can use bluetooth-sys directly
- Future-proof: Embassy support ready (asynch/ module)

Documentation:
- docs/RADIO_ARCHITECTURE.md (724 lines)
- docs/BLUETOOTH_REFACTORING_DECISIONS.md (661 lines)

All 6 BLE examples compile and work unchanged.
```

---

## Status

**Ready to execute**: All steps documented
**Estimated time**: 2-3 hours
**Risk**: Low (well-documented, reversible)
**Testing**: Compile all examples

**Documentation references:**
- Architecture: docs/RADIO_ARCHITECTURE.md
- Decisions: docs/BLUETOOTH_REFACTORING_DECISIONS.md
- This file: REFACTORING_STEPS.md

---

**Last Updated**: 2026-01-03
