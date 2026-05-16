# Unified Radio Architecture: efr32mg24-radio Module

**Status**: Design Document (Phase D4 - Future)
**Author**: Architecture Team
**Date**: 2026-01-03
**References**:
- [RADIO_ARCHITECTURE.md](RADIO_ARCHITECTURE.md) - Base radio architecture
- [BLUETOOTH_REFACTORING_DECISIONS.md](BLUETOOTH_REFACTORING_DECISIONS.md) - BLE refactoring decisions

---

## Executive Summary

Proposal for a unified `efr32mg24-radio` crate that provides feature-gated access to all radio protocols (BLE, Zigbee, Thread) supported by the EFR32MG24's Dynamic Multiprotocol capability.

**Key Benefits**:
- Single dependency for all radio protocols
- Shared RAIL (Radio Abstraction Interface Layer) abstractions
- Multi-protocol support (BLE + Zigbee concurrently)
- Zero-cost feature flags (users only pay for what they use)

---

## Industry Precedents

### Pattern A: Separate Crates (Nordic - nrf-rs)

**Structure**:
```
nrf-softdevice/       ← BLE only
nrf-ieee802154/       ← Zigbee/Thread only
```

**Rationale**:
- Nordic SoftDevice is mutually exclusive firmware blob
- BLE and IEEE 802.15.4 cannot run simultaneously
- Different radio stacks are incompatible

**Conclusion**: **Not applicable to EFR32MG24** (we have dynamic multiprotocol)

---

### Pattern B: Unified Crate with Features (Espressif - esp-rs)

**Structure**:
```rust
// esp-wifi/Cargo.toml
[features]
wifi = []
ble = []
esp-now = []
default = ["wifi"]

// User enables what they need
esp-wifi = { version = "0.5", features = ["wifi", "ble"] }
```

**Rationale**:
- ESP32 supports simultaneous protocols (Wi-Fi + BLE)
- Shared radio hardware managed by vendor SDK
- Feature flags provide zero-cost abstraction

**Conclusion**: **Perfect match for EFR32MG24!**

---

## EFR32MG24 Capabilities

### Dynamic Multiprotocol (DMP)

Silicon Labs EFR32MG24 supports **concurrent** multi-protocol operation:

```
┌─────────────────────────────────────┐
│      Application Layer              │
├─────────────┬───────────┬───────────┤
│ BLE Stack   │ Zigbee    │ Thread    │
│ (BGAPI)     │ Stack     │ Stack     │
├─────────────┴───────────┴───────────┤
│    RAIL (Radio Abstraction Layer)   │  ← Shared!
├──────────────────────────────────────┤
│      2.4 GHz Radio Hardware          │
└──────────────────────────────────────┘
```

**Key Features**:
- **Concurrent BLE + Zigbee**: Time-sliced radio sharing
- **Shared RAIL layer**: Common radio configuration
- **Dynamic scheduling**: Automatic priority management
- **Separate binary blobs**: `libbluetooth.a`, `libzigbee.a`, `libthread.a`

**Source**:
- AN1269: Dynamic Multiprotocol Development with Bluetooth and Zigbee
- UG435.06: Multiprotocol User's Guide

---

## Proposed Architecture

### Crate Structure

```
efr32mg24-radio/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              ← Feature-gated re-exports
│   ├── rail/               ← Common RAIL abstractions
│   │   ├── mod.rs
│   │   ├── config.rs       ← Radio configuration
│   │   ├── power.rs        ← TX power management
│   │   └── scheduler.rs    ← Multiprotocol scheduling
│   ├── ble.rs              ← #[cfg(feature = "ble")]
│   ├── zigbee.rs           ← #[cfg(feature = "zigbee")]
│   └── thread.rs           ← #[cfg(feature = "thread")]
└── examples/
    ├── ble_only.rs
    ├── zigbee_only.rs
    ├── ble_zigbee_concurrent.rs
    └── thread_border_router.rs
```

### Cargo.toml

```toml
[package]
name = "efr32mg24-radio"
version = "0.1.0"
edition = "2021"
description = "Unified radio protocol support for EFR32MG24 (BLE, Zigbee, Thread)"

[dependencies]
# Core FFI crates (underlying implementation)
efr32mg24-rail-sys = { version = "0.1", path = "../efr32mg24-rail-sys" }
efr32mg24-bluetooth-sys = { version = "0.1", path = "../efr32mg24-bluetooth-sys", optional = true }
efr32mg24-zigbee-sys = { version = "0.1", path = "../efr32mg24-zigbee-sys", optional = true }
efr32mg24-thread-sys = { version = "0.1", path = "../efr32mg24-thread-sys", optional = true }

# Common dependencies
embedded-hal = "1.0"
heapless = "0.8"

[features]
# Individual protocols
ble = ["efr32mg24-bluetooth-sys"]
zigbee = ["efr32mg24-zigbee-sys"]
thread = ["efr32mg24-thread-sys"]

# Protocol combinations (common use cases)
ble-zigbee = ["ble", "zigbee"]          # Smart home hub
ble-thread = ["ble", "thread"]          # Matter device
all-protocols = ["ble", "zigbee", "thread"]

# Async support (future)
embassy = ["efr32mg24-bluetooth-sys?/embassy"]

# Default: No protocols (user must choose)
default = []
```

### lib.rs Structure

```rust
#![no_std]
#![doc = include_str!("../README.md")]

//! Unified radio protocol support for EFR32MG24
//!
//! # Feature Flags
//!
//! - `ble`: Bluetooth Low Energy 5.3 support
//! - `zigbee`: Zigbee 3.0 support
//! - `thread`: Thread 1.3 / Matter support
//! - `ble-zigbee`: Concurrent BLE + Zigbee (dynamic multiprotocol)

// Common RAIL abstractions (always available)
pub mod rail;

// Protocol modules (feature-gated)
#[cfg(feature = "ble")]
pub mod ble {
    //! Bluetooth Low Energy 5.3
    //!
    //! Re-exports from `efr32mg24_bluetooth_sys::blocking`.
    pub use efr32mg24_bluetooth_sys::blocking::*;
}

#[cfg(feature = "zigbee")]
pub mod zigbee {
    //! Zigbee 3.0
    //!
    //! Re-exports from `efr32mg24_zigbee_sys::blocking`.
    pub use efr32mg24_zigbee_sys::blocking::*;
}

#[cfg(feature = "thread")]
pub mod thread {
    //! Thread 1.3 / Matter
    //!
    //! Re-exports from `efr32mg24_thread_sys::blocking`.
    pub use efr32mg24_thread_sys::blocking::*;
}

// Multiprotocol support
#[cfg(all(feature = "ble", feature = "zigbee"))]
pub mod multiprotocol;
```

---

## RAIL Module (Common Abstractions)

### Purpose

Provide shared radio configuration and scheduling for all protocols.

### rail/mod.rs

```rust
//! Radio Abstraction Interface Layer (RAIL)
//!
//! Common radio configuration shared across all protocols.

pub mod config;
pub mod power;

#[cfg(any(
    all(feature = "ble", feature = "zigbee"),
    all(feature = "ble", feature = "thread"),
    all(feature = "zigbee", feature = "thread")
))]
pub mod scheduler;

pub use config::RadioConfig;
pub use power::{TxPower, TxPowerDbm};

#[cfg(any(
    all(feature = "ble", feature = "zigbee"),
    all(feature = "ble", feature = "thread")
))]
pub use scheduler::MultiprotocolScheduler;
```

### rail/config.rs

```rust
//! Common radio configuration

use efr32mg24_rail_sys as rail;

/// Radio configuration
///
/// Shared configuration used by all protocol stacks.
pub struct RadioConfig {
    /// 2.4 GHz band configuration
    band: Band,

    /// TX power in dBm (-20 to +20)
    tx_power: i16,
}

impl RadioConfig {
    /// Create default radio configuration
    pub fn new() -> Self {
        Self {
            band: Band::Band2400MHz,
            tx_power: 0, // 0 dBm default
        }
    }

    /// Set TX power
    pub fn tx_power(mut self, dbm: i16) -> Self {
        self.tx_power = dbm.clamp(-20, 20);
        self
    }
}

/// Radio band
#[derive(Debug, Clone, Copy)]
pub enum Band {
    /// 2.4 GHz ISM band (2400-2483.5 MHz)
    Band2400MHz,
}
```

### rail/power.rs

```rust
//! TX power management

/// TX power in dBm
///
/// Valid range: -20 dBm to +20 dBm (EFR32MG24 high-power variant)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxPowerDbm(i16);

impl TxPowerDbm {
    /// Create TX power value (clamped to valid range)
    pub fn new(dbm: i16) -> Self {
        Self(dbm.clamp(-20, 20))
    }

    /// Get value in dBm
    pub fn dbm(&self) -> i16 {
        self.0
    }

    /// Convert to RAIL TX power units (0.1 dBm resolution)
    pub(crate) fn to_rail_units(&self) -> i16 {
        self.0 * 10
    }
}

/// Common TX power presets
impl TxPowerDbm {
    pub const MIN: Self = Self(-20);        // Minimum power
    pub const LOW: Self = Self(-10);        // Low power (beacon mode)
    pub const MEDIUM: Self = Self(0);       // Medium power (default)
    pub const HIGH: Self = Self(10);        // High power
    pub const MAX: Self = Self(20);         // Maximum power (B-series only)
}
```

### rail/scheduler.rs (Multiprotocol)

```rust
//! Dynamic multiprotocol scheduler
//!
//! Only available when multiple protocol features are enabled.

use efr32mg24_rail_sys as rail;

/// Multiprotocol scheduler
///
/// Manages time-sliced radio access for concurrent protocols.
///
/// # Example
///
/// ```no_run
/// use efr32mg24_radio::rail::MultiprotocolScheduler;
/// use efr32mg24_radio::{ble, zigbee};
///
/// let mut scheduler = MultiprotocolScheduler::new();
/// scheduler.set_ble_priority(ProtocolPriority::High);
/// scheduler.set_zigbee_priority(ProtocolPriority::Normal);
/// ```
pub struct MultiprotocolScheduler {
    // Internal RAIL scheduler state
}

impl MultiprotocolScheduler {
    /// Create new multiprotocol scheduler
    pub fn new() -> Self {
        // Initialize RAIL multiprotocol support
        Self {}
    }

    /// Set BLE priority
    #[cfg(feature = "ble")]
    pub fn set_ble_priority(&mut self, priority: ProtocolPriority) {
        // Configure via RAIL
    }

    /// Set Zigbee priority
    #[cfg(feature = "zigbee")]
    pub fn set_zigbee_priority(&mut self, priority: ProtocolPriority) {
        // Configure via RAIL
    }
}

/// Protocol priority for multiprotocol scheduling
#[derive(Debug, Clone, Copy)]
pub enum ProtocolPriority {
    /// Low priority (background tasks)
    Low,

    /// Normal priority (default)
    Normal,

    /// High priority (time-critical operations)
    High,
}
```

---

## Usage Examples

### Example 1: BLE Only

```toml
[dependencies]
efr32mg24-radio = { version = "0.1", features = ["ble"] }
```

```rust
use efr32mg24_radio::ble::{Bluetooth, BleEvent};

let mut ble = Bluetooth::new(&mut delay)?;
ble.set_device_name("My Device")?;
ble.start_connectable(100, 0)?;

loop {
    if let Some(event) = ble.poll_event() {
        match event {
            BleEvent::Connected { .. } => { /* handle */ }
            _ => {}
        }
    }
}
```

### Example 2: Zigbee Only

```toml
[dependencies]
efr32mg24-radio = { version = "0.1", features = ["zigbee"] }
```

```rust
use efr32mg24_radio::zigbee::{Zigbee, ZigbeeEvent};

let mut zigbee = Zigbee::new(&mut delay)?;
zigbee.join_network()?;

loop {
    if let Some(event) = zigbee.poll_event() {
        // Handle Zigbee events
    }
}
```

### Example 3: BLE + Zigbee Concurrent (Dynamic Multiprotocol)

```toml
[dependencies]
efr32mg24-radio = { version = "0.1", features = ["ble-zigbee"] }
```

```rust
use efr32mg24_radio::{ble, zigbee, rail};

// Initialize multiprotocol scheduler
let mut scheduler = rail::MultiprotocolScheduler::new();
scheduler.set_ble_priority(rail::ProtocolPriority::High);
scheduler.set_zigbee_priority(rail::ProtocolPriority::Normal);

// Initialize both protocols
let mut ble = ble::Bluetooth::new(&mut delay)?;
let mut zigbee = zigbee::Zigbee::new(&mut delay)?;

loop {
    // Poll both protocols (RAIL handles radio time-slicing)
    if let Some(event) = ble.poll_event() {
        // Handle BLE events
    }

    if let Some(event) = zigbee.poll_event() {
        // Handle Zigbee events
    }
}
```

---

## Migration Path

### Phase D2 (Current - Complete)

```
✅ efr32mg24-bluetooth-sys with blocking API
✅ Safe BLE wrappers
✅ Event-driven architecture
✅ BSP thin wrapper
```

### Phase D3 (Next - Zigbee)

```
1. Create efr32mg24-zigbee-sys (mirror BLE structure)
2. Implement blocking Zigbee API
3. Add Zigbee examples to BSP
4. Document Zigbee architecture
```

### Phase D4 (Unified Radio)

```
1. Create efr32mg24-radio crate
2. Implement rail/ common abstractions
3. Add feature-gated ble/zigbee/thread modules
4. Implement MultiprotocolScheduler
5. Create concurrent protocol examples
```

### Phase D5 (BSP Integration)

```
1. Update xiao-mg24-bsp to use efr32mg24-radio
2. Remove direct *-sys dependencies
3. Add radio feature flags to BSP
4. Update all examples
```

---

## Benefits Analysis

### For Users

**Single Dependency**:
```toml
# Before (multiple crates)
efr32mg24-bluetooth-sys = "0.1"
efr32mg24-zigbee-sys = "0.1"
efr32mg24-thread-sys = "0.1"

# After (unified)
efr32mg24-radio = { version = "0.1", features = ["ble", "zigbee"] }
```

**Zero Cost**:
- Users only compile protocols they enable
- No runtime overhead for disabled features
- Binary size matches single-protocol apps

**Multiprotocol Support**:
- Simple feature flags enable concurrent protocols
- Shared RAIL configuration
- Automatic scheduling

### For Maintainers

**Consistent Architecture**:
- All protocols follow same structure
- Shared RAIL abstractions reduce duplication
- Single documentation location

**Versioning**:
- Synchronized protocol versions
- Breaking changes coordinated
- Single release cycle

---

## Comparison with Current Architecture

### Current (Phase D2)

```
Workspace:
├── efr32mg24-bluetooth-sys    (BLE only)
├── efr32mg24-rail-sys         (RAIL FFI)
└── xiao-mg24-bsp              (Board wrapper)

User code:
use efr32mg24_bluetooth_sys::blocking::Bluetooth;
```

**Pros**:
- Clean separation of concerns
- Easy to understand
- No feature flag complexity

**Cons**:
- Multiple dependencies for multi-protocol
- No shared abstractions
- BSP must manage multiple *-sys crates

### Future (Phase D4)

```
Workspace:
├── efr32mg24-bluetooth-sys    (BLE FFI + wrappers)
├── efr32mg24-zigbee-sys       (Zigbee FFI + wrappers)
├── efr32mg24-thread-sys       (Thread FFI + wrappers)
├── efr32mg24-rail-sys         (RAIL FFI)
├── efr32mg24-radio            (Unified convenience crate)
└── xiao-mg24-bsp              (Board wrapper)

User code:
use efr32mg24_radio::ble::Bluetooth;
use efr32mg24_radio::zigbee::Zigbee;
```

**Pros**:
- Single dependency
- Shared RAIL abstractions
- Multiprotocol support built-in
- Consistent API across protocols

**Cons**:
- More complex workspace
- Feature flag configuration required
- Slightly longer compile times (full workspace)

---

## Design Decisions

### Decision 1: Keep *-sys Crates Separate

**Rationale**:
- Follows esp-wifi pattern (esp-wifi-sys separate from esp-wifi)
- Clean FFI boundary
- Allows advanced users to use *-sys directly
- Easier to test individual protocols

### Decision 2: Feature Flags for Protocols

**Rationale**:
- Zero-cost abstraction (disabled = not compiled)
- User control over binary size
- Industry standard pattern (esp-wifi, embassy)

### Decision 3: RAIL as Common Module

**Rationale**:
- All protocols use RAIL underneath
- Shared configuration reduces duplication
- Multiprotocol scheduling requires shared state

### Decision 4: Re-exports, Not Wrappers

**Rationale**:
```rust
// Good: Re-export (zero cost)
pub use efr32mg24_bluetooth_sys::blocking::*;

// Avoid: Additional wrapper (overhead)
pub struct Bluetooth {
    inner: bluetooth_sys::Bluetooth,
}
```

### Decision 5: Blocking First, Async Later

**Rationale**:
- Blocking API is simpler
- Embassy support via feature flag
- Matches bluetooth-sys structure

---

## Future Extensions

### Embassy Async Support

```toml
[features]
embassy = [
    "efr32mg24-bluetooth-sys?/embassy",
    "efr32mg24-zigbee-sys?/embassy",
]
```

```rust
#[cfg(feature = "embassy")]
pub mod asynch {
    pub use efr32mg24_bluetooth_sys::asynch::*;
}
```

### Matter Support

```toml
[features]
matter = ["thread", "ble"]  # Matter requires both
```

```rust
#[cfg(feature = "matter")]
pub mod matter {
    //! Matter protocol (Thread + BLE commissioning)
}
```

---

## References

### Silicon Labs Documentation

- **AN1269**: Dynamic Multiprotocol Development with Bluetooth and Zigbee
- **UG435.06**: Multiprotocol User's Guide
- **AN1268**: Simplicity Studio 5 Multiprotocol Application Developer's Guide

### Industry Examples

- **esp-wifi**: https://github.com/esp-rs/esp-wifi (unified radio pattern)
- **nrf-softdevice**: https://github.com/embassy-rs/nrf-softdevice (separate crates pattern)

### Internal Documentation

- [RADIO_ARCHITECTURE.md](RADIO_ARCHITECTURE.md) - Base architecture
- [BLUETOOTH_REFACTORING_DECISIONS.md](BLUETOOTH_REFACTORING_DECISIONS.md) - BLE decisions

---

## Implementation Checklist

### Phase D4 Preparation

- [ ] Complete Zigbee stack implementation (Phase D3)
- [ ] Verify RAIL scheduler capabilities
- [ ] Design multiprotocol API
- [ ] Create test plan for concurrent protocols

### Phase D4 Implementation

- [ ] Create efr32mg24-radio crate structure
- [ ] Implement rail/ common module
- [ ] Add feature-gated protocol modules
- [ ] Implement MultiprotocolScheduler
- [ ] Write concurrent protocol examples
- [ ] Update documentation

### Phase D5 BSP Migration

- [ ] Update xiao-mg24-bsp Cargo.toml
- [ ] Migrate all examples to unified radio
- [ ] Test all protocol combinations
- [ ] Update README and docs

---

**Last Updated**: 2026-01-03
**Status**: Design Document (Phase D4 - Future)
**Next Review**: After Phase D3 (Zigbee) completion

<!-- META: last_updated=2026-01-03 version=1.0.0 status=design template-version=1.0.0 -->
