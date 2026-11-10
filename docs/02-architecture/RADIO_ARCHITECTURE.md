# Radio Crate Architecture Guidelines

**Date**: 2026-01-03
**Purpose**: Define proper layering for radio crates (Bluetooth, Zigbee, Thread, RAIL)
**Applies To**: efr32mg24-bluetooth, efr32mg24-zigbee, efr32mg24-thread, efr32mg24-rail

---

## Executive Summary

**Problem Identified**: 1,348 lines of generic BLE code mistakenly placed in board-specific BSP layer.

**Root Cause**: Misunderstanding of proper *-sys vs high-level crate separation.

**Solution**: Follow industry standard: Split into `-sys` (raw FFI) and standalone crate (safe wrappers).

---

## Industry Research

### Sources

- [nrf-softdevice GitHub](https://github.com/embassy-rs/nrf-softdevice)
- [esp-wifi GitHub](https://github.com/esp-rs/esp-wifi)
- [esp-hal/esp-radio GitHub](https://github.com/esp-rs/esp-hal)

### Nordic nRF Pattern (Reference Implementation)

```
┌───────────────────────────────────┐
│  nrf52-dk-bsp                     │  ← Board-specific wrapper
│  (board crate)                    │    (optional convenience)
└───────────────────────────────────┘
            ↓ uses
┌───────────────────────────────────┐
│  nrf-softdevice                   │  ← High-level safe API
│  - BLE central/peripheral         │    (SEPARATE CRATE)
│  - GATT client/server             │
│  - L2CAP, async/await             │
│  - ~5,000 lines safe wrappers     │
└───────────────────────────────────┘
            ↓ depends on
┌───────────────────────────────────┐
│  nrf-softdevice-s140              │  ← Raw FFI bindings
│  - bindgen output                 │    (-sys equivalent)
│  - SVC call inline ASM            │
│  - NO safe wrappers               │
└───────────────────────────────────┘
            ↓ links
┌───────────────────────────────────┐
│  s140_nrf52_7.x.x_softdevice.hex  │  ← Binary blob
└───────────────────────────────────┘
```

### Espressif ESP Pattern (Reference Implementation)

```
┌───────────────────────────────────┐
│  esp32c3-hal                      │  ← Chip HAL (NO radio)
│  - GPIO, SPI, I2C, UART           │    (peripherals only)
│  - NO WiFi/BLE code               │
└───────────────────────────────────┘
            ↓ board uses
┌───────────────────────────────────┐
│  esp-radio                        │  ← High-level safe API
│  - WiFi controller                │    (SEPARATE CRATE)
│  - BLE central/peripheral         │
│  - ESP-NOW                        │
│  - ~10,000 lines safe wrappers    │
└───────────────────────────────────┘
            ↓ depends on
┌───────────────────────────────────┐
│  esp-wifi-sys-esp32c3             │  ← Raw FFI bindings
│  - bindgen output                 │    (-sys crate)
│  - C wireless driver bindings     │
│  - NO safe wrappers               │
└───────────────────────────────────┘
            ↓ links
┌───────────────────────────────────┐
│  libcoexist.a, libcore.a, etc     │  ← Binary blobs
└───────────────────────────────────┘
```

---

## Correct EFR32MG24 Architecture

### Three-Layer Pattern

```
┌───────────────────────────────────┐
│  xiao-mg24-bsp                    │  ← Layer 3: Board-specific
│  - XiaoBle wrapper (~50 lines)    │    Optional convenience
│  - Default device name "XIAO MG24"│
│  - Wraps Bluetooth::new()         │
└───────────────────────────────────┘
            ↓ uses
┌───────────────────────────────────┐
│  efr32mg24-bluetooth              │  ← Layer 2: Safe wrappers
│  - Bluetooth struct               │    SEPARATE CRATE (NEW!)
│  - BleEvent enum                  │
│  - CharHandle, GapAdBuilder       │
│  - All BGAPI safe wrappers        │
│  - ~1,300 lines safe code         │
└───────────────────────────────────┘
            ↓ depends on
┌───────────────────────────────────┐
│  efr32mg24-bluetooth-sys          │  ← Layer 1: Raw FFI
│  - include!(bindings.rs)          │    -sys crate (current)
│  - BGAPI FFI functions            │
│  - NO safe wrappers               │
└───────────────────────────────────┘
            ↓ links
┌───────────────────────────────────┐
│  libbluetooth_efr32xg24_*.a       │  ← Binary blobs
└───────────────────────────────────┘
```

### File Structure

```
Workspace root/
├── crates/efr32mg24-bluetooth-sys/        ← LAYER 1: Raw FFI only
│   ├── Cargo.toml                  (deps: cty only)
│   ├── src/lib.rs                  (include bindings)
│   ├── build.rs                    (bindgen + blob linking)
│   ├── blobs/                      (libbluetooth_*.a)
│   └── vendor/                     (BGAPI headers)
│
├── efr32mg24-bluetooth/            ← LAYER 2: Safe wrappers (NEW!)
│   ├── Cargo.toml                  (deps: bluetooth-sys, heapless)
│   ├── src/
│   │   ├── lib.rs                  (public API)
│   │   ├── stack.rs                (Bluetooth struct)
│   │   ├── events.rs               (BleEvent enum)
│   │   ├── types.rs                (CharHandle, GapAdBuilder)
│   │   ├── error.rs                (BleError)
│   │   └── README.md
│   └── examples/                   (generic BLE examples)
│
└── crates/xiao-mg24-bsp/                  ← LAYER 3: Board wrapper
    ├── Cargo.toml                  (deps: efr32mg24-bluetooth)
    ├── src/
    │   ├── radio/
    │   │   └── ble.rs              (XiaoBle wrapper ~50 lines)
    │   └── ...
    └── examples/                   (board-specific examples)
```

---

## Layer Responsibilities

### Layer 1: *-sys Crate (Raw FFI)

**Purpose:** Provide 1:1 Rust bindings to C API

**Responsibilities:**
- ✅ Generate bindings with bindgen
- ✅ Link against binary blobs
- ✅ Provide raw unsafe FFI functions
- ✅ Verify blob integrity (SHA256)
- ✅ Platform-independent build system

**Does NOT contain:**
- ❌ Safe wrappers
- ❌ High-level types (enums, structs)
- ❌ Error handling beyond raw codes
- ❌ Convenience methods

**Dependencies:**
- `cty` (C type definitions)
- Build deps: `bindgen`, `sha2`, `cc`

**Examples:**
- `efr32mg24-bluetooth-sys` ✓ (current - correct)
- `efr32mg24-rail-sys` ✓ (current - correct)
- `nrf-softdevice-s140`
- `esp-wifi-sys-esp32c3`

**Cargo.toml:**
```toml
[dependencies]
cty = "0.2"

[build-dependencies]
bindgen = "0.69"
sha2 = "0.10"
```

---

### Layer 2: Standalone Safe Wrapper Crate

**Purpose:** Provide safe, idiomatic Rust API over raw FFI

**Responsibilities:**
- ✅ Safe wrapper structs (Bluetooth, RailHandle, etc.)
- ✅ Type-safe enums (BleEvent, RailStatus, etc.)
- ✅ Utility types (CharHandle, GapAdBuilder, etc.)
- ✅ Error handling (BleError with proper variants)
- ✅ All BGAPI/RAIL safe function wrappers
- ✅ Memory safety guarantees
- ✅ Generic across all EFR32MG24 variants

**Does NOT contain:**
- ❌ Board-specific configurations
- ❌ Pin definitions
- ❌ Board-specific defaults
- ❌ Hardware peripheral initialization (GPIO, etc.)

**Dependencies:**
- `efr32mg24-*-sys` (the raw FFI layer)
- `heapless` (for no_std collections)
- `embedded-hal` traits (where applicable)

**Examples:**
- `efr32mg24-bluetooth` (to be created)
- `efr32mg24-rail` (to be created)
- `nrf-softdevice`
- `esp-radio`

**Cargo.toml:**
```toml
[dependencies]
efr32mg24-bluetooth-sys = { version = "0.1", path = "../efr32mg24-bluetooth-sys" }
heapless = "0.8"
embedded-hal = "1.0"
```

---

### Layer 3: Board Support Package (BSP)

**Purpose:** Provide board-specific convenience and defaults

**Responsibilities:**
- ✅ Board-specific wrapper structs (XiaoBle, XiaoRail, etc.)
- ✅ Board-specific default configurations
- ✅ Board pin definitions and initialization
- ✅ Convenience methods forwarding to Layer 2
- ✅ Board-specific examples

**Does NOT contain:**
- ❌ Generic protocol logic
- ❌ Event parsing
- ❌ Protocol state machines
- ❌ Anything usable on other boards

**Dependencies:**
- `efr32mg24-bluetooth` (Layer 2, NOT -sys!)
- `efr32mg24-hal` (for GPIO, delay, etc.)
- `efr32mg24-pac`

**Examples:**
- `xiao-mg24-bsp`
- `nrf52-dk` (Nordic board crates)
- Custom user board crates

**Cargo.toml:**
```toml
[dependencies]
efr32mg24-bluetooth = { version = "0.1", path = "../efr32mg24-bluetooth" }
efr32mg24-hal = { path = "../efr32mg24-hal", features = ["rt"] }
efr32mg24-pac = { path = "../efr32mg24-pac" }
```

---

## Decision Matrix: Where Does Code Belong?

### Quick Test

Ask these questions about your code:

| Question | Yes = Layer | No = Continue |
|----------|-------------|---------------|
| Is this raw FFI (`unsafe fn`)? | Layer 1 (*-sys) | ↓ |
| Does this parse protocol events? | Layer 2 (safe wrapper) | ↓ |
| Does this wrap FFI with safe types? | Layer 2 (safe wrapper) | ↓ |
| Is this board-specific? | Layer 3 (BSP) | ↓ |
| Is this XIAO-specific? | Layer 3 (BSP) | ERROR! |

### Detailed Decision Tree

```
Code component to place:
│
├─ Contains `unsafe { ffi::* }`?
│  ├─ YES: Is it ONLY bindgen output?
│  │  ├─ YES → Layer 1 (-sys crate)
│  │  └─ NO → Layer 2 (safe wrapper calling -sys)
│  └─ NO: Continue...
│
├─ Is it a safe wrapper over FFI?
│  ├─ YES → Layer 2 (safe wrapper)
│  └─ NO: Continue...
│
├─ Does it parse protocol-specific data?
│  ├─ YES → Layer 2 (safe wrapper)
│  └─ NO: Continue...
│
├─ Does it work on ANY EFR32MG24 device?
│  ├─ YES → Layer 2 (safe wrapper)
│  └─ NO: Continue...
│
├─ Is it board-specific (XIAO pins, etc.)?
│  ├─ YES → Layer 3 (BSP)
│  └─ NO → ERROR: Unknown layer!
```

---

## Examples from Current BLE Code

### Example 1: BleEvent Enum

```rust
#[derive(Debug, Clone)]
pub enum BleEvent {
    Connected { connection: u8, address: [u8; 6], ... },
    Disconnected { connection: u8, reason: u16 },
    GattWrite { connection: u8, attribute: u16, value: Vec<u8, 512> },
    // ...
}
```

**Analysis:**
- ❌ Not raw FFI (safe types)
- ✅ Parses BGAPI events
- ✅ Works on any EFR32MG24
- ❌ Not board-specific

**Correct Layer:** **Layer 2** (efr32mg24-bluetooth)

**Current (Wrong):** BSP (xiao-mg24-bsp)

---

### Example 2: CharHandle

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharHandle(pub u16);

impl CharHandle {
    pub fn value(&self) -> u16 { self.0 }
}
```

**Analysis:**
- ❌ Not raw FFI
- ✅ Type-safe GATT wrapper
- ✅ Works on any EFR32MG24
- ❌ Not board-specific

**Correct Layer:** **Layer 2** (efr32mg24-bluetooth)

**Current (Wrong):** BSP (xiao-mg24-bsp)

---

### Example 3: GapAdBuilder

```rust
pub struct GapAdBuilder {
    buffer: [u8; 31],
    offset: usize,
}

impl GapAdBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn add_name(&mut self, name: &str) -> Result<(), BleError> { /* ... */ }
    pub fn add_service_uuid_16(&mut self, uuid: u16) -> Result<(), BleError> { /* ... */ }
}
```

**Analysis:**
- ❌ Not raw FFI
- ✅ GAP AD format encoding (protocol-level)
- ✅ Works on any EFR32MG24
- ❌ Not board-specific

**Correct Layer:** **Layer 2** (efr32mg24-bluetooth)

**Current (Wrong):** BSP (xiao-mg24-bsp)

---

### Example 4: Bluetooth::poll_event()

```rust
pub fn poll_event(&mut self) -> Option<BleEvent> {
    let mut evt = core::mem::MaybeUninit::<sl_bt_msg_t>::uninit();
    let status = unsafe { sl_bt_pop_event(evt.as_mut_ptr()) };

    if status != 0 { return None; }

    let evt = unsafe { evt.assume_init() };
    BleEvent::parse(&evt)
}
```

**Analysis:**
- ✅ Calls raw FFI
- ✅ Provides safe wrapper
- ✅ Works on any EFR32MG24
- ❌ Not board-specific

**Correct Layer:** **Layer 2** (efr32mg24-bluetooth)

**Current (Wrong):** BSP (xiao-mg24-bsp as XiaoBle::poll_event)

---

### Example 5: XiaoBle Wrapper

```rust
pub struct XiaoBle {
    bluetooth: Bluetooth,  // Wraps Layer 2
}

impl XiaoBle {
    pub fn init(delay: &mut Delay) -> Result<Self, BleError> {
        let mut bluetooth = Bluetooth::new(delay)?;
        // XIAO-specific default
        bluetooth.set_device_name("XIAO MG24")?;
        Ok(Self { bluetooth })
    }
}
```

**Analysis:**
- ❌ Not raw FFI
- ❌ Not generic protocol logic
- ✅ XIAO-specific defaults ("XIAO MG24" name)
- ✅ Board-specific

**Correct Layer:** **Layer 3** (xiao-mg24-bsp)

**Current:** BSP ✓ (but XiaoBle should wrap Bluetooth from Layer 2, not contain all logic)

---

## Migration Path

### Phase 1: Create New Crate

```bash
cargo new efr32mg24-bluetooth --lib
```

**Cargo.toml:**
```toml
[package]
name = "efr32mg24-bluetooth"
version = "0.1.0"
edition = "2021"

[dependencies]
efr32mg24-bluetooth-sys = { path = "../efr32mg24-bluetooth-sys" }
heapless = "0.8"
```

### Phase 2: Move Code

**From BSP → New Crate:**

| File | Source | Destination | Lines |
|------|--------|-------------|-------|
| error.rs | crates/xiao-mg24-bsp/src/radio/ble/mod.rs | efr32mg24-bluetooth/src/error.rs | ~50 |
| types.rs | crates/xiao-mg24-bsp/src/radio/ble/types.rs | efr32mg24-bluetooth/src/types.rs | ~350 |
| events.rs | crates/xiao-mg24-bsp/src/radio/ble/events.rs | efr32mg24-bluetooth/src/events.rs | ~280 |
| stack.rs | crates/xiao-mg24-bsp/src/radio/ble/mod.rs (XiaoBle → Bluetooth) | efr32mg24-bluetooth/src/stack.rs | ~700 |

### Phase 3: Refactor BSP

**crates/xiao-mg24-bsp/src/radio/ble.rs (new minimal file):**
```rust
//! XIAO MG24 BLE convenience wrapper

pub use efr32mg24_bluetooth::{BleEvent, BleError, CharHandle, GapAdBuilder};
use efr32mg24_bluetooth::Bluetooth;

pub struct XiaoBle {
    bluetooth: Bluetooth,
}

impl XiaoBle {
    pub fn init(delay: &mut impl DelayNs) -> Result<Self, BleError> {
        let mut bluetooth = Bluetooth::new(delay)?;
        bluetooth.set_device_name("XIAO MG24")?;
        Ok(Self { bluetooth })
    }

    // Forward all methods to bluetooth
    pub fn poll_event(&mut self) -> Option<BleEvent> {
        self.bluetooth.poll_event()
    }
    // ... etc
}
```

### Phase 4: Update Examples

**Before:**
```rust
use xiao_mg24_bsp::radio::ble::{XiaoBle, BleEvent};
```

**After (Option A - Generic):**
```rust
use efr32mg24_bluetooth::{Bluetooth, BleEvent};
let mut ble = Bluetooth::new(&mut delay)?;
```

**After (Option B - Board wrapper):**
```rust
use xiao_mg24_bsp::radio::ble::{XiaoBle, BleEvent};
let mut ble = XiaoBle::init(&mut delay)?;  // Still works!
```

---

## Common Mistakes to Avoid

### ❌ Mistake 1: Putting Safe Wrappers in -sys Crate

**Wrong:**
```
crates/efr32mg24-bluetooth-sys/
├── src/
│   ├── ffi.rs         ← Raw FFI ✓
│   ├── bluetooth.rs   ← Safe wrappers ✗ WRONG!
│   └── events.rs      ← Safe wrappers ✗ WRONG!
```

**Why Wrong:** -sys crates should be pure FFI

**Fix:** Move safe wrappers to separate crate

---

### ❌ Mistake 2: Putting Protocol Logic in BSP

**Wrong:**
```rust
// In xiao-mg24-bsp
impl XiaoBle {
    pub fn poll_event(&mut self) -> Option<BleEvent> {
        // 280 lines of BGAPI event parsing ✗ WRONG!
    }
}
```

**Why Wrong:** Event parsing works on all EFR32MG24, not just XIAO

**Fix:** Move to Layer 2 (efr32mg24-bluetooth)

---

### ❌ Mistake 3: Putting Generic Types in BSP

**Wrong:**
```rust
// In crates/xiao-mg24-bsp/src/radio/ble/types.rs
pub struct CharHandle(pub u16);  // ✗ WRONG!
```

**Why Wrong:** CharHandle works on all EFR32MG24

**Fix:** Move to Layer 2 (efr32mg24-bluetooth/src/types.rs)

---

### ❌ Mistake 4: Mixing HAL with Radio

**Wrong:**
```
crates/efr32mg24-hal/
└── src/
    ├── gpio/     ← Peripherals ✓
    ├── spi/      ← Peripherals ✓
    └── radio/    ← ✗ WRONG! Radio doesn't belong in HAL
```

**Why Wrong:** Radio uses proprietary blobs, HAL is peripheral-only

**Fix:** Keep radio in separate *-bluetooth, *-zigbee crates

---

## Checklist for New Radio Implementations

### For Zigbee (efr32mg24-zigbee-sys + efr32mg24-zigbee):

- [ ] Create efr32mg24-zigbee-sys (Layer 1: raw FFI only)
- [ ] Create efr32mg24-zigbee (Layer 2: safe wrappers)
- [ ] BSP has minimal XiaoZigbee wrapper (Layer 3)
- [ ] No Zigbee code in efr32mg24-hal
- [ ] Follow 3-layer pattern exactly

### For Thread (efr32mg24-thread-sys + efr32mg24-thread):

- [ ] Create efr32mg24-thread-sys (Layer 1: raw FFI only)
- [ ] Create efr32mg24-thread (Layer 2: safe wrappers)
- [ ] BSP has minimal XiaoThread wrapper (Layer 3)
- [ ] No Thread code in efr32mg24-hal
- [ ] Follow 3-layer pattern exactly

### For Matter (uses Thread underneath):

- [ ] Depends on efr32mg24-thread (not thread-sys!)
- [ ] No Matter code in HAL or BSP
- [ ] Separate efr32mg24-matter crate

---

## Summary

### Correct 3-Layer Architecture

1. **Layer 1 (*-sys):** Raw FFI only, no safe wrappers
2. **Layer 2 (standalone):** Safe wrappers, protocol logic, generic types
3. **Layer 3 (BSP):** Board-specific defaults and convenience

### Key Principles

✅ **Do:**
- Keep -sys crates minimal (bindgen + blob linking)
- Put ALL safe wrappers in separate crate (Layer 2)
- Make Layer 2 generic (works on any EFR32MG24)
- Keep BSP minimal (board-specific only)

❌ **Don't:**
- Put safe wrappers in -sys crates
- Put protocol logic in BSP
- Put radio code in HAL
- Mix layers

### Industry Validation

- ✅ Nordic: `nrf-softdevice-s140` (FFI) + `nrf-softdevice` (safe)
- ✅ Espressif: `esp-wifi-sys` (FFI) + `esp-radio` (safe)
- ✅ Our pattern: `efr32mg24-*-sys` (FFI) + `efr32mg24-*` (safe)

---

---

## Embassy Integration Strategy

### Key Decision: ONE Crate with Optional Embassy

**Why not separate crates?**
- Our deps are minimal (only heapless for blocking API)
- Embassy is optional via feature flags
- Simpler than maintaining 3 crates (sys, blocking, async)

### Structure with Embassy

```
crates/efr32mg24-bluetooth-sys/
├── src/
│   ├── ffi.rs              ← Raw FFI (always)
│   ├── blocking/           ← Blocking API (always)
│   │   ├── stack.rs
│   │   ├── events.rs
│   │   ├── types.rs
│   │   └── error.rs
│   └── asynch/             ← Async API (#[cfg(feature = "embassy")])
│       ├── stack.rs
│       └── gatt.rs
└── Cargo.toml
```

### Cargo.toml Features

```toml
[dependencies]
cty = "0.2"
heapless = "0.8"

# Optional Embassy support
embassy-sync = { version = "0.6", optional = true }
embassy-time = { version = "0.3", optional = true }
embassy-futures = { version = "0.1", optional = true }

[features]
default = []
embassy = ["embassy-sync", "embassy-time", "embassy-futures"]
```

### Usage Patterns

**Blocking (default):**
```rust
use efr32mg24_bluetooth_sys::blocking::Bluetooth;
let mut ble = Bluetooth::new(&mut delay)?;
```

**Async (with feature):**
```toml
efr32mg24-bluetooth-sys = { version = "0.1", features = ["embassy"] }
```
```rust
use efr32mg24_bluetooth_sys::asynch::AsyncBluetooth;
let mut ble = AsyncBluetooth::new().await?;
```

### Benefits

✅ Simple: One crate to maintain
✅ Flexible: Users choose blocking or async
✅ Minimal: Blocking API has zero embassy deps
✅ Future-proof: Async support when needed

---

**Document Status:** FINAL (Updated with Embassy)
**Review Required:** Before implementing Zigbee/Thread
**Last Updated:** 2026-01-03

<!-- META: last_updated=2026-01-03 version=1.1.0 status=final maintained-by=architecture-team template-version=1.0 embassy-strategy=documented -->
