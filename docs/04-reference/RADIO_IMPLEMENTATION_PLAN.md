# EFR32MG24 2.4 GHz Radio Implementation Plan

**Created**: December 27, 2025
**Phase**: D (Radio & Security)
**Status**: Planning Complete
**Identifier Prefix**: D1-D4

---

## Executive Summary

This document provides a **complete implementation plan** for adding 2.4 GHz radio support to the EFR32MG24 Rust HAL, covering **ALL** compatible wireless protocols:

- **Bluetooth 5.3 LE** (Low Energy)
- **Zigbee 3.0**
- **Thread 1.3** (OpenThread)
- **Matter** (IP-based IoT standard)

**Approach**: FFI bindings to Silicon Labs proprietary RAIL (Radio Abstraction Interface Layer) library and protocol stack binaries, following industry-proven patterns from Nordic (nrf-softdevice) and Espressif (esp-idf-hal) ecosystems.

**Total Estimated Effort**: 150-200 hours across 9-12 months
**Hardware Required**: XIAO MG24 Sense boards (minimum 2 for TX/RX testing)

---

## Table of Contents


## Arduino Implementation Analysis

**Reference**: [ARDUINO_RADIO_ANALYSIS.md](ARDUINO_RADIO_ANALYSIS.md) - Comprehensive analysis of Silicon Labs Arduino Core radio implementation

**Key Findings from Arduino Research**:

The [SiliconLabs Arduino Core](https://github.com/SiliconLabs/arduino) validates our architectural approach and provides critical insights:

### Validated Architecture Decisions

1. **Three-Layer Architecture**: Arduino uses GSDK → Core → Libraries, matching our -sys → safe wrapper → HAL design
2. **Compile-Time Protocol Selection**: Arduino uses IDE menu variants; we use Cargo features (same concept)
3. **No Direct RAIL Exposure**: Arduino wraps RAIL with protocol-specific APIs (Matter, ezBLE), not raw RAIL access
4. **Bundled Blobs**: Arduino ships pre-compiled .a files in the core; we'll bundle in -sys crates

### Critical Implementation Patterns

**Matter + BLE Coexistence Warning**:
> "When Matter is selected, BLE is included but is not usable by the user, as the Matter SDK takes ownership of it."

**Memory Budget Reality Check**:
- **Matter Stack**: ~450 KB Flash, ~160 KB RAM (62% of EFR32MG24B220's 256 KB!)
- **BLE Stack**: ~120 KB Flash, ~36 KB RAM
- **Base HAL**: ~25 KB Flash, ~8 KB RAM

**Callback Mechanism**: Arduino uses FreeRTOS semaphores for C → Arduino event signaling. Our Rust approach will use `critical-section` + atomics/queues (lighter weight, no RTOS required).

### Revised Implementation Priority

Based on Arduino's resource warnings ("Matter is quite resource heavy"):

1. **D2-BLE** (Phase 1): Bluetooth 5.3 - Simpler, broader appeal, faster to ship
2. **D3-Thread** (Phase 2): OpenThread - Foundation for Matter
3. **D4-Matter** (Phase 3): Matter over Thread - Most complex, highest memory
4. **D5-Zigbee** (Phase 4): Zigbee 3.0 - Optional (Arduino doesn't have this yet)

**Rationale**: Start with BLE (most users), build foundation, then add complex stacks.

### API Style Lessons

Arduino's `MatterLightbulb` class demonstrates the right abstraction level:

```cpp
// Arduino: User-friendly, type-safe
MatterLightbulb bulb;
bulb.begin();
bulb.set_onoff(true);
```

Our Rust equivalent will follow similar patterns:

```rust
// Rust: Similar ergonomics + Result types
let mut bulb = MatterLightbulb::new()
    .device_name("Rust Bulb")
    .build()?;
bulb.set_onoff(true)?;
```

**Don't Do**: Expose raw RAIL functions like `RAIL_ConfigChannels()` to end users.

---


1. [Radio Hardware Architecture](#1-radio-hardware-architecture)
2. [Silicon Labs Software Stack](#2-silicon-labs-software-stack)
3. [FFI Integration Strategy](#3-ffi-integration-strategy)
4. [Implementation Phases](#4-implementation-phases)
5. [Protocol-Specific Implementation](#5-protocol-specific-implementation)
6. [Licensing and Legal](#6-licensing-and-legal)
7. [Testing and Validation](#7-testing-and-validation)
8. [Documentation Requirements](#8-documentation-requirements)
9. [Timeline and Milestones](#9-timeline-and-milestones)
10. [Risk Assessment](#10-risk-assessment)

---

## 1. Radio Hardware Architecture

## Revised Architecture: Generic HAL + Protocol-Specific BSP

### Core Principle

**HAL provides generic radio traits, BSP provides concrete protocol implementations.**

```
┌─────────────────────────────────────────────────────────────┐
│  User Application (examples, end-user code)                 │
├─────────────────────────────────────────────────────────────┤
│  xiao-mg24-bsp (Board Support Package)                      │
│  - Protocol-specific wrappers (BLE, Zigbee)                 │
│  - Concrete implementations of HAL traits                   │
│  - Board-specific configuration                             │
├─────────────────────────────────────────────────────────────┤
│  efr32mg24-hal (Hardware Abstraction Layer)                 │
│  - Generic radio traits (RadioBle, RadioZigbee)             │
│  - Generic types (GapAdvertiser, ZigbeeCoordinator)         │
│  - Peripheral ownership (Radio<STATE>)                      │
├─────────────────────────────────────────────────────────────┤
│  efr32mg24-{bluetooth,zigbee,thread}-sys                    │
│  - Unsafe FFI to protocol stacks                            │
│  - Blob linking (libbluetooth_*.a, libzigbee_*.a)           │
├─────────────────────────────────────────────────────────────┤
│  efr32mg24-rail-sys                                         │
│  - Unsafe FFI to RAIL library                               │
│  - Blob linking (librail_*.a)                               │
└─────────────────────────────────────────────────────────────┘
```

### HAL Layer (Generic Traits)

**crates/efr32mg24-hal/src/radio/mod.rs**:
```rust
/// Generic radio peripheral
pub struct Radio<STATE> {
    _state: PhantomData<STATE>,
}

/// Radio states (type-state pattern)
pub struct Disabled;
pub struct EnabledBle;
pub struct EnabledZigbee;
pub struct EnabledThread;

impl Radio<Disabled> {
    pub fn enable_ble(self) -> Radio<EnabledBle> { ... }
    pub fn enable_zigbee(self) -> Radio<EnabledZigbee> { ... }
}
```

**crates/efr32mg24-hal/src/radio/ble.rs** (Generic BLE traits):
```rust
/// Generic BLE GAP advertiser trait
pub trait BleAdvertiser {
    type Error;
    
    fn set_device_name(&mut self, name: &str) -> Result<(), Self::Error>;
    fn start_advertising(&mut self) -> Result<(), Self::Error>;
    fn stop_advertising(&mut self) -> Result<(), Self::Error>;
}

/// Generic BLE GATT server trait
pub trait BleGattServer {
    type Error;
    type ServiceHandle;
    
    fn add_service(&mut self, uuid: Uuid) -> Result<Self::ServiceHandle, Self::Error>;
    fn add_characteristic(&mut self, ...) -> Result<(), Self::Error>;
}
```

**crates/efr32mg24-hal/src/radio/zigbee.rs** (Generic Zigbee traits):
```rust
/// Generic Zigbee coordinator trait
pub trait ZigbeeCoordinator {
    type Error;
    type DeviceHandle;
    
    fn start_network(&mut self, pan_id: u16) -> Result<(), Self::Error>;
    fn permit_join(&mut self, duration_sec: u8) -> Result<(), Self::Error>;
    fn get_joined_devices(&self) -> &[Self::DeviceHandle];
}

/// Generic Zigbee endpoint trait
pub trait ZigbeeEndpoint {
    type Error;
    
    fn send_on_off(&mut self, state: bool) -> Result<(), Self::Error>;
    fn send_level(&mut self, level: u8) -> Result<(), Self::Error>;
}
```

### BSP Layer (Concrete Implementations)

**crates/xiao-mg24-bsp/src/radio/ble.rs**:
```rust
use efr32mg24_hal::radio::{Radio, EnabledBle, BleAdvertiser, BleGattServer};
use efr32mg24_bluetooth_sys as ffi;

/// XIAO MG24 BLE peripheral
pub struct XiaoBle {
    radio: Radio<EnabledBle>,
    // Internal state
}

impl XiaoBle {
    /// Create BLE peripheral (board-specific convenience)
    pub fn new(radio: Radio<EnabledBle>) -> Result<Self, Error> {
        // Board-specific initialization
        Ok(Self { radio })
    }
    
    /// Easy advertising (board-specific helper)
    pub fn advertise_as(name: &str) -> Result<Self, Error> {
        let mut ble = Self::new(...)?;
        ble.set_device_name(name)?;
        ble.start_advertising()?;
        Ok(ble)
    }
}

// Implement HAL traits
impl BleAdvertiser for XiaoBle {
    type Error = Error;
    
    fn set_device_name(&mut self, name: &str) -> Result<(), Error> {
        // Call FFI
        unsafe { ffi::sl_bt_gatt_server_write_attribute_value(...) }?;
        Ok(())
    }
    
    fn start_advertising(&mut self) -> Result<(), Error> {
        unsafe { ffi::sl_bt_advertiser_start(...) }?;
        Ok(())
    }
}

impl BleGattServer for XiaoBle {
    // ... concrete implementation
}
```

**crates/xiao-mg24-bsp/src/radio/zigbee.rs**:
```rust
use efr32mg24_hal::radio::{Radio, EnabledZigbee, ZigbeeCoordinator};
use efr32mg24_zigbee_sys as ffi;

/// XIAO MG24 Zigbee coordinator
pub struct XiaoZigbeeCoordinator {
    radio: Radio<EnabledZigbee>,
}

impl XiaoZigbeeCoordinator {
    pub fn new(radio: Radio<EnabledZigbee>) -> Result<Self, Error> {
        Ok(Self { radio })
    }
    
    /// Start coordinator with board defaults
    pub fn start_with_defaults() -> Result<Self, Error> {
        let mut coord = Self::new(...)?;
        coord.start_network(0x1234)?; // Default PAN ID
        Ok(coord)
    }
}

impl ZigbeeCoordinator for XiaoZigbeeCoordinator {
    type Error = Error;
    type DeviceHandle = ZigbeeDevice;
    
    fn start_network(&mut self, pan_id: u16) -> Result<(), Error> {
        unsafe { ffi::emberNetworkInit() }?;
        unsafe { ffi::emberFormNetwork(pan_id, ...) }?;
        Ok(())
    }
    
    fn permit_join(&mut self, duration_sec: u8) -> Result<(), Error> {
        unsafe { ffi::emberPermitJoining(duration_sec) }?;
        Ok(())
    }
}
```

### User Application

**crates/xiao-mg24-bsp/examples/zigbee_coordinator.rs**:
```rust
#![no_std]
#![no_main]

use xiao_mg24_bsp::{
    Board,
    radio::zigbee::XiaoZigbeeCoordinator,
};
use efr32mg24_hal::radio::ZigbeeCoordinator; // Import trait

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    
    // Board-specific convenience
    let mut coordinator = XiaoZigbeeCoordinator::start_with_defaults()
        .expect("Failed to start Zigbee");
    
    // Generic trait methods
    coordinator.permit_join(60).unwrap();
    
    loop {
        for device in coordinator.get_joined_devices() {
            // Handle devices
        }
    }
}
```

**crates/xiao-mg24-bsp/examples/ble_beacon.rs**:
```rust
use xiao_mg24_bsp::radio::ble::XiaoBle;
use efr32mg24_hal::radio::BleAdvertiser; // Import trait

#[entry]
fn main() -> ! {
    let board = Board::take().unwrap();
    
    // Board-specific convenience
    let mut ble = XiaoBle::advertise_as("XIAO Beacon")
        .expect("Failed to start BLE");
    
    // Generic trait methods work
    ble.stop_advertising().unwrap();
    ble.set_device_name("New Name").unwrap();
    ble.start_advertising().unwrap();
    
    loop { /* ... */ }
}
```

### Revised Implementation Priority

Based on user requirements:

1. **D1-RAIL-sys** (Foundation) - Internal use only
2. **D2-BLE** (Phase 1: Most Important) - Bluetooth 5.3 LE
3. **D3-Zigbee** (Phase 2: Second Priority) - Zigbee 3.0 coordinator/router/end-device
4. **D4-Thread** (Phase 3: Optional) - Thread 1.3 for future Matter support
5. **D5-Matter** (Phase 4: Optional) - Matter over Thread (if needed)

**Rationale**:
- BLE + Zigbee cover 90% of IoT sensor use cases
- Thread/Matter are more smart-home focused (lower priority)
- Generic HAL allows future protocols without breaking changes

## 2. Silicon Labs Software Stack

### 2.1 Gecko SDK Components

**Gecko SDK 4.4.5+** (Latest as of December 2025):

| Component | Type | Size | License | Status |
|-----------|------|------|---------|--------|
| **RAIL** (Radio) | Binary blob | ~100 KB | MSLA | Required (foundation) |
| **Bluetooth Stack** | Binary blob | ~200 KB | MSLA | Phase D2 |
| **Zigbee Stack** | Binary blob | ~250 KB | MSLA | Phase D3 |
| **OpenThread** | Open source | ~150 KB | BSD-3-Clause | Phase D4 |
| **Matter** | Open source + blobs | ~300 KB | Apache-2.0 + MSLA | Phase D4 |
| **Crypto (PSA)** | Binary blob | ~50 KB | MSLA | Integrated with stacks |

**Total Flash Required**: ~550 KB for full multiprotocol support (leaves ~1 MB for application)

### 2.2 RAIL API Versions

**RAIL 2.x** (Current, stable):
- API: `rail.h`, `rail_types.h`, `rail_chip_specific.h`
- Version: 2.19.2 (September 2025)
- Status: **Recommended** for new projects
- Deprecation: December 2025 (but will be supported until 2027+)

**RAIL 3.0** (Future):
- API: `sl_rail_*.h` files
- Status: In evaluation, not production-ready
- Migration: Can defer to Phase D5+

**Decision**: Use RAIL 2.x API for initial implementation.

### 2.3 Blob Extraction Process

**Source**: Gecko SDK 4.4.5 from GitHub

```bash
# 1. Clone Gecko SDK (or download release)
git clone https://github.com/SiliconLabs/gecko_sdk.git
cd gecko_sdk
git checkout v4.4.5

# 2. Extract RAIL blobs for EFR32MG24B220
cp platform/radio/rail_lib/autogen/librail_release/librail_efr32xg24_gcc_release.a \
   crates/efr32mg24-rail-sys/blobs/efr32mg24b220/

# 3. Extract Bluetooth blobs
cp protocol/bluetooth/lib/EFR32MG24/GCC/libbluetooth.a \
   crates/efr32mg24-bluetooth-sys/blobs/

# 4. Extract Zigbee blobs
cp protocol/zigbee/stack/release/gcc/libzigbee-stack-cortexm3-gcc-efr32mg24-rail.a \
   efr32mg24-zigbee-sys/blobs/

# 5. Extract OpenThread (note: source available!)
cp protocol/openthread/platform-abstraction/efr32/*.c \
   efr32mg24-openthread/vendor/

# 6. Extract Matter (combination of source + blobs)
cp third_party/matter_support/*.a \
   efr32mg24-matter-sys/blobs/
```

**IMPORTANT**: Document exact Gecko SDK version in README for reproducibility.

---

## 3. FFI Integration Strategy

### 3.1 Crate Architecture (Three-Layer Pattern)

Following **nrf-softdevice** and **esp-idf-hal** patterns:

```text
┌────────────────────────────────────────────────────────┐
│  Layer 1: -sys Crates (Unsafe FFI Bindings)           │
│  - efr32mg24-rail-sys                                  │
│  - efr32mg24-bluetooth-sys                             │
│  - efr32mg24-zigbee-sys                                │
│  - efr32mg24-openthread-sys                            │
│  - efr32mg24-matter-sys                                │
│                                                         │
│  Generated with bindgen from C headers                 │
│  Links binary blobs via build.rs                       │
│  100% unsafe, no abstraction                           │
└────────────────────────────────────────────────────────┘
                          ▲
                          │ depends on
┌────────────────────────────────────────────────────────┐
│  Layer 2: Safe Rust Wrappers                          │
│  - efr32mg24-rail                                      │
│  - efr32mg24-bluetooth                                 │
│  - efr32mg24-zigbee                                    │
│  - efr32mg24-openthread                                │
│  - efr32mg24-matter                                    │
│                                                         │
│  Safe abstractions, ownership, error handling          │
│  Resource management (Drop implementations)            │
│  Type-safe APIs                                        │
└────────────────────────────────────────────────────────┘
                          ▲
                          │ depends on
┌────────────────────────────────────────────────────────┐
│  Layer 3: HAL Integration                             │
│  - efr32mg24-hal::radio                                │
│                                                         │
│  Integrates radio with existing HAL:                   │
│  - Clock management (HFXO required)                    │
│  - Power management (EMU integration)                  │
│  - DMA integration (for async)                         │
│  - embedded-hal traits (where applicable)              │
└────────────────────────────────────────────────────────┘
```

### 3.2 Build System Integration

**build.rs Pattern** (for all -sys crates):

```rust
// crates/efr32mg24-rail-sys/build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Ensure we're building for ARM Cortex-M33
    if !target.starts_with("thumbv8m.main") {
        panic!("This crate only supports thumbv8m.main-none-eabihf");
    }

    // Link RAIL blob
    let blob_path = "blobs/efr32mg24b220";
    println!("cargo:rustc-link-search=native={}", blob_path);
    println!("cargo:rustc-link-lib=static=rail_efr32xg24_gcc_release");

    // Link EMLIB dependencies (Silicon Labs peripheral library)
    println!("cargo:rustc-link-lib=static=emlib");

    // Generate bindings from RAIL headers
    let bindings = bindgen::Builder::default()
        .header("vendor/rail/rail.h")
        .header("vendor/rail/rail_types.h")
        .use_core()
        .ctypes_prefix("cty")
        .derive_default(true)
        .prepend_enum_name(false)
        .generate()
        .expect("Unable to generate RAIL bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Rerun if headers change
    println!("cargo:rerun-if-changed=vendor/rail/rail.h");
    println!("cargo:rerun-if-changed=vendor/rail/rail_types.h");
}
```

### 3.3 Memory Layout Adjustments

**memory.x modifications** for radio stack requirements:

```text
/* crates/efr32mg24-hal/memory.x - Updated for radio support */

MEMORY
{
  /* Flash: 1536 KB total */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1536K

  /* RAM: 256 KB total, partitioned for radio */
  RAM       : ORIGIN = 0x20000000, LENGTH = 160K  /* Application heap/stack */
  RAIL_RAM  : ORIGIN = 0x20028000, LENGTH = 32K   /* RAIL working memory */
  BT_RAM    : ORIGIN = 0x20030000, LENGTH = 50K   /* Bluetooth heap */
  ZB_RAM    : ORIGIN = 0x2003C800, LENGTH = 14K   /* Zigbee (optional) */

  /* Note: Cannot use full multiprotocol simultaneously without careful planning */
}

/* Stack grows down from top of RAM region */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
_stack_size = 32K;  /* Increased from 16K for radio operations */

/* RAIL requires minimum 32KB combined stack+heap for all operations */
```

**Trade-offs**:
- Bluetooth-only: 160K app RAM + 50K BT RAM = 210K usable
- Zigbee-only: 160K app RAM + 32K RAIL RAM = 192K usable
- Thread-only: 160K app RAM + 32K RAIL RAM = 192K usable
- Multiprotocol (BLE + Thread): Requires dynamic switching, cannot run simultaneously

### 3.4 Callback Mechanisms

**C → Rust Callback Pattern** (critical for event handling):

```rust
// efr32mg24-rail/src/callbacks.rs

use efr32mg24_rail_sys as ffi;
use core::sync::atomic::{AtomicPtr, Ordering};
use critical_section::Mutex;

// Type-erased callback storage (safe because single-threaded embedded)
static RAIL_EVENT_CALLBACK: Mutex<AtomicPtr<u8>> = Mutex::new(AtomicPtr::new(core::ptr::null_mut()));

// C-compatible wrapper function
unsafe extern "C" fn rail_event_callback_wrapper(
    handle: ffi::RAIL_Handle_t,
    events: u64,
) {
    critical_section::with(|cs| {
        let callback_ptr = RAIL_EVENT_CALLBACK.borrow(cs).load(Ordering::Acquire);
        if !callback_ptr.is_null() {
            let callback: &mut dyn FnMut(RailEvents) = &mut *(callback_ptr as *mut _);
            let rust_events = RailEvents::from_bits_truncate(events);
            callback(rust_events);
        }
    });
}

// Safe Rust API for setting callback
pub fn set_rail_callback<F>(callback: &mut F)
where
    F: FnMut(RailEvents) + 'static
{
    critical_section::with(|cs| {
        let callback_ptr = callback as *mut F as *mut u8;
        RAIL_EVENT_CALLBACK.borrow(cs).store(callback_ptr, Ordering::Release);
    });
}
```

**Safety Requirements**:
1. Callback must be valid for lifetime of RAIL handle
2. Only one callback per peripheral (enforced by ownership)
3. Critical sections prevent data races
4. Static storage required (no heap allocation in callbacks)

---

## 4. Implementation Phases

## 4. Implementation Phases

### Overview

Radio implementation will follow a **staged rollout** across 5 phases, prioritizing most-used protocols first:

| **Phase** | **Identifier** | **Protocol** | **Priority** | **Estimated Effort** |
|-----------|----------------|--------------|--------------|---------------------|
| Foundation | D1-RAIL-sys | RAIL Library (internal) | Critical | 38-57 hours (5-7 weeks) |
| Phase 1 | D2-BLE | Bluetooth 5.3 LE | **Highest** | 60-78 hours (8-10 weeks) |
| Phase 2 | D3-Zigbee | Zigbee 3.0 | **High** | 52-68 hours (7-9 weeks) |
| Phase 3 | D4-Thread | Thread 1.3 (OpenThread) | Medium | 45-55 hours (6-7 weeks) |
| Phase 4 | D5-Matter | Matter over Thread | Optional | 58-72 hours (8-10 weeks) |

**Total**: 253-330 hours (34-45 weeks) for all 5 phases

**Critical Path**: D1-RAIL → D2-BLE → D3-Zigbee (covers 90% of IoT sensor use cases)

### Phase Rationale

**Why BLE First?**
- Most widely used protocol (smartphones, laptops, IoT)
- Simpler than Zigbee/Thread (no mesh networking complexity)
- Arduino pattern validated
- Fastest time to value

**Why Zigbee Second?**
- **User requirement**: Critical for industrial IoT sensor applications
- Covers home automation, industrial monitoring, smart meters
- Mature ecosystem (Zigbee 3.0 standardized)
- Complements BLE (Zigbee for mesh, BLE for direct device access)

**Why Thread/Matter Later?**
- More niche (smart home focused)
- Higher complexity (IPv6, CoAP)
- Higher memory requirements (Matter: 62% of RAM!)
- Can leverage Thread for Matter if needed

---

### D1-RAIL-sys: RAIL Foundation (Internal Use Only)

**Identifier**: `D1-RAIL.1` (Stage 1: Memory-to-memory, etc. for substages)

**Purpose**: Create unsafe FFI bindings to RAIL library as **internal dependency** for protocol stacks.

**Duration**: 5-7 weeks (38-57 hours)

**CRITICAL**: RAIL is NOT exposed in HAL public API. It's an internal dependency for `-sys` crates.

#### Deliverables

1. **efr32mg24-rail-sys** crate:
   - `build.rs`: Link `librail_efr32xg24_gcc_release.a`
   - `src/bindings.rs`: Auto-generated FFI bindings (bindgen)
   - Binary blob bundled in crate

2. **Minimal safe wrapper** (efr32mg24-rail, internal):
   - RAIL handle management
   - Channel configuration
   - TX/RX primitives (used by BLE/Zigbee stacks)
   - NOT public API

#### Acceptance Criteria

- [ ] RAIL blob linked successfully
- [ ] Bindings compile without warnings
- [ ] Can initialize RAIL handle (internal test)
- [ ] Can configure 2.4 GHz channel (internal test)
- [ ] No public API (module is `pub(crate)`)

---

### D2-BLE: Bluetooth 5.3 LE (Phase 1)

**Identifier**: `D2-BLE.1`, `D2-BLE.2`, etc. (substages for features)

**Priority**: **Highest** (most requested feature)

**Duration**: 8-10 weeks (60-78 hours)

#### Architecture

**Generic HAL Traits** (crates/efr32mg24-hal/src/radio/ble.rs):
```rust
/// Generic BLE advertiser trait
pub trait BleAdvertiser {
    type Error;
    fn set_device_name(&mut self, name: &str) -> Result<(), Self::Error>;
    fn start_advertising(&mut self) -> Result<(), Self::Error>;
    fn stop_advertising(&mut self) -> Result<(), Self::Error>;
}

/// Generic GATT server trait
pub trait BleGattServer {
    type Error;
    type ServiceHandle;
    fn add_service(&mut self, uuid: Uuid) -> Result<Self::ServiceHandle, Self::Error>;
}
```

**Concrete Implementation** (crates/xiao-mg24-bsp/src/radio/ble.rs):
```rust
pub struct XiaoBle {
    radio: Radio<EnabledBle>,
}

impl BleAdvertiser for XiaoBle { /* ... */ }
impl BleGattServer for XiaoBle { /* ... */ }
```

#### Substages

**D2-BLE.1: GAP Foundation** (2-3 weeks, 15-22 hours)
- [x] efr32mg24-bluetooth-sys crate (FFI bindings)
- [ ] Generic BLE traits in HAL
- [ ] Radio<EnabledBle> type-state
- [ ] Basic advertising (non-connectable beacons)
- [ ] Device name, TX power configuration
- [ ] Example: `ble_beacon.rs` (BSP)

**D2-BLE.2: GAP Connections** (2-3 weeks, 15-22 hours)
- [ ] Connection establishment (peripheral role)
- [ ] Connection parameters (interval, latency, timeout)
- [ ] Connection event callbacks
- [ ] Disconnection handling
- [ ] Example: `ble_peripheral.rs` (BSP)

**D2-BLE.3: GATT Server** (3-4 weeks, 22-28 hours)
- [ ] Service/characteristic definition
- [ ] Attribute read/write handlers
- [ ] Notifications/indications
- [ ] Custom UUIDs
- [ ] Example: `ble_gatt_server.rs` (heart rate sensor, BSP)

**D2-BLE.4: Central Role** (1-2 weeks, 8-12 hours) - Optional
- [ ] Scanning for peripherals
- [ ] Connection initiation
- [ ] GATT client operations
- [ ] Example: `ble_central.rs` (BSP)

#### Deliverables

1. **efr32mg24-bluetooth-sys** crate:
   - BGAPI FFI bindings
   - `libbluetooth_efr32xg24_gcc_release.a` blob

2. **Generic BLE traits** (HAL):
   - `BleAdvertiser`, `BleGattServer`, `BleGattClient`
   - Type-state: `Radio<EnabledBle>`

3. **Concrete implementation** (BSP):
   - `XiaoBle` struct
   - Trait implementations
   - Board-specific helpers (`advertise_as()`, etc.)

4. **Examples** (BSP):
   - `ble_beacon.rs` - Non-connectable advertising
   - `ble_peripheral.rs` - Connectable peripheral
   - `ble_gatt_server.rs` - Heart rate sensor GATT service
   - `ble_central.rs` - Scanner and central role

---

### D3-Zigbee: Zigbee 3.0 (Phase 2)

**Identifier**: `D3-Zigbee.1`, `D3-Zigbee.2`, etc.

**Priority**: **High** (user requirement: 2nd most important)

**Duration**: 7-9 weeks (52-68 hours)

#### Architecture

**Generic HAL Traits** (crates/efr32mg24-hal/src/radio/zigbee.rs):
```rust
/// Generic Zigbee coordinator trait
pub trait ZigbeeCoordinator {
    type Error;
    type DeviceHandle;
    fn start_network(&mut self, pan_id: u16) -> Result<(), Self::Error>;
    fn permit_join(&mut self, duration_sec: u8) -> Result<(), Self::Error>;
    fn get_joined_devices(&self) -> &[Self::DeviceHandle];
}

/// Generic Zigbee endpoint trait (ZCL)
pub trait ZigbeeEndpoint {
    type Error;
    fn send_on_off(&mut self, dest: u16, state: bool) -> Result<(), Self::Error>;
    fn send_level(&mut self, dest: u16, level: u8) -> Result<(), Self::Error>;
}
```

**Concrete Implementation** (crates/xiao-mg24-bsp/src/radio/zigbee.rs):
```rust
pub struct XiaoZigbeeCoordinator {
    radio: Radio<EnabledZigbee>,
}

impl ZigbeeCoordinator for XiaoZigbeeCoordinator { /* ... */ }
```

#### Substages

**D3-Zigbee.1: EmberZNet Foundation** (2-3 weeks, 16-22 hours)
- [ ] efr32mg24-zigbee-sys crate (FFI to EmberZNet stack)
- [ ] Generic Zigbee traits in HAL
- [ ] Radio<EnabledZigbee> type-state
- [ ] Network formation (coordinator)
- [ ] Permit join control
- [ ] Example: `zigbee_coordinator.rs` (BSP)

**D3-Zigbee.2: ZCL Foundation** (2-3 weeks, 16-22 hours)
- [ ] Zigbee Cluster Library (ZCL) foundation
- [ ] On/Off cluster (0x0006)
- [ ] Level Control cluster (0x0008)
- [ ] Basic cluster (0x0000)
- [ ] Example: `zigbee_light_controller.rs` (BSP)

**D3-Zigbee.3: Router/End Device** (2-3 weeks, 14-18 hours)
- [ ] Router role (network extension)
- [ ] End device role (battery-powered sensors)
- [ ] Sleepy end device support
- [ ] Example: `zigbee_temperature_sensor.rs` (BSP)

**D3-Zigbee.4: Advanced ZCL** (1-2 weeks, 6-12 hours) - Optional
- [ ] Color Control cluster (0x0300) - smart bulbs
- [ ] Thermostat cluster (0x0201)
- [ ] IAS Zone cluster (0x0500) - security sensors
- [ ] Example: `zigbee_smart_bulb.rs` (BSP)

#### Deliverables

1. **efr32mg24-zigbee-sys** crate:
   - EmberZNet FFI bindings
   - `libzigbee_stack_efr32xg24_gcc_release.a` blob

2. **Generic Zigbee traits** (HAL):
   - `ZigbeeCoordinator`, `ZigbeeRouter`, `ZigbeeEndpoint`
   - Type-state: `Radio<EnabledZigbee>`

3. **Concrete implementation** (BSP):
   - `XiaoZigbeeCoordinator`, `XiaoZigbeeRouter`, `XiaoZigbeeEndDevice`
   - Trait implementations
   - ZCL cluster helpers

4. **Examples** (BSP):
   - `zigbee_coordinator.rs` - Network coordinator
   - `zigbee_light_controller.rs` - Smart light switch (on/off, dimming)
   - `zigbee_temperature_sensor.rs` - Battery-powered sensor
   - `zigbee_smart_bulb.rs` - RGB smart bulb

---

### D4-Thread: Thread 1.3 (Phase 3)

**Identifier**: `D4-Thread.1`, etc.

**Priority**: Medium (foundation for Matter, lower priority than BLE/Zigbee)

**Duration**: 6-7 weeks (45-55 hours)

**Key Advantage**: OpenThread is **open source** (BSD 3-Clause), unlike BLE/Zigbee stacks!

#### Substages

**D4-Thread.1: OpenThread Foundation** (3-4 weeks, 25-30 hours)
- [ ] efr32mg24-openthread crate (no -sys needed, it's open source!)
- [ ] Generic Thread traits in HAL
- [ ] Network commissioning (Thread joiner)
- [ ] IPv6 networking
- [ ] Example: `thread_router.rs` (BSP)

**D4-Thread.2: CoAP and Border Router** (2-3 weeks, 15-20 hours)
- [ ] CoAP client/server
- [ ] Thread Border Router configuration
- [ ] DNS-SD service discovery
- [ ] Example: `thread_coap_sensor.rs` (BSP)

**D4-Thread.3: Sleepy End Device** (1 week, 5-8 hours)
- [ ] Low-power sleepy end device role
- [ ] Data polling
- [ ] Example: `thread_sleepy_sensor.rs` (BSP)

#### Deliverables

1. **efr32mg24-openthread** crate:
   - OpenThread source compilation (no blob!)
   - Platform abstraction layer (PAL) for EFR32MG24

2. **Generic Thread traits** (HAL):
   - `ThreadRouter`, `ThreadEndDevice`
   - IPv6/UDP/CoAP abstractions

3. **Concrete implementation** (BSP):
   - `XiaoThreadRouter`, `XiaoThreadEndDevice`

4. **Examples** (BSP):
   - `thread_router.rs` - Thread router node
   - `thread_coap_sensor.rs` - CoAP temperature sensor
   - `thread_sleepy_sensor.rs` - Low-power sensor

---

### D5-Matter: Matter over Thread (Phase 4)

**Identifier**: `D5-Matter.1`, etc.

**Priority**: Optional (high memory requirements, niche use case)

**Duration**: 8-10 weeks (58-72 hours)

**Critical Constraint**: Requires ~160 KB RAM (62% of 256 KB total!)

#### Substages

**D5-Matter.1: Matter Foundation** (4-5 weeks, 30-38 hours)
- [ ] efr32mg24-matter-sys crate
- [ ] Device commissioning via QR code/BLE
- [ ] Generic Matter traits in HAL
- [ ] Cluster model foundation
- [ ] Example: `matter_lightbulb.rs` (BSP)

**D5-Matter.2: Matter Clusters** (3-4 weeks, 22-28 hours)
- [ ] On/Off cluster
- [ ] Level Control cluster
- [ ] Color Control cluster
- [ ] Example: `matter_smart_bulb.rs` (BSP)

**D5-Matter.3: Matter + BLE Coexistence** (1-2 weeks, 6-10 hours)
- [ ] BLE release after commissioning
- [ ] Simultaneous operation
- [ ] Example: `matter_with_ble.rs` (BSP)

#### Deliverables

1. **efr32mg24-matter-sys** crate:
   - Matter SDK FFI bindings
   - `libmatter_efr32xg24_gcc_release.a` blob

2. **Generic Matter traits** (HAL):
   - `MatterDevice`, `MatterCluster`

3. **Concrete implementation** (BSP):
   - `XiaoMatterLightbulb`, `XiaoMatterSwitch`

4. **Examples** (BSP):
   - `matter_lightbulb.rs` - Matter smart bulb
   - `matter_switch.rs` - Matter light switch
   - `matter_with_ble.rs` - Matter + BLE coexistence

---

### Implementation Order Summary

```
D1-RAIL-sys (5-7 weeks)
    ↓
D2-BLE.1 → D2-BLE.2 → D2-BLE.3 → D2-BLE.4 (8-10 weeks)
    ↓
D3-Zigbee.1 → D3-Zigbee.2 → D3-Zigbee.3 → D3-Zigbee.4 (7-9 weeks)
    ↓ (Optional from here)
D4-Thread.1 → D4-Thread.2 → D4-Thread.3 (6-7 weeks)
    ↓
D5-Matter.1 → D5-Matter.2 → D5-Matter.3 (8-10 weeks)
```

**Critical Path** (required): D1 → D2 → D3 (20-26 weeks, ~150-193 hours)
**Full Implementation** (optional): All phases (34-43 weeks, ~253-330 hours)

---

## 5. Protocol-Specific Implementation

### 5.1 Bluetooth 5.3 LE Complete Implementation

#### 5.1.1 Architecture

```text
┌─────────────────────────────────────────────────┐
│  Bluetooth Application                          │
│  - Peripheral (sensor device)                   │
│  - Central (hub device)                         │
│  - Observer (scanner)                           │
│  - Broadcaster (beacon)                         │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  GAP (Generic Access Profile)                   │
│  - Advertising (connectable, scannable)         │
│  - Discovery (active, passive scanning)         │
│  - Connection management                        │
│  - LE Privacy (resolvable addresses)            │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  GATT (Generic Attribute Profile)               │
│  - Server: Services + Characteristics           │
│  - Client: Service discovery, read/write        │
│  - Notifications/Indications                    │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  SM (Security Manager)                          │
│  - Pairing (Just Works, Passkey, OOB)           │
│  - Bonding (key storage)                        │
│  - LE Secure Connections                        │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  Link Layer (handled by Bluetooth controller)  │
│  - libbluetooth.a (Silicon Labs blob)           │
└─────────────────────────────────────────────────┘
```

#### 5.1.2 API Design (Safe Rust)

```rust
// efr32mg24-bluetooth/src/gap.rs

pub struct BluetoothStack {
    controller: BluetoothController,
    address: BleAddress,
}

impl BluetoothStack {
    /// Initialize Bluetooth stack
    pub fn new(rail: RailHandle) -> Result<Self, BtError> {
        // Initialize controller with RAIL handle
        let controller = BluetoothController::init(rail)?;

        // Get device address (from factory calibration)
        let address = controller.get_address()?;

        Ok(Self { controller, address })
    }

    /// Start advertising as peripheral
    pub fn advertise(&mut self, params: AdvertisingParams) -> Result<(), BtError> {
        self.controller.set_advertising_data(&params.data)?;
        self.controller.start_advertising(params.interval, params.tx_power)?;
        Ok(())
    }

    /// Start scanning as central
    pub fn scan(&mut self, params: ScanParams) -> Result<ScanResult, BtError> {
        self.controller.start_scan(params.window, params.interval)?;
        // Returns iterator over discovered devices
    }
}

// Example usage
let mut bt = BluetoothStack::new(rail)?;

// Advertise as iBeacon
let beacon = AdvertisingParams::ibeacon(
    uuid: [0x01, 0x12, 0x23, ...],
    major: 1,
    minor: 100,
);
bt.advertise(beacon)?;
```

#### 5.1.3 GATT Server Example

```rust
// efr32mg24-bluetooth/src/gatt.rs

pub struct GattServer {
    services: Vec<GattService>,
}

impl GattServer {
    pub fn add_service(&mut self, service: GattService) -> ServiceHandle {
        self.services.push(service);
        ServiceHandle(self.services.len() - 1)
    }
}

// Example: Battery Service
let mut gatt = GattServer::new();

let battery_service = GattService::new(UUID_BATTERY_SERVICE)
    .add_characteristic(GattCharacteristic {
        uuid: UUID_BATTERY_LEVEL,
        properties: CharProperties::READ | CharProperties::NOTIFY,
        value: vec![85], // 85% battery
    });

let svc_handle = gatt.add_service(battery_service);

// Update battery level
gatt.update_characteristic(svc_handle, UUID_BATTERY_LEVEL, vec![80])?;
gatt.notify_subscribers(svc_handle, UUID_BATTERY_LEVEL)?;
```

### 5.2 Zigbee 3.0 Complete Implementation

#### 5.2.1 Architecture

```text
┌─────────────────────────────────────────────────┐
│  Zigbee Application                              │
│  - Coordinator (network formation)               │
│  - Router (message routing)                      │
│  - End Device (data collection)                  │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  ZCL (Zigbee Cluster Library)                    │
│  - On/Off cluster (0x0006)                       │
│  - Level Control cluster (0x0008)                │
│  - Color Control cluster (0x0300)                │
│  - Temperature Measurement (0x0402)              │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  ZDO (Zigbee Device Object)                      │
│  - Network management                            │
│  - Binding table                                 │
│  - Service discovery                             │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  APS (Application Support Sublayer)              │
│  - Group addressing                              │
│  - Fragmentation/reassembly                      │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  NWK (Network Layer)                             │
│  - Routing, address assignment                   │
│  - libzigbee-stack.a (Silicon Labs blob)         │
└─────────────────────────────────────────────────┘
```

#### 5.2.2 API Design

```rust
// efr32mg24-zigbee/src/lib.rs

pub struct ZigbeeStack {
    role: DeviceRole,
    network_state: NetworkState,
}

pub enum DeviceRole {
    Coordinator,
    Router,
    EndDevice { sleepy: bool },
}

impl ZigbeeStack {
    /// Form Zigbee network as coordinator
    pub fn form_network(&mut self, config: NetworkConfig) -> Result<NetworkId, ZbError> {
        // Configure as coordinator
        // Set PAN ID, channel, security keys
        // Start network formation
    }

    /// Join existing network as router or end device
    pub fn join_network(&mut self, params: JoinParams) -> Result<(), ZbError> {
        // Scan for networks
        // Association request
        // Network key negotiation
    }
}

// Example: Zigbee light bulb
let mut zb = ZigbeeStack::new(DeviceRole::EndDevice { sleepy: false })?;
zb.join_network(JoinParams::default())?;

// Register On/Off cluster
let on_off = OnOffCluster::new();
zb.register_cluster(on_off)?;

// Handle incoming commands
loop {
    if let Some(cmd) = zb.poll_command()? {
        match cmd {
            ZclCommand::OnOff(OnOffCommand::On) => {
                // Turn on light
                led.set_high();
                zb.send_response(ZclStatus::Success)?;
            }
            _ => {}
        }
    }
}
```

### 5.3 Thread 1.3 Complete Implementation

#### 5.3.1 Architecture (OpenThread)

**Key Advantage**: OpenThread is **open source** (BSD-3-Clause), reducing dependency on proprietary blobs!

```text
┌─────────────────────────────────────────────────┐
│  Thread Application                              │
│  - Leader (network formation)                    │
│  - Router (routing)                              │
│  - End Device (data)                             │
│  - Sleepy End Device (low power)                 │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  OpenThread Stack (Open Source!)                 │
│  - IPv6 networking                               │
│  - CoAP application protocol                     │
│  - Thread Commissioning                          │
│  - Source: github.com/openthread/openthread      │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  Platform Abstraction Layer (PAL)                │
│  - Radio driver (uses RAIL)                      │
│  - Alarm/timer                                   │
│  - Random number generator                       │
│  - Non-volatile storage                          │
└─────────────────────────────────────────────────┘
```

#### 5.3.2 API Design

```rust
// efr32mg24-openthread/src/lib.rs

pub struct ThreadStack {
    instance: OtInstance,
    role: ThreadRole,
}

pub enum ThreadRole {
    Leader,
    Router,
    EndDevice,
    SleepyEndDevice,
}

impl ThreadStack {
    /// Initialize Thread stack
    pub fn new(rail: RailHandle, config: ThreadConfig) -> Result<Self, ThreadError> {
        let instance = OtInstance::init(rail)?;
        instance.set_network_name(&config.network_name)?;
        instance.set_pan_id(config.pan_id)?;

        Ok(Self {
            instance,
            role: config.role,
        })
    }

    /// Start Thread network
    pub fn start(&mut self) -> Result<(), ThreadError> {
        self.instance.thread_start()?;

        // Wait for role assignment
        while self.instance.get_state() != ThreadState::Leader {
            // Poll for state change
        }

        Ok(())
    }

    /// Send CoAP request
    pub fn coap_request(&mut self, uri: &str, data: &[u8]) -> Result<CoapResponse, ThreadError> {
        self.instance.coap_send_request(uri, data)
    }
}
```

### 5.4 Matter Complete Implementation

#### 5.4.1 Architecture

Matter builds on Thread (or WiFi) for IP connectivity:

```text
┌─────────────────────────────────────────────────┐
│  Matter Application Layer                        │
│  - Device types: Light, Switch, Sensor, etc.     │
│  - Clusters: On/Off, Level, Temperature          │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  Matter Data Model                               │
│  - Interaction model (read/write/subscribe)      │
│  - Access control                                │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  Matter Commissioning                            │
│  - QR code generation                            │
│  - BLE commissioning (initial setup)             │
│  - Network credential provisioning               │
└─────────────────────────────────────────────────┘
                    ▼
┌─────────────────────────────────────────────────┐
│  Transport Layer                                 │
│  - Thread (primary for EFR32MG24)                │
│  - BLE (commissioning only)                      │
└─────────────────────────────────────────────────┘
```

#### 5.4.2 API Design

```rust
// efr32mg24-matter/src/lib.rs

pub struct MatterDevice {
    thread: ThreadStack,
    device_type: MatterDeviceType,
    commissioning: CommissioningState,
}

pub enum MatterDeviceType {
    OnOffLight,
    DimmableLight,
    ColorLight,
    TemperatureSensor,
    DoorLock,
    Thermostat,
}

impl MatterDevice {
    /// Create Matter device
    pub fn new(
        thread: ThreadStack,
        device_type: MatterDeviceType,
        vendor_id: u16,
        product_id: u16,
    ) -> Result<Self, MatterError> {
        let commissioning = CommissioningState::NotCommissioned;

        Ok(Self {
            thread,
            device_type,
            commissioning,
        })
    }

    /// Generate QR code for commissioning
    pub fn generate_qr_code(&self) -> String {
        // Generate Matter QR code payload
        format!("MT:Y.K9042C00KA0648G00") // Example
    }

    /// Start commissioning via BLE
    pub fn start_commissioning(&mut self, ble: &mut BluetoothStack) -> Result<(), MatterError> {
        // Start BLE advertising for Matter commissioning
        ble.advertise(AdvertisingParams::matter_commissioning())?;

        // Handle commissioning protocol
        // Receive network credentials
        // Join Thread network
        // Complete commissioning

        self.commissioning = CommissioningState::Commissioned;
        Ok(())
    }

    /// Handle Matter interaction (read/write/subscribe)
    pub fn handle_interaction(&mut self, interaction: Interaction) -> Result<Response, MatterError> {
        match interaction {
            Interaction::Read { cluster_id, attribute_id } => {
                // Read attribute value
            }
            Interaction::Write { cluster_id, attribute_id, value } => {
                // Write attribute value
            }
            Interaction::Subscribe { cluster_id, attribute_id } => {
                // Subscribe to attribute changes
            }
        }
    }
}

// Example: Matter light bulb
let thread = ThreadStack::new(rail, ThreadConfig::default())?;
let mut light = MatterDevice::new(
    thread,
    MatterDeviceType::OnOffLight,
    vendor_id: 0xFFF1, // Test vendor ID
    product_id: 0x8000,
)?;

// Generate QR code (print to console or display on screen)
let qr_code = light.generate_qr_code();
println!("Scan this QR code: {}", qr_code);

// Start commissioning
light.start_commissioning(&mut ble)?;

// Handle interactions
loop {
    if let Some(interaction) = light.poll_interaction()? {
        light.handle_interaction(interaction)?;
    }
}
```

---

## 6. Licensing and Legal

### 6.1 Licensing Matrix

| Component | License | Redistribution | Commercial Use | Notes |
|-----------|---------|----------------|----------------|-------|
| **RAIL blob** | MSLA | ✅ (for SiLabs devices) | ✅ | Proprietary |
| **Bluetooth blob** | MSLA | ✅ | ✅ | Proprietary |
| **Zigbee blob** | MSLA | ✅ | ✅ | Proprietary |
| **OpenThread** | BSD-3-Clause | ✅ | ✅ | Open source! |
| **Matter** | Apache-2.0 | ✅ | ✅ | Open source core |
| **RAIL headers** | Zlib | ✅ | ✅ | Permissive |
| **Our Rust code** | MIT OR Apache-2.0 | ✅ | ✅ | Dual license |

### 6.2 MSLA Key Terms

Silicon Labs **Master Software License Agreement** (MSLA):

**Permissions**:
- ✅ Use with Silicon Labs devices (EFR32MG24)
- ✅ Redistribute blobs with applications
- ✅ Commercial products allowed
- ✅ Modification of wrapper code (not blobs)

**Restrictions**:
- ❌ Cannot use with non-Silicon Labs hardware
- ❌ No reverse engineering of blobs
- ❌ No warranty from Silicon Labs

**Important**: MSLA is **compatible** with MIT/Apache-2.0 for wrapper code, following precedent from nrf-softdevice and esp-idf-hal.

### 6.3 Licensing Strategy

**Our Repository**:

1. **Core HAL** (efr32mg24-hal): `MIT OR Apache-2.0`
   - No radio code, pure Rust
   - Standard embedded Rust license

2. **Radio -sys crates**: `Zlib AND MSLA`
   - Bindings generated from Zlib-licensed headers
   - Links MSLA-licensed blobs
   - Clear NOTICE file

3. **Radio safe wrappers**: `MIT OR Apache-2.0`
   - Our original Rust code
   - Depends on -sys (which is MSLA)
   - Users opt-in via feature flags

**Feature Flags**:

```toml
# Cargo.toml
[features]
default = []  # No radio by default
radio-rail = ["efr32mg24-rail-sys", "efr32mg24-rail"]
radio-bluetooth = ["radio-rail", "efr32mg24-bluetooth-sys", "efr32mg24-bluetooth"]
radio-zigbee = ["radio-rail", "efr32mg24-zigbee-sys", "efr32mg24-zigbee"]
radio-thread = ["radio-rail", "efr32mg24-openthread"]
radio-matter = ["radio-thread", "efr32mg24-matter"]
radio-all = ["radio-bluetooth", "radio-zigbee", "radio-thread", "radio-matter"]
```

**NOTICE File** (required in radio crates):

```text
NOTICE: Licensing Information for efr32mg24-rail

This crate uses Silicon Labs RAIL (Radio Abstraction Interface Layer)
library, which is proprietary software licensed under the Silicon Labs
Master Software License Agreement (MSLA).

RAIL Library License: MSLA
- Full text: https://www.silabs.com/documents/public/software-license-agreements/

By enabling the `radio-rail` feature and using this crate, you agree
to the terms of the MSLA. The RAIL library may only be used with
Silicon Labs wireless devices.

Rust wrapper code (this crate) is licensed under MIT OR Apache-2.0.

Silicon Labs, EFR32, and RAIL are trademarks of Silicon Laboratories Inc.
```

### 6.4 Certification Considerations

**FCC/CE Certification**:

Using Silicon Labs certified blobs **helps** with certification, but:
- ❌ Does NOT provide automatic certification
- ✅ Provides certified radio performance (modular certification)
- ✅ Reduces testing burden (pre-qualified RF)
- ❌ User still responsible for:
  - Antenna design
  - PCB layout
  - Final product testing

**Recommendation**: Document that users are responsible for product certification.

---

## 7. Testing and Validation

### 7.1 Unit Testing (No Hardware)

**Build-Only Tests**:

```rust
// efr32mg24-rail/tests/config.rs

#[test]
fn rail_config_builds() {
    let config = RailConfig::new_2_4ghz();
    assert_eq!(config.frequency_band(), FrequencyBand::Ghz2_4);
}

#[test]
fn rail_events_bitflags() {
    let events = RailEvents::TX_COMPLETE | RailEvents::RX_READY;
    assert!(events.contains(RailEvents::TX_COMPLETE));
}

#[test]
fn bluetooth_gap_params() {
    let adv = AdvertisingParams::default();
    assert_eq!(adv.interval_ms(), 100); // Default 100ms
}
```

### 7.2 Integration Testing (Hardware Required)

**Phase D1 Tests** (RAIL Foundation):
- [ ] RAIL initializes without panic
- [ ] Radio powers on (measure current: ~20 mA active)
- [ ] TX FIFO write succeeds
- [ ] Packet transmission works (verify with spectrum analyzer)
- [ ] Packet reception works (loopback test with 2 boards)
- [ ] Event callbacks fire correctly

**Phase D2 Tests** (Bluetooth):
- [ ] BLE advertising starts (verify with mobile phone scanner)
- [ ] Mobile phone detects device name
- [ ] Connection establishment works
- [ ] GATT read/write works (use nRF Connect app)
- [ ] Notifications received on mobile
- [ ] Secure pairing completes
- [ ] Bonding persists across resets

**Phase D3 Tests** (Zigbee):
- [ ] Coordinator forms network
- [ ] Router joins network
- [ ] End device joins network
- [ ] On/Off cluster commands work
- [ ] Network survives power cycling
- [ ] Interoperability with Philips Hue bridge

**Phase D4 Tests** (Thread + Matter):
- [ ] Thread network forms
- [ ] CoAP requests work
- [ ] Border router connectivity
- [ ] Matter commissioning via QR code
- [ ] Google Home integration
- [ ] Apple HomeKit integration
- [ ] Samsung SmartThings integration

### 7.3 Test Hardware Setup

**Minimum Setup**:
- 2× XIAO MG24 Sense boards (TX/RX loopback testing)
- 1× Mobile phone (Android/iOS) with BLE scanner (nRF Connect)
- 1× USB-C cable for power/debugging
- 1× SWD debugger (J-Link or Black Magic Probe)

**Advanced Setup**:
- Logic analyzer (Saleae or similar) - timing validation
- Spectrum analyzer (TinySA Ultra) - RF verification
- Current measurement (µCurrent Gold) - power profiling
- Zigbee hub (Home Assistant with SLZB-06)
- Thread border router (Home Assistant Yellow)
- Matter controller (Google Nest Hub 2nd Gen)

### 7.4 Continuous Integration

**CI/CD Pipeline** (extends existing):

```yaml
# .github/workflows/radio.yml
name: Radio Tests

on: [push, pull_request]

jobs:
  build-radio-crates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install ARM toolchain
        run: rustup target add thumbv8m.main-none-eabihf

      - name: Cache Gecko SDK blobs
        uses: actions/cache@v3
        with:
          path: |
            crates/efr32mg24-rail-sys/blobs
            crates/efr32mg24-bluetooth-sys/blobs
          key: gecko-sdk-4.4.5-blobs

      - name: Build RAIL -sys crate
        run: cargo build --package efr32mg24-rail-sys --target thumbv8m.main-none-eabihf

      - name: Build RAIL safe wrapper
        run: cargo build --package efr32mg24-rail --target thumbv8m.main-none-eabihf

      - name: Build HAL with radio feature
        run: cargo build --package efr32mg24-hal --features radio-rail --target thumbv8m.main-none-eabihf

      - name: Run unit tests (host)
        run: cargo test --package efr32mg24-rail --lib

      - name: Build all radio examples
        run: |
          for example in 13_radio_init 16_ble_beacon 22_zigbee_light 26_thread_joiner; do
            cargo build --example $example --features radio-all --target thumbv8m.main-none-eabihf --release
          done

      - name: Check binary sizes
        run: |
          arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/examples/*
```

---

## 8. Documentation Requirements

### 8.1 Module README Structure

Each radio module needs comprehensive README following existing HAL pattern:

**Template** (e.g., `crates/efr32mg24-hal/src/radio/README.md`):

```markdown
# Radio Module (D1-RAIL)

**Identifier**: D1-RAIL
**Phase**: D (Radio & Security)
**Stage**: 1
**Status**: Production-Ready

## Overview

2.4 GHz radio support via Silicon Labs RAIL library FFI bindings.

## Supported Protocols

- ✅ Bluetooth 5.3 LE (Phase D2)
- ✅ Zigbee 3.0 (Phase D3)
- ✅ Thread 1.3 (Phase D4)
- ✅ Matter (Phase D4)

## Hardware Architecture

[Mermaid diagram of radio architecture]

## Usage Examples

[Code examples for initialization, TX, RX]

## Safety Considerations

[Documentation of all unsafe blocks]

## Licensing

**IMPORTANT**: Radio features use Silicon Labs RAIL library under MSLA.
Core HAL remains MIT OR Apache-2.0.

See NOTICE file for details.

## Performance

- TX throughput: 1 Mbps (BLE) / 250 kbps (Zigbee) / 250 kbps (Thread)
- RX latency: <10 ms
- Current consumption: 6-20 mA active, <5 µA deep sleep

## References

- [RAIL API Documentation](https://docs.silabs.com/rail/latest/)
- [Bluetooth Core Spec 5.3](https://www.bluetooth.com/specifications/specs/core-specification-5-3/)
- [Zigbee Specification](https://csa-iot.org/developer-resource/specifications-download-request/)
- [Thread Specification](https://www.threadgroup.org/support#specifications)
- [Matter Specification](https://csa-iot.org/all-solutions/matter/)
```

### 8.2 Example Documentation

Each example needs header comments:

```rust
//! Bluetooth LE Beacon Example (16_ble_beacon.rs)
//!
//! **Identifier**: D2-BLE.2
//!
//! This example demonstrates BLE advertising as an iBeacon and Eddystone beacon.
//!
//! # Hardware Requirements
//!
//! - XIAO MG24 Sense board
//! - 39 MHz HFXO configured
//!
//! # Expected Behavior
//!
//! 1. Initializes Bluetooth stack
//! 2. Starts iBeacon advertising with UUID
//! 3. Mobile phones can detect the beacon
//! 4. Prints beacon data to console
//!
//! # Testing
//!
//! Use nRF Connect app on mobile phone:
//! - Open nRF Connect
//! - Start scanning
//! - Look for "XIAO Beacon" or iBeacon UUID
//!
//! # Build and Flash
//!
//! ```bash
//! cargo build --example 16_ble_beacon --features radio-bluetooth --release
//! cargo run --example 16_ble_beacon --features radio-bluetooth --release
//! ```
//!
//! # Licensing
//!
//! This example uses Bluetooth stack under Silicon Labs MSLA.
```

### 8.3 Top-Level Documentation Updates

**README.md additions**:

```markdown
## Radio Support (Phase D)

EFR32MG24 HAL provides complete 2.4 GHz radio support for:

- **Bluetooth 5.3 LE**: GAP, GATT, Security Manager
- **Zigbee 3.0**: Coordinator, Router, End Device, ZCL clusters
- **Thread 1.3**: OpenThread stack integration
- **Matter**: Device commissioning and clusters

### Enabling Radio Features

```toml
[dependencies]
efr32mg24-hal = { version = "0.4", features = ["radio-bluetooth"] }
```

Available features:
- `radio-rail` - RAIL foundation (required for all protocols)
- `radio-bluetooth` - Bluetooth 5.3 LE stack
- `radio-zigbee` - Zigbee 3.0 stack
- `radio-thread` - Thread 1.3 (OpenThread)
- `radio-matter` - Matter support
- `radio-all` - All protocols

### Licensing

**Core HAL**: MIT OR Apache-2.0 (no restrictions)

**Radio features**: Silicon Labs MSLA (restricts to SiLabs devices only)

By enabling radio features, you agree to MSLA terms.
See [NOTICE](./NOTICE) for details.
```

---

## 9. Timeline and Milestones

### 9.1 Phase Breakdown

| Phase | Identifier | Description | Effort (hours) | Duration (weeks) | Dependencies |
|-------|------------|-------------|----------------|------------------|--------------|
| **D1** | D1-RAIL | RAIL foundation | 38-57 | 5-7 | None (can start now) |
| **D2** | D2-BLE | Bluetooth 5.3 | 60-78 | 8-10 | D1-RAIL complete |
| **D3** | D3-Zigbee | Zigbee 3.0 | 45-55 | 6-7 | D1-RAIL complete |
| **D4** | D4-Thread | Thread + Matter | 52-65 | 7-9 | D1-RAIL complete |

**Total**: 195-255 hours (~9-12 months part-time, 4-6 months full-time)

**Parallelization**: D2, D3, D4 can be done in parallel after D1 is complete.

### 9.2 Milestone Definitions

**Milestone 1: RAIL Foundation Complete** (D1-RAIL.4)
- ✅ Radio initializes without errors
- ✅ TX/RX loopback test passes
- ✅ Event-driven operation works
- ✅ Example `13_radio_init.rs` compiles and runs
- **Deliverable**: Can transmit and receive raw packets

**Milestone 2: Bluetooth Basic** (D2-BLE.2)
- ✅ Mobile phone detects BLE advertising
- ✅ Connection establishment works
- ✅ Example `16_ble_beacon.rs` works with nRF Connect
- **Deliverable**: BLE peripheral device operational

**Milestone 3: Bluetooth Complete** (D2-BLE.5)
- ✅ GATT server/client works
- ✅ Secure pairing works
- ✅ Low-power sleep coordination works
- ✅ All 5 BLE examples (16-20) work
- **Deliverable**: Production-ready BLE stack

**Milestone 4: Zigbee Complete** (D3-Zigbee.4)
- ✅ Zigbee network forms and joins
- ✅ On/Off cluster works
- ✅ Interoperates with commercial Zigbee devices
- ✅ All 4 Zigbee examples (22-25) work
- **Deliverable**: Production-ready Zigbee stack

**Milestone 5: Thread + Matter Complete** (D4-Thread.4)
- ✅ Thread network forms
- ✅ Matter device commissions via QR code
- ✅ Google Home / Apple HomeKit integration works
- ✅ All 4 Thread/Matter examples (26-29) work
- **Deliverable**: Production-ready Matter devices

### 9.3 Critical Path

```text
Start
  ↓
[D1-RAIL.1] Build infrastructure (8-12h)
  ↓
[D1-RAIL.2] Basic initialization (10-15h)
  ↓
[D1-RAIL.3] TX/RX foundation (12-18h)
  ↓
[D1-RAIL.4] Event system (8-12h)
  ↓
Milestone 1: RAIL Complete
  ↓
┌─────────────┬──────────────┬──────────────┐
│ [D2-BLE]    │ [D3-Zigbee]  │ [D4-Thread]  │ ← Parallel
│ Bluetooth   │ Zigbee       │ Thread       │
│ (60-78h)    │ (45-55h)     │ (52-65h)     │
└─────────────┴──────────────┴──────────────┘
  ↓            ↓              ↓
Milestone 2-5: All protocols complete
  ↓
Phase D Complete
```

**Fastest path**: 4-5 months (full-time, all protocols in parallel after D1)
**Realistic path**: 9-12 months (part-time, sequential protocol implementation)

---

## 10. Risk Assessment

### 10.1 High-Priority Risks

**Risk 1: RAIL Blob Linking Failures**
- **Impact**: Cannot compile radio features (CRITICAL)
- **Likelihood**: Medium (symbol conflicts, ABI mismatches)
- **Mitigation**:
  - Start with minimal C test program to validate linking
  - Use exact GCC version as Silicon Labs toolchain (ARM GCC 12.2.1)
  - Test on Linux, macOS, and Windows
  - Document working toolchain versions

**Risk 2: MSLA Licensing Conflicts**
- **Impact**: Cannot publish to crates.io, legal issues
- **Likelihood**: Low (precedent exists with nrf-softdevice, esp-idf-hal)
- **Mitigation**:
  - Consult IP lawyer before publishing
  - Make radio features optional (feature flags)
  - Clear NOTICE file with licensing terms
  - Follow industry patterns from Nordic/Espressif

**Risk 3: Memory Constraints**
- **Impact**: Stack overflow, heap exhaustion, crashes
- **Likelihood**: Medium (256 KB RAM must support app + radio stacks)
- **Mitigation**:
  - Profile memory usage in D1-RAIL.2
  - Adjust memory.x linker script early
  - Test with realistic application loads
  - Consider disabling unused protocol features

**Risk 4: Callback Stability**
- **Impact**: Radio crashes due to undefined behavior in C→Rust callbacks
- **Likelihood**: Medium (callback mechanism is complex)
- **Mitigation**:
  - Extensive testing of callback mechanism (D1-RAIL.4)
  - Use static storage only (no heap in callbacks)
  - Thorough SAFETY documentation for all unsafe blocks
  - Fuzz testing with random event sequences

### 10.2 Medium-Priority Risks

**Risk 5: Hardware Availability**
- **Impact**: Cannot test on actual devices, delays implementation
- **Likelihood**: Low (XIAO MG24 Sense readily available)
- **Mitigation**: Order 3-5 boards early, budget $150-200

**Risk 6: Protocol Interoperability**
- **Impact**: Devices don't work with commercial ecosystems (Google Home, etc.)
- **Likelihood**: Medium (complex protocols, many edge cases)
- **Mitigation**:
  - Follow Silicon Labs examples closely
  - Test with multiple commercial products early
  - Use protocol analyzers (Wireshark with BLE/Zigbee plugins)
  - Join Matter and Thread certification programs

**Risk 7: Certification Requirements**
- **Impact**: Cannot legally sell products commercially
- **Likelihood**: High (for commercial products)
- **Mitigation**:
  - Document that users are responsible for certification
  - Provide reference designs matching Silicon Labs certified configs
  - Partner with certified test labs (budget $5K-$15K per product)

### 10.3 Low-Priority Risks

**Risk 8: Gecko SDK API Changes**
- **Impact**: Blobs incompatible with newer SDK versions
- **Likelihood**: Low (RAIL 2.x stable, long-term support)
- **Mitigation**:
  - Pin to specific Gecko SDK version (4.4.5+)
  - Document SDK version in README
  - Monitor Silicon Labs release notes

**Risk 9: Performance Issues**
- **Impact**: Poor throughput, high latency
- **Likelihood**: Low (Silicon Labs stacks are production-proven)
- **Mitigation**:
  - Benchmark early (D1-RAIL.3)
  - Profile with oscilloscope/logic analyzer
  - Optimize critical paths if needed

**Risk 10: Community Adoption**
- **Impact**: Low usage, no contributors
- **Likelihood**: Medium (niche MCU, complex topic)
- **Mitigation**:
  - Comprehensive documentation
  - Working examples for all protocols
  - Active engagement in Rust embedded community
  - Blog posts, conference talks

---

## Summary and Next Steps

### Phase D Implementation Checklist

**Immediate Actions** (next 2 weeks):
- [ ] Review and approve this implementation plan
- [ ] Order 3× XIAO MG24 Sense boards ($100 total)
- [ ] Download Gecko SDK 4.4.5 (~2 GB)
- [ ] Extract RAIL blobs for EFR32MG24B220
- [ ] Create `efr32mg24-rail-sys` crate skeleton
- [ ] Implement build.rs with bindgen integration
- [ ] Verify blob linking with minimal test

**Phase D1 Goals** (next 5-7 weeks):
- [ ] Complete D1-RAIL.1 through D1-RAIL.4
- [ ] Have working TX/RX radio communication
- [ ] Example `13_radio_init.rs` operational
- [ ] Milestone 1 achieved: RAIL Foundation Complete

**Long-Term Goals** (9-12 months):
- [ ] All protocols implemented (Bluetooth, Zigbee, Thread, Matter)
- [ ] 15+ radio examples working
- [ ] Hardware tested on XIAO MG24 Sense
- [ ] Documentation complete
- [ ] Published to crates.io with clear licensing

### Success Criteria

Phase D is **COMPLETE** when:
1. ✅ All 4 protocols implemented (Bluetooth, Zigbee, Thread, Matter)
2. ✅ All 15+ radio examples compile and run on hardware
3. ✅ Interoperability validated with commercial ecosystems
4. ✅ Zero unsafe blocks without SAFETY documentation
5. ✅ Comprehensive READMEs for all radio modules
6. ✅ Clear licensing documentation (MSLA + MIT/Apache-2.0)
7. ✅ CI/CD pipeline validates all radio builds

---

**Document Version**: 1.0
**Last Updated**: December 27, 2025
**Maintained By**: EFR32MG24 HAL Project Team
**Feedback**: Open issues at https://github.com/bitscrafts/EFR32MG2X-RS/issues

<\!-- META: last_updated=2025-12-27 version=1.0.0 status=planning maintained_by=orchestrator -->
