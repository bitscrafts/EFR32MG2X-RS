# Bluetooth Architecture Refactoring Decisions

**Date**: 2026-01-03
**Issue**: 1,348 lines of generic BLE code trapped in board-specific BSP layer
**Resolution**: Move to proper layer in bluetooth-sys with blocking/async split

---

## Decision Log

### Decision 1: One Crate vs Two Crates

**Question**: Should we create a separate `efr32mg24-bluetooth` crate for safe wrappers, or put everything in `efr32mg24-bluetooth-sys`?

**Options Considered:**

**Option A: TWO Crates (sys + bluetooth)**
```
crates/efr32mg24-bluetooth-sys/     ← Raw FFI only
efr32mg24-bluetooth/         ← Safe wrappers
```

**Pros:**
- Clear separation (FFI vs safe API)
- Matches nrf-softdevice pattern exactly
- Independent versioning

**Cons:**
- More crates to maintain
- More build overhead
- Overkill for our minimal dependencies

**Option B: ONE Crate (sys with modules) ✅ CHOSEN**
```
crates/efr32mg24-bluetooth-sys/
├── ffi.rs           ← Raw FFI
├── blocking/        ← Safe blocking API
└── asynch/          ← Safe async API (future)
```

**Pros:**
- Simpler workspace (one crate per protocol)
- Minimal dependencies (only heapless for blocking)
- Matches esp-wifi/esp-radio pattern
- Less overhead

**Cons:**
- Mixed FFI and safe API in same crate
- Name ends with -sys but has safe wrappers

**DECISION: Option B (One Crate)**

**Rationale:**
1. **Minimal dependencies** - Blocking API only adds `heapless` (small, no_std)
2. **Not like Nordic** - They have heavy Embassy deps in safe wrapper (futures, embassy-executor, etc.)
3. **Like Espressif** - esp-wifi is one crate with both FFI and safe API
4. **Simpler** - One crate to version, publish, and maintain
5. **Name convention** - Can be relaxed since -sys traditionally means FFI, but can also mean "system-level" including safe wrappers

**Source Reference:**
- [esp-wifi](https://github.com/esp-rs/esp-wifi) - Single crate with FFI and safe API
- [nrf-softdevice](https://github.com/embassy-rs/nrf-softdevice) - Separate crates due to heavy Embassy dependencies

---

### Decision 2: Module Organization (blocking vs asynch)

**Question**: How should we organize the code to support both blocking and async APIs?

**Options Considered:**

**Option A: Flat structure with cfg**
```
src/
├── ffi.rs
├── bluetooth.rs      // #[cfg(not(feature = "embassy"))]
└── async_bluetooth.rs // #[cfg(feature = "embassy")]
```

**Pros:**
- Simple file structure
- Clear separation

**Cons:**
- cfg everywhere
- Hard to share types
- Duplicate code

**Option B: Module-based separation ✅ CHOSEN**
```
src/
├── lib.rs
├── ffi.rs
├── blocking/
│   ├── mod.rs
│   ├── error.rs
│   ├── types.rs
│   ├── events.rs
│   └── stack.rs
└── asynch/           // #[cfg(feature = "embassy")]
    ├── mod.rs
    ├── stack.rs
    └── gatt.rs
```

**Pros:**
- Clear namespace separation (`blocking::` vs `asynch::`)
- Easy to share types (both use `blocking::BleEvent`)
- Matches nrf-softdevice pattern
- Clean feature gating (only `asynch/` is behind feature)

**Cons:**
- More files
- Deeper nesting

**DECISION: Option B (Module-based)**

**Rationale:**
1. **Clear namespaces** - `use bluetooth_sys::blocking::Bluetooth` vs `use bluetooth_sys::asynch::AsyncBluetooth`
2. **Type sharing** - Async can reuse `blocking::BleEvent`, `blocking::BleError`
3. **Industry standard** - Matches nrf-softdevice's `ble::` and `raw::` modules
4. **Maintainable** - Easy to see what's blocking vs async

**Source Reference:**
- [nrf-softdevice module structure](https://github.com/embassy-rs/nrf-softdevice) uses `ble::` and `raw::`

---

### Decision 3: Keep HAL Radio-Free

**Question**: Should radio code live in HAL or separate crates?

**Options Considered:**

**Option A: Put radio in HAL**
```
crates/efr32mg24-hal/src/
├── gpio/
├── spi/
└── radio/           ← BLE, Zigbee, Thread
```

**Pros:**
- All drivers in one place
- Simpler for users (one import)

**Cons:**
- Violates single responsibility
- Mixes peripheral drivers with proprietary blobs
- HAL becomes non-portable

**Option B: Keep radio separate ✅ CHOSEN**
```
crates/efr32mg24-hal/        ← Peripherals only (GPIO, SPI, etc.)
crates/efr32mg24-bluetooth-sys/  ← Bluetooth (proprietary blob)
efr32mg24-zigbee-sys/     ← Zigbee (proprietary blob)
efr32mg24-thread-sys/     ← Thread (proprietary blob)
```

**Pros:**
- Clear separation (open-source peripherals vs proprietary radio)
- HAL stays portable
- Radio crates can be optional
- Matches industry (nrf-hal has no radio, nrf-softdevice separate)

**Cons:**
- More crates
- Users need multiple imports

**DECISION: Option B (Separate Radio Crates)**

**Rationale:**
1. **Licensing** - Radio uses proprietary Silicon Labs blobs (MSLA license)
2. **Portability** - HAL peripherals are generic, radio is chip-specific
3. **Industry standard** - Nordic separates `nrf-hal` and `nrf-softdevice`
4. **Optional** - Users who don't need BLE don't pull in 11MB of blobs

**Documented in:** docs/RADIO_ARCHITECTURE.md

---

### Decision 4: Where Does Embassy Live?

**Question**: Should Embassy support be a separate crate or a feature in bluetooth-sys?

**Options Considered:**

**Option A: Separate async crate**
```
crates/efr32mg24-bluetooth-sys/      ← FFI + blocking
efr32mg24-bluetooth-async/    ← Embassy async
```

**Pros:**
- Clean dependency separation
- No embassy deps unless needed

**Cons:**
- Yet another crate to maintain
- Duplicates types/enums
- Version sync issues

**Option B: Feature flag in same crate ✅ CHOSEN**
```
crates/efr32mg24-bluetooth-sys/
├── blocking/        ← Always available
└── asynch/          ← #[cfg(feature = "embassy")]
```

**Cargo.toml:**
```toml
[dependencies]
heapless = "0.8"
embassy-sync = { version = "0.6", optional = true }

[features]
embassy = ["embassy-sync", "embassy-time"]
```

**Pros:**
- Single crate to maintain
- Types shared easily
- Simple feature flag
- Blocking users pay zero cost

**Cons:**
- Crate has optional heavy dependencies
- Slightly larger Cargo.toml

**DECISION: Option B (Feature Flag)**

**Rationale:**
1. **Zero cost** - Blocking users don't pull embassy crates
2. **Type sharing** - Async reuses `blocking::BleEvent` etc.
3. **Simpler** - One crate to version/publish
4. **Matches esp-radio** - They use feature flags for async
5. **Future-proof** - Embassy support ready for Phase E

**Source Reference:**
- [nrf-softdevice features](https://lib.rs/crates/nrf-softdevice/features) uses feature flags

---

### Decision 5: Naming Convention (XiaoBle → Bluetooth)

**Question**: What should the generic BLE stack struct be called?

**Options Considered:**

**Option A: Keep XiaoBle name**
- Pro: Matches current code
- Con: XIAO-specific name for generic struct

**Option B: BleStack**
- Pro: Descriptive
- Con: Verbose

**Option C: Bluetooth ✅ CHOSEN**
- Pro: Simple, clear, matches Bluetooth protocol
- Con: None

**DECISION: Option C (Bluetooth)**

**Rationale:**
1. **Generic** - Works on any EFR32MG24, not just XIAO
2. **Simple** - Short, clear name
3. **Consistent** - Matches async `AsyncBluetooth`
4. **Industry** - nrf-softdevice uses `Softdevice`, we use `Bluetooth`

**Naming Table:**

| Layer | Struct Name | Location | Purpose |
|-------|-------------|----------|---------|
| Generic | `Bluetooth` | bluetooth-sys/blocking | Generic BLE stack |
| Generic | `AsyncBluetooth` | bluetooth-sys/asynch | Generic async BLE |
| Board | `XiaoBle` | xiao-mg24-bsp | XIAO-specific wrapper |

---

### Decision 6: Error Handling Strategy

**Question**: Should we use a single error enum or separate per-module?

**Options Considered:**

**Option A: Separate errors**
```rust
pub enum AdvertisingError { ... }
pub enum GattError { ... }
pub enum ConnectionError { ... }
```

**Pros:**
- Granular error types
- Type-safe per operation

**Cons:**
- Many error types
- Conversion overhead
- Complex API

**Option B: Single BleError enum ✅ CHOSEN**
```rust
pub enum BleError {
    InvalidParameter,
    NotConnected,
    BufferOverflow,
    BgapiError(u16),
    // ...
}
```

**Pros:**
- Simple API
- Easy to use (one Result type)
- Matches BGAPI flat error codes

**Cons:**
- Less granular
- Some errors don't apply to all operations

**DECISION: Option B (Single BleError)**

**Rationale:**
1. **Simple** - One `Result<T, BleError>` everywhere
2. **BGAPI alignment** - Silicon Labs uses flat error codes
3. **Practical** - Users just want to know "did it fail?"
4. **Matches pattern** - Most embedded crates use single error enum

---

### Decision 7: Event Polling vs Callbacks

**Question**: How should BLE events be delivered to the application?

**Options Considered:**

**Option A: Callback-based**
```rust
ble.on_connected(|conn| { ... });
ble.on_gatt_write(|attr, data| { ... });
```

**Pros:**
- Event-driven
- Familiar to C developers

**Cons:**
- Requires static lifetime management
- Complex in embedded (no heap)
- Interrupt context issues

**Option B: Polling-based ✅ CHOSEN**
```rust
loop {
    if let Some(event) = ble.poll_event() {
        match event { ... }
    }
}
```

**Pros:**
- Simple to implement
- No lifetime issues
- User controls when to poll
- Matches BGAPI `sl_bt_pop_event()`

**Cons:**
- Must poll in loop
- Can miss events if not polling

**DECISION: Option B (Polling)**

**Rationale:**
1. **BGAPI design** - Silicon Labs uses `sl_bt_pop_event()` (polling)
2. **Embedded-friendly** - No closures, no allocations
3. **Simple** - Clear control flow
4. **Async-ready** - Can wrap with async later (Phase E)

**Note:** Async API (Phase E) will provide `async fn next_event()` for event-driven style.

---

### Decision 8: GATT Handle Management

**Question**: How should GATT characteristic handles be managed?

**Options Considered:**

**Option A: Automatic builder**
```rust
let db = GattDatabase::new()
    .service(0x180D)
        .characteristic(0x2A37, props::NOTIFY)
        .build();
let handle = db.heart_rate_handle();
```

**Pros:**
- Automatic handle assignment
- Type-safe
- No magic numbers

**Cons:**
- Complex to implement
- Requires macro magic
- Not matching GATT Configurator

**Option B: Manual handles (current) ✅ CHOSEN**
```rust
const GATTDB_HEART_RATE: CharHandle = CharHandle(3);
ble.notify(GATTDB_HEART_RATE, &data)?;
```

**Pros:**
- Simple
- Matches GATT Configurator output
- Clear and explicit
- Works today

**Cons:**
- Manual synchronization with GATT Configurator
- Magic numbers
- Error-prone

**DECISION: Option B (Manual) for now, Option A deferred**

**Rationale:**
1. **Works now** - All 6 examples use manual handles successfully
2. **Simple** - No complex builder needed
3. **GATT Configurator** - Users already have handles from tool
4. **Future work** - Can add builder later as convenience (not required)

**Status:** Manual handles in production, GATT builder marked as "nice to have"

---

### Decision 9: BSP Wrapper Depth

**Question**: How much functionality should XiaoBle have?

**Options Considered:**

**Option A: Thick wrapper (current - WRONG)**
```rust
impl XiaoBle {
    // 700+ lines of BLE logic
    pub fn poll_event(&mut self) -> Option<BleEvent> {
        // Complex event parsing logic
    }
}
```

**Option B: Thin wrapper ✅ CHOSEN**
```rust
pub struct XiaoBle {
    bluetooth: Bluetooth,  // Wraps generic stack
}

impl XiaoBle {
    pub fn init(delay: &mut impl DelayNs) -> Result<Self, BleError> {
        let mut bluetooth = Bluetooth::new(delay)?;
        bluetooth.set_device_name("XIAO MG24")?;  // Only board-specific!
        Ok(Self { bluetooth })
    }

    // Just forward to bluetooth
    pub fn poll_event(&mut self) -> Option<BleEvent> {
        self.bluetooth.poll_event()
    }
}
```

**Pros:**
- Minimal code (~50 lines)
- Clear separation
- Easy to maintain
- Generic logic in bluetooth-sys

**Cons:**
- Indirection (one extra function call)

**DECISION: Option B (Thin Wrapper)**

**Rationale:**
1. **Single responsibility** - BSP only adds board defaults
2. **Reusable** - Generic logic in bluetooth-sys benefits all boards
3. **Industry standard** - Board crates are thin wrappers
4. **Maintainable** - Less code to update

**Board-specific items:**
- Default device name ("XIAO MG24")
- Pin configurations (future)
- Board-specific initialization

**Everything else:** In bluetooth-sys

---

### Decision 10: Module Visibility

**Question**: What should be public in the API?

**Chosen Strategy:**

```rust
// crates/efr32mg24-bluetooth-sys/src/lib.rs

// Raw FFI (for advanced users)
pub mod ffi {
    pub use super::ffi_impl::*;
}

// Blocking API (main API)
pub mod blocking {
    pub use super::blocking_impl::*;
}

// Async API (future)
#[cfg(feature = "embassy")]
pub mod asynch {
    pub use super::asynch_impl::*;
}

// Re-export blocking API at root for convenience
pub use blocking::*;
```

**Usage patterns:**

```rust
// Explicit (recommended)
use efr32mg24_bluetooth_sys::blocking::{Bluetooth, BleEvent};

// Convenient (also works)
use efr32mg24_bluetooth_sys::{Bluetooth, BleEvent};

// Raw FFI (advanced)
use efr32mg24_bluetooth_sys::ffi;
unsafe { ffi::sl_bt_system_start_bluetooth(); }
```

**Rationale:**
1. **Clear namespaces** - Easy to see what's blocking vs async
2. **Convenience** - Can import from root for blocking API
3. **Explicit** - Users can be explicit with `blocking::`
4. **Future-proof** - Async lives in separate namespace

---

## Summary of Decisions

| Decision | Chosen Option | Rationale |
|----------|---------------|-----------|
| 1. Crate structure | ONE crate (bluetooth-sys) | Minimal deps, simpler |
| 2. Module org | blocking/ + asynch/ | Clear namespaces |
| 3. HAL radio | Keep HAL radio-free | Separation of concerns |
| 4. Embassy | Feature flag in same crate | Zero cost for blocking |
| 5. Naming | Bluetooth (not XiaoBle) | Generic, not board-specific |
| 6. Errors | Single BleError enum | Simple API |
| 7. Events | Polling (not callbacks) | Matches BGAPI design |
| 8. GATT handles | Manual (builder future) | Works now, simple |
| 9. BSP wrapper | Thin (~50 lines) | Board defaults only |
| 10. Visibility | blocking:: + root re-export | Convenient + explicit |

---

## Code Movement Plan

### From BSP → bluetooth-sys/src/blocking/

| File | Source | Destination | Lines | Changes |
|------|--------|-------------|-------|---------|
| error.rs | crates/xiao-mg24-bsp/src/radio/ble/mod.rs | bluetooth-sys/src/blocking/error.rs | ~50 | Extract BleError enum |
| types.rs | crates/xiao-mg24-bsp/src/radio/ble/types.rs | bluetooth-sys/src/blocking/types.rs | ~350 | Move CharHandle, GapAdBuilder |
| events.rs | crates/xiao-mg24-bsp/src/radio/ble/events.rs | bluetooth-sys/src/blocking/events.rs | ~280 | Move BleEvent enum |
| stack.rs | crates/xiao-mg24-bsp/src/radio/ble/mod.rs | bluetooth-sys/src/blocking/stack.rs | ~700 | XiaoBle → Bluetooth |

**Total movement:** 1,380 lines from BSP → bluetooth-sys

### BSP becomes minimal

**crates/xiao-mg24-bsp/src/radio/ble.rs** (NEW - ~50 lines):
```rust
pub use efr32mg24_bluetooth_sys::blocking::*;

pub struct XiaoBle {
    bluetooth: Bluetooth,
}

impl XiaoBle {
    pub fn init(delay: &mut impl DelayNs) -> Result<Self, BleError> {
        let mut bluetooth = Bluetooth::new(delay)?;
        bluetooth.set_device_name("XIAO MG24")?;  // Board-specific
        Ok(Self { bluetooth })
    }

    // Forward all methods
    pub fn poll_event(&mut self) -> Option<BleEvent> {
        self.bluetooth.poll_event()
    }
    // ... etc
}
```

---

## Benefits of This Architecture

### For Users

✅ **Simple blocking API** - No Embassy required for basic apps
✅ **Optional async** - Add `features = ["embassy"]` when ready
✅ **Generic** - Can use bluetooth-sys directly (no BSP needed)
✅ **Board convenience** - BSP adds defaults for XIAO

### For Maintainers

✅ **Clear layers** - FFI, blocking, async, board wrapper
✅ **Reusable** - Generic code benefits all boards
✅ **Testable** - Can test bluetooth-sys independently
✅ **Documented** - Clear decisions for future protocols

### For Future Work

✅ **Zigbee/Thread** - Follow same pattern
✅ **Embassy** - asynch/ module ready (Phase E)
✅ **New boards** - Just wrap Bluetooth (like XiaoBle)
✅ **GATT builder** - Can add later without breaking changes

---

## References

**Industry Research:**
- [nrf-softdevice](https://github.com/embassy-rs/nrf-softdevice) - Nordic BLE pattern
- [esp-wifi/esp-radio](https://github.com/esp-rs/esp-wifi) - Espressif WiFi/BLE pattern
- [nrf-softdevice features](https://lib.rs/crates/nrf-softdevice/features) - Feature flags

**Project Documentation:**
- [docs/RADIO_ARCHITECTURE.md](RADIO_ARCHITECTURE.md) - Complete architecture guide
- [docs/STATUS.md](STATUS.md) - Project status
- [docs/BACKLOG.md](BACKLOG.md) - Task tracking

---

## Timeline

**2026-01-03 09:00** - Identified architectural problem (1,348 lines in wrong layer)
**2026-01-03 10:30** - Researched industry patterns (nrf, esp)
**2026-01-03 12:00** - Documented architecture (RADIO_ARCHITECTURE.md)
**2026-01-03 13:00** - Made decisions (this document)
**2026-01-03 13:30** - Starting refactoring...

---

**Document Status:** FINAL
**Applies To:** efr32mg24-bluetooth-sys refactoring (2026-01-03)
**Future Use:** Template for Zigbee/Thread/Matter implementations

<!-- META: last_updated=2026-01-03 version=1.0.0 status=final maintained-by=architecture-team decision-record=true template-version=1.0 -->
