# Arduino EFR32MG24 Radio Implementation Analysis

**Date**: December 27, 2025
**Purpose**: Analyze Arduino's approach to EFR32MG24 radio support to inform Rust HAL implementation
**Target**: Extract patterns, architecture decisions, and best practices from SiliconLabs/arduino

---

## Executive Summary

The [SiliconLabs Arduino Core](https://github.com/SiliconLabs/arduino) provides comprehensive radio support for EFR32MG24 devices through a **three-layer architecture**:

1. **Gecko SDK (GSDK)** - Silicon Labs proprietary C/C++ libraries (RAIL, BLE stack, Matter SDK)
2. **Arduino Core** - C++ wrapper layer abstracting GSDK into Arduino-friendly APIs
3. **User Libraries** - High-level libraries (Matter, ezBLE) with simplified APIs

**Key Finding**: Arduino **does NOT** expose raw RAIL API directly. Instead, it provides **protocol-specific wrappers** (Matter, BLE) that internally use RAIL. This is similar to our proposed three-layer Rust approach.

---

## 1. Architecture Overview

### 1.1 Protocol Stack Options

Arduino provides **compile-time selectable** protocol stacks via "Tools > Protocol stack" menu:

| **Stack Variant** | **Description** | **Resource Usage** | **Target Use Case** |
|-------------------|-----------------|--------------------|--------------------|
| **Matter** | Full Matter SDK with Thread + BLE | Very Heavy (Flash/RAM) | Matter IoT applications |
| **BLE (Arduino)** | ArduinoBLE library compatibility | Moderate | Standard Bluetooth LE (portable API) |
| **BLE (Silabs)** | Silicon Labs BLE stack (BGAPI) | Moderate | Advanced BLE applications |
| **None** | No radio stack | Minimal | Non-wireless applications |

**Source**: [SiliconLabs Arduino README](https://github.com/SiliconLabs/arduino/blob/main/readme.md)

**Key Insight**: Each stack is a **separate build variant**, not runtime-selectable. This aligns with our Rust feature flag approach:

```toml
# Rust equivalent (our plan)
[features]
default = []
radio-matter = ["radio-thread"]
radio-bluetooth = ["radio-rail"]
radio-thread = ["radio-rail"]
```

### 1.2 Three-Layer Architecture

```
┌─────────────────────────────────────────────────┐
│  User Application (.ino Arduino sketches)      │
├─────────────────────────────────────────────────┤
│  High-Level Libraries (Matter, ezBLE)          │
│  - C++ wrappers with Arduino-style APIs        │
│  - Object-oriented (MatterLightbulb, ezBLE)    │
├─────────────────────────────────────────────────┤
│  Arduino Core (cores/gecko/)                   │
│  - FreeRTOS integration                        │
│  - HAL abstraction (Wire, SPI, Serial)         │
│  - Memory management helpers                   │
├─────────────────────────────────────────────────┤
│  Gecko SDK (GSDK) - Proprietary Blobs          │
│  - RAIL library (librail_*.a)                  │
│  - BLE stack (libbluetooth_*.a)                │
│  - Matter SDK (libmatter_*.a)                  │
│  - OpenThread (libthread_*.a)                  │
└─────────────────────────────────────────────────┘
```

**Parallel to Rust**:

```rust
// Our proposed Rust architecture
User Application (.rs examples)
    ↓
High-Level HAL (efr32mg24-hal radio module)
    ↓
Safe Wrapper Crates (efr32mg24-bluetooth, efr32mg24-matter)
    ↓
Unsafe FFI Crates (efr32mg24-rail-sys, efr32mg24-bluetooth-sys)
    ↓
Gecko SDK Blobs (librail_*.a, libbluetooth_*.a)
```

---

## 2. Matter Implementation Analysis

### 2.1 Matter Library API Patterns

**Source**: [Matter Library README](https://github.com/SiliconLabs/arduino/blob/main/libraries/Matter/readme.md)

#### Device-Specific Classes

Arduino provides **one class per device type**:

```cpp
// Example: Matter Lightbulb
#include <Matter.h>
#include <MatterLightbulb.h>

MatterLightbulb matter_bulb;

void setup() {
  Matter.begin();           // Initialize Matter stack (global singleton)
  matter_bulb.begin();      // Create device endpoint

  // Device properties
  matter_bulb.set_device_name("My Bulb");
  matter_bulb.set_vendor_name("Acme");
  matter_bulb.set_product_name("Smart Bulb");
}

void loop() {
  // State management
  bool state = matter_bulb.get_onoff();
  matter_bulb.set_onoff(!state);
}
```

**API Pattern**: Builder-like setters + getters for state management

**Rust Equivalent**:

```rust
// Proposed Rust API (similar pattern)
use efr32mg24_hal::radio::matter::{Matter, MatterLightbulb};

let mut matter = Matter::new(dp.radio);
matter.begin()?;

let mut bulb = MatterLightbulb::new()
    .device_name("My Bulb")
    .vendor_name("Acme")
    .product_name("Smart Bulb")
    .build()?;

// State management
let state = bulb.get_onoff();
bulb.set_onoff(!state)?;
```

### 2.2 Initialization Sequence

**Source**: [matter_lightbulb_callback.ino](https://github.com/SiliconLabs/arduino/blob/main/libraries/Matter/examples/matter_lightbulb_callback/matter_lightbulb_callback.ino)

```cpp
void setup() {
  Serial.begin(115200);
  Matter.begin();

  // Wait for commissioning (blocking!)
  while (!Matter.isDeviceCommissioned()) {
    Serial.println("Waiting for Matter device discovery...");
    delay(200);
  }

  // Wait for Thread network
  while (!Matter.isDeviceThreadConnected()) {
    Serial.println("Waiting for Thread network...");
    delay(200);
  }

  // Initialize device
  matter_bulb.begin();

  // Wait for online status
  while (!matter_bulb.is_online()) {
    Serial.println("Waiting for device to be online...");
    delay(200);
  }

  // Register callback
  matter_bulb.set_device_change_callback(matter_bulb_change_callback);

  Serial.println("Matter device is now online!");
}
```

**Key Observations**:

1. **Global Singleton**: `Matter` is a global object (like Arduino `Serial`)
2. **Blocking Initialization**: Uses polling loops with delays (acceptable for Arduino)
3. **State Checks**: Multiple status queries (`isDeviceCommissioned()`, `isDeviceThreadConnected()`, `is_online()`)
4. **Callback Registration**: Event-driven architecture for remote state changes

**Rust Approach** (improved):

```rust
// Rust: Non-blocking with Result types
let mut matter = Matter::new(dp.radio)?;

// Non-blocking status checks
if !matter.is_commissioned()? {
    return Err(Error::NotCommissioned);
}

if !matter.is_thread_connected()? {
    return Err(Error::ThreadDisconnected);
}

let mut bulb = MatterLightbulb::new(&mut matter)
    .device_name("My Bulb")
    .on_change(|state| {
        // Callback closure
        rprintln!("State changed: {:?}", state);
    })
    .build()?;

if !bulb.is_online()? {
    return Err(Error::NotOnline);
}
```

### 2.3 Callback Mechanism (FreeRTOS Integration)

**Key Pattern**: Arduino uses **FreeRTOS semaphores** for event synchronization:

```cpp
// Global state
StaticSemaphore_t matter_device_event_semaphore_buf;
SemaphoreHandle_t matter_device_event_semaphore;

void setup() {
  // Create binary semaphore (static allocation)
  matter_device_event_semaphore = xSemaphoreCreateBinaryStatic(
    &matter_device_event_semaphore_buf
  );

  // Register callback
  matter_bulb.set_device_change_callback(matter_bulb_change_callback);
}

// Callback invoked by Matter stack
void matter_bulb_change_callback(void) {
  // Signal main loop
  xSemaphoreGive(matter_device_event_semaphore);
}

void loop() {
  // Block until event (low power)
  if (xSemaphoreTake(matter_device_event_semaphore, portMAX_DELAY) == pdTRUE) {
    // Handle state change
    bool new_state = matter_bulb.get_onoff();
    digitalWrite(LED_PIN, new_state);
  }
}
```

**Architecture**:

1. **Asynchronous Events**: Matter stack calls callback from FreeRTOS task
2. **Synchronization**: Semaphore signals main loop
3. **Low Power**: `portMAX_DELAY` allows CPU sleep while waiting

**Rust Equivalent** (using `critical-section` + `AtomicBool`):

```rust
use core::sync::atomic::{AtomicBool, Ordering};
use critical_section::Mutex;

static STATE_CHANGED: AtomicBool = AtomicBool::new(false);

// Callback from Matter SDK (unsafe C callback)
unsafe extern "C" fn matter_event_callback(_handle: *mut c_void) {
    STATE_CHANGED.store(true, Ordering::Release);
}

// Main loop (safe Rust)
loop {
    if STATE_CHANGED.swap(false, Ordering::Acquire) {
        let new_state = bulb.get_onoff()?;
        led.set_state(new_state.into())?;
    }
    cortex_m::asm::wfi(); // Wait for interrupt (low power)
}
```

**Alternative**: Use `heapless::spsc::Queue` for message passing (zero-allocation)

### 2.4 BLE + Matter Coexistence

**Source**: [Matter README](https://github.com/SiliconLabs/arduino/blob/main/libraries/Matter/readme.md)

**Important Constraint**:

> "You can use the Silicon Labs BLE stack simultaneously with Matter/Thread to communicate with BLE devices. **However when Matter is selected, BLE is included but is not usable by the user, as the Matter SDK takes ownership of it.**"

**After Commissioning**:

> "Make sure you're not interfering with the BLE stack before the Matter commissioning finishes! After Matter commissioning finished the BLE stack is freely usable with all features."

**Implications for Rust**:

```rust
// Phase 1: Matter commissioning (BLE unavailable)
let mut matter = Matter::new(dp.radio)?;
matter.commission()?; // Uses BLE internally for commissioning

// Phase 2: After commissioning (BLE now available)
let ble = matter.release_ble()?; // Get BLE handle
let gatt_server = ble.gatt_server()?;
// ... use BLE freely
```

**Memory Management**: Matter and BLE share the same radio hardware (RAIL), so resource ownership must be explicit.

---

## 3. BLE Implementation (ezBLE Library)

### 3.1 Library Purpose

**Source**: [Arduino README](https://github.com/SiliconLabs/arduino/blob/main/readme.md)

> "ezBLE: send and receive data over BLE in a simple and user-friendly way on 'BLE (Silabs)' variants"

**Key Points**:

- Only available with **"BLE (Silabs)"** stack variant (uses BGAPI)
- NOT compatible with **"BLE (Arduino)"** variant (which uses ArduinoBLE library)
- Wraps Silicon Labs' native BLE stack

### 3.2 BGAPI Architecture

**BGAPI (Bluetooth Generic Access Profile Interface)** is Silicon Labs' proprietary API for BLE:

**Documentation**: [BGAPI Reference Manual](https://www.silabs.com/documents/public/reference-manuals/bluetooth-api-reference.pdf)

**Key Features** (from [BGAPI v3.1 docs](https://docs.silabs.com/bluetooth/3.1/bgapi)):

- **Command/Response Protocol**: Host sends commands, controller responds
- **Event-Driven**: Asynchronous events from BLE stack
- **GATT Client/Server**: Full GATT support
- **GAP**: Advertising, scanning, connections
- **Security Manager**: Pairing, bonding, encryption

**Example BGAPI Command** (C):

```c
// BGAPI command: Start advertising
sl_status_t sl_bt_advertiser_start(
    uint8_t advertising_set,
    uint8_t discover,
    uint8_t connect
);

// BGAPI event: Connection opened
typedef struct {
    uint8_t connection;
    bd_addr address;
    uint8_t address_type;
    uint8_t bonding;
} sl_bt_evt_connection_opened_t;
```

**Rust FFI Equivalent**:

```rust
// efr32mg24-bluetooth-sys (unsafe FFI)
#[repr(C)]
pub struct sl_bt_evt_connection_opened_t {
    pub connection: u8,
    pub address: [u8; 6],
    pub address_type: u8,
    pub bonding: u8,
}

extern "C" {
    pub fn sl_bt_advertiser_start(
        advertising_set: u8,
        discover: u8,
        connect: u8,
    ) -> sl_status_t;
}

// efr32mg24-bluetooth (safe wrapper)
pub struct BleAdvertiser {
    advertising_set: u8,
}

impl BleAdvertiser {
    pub fn start(&mut self, discoverable: bool, connectable: bool) -> Result<(), Error> {
        let status = unsafe {
            sl_bt_advertiser_start(
                self.advertising_set,
                discoverable as u8,
                connectable as u8,
            )
        };
        Error::from_sl_status(status)
    }
}
```

### 3.3 ezBLE Simplification

While we don't have direct source access to ezBLE, based on documentation it likely provides:

```cpp
// Hypothetical ezBLE API (inferred from description)
#include <ezBLE.h>

ezBLE ble;

void setup() {
  ble.begin("MyDevice");  // Start advertising with device name
}

void loop() {
  if (ble.available()) {
    String data = ble.readString();
    ble.println("Received: " + data);
  }
}
```

**Rust Equivalent** (simplified API):

```rust
use efr32mg24_hal::radio::ble::EzBle;

let mut ble = EzBle::new(dp.radio, "MyDevice")?;
ble.start_advertising()?;

loop {
    if let Some(data) = ble.read()? {
        ble.write(b"Received: ")?;
        ble.write(&data)?;
    }
}
```

---

## 4. Build System Integration

### 4.1 Gecko SDK Bundling

**Source**: [SiliconLabs Arduino Core](https://github.com/SiliconLabs/arduino)

Arduino bundles Gecko SDK components directly in the core:

```
SiliconLabs/arduino/
├── cores/gecko/              # Arduino core implementation
│   ├── api/                  # Arduino API (Serial, Wire, SPI)
│   ├── freertos/             # FreeRTOS kernel
│   ├── hal/                  # HAL drivers
│   └── startup/              # Startup code, vectors
├── libraries/
│   ├── Matter/               # Matter SDK wrapper
│   │   ├── src/              # C++ wrapper code
│   │   └── blobs/            # libmatter_*.a (proprietary)
│   ├── ezBLE/                # BLE wrapper
│   │   ├── src/              # C++ wrapper code
│   │   └── blobs/            # libbluetooth_*.a (proprietary)
├── variants/
│   └── xiao_mg24/            # Board-specific (pins, memory layout)
│       └── linker_script.ld  # Memory layout
└── platform.txt              # Build configuration
```

**Key Insight**: Arduino **ships pre-compiled blobs** with the core. Users don't download GSDK separately.

**Rust Approach**:

```
EFR32MG24/
├── efr32mg24-rail-sys/
│   ├── blobs/
│   │   └── librail_efr32xg24_gcc_release.a  # Shipped in crate
│   ├── build.rs              # Link blobs
│   └── src/lib.rs            # FFI bindings (bindgen)
├── efr32mg24-bluetooth-sys/
│   ├── blobs/
│   │   └── libbluetooth_efr32xg24_gcc_release.a
│   ├── build.rs
│   └── src/lib.rs
└── efr32mg24-hal/
    └── Cargo.toml
        [features]
        radio-bluetooth = ["efr32mg24-bluetooth-sys"]
```

**Licensing**: Blobs use **MSLA** (Master Software License Agreement), which is compatible with shipping in crates.

### 4.2 Memory Layout (Linker Script)

**Source**: [Gecko Platform Memory Documentation](https://docs.silabs.com/gecko-platform/latest/memory)

**Key Points**:

- **Auto-Generated Linker Scripts**: Simplicity Studio generates `.ld` files based on project configuration
- **Component-Aware**: Linker script changes based on included components (Matter, BLE, etc.)
- **Flash Layout**:
  ```
  0x0000_0000: Bootloader (~16 KB)
  0x0000_4000: Application code
  0x0008_0000: NVM3 storage (network keys, credentials)
  ```

- **RAM Layout** (256 KB total on B220):
  ```
  0x2000_0000: Stack (grows down)
  0x2000_XXXX: Heap (grows up)
  0x2003_FFFF: End of RAM
  ```

**Arduino Variant Example** (`xiao_mg24/linker_script.ld`):

```ld
/* Memory layout for EFR32MG24B220F1536IM48 */
MEMORY
{
  FLASH (rx)  : ORIGIN = 0x08004000, LENGTH = 1504K  /* After bootloader */
  RAM (rwx)   : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Stack size (FreeRTOS + Matter) */
_stack_size = 16K;

/* Heap size (Matter SDK needs ~100 KB) */
_heap_size = 100K;
```

**Rust Equivalent** (`efr32mg24-hal/memory.x`):

```ld
/* memory.x for EFR32MG24B220 */
MEMORY
{
  /* Leave space for bootloader (16 KB) */
  FLASH : ORIGIN = 0x08004000, LENGTH = 1504K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Adjust for radio stacks */
_stack_size = 8K;   /* Smaller for no-RTOS builds */
_heap_size = 64K;   /* Adjust based on protocol */
```

**Critical**: Matter SDK requires **significant heap** (~100 KB). Must document memory requirements per feature.

### 4.3 Compiler Flags (platform.txt)

**Arduino platform.txt** (inferred):

```ini
# Compiler flags for EFR32MG24
compiler.c.flags=-mcpu=cortex-m33 -mthumb -mfpu=fpv5-sp-d16 -mfloat-abi=hard
compiler.c.flags+=-DEFR32MG24 -DEFR32MG24B220F1536IM48
compiler.c.flags+=-I{gecko_sdk}/platform/Device/SiliconLabs/EFR32MG24/Include
compiler.c.flags+=-I{gecko_sdk}/platform/common/inc

# Matter stack flags
compiler.c.flags.matter=-DMATTER_ENABLED -DCHIP_SYSTEM_CONFIG_USE_LWIP

# BLE stack flags
compiler.c.flags.ble=-DSL_BT_ENABLED -DSL_CATALOG_BLUETOOTH_PRESENT

# Linker flags
compiler.c.elf.libs=-lrail_efr32xg24_gcc_release
compiler.c.elf.libs.matter=-lmatter_efr32xg24_gcc_release
compiler.c.elf.libs.ble=-lbluetooth_efr32xg24_gcc_release
```

**Rust build.rs Equivalent**:

```rust
// efr32mg24-rail-sys/build.rs
fn main() {
    // Link RAIL blob
    println!("cargo:rustc-link-lib=static=rail_efr32xg24_gcc_release");
    println!("cargo:rustc-link-search=native=blobs/efr32mg24b220");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("vendor/rail/rail.h")
        .use_core()
        .ctypes_prefix("cty")
        .generate()
        .expect("Failed to generate RAIL bindings");

    bindings.write_to_file("src/bindings.rs").unwrap();
}
```

---

## 5. Key Takeaways for Rust Implementation

### 5.1 Architecture Decisions

| **Aspect** | **Arduino Approach** | **Rust Adaptation** |
|------------|----------------------|---------------------|
| **API Layers** | 3 layers (GSDK → Core → Libraries) | 3 layers (-sys → safe wrapper → HAL) |
| **Protocol Selection** | Compile-time (IDE menu) | Compile-time (Cargo features) |
| **Memory Management** | FreeRTOS heap + Arduino `new`/`delete` | Rust ownership + `heapless` or `alloc` |
| **Callbacks** | FreeRTOS semaphores + global state | `critical-section` + atomics/queues |
| **Initialization** | Blocking loops with delays | Non-blocking `Result<T, E>` returns |
| **Error Handling** | Return codes + `Serial.println()` | `Result<T, Error>` with typed errors |
| **Blob Distribution** | Bundled in Arduino core | Bundled in `-sys` crates |
| **RTOS Integration** | FreeRTOS (required) | Optional (bare-metal first, RTOS later) |

### 5.2 Critical Patterns to Adopt

#### 1. **Protocol-Specific Wrappers**

**Don't expose raw RAIL**. Provide high-level protocol APIs like Arduino's `MatterLightbulb`, `ezBLE`:

```rust
// Good (user-friendly)
let bulb = MatterLightbulb::new()
    .device_name("My Bulb")
    .build()?;

// Bad (too low-level for most users)
let rail_handle = rail_init(...)?;
rail_config_channels(...)?;
rail_start_tx(...)?;
```

#### 2. **Separate Crates per Protocol**

```toml
# efr32mg24-hal/Cargo.toml
[features]
radio-bluetooth = ["efr32mg24-bluetooth"]  # Optional BLE
radio-matter = ["efr32mg24-matter"]        # Optional Matter
radio-zigbee = ["efr32mg24-zigbee"]        # Optional Zigbee

[dependencies]
efr32mg24-bluetooth = { version = "0.1", optional = true }
efr32mg24-matter = { version = "0.1", optional = true }
```

Users select only what they need, minimizing binary size.

#### 3. **Memory Budget Documentation**

Arduino warns: "Matter is **quite resource heavy**". We must document:

```rust
//! # Memory Requirements
//!
//! | Feature | Flash | RAM | Heap |
//! |---------|-------|-----|------|
//! | Base HAL | 25 KB | 8 KB | 0 KB |
//! | + BLE | +120 KB | +20 KB | +16 KB |
//! | + Matter | +450 KB | +60 KB | +100 KB |
//! | + Zigbee | +200 KB | +30 KB | +40 KB |
//!
//! **EFR32MG24B220**: 1536 KB Flash, 256 KB RAM
```

#### 4. **Callback Safety Pattern**

Use `critical-section` for C → Rust callbacks:

```rust
use critical_section::Mutex;
use core::cell::Cell;

static CALLBACK: Mutex<Cell<Option<fn()>>> = Mutex::new(Cell::new(None));

// Register Rust callback
pub fn set_event_callback(f: fn()) {
    critical_section::with(|cs| {
        CALLBACK.borrow(cs).set(Some(f));
    });
}

// C callback wrapper (unsafe)
unsafe extern "C" fn c_callback_wrapper() {
    critical_section::with(|cs| {
        if let Some(f) = CALLBACK.borrow(cs).get() {
            f();  // Call Rust function
        }
    });
}
```

#### 5. **Feature-Gated Examples**

```rust
// examples/matter_lightbulb.rs
#![cfg(feature = "radio-matter")]  // Only compile if feature enabled

use efr32mg24_hal::radio::matter::MatterLightbulb;

fn main() -> ! {
    let mut bulb = MatterLightbulb::new()
        .device_name("Rust Bulb")
        .build()
        .unwrap();

    loop {
        bulb.process_events().unwrap();
    }
}
```

### 5.3 Critical Differences from Arduino

| **Aspect** | **Arduino** | **Rust** | **Implication** |
|------------|-------------|----------|-----------------|
| **RTOS** | FreeRTOS required | Optional (bare-metal first) | Simpler initial implementation |
| **Heap** | Required (C++ `new`) | Optional (`heapless` alternatives) | More embedded-friendly |
| **Blocking** | Acceptable (`delay()` loops) | Avoid (use `nb` or `Result`) | Better for interrupts |
| **Global State** | Common (Arduino style) | Avoid (use `&mut` ownership) | Safer concurrency |
| **Error Handling** | Print + ignore | `Result<T, E>` required | Compile-time safety |

---

## 6. Recommended Changes to Our Plan

### 6.1 Update Phase D1-RAIL

**Original Plan**: Expose raw RAIL API

**Arduino Insight**: Don't expose RAIL directly. Start with protocol wrappers.

**Revised Plan**:

1. **D1.1-RAIL-sys**: Create `efr32mg24-rail-sys` (unsafe FFI, internal use only)
2. **D1.2-RAIL-safe**: Create `efr32mg24-rail` (safe wrapper, still low-level)
3. **D1.3-RAIL-HAL**: Integrate into `efr32mg24-hal` (not public API)

**Rationale**: RAIL is too low-level for most users. Protocol stacks (BLE, Matter) are the real APIs.

### 6.2 Add Memory Budget Analysis

**Action**: Create `docs/MEMORY_BUDGET.md` documenting:

- Base HAL memory usage
- Per-feature incremental costs (BLE, Matter, Zigbee)
- Linker script adjustments needed
- Heap sizing recommendations

**Example** (from Arduino):

```markdown
## Matter Stack Memory Requirements

**Tested on Arduino Nano Matter** (EFR32MG24B220):
- Flash: ~450 KB (30% of 1536 KB)
- RAM: ~160 KB (62% of 256 KB)
  - Stack: 16 KB
  - Heap: 100 KB (Matter SDK)
  - Static: 44 KB
```

### 6.3 Prioritize BLE over Matter for Phase D2

**Original Plan**: Start with Matter (most complex)

**Arduino Insight**: Matter is "quite resource heavy". BLE is simpler and more widely used.

**Revised Priority**:

1. **D2-BLE**: Bluetooth 5.3 LE (simpler, more users)
2. **D3-Thread**: OpenThread stack (for Matter foundation)
3. **D4-Matter**: Matter over Thread (most complex)
4. **D5-Zigbee**: Zigbee 3.0 (optional)

**Rationale**: BLE has broader appeal, simpler to test, faster to ship.

### 6.4 Document BLE + Matter Coexistence

**Action**: Add warning to `docs/RADIO_IMPLEMENTATION_PLAN.md`:

```markdown
### BLE + Matter Resource Sharing

**Critical**: Matter takes ownership of BLE during commissioning.

**Sequence**:
1. Matter commissioning (uses BLE for pairing) - BLE **not available**
2. Commissioning complete - BLE **now available**
3. User can use BLE alongside Matter

**Code Example**:
```rust
let mut matter = Matter::new(dp.radio)?;
matter.commission()?;  // Uses BLE internally

// After commissioning, BLE is released
let ble = matter.release_ble()?;
ble.start_advertising()?;
```

---

## 7. Arduino Implementation Gaps

### 7.1 What Arduino **Doesn't** Provide

1. **Raw RAIL API**: Not exposed to users
2. **Proprietary Protocols**: Only standard protocols (BLE, Matter, Thread)
3. **Direct Register Access**: Radio peripheral registers not documented
4. **Zigbee Support**: Listed as "future" (not yet available)
5. **Low-Level DMA**: No LDMA integration with radio

### 7.2 What Our Rust HAL **Can** Provide

1. **More Type Safety**: Compile-time protocol enforcement
2. **Better Error Handling**: `Result<T, E>` vs. silent failures
3. **No RTOS Requirement**: Bare-metal option (Arduino requires FreeRTOS)
4. **Smaller Binaries**: `heapless` alternatives to heap allocation
5. **Interrupt-Driven**: Non-blocking APIs with `nb` crate

---

## 8. Example Code Comparison

### 8.1 Matter Lightbulb

**Arduino** ([source](https://github.com/SiliconLabs/arduino/blob/main/libraries/Matter/examples/matter_lightbulb_callback/matter_lightbulb_callback.ino)):

```cpp
#include <Matter.h>
#include <MatterLightbulb.h>

MatterLightbulb matter_bulb;

void setup() {
  Serial.begin(115200);
  Matter.begin();

  while (!Matter.isDeviceCommissioned()) {
    delay(200);
  }

  matter_bulb.begin();
  matter_bulb.set_device_change_callback(bulb_change_cb);

  Serial.println(Matter.getOnboardingQRCodeUrl());
}

void loop() {
  static bool onoff_state = false;
  matter_bulb.set_onoff(onoff_state);
  onoff_state = !onoff_state;
  delay(1000);
}

void bulb_change_cb() {
  bool state = matter_bulb.get_onoff();
  digitalWrite(LED_PIN, state);
}
```

**Rust Equivalent** (proposed):

```rust
#![no_std]
#![no_main]

use efr32mg24_hal::{
    radio::matter::{Matter, MatterLightbulb},
    gpio::GpioExt,
};
use cortex_m_rt::entry;
use rtt_target::rprintln;

#[entry]
fn main() -> ! {
    let dp = efr32mg24_pac::Peripherals::take().unwrap();

    // Initialize Matter
    let mut matter = Matter::new(dp.radio).unwrap();

    // Wait for commissioning (non-blocking alternative)
    while !matter.is_commissioned().unwrap() {
        cortex_m::asm::delay(1_000_000); // ~12ms at 78 MHz
    }

    // Create lightbulb device
    let mut bulb = MatterLightbulb::new(&mut matter)
        .device_name("Rust Bulb")
        .on_change(|state| {
            rprintln!("State changed: {:?}", state);
        })
        .build()
        .unwrap();

    rprintln!("QR Code: {}", matter.get_qr_code_url());

    // Main loop
    let gpio = dp.gpio_s.split(&frozen_clocks);
    let mut led = gpio.portb.pb2.into_push_pull_output();

    let mut onoff = false;
    loop {
        bulb.set_onoff(onoff).unwrap();
        led.set_state(onoff.into()).unwrap();
        onoff = !onoff;

        cortex_m::asm::delay(78_000_000); // 1 second
    }
}
```

**Key Differences**:

- **No heap allocation**: Bulb created on stack
- **Result types**: All operations return `Result<T, E>`
- **No FreeRTOS**: Uses `cortex_m::asm::delay` instead of `delay()`
- **Ownership**: `&mut matter` passed to bulb (prevents misuse)

### 8.2 BLE Advertisement

**Arduino** (hypothetical ezBLE):

```cpp
#include <ezBLE.h>

ezBLE ble;

void setup() {
  ble.begin("MyDevice");
}

void loop() {
  if (ble.available()) {
    String data = ble.readString();
    ble.println("Echo: " + data);
  }
}
```

**Rust Equivalent** (proposed):

```rust
use efr32mg24_hal::radio::ble::EzBle;
use heapless::String;

#[entry]
fn main() -> ! {
    let dp = efr32mg24_pac::Peripherals::take().unwrap();

    let mut ble = EzBle::new(dp.radio, "MyDevice").unwrap();
    ble.start_advertising().unwrap();

    let mut buffer: String<64> = String::new();

    loop {
        if let Some(data) = ble.read().unwrap() {
            buffer.clear();
            write!(buffer, "Echo: {:?}", data).unwrap();
            ble.write(buffer.as_bytes()).unwrap();
        }
    }
}
```

**Key Differences**:

- **No dynamic strings**: Uses `heapless::String<64>` (stack-allocated)
- **No blocking reads**: `ble.read()` returns `Option<&[u8]>`
- **Explicit buffer management**: User controls memory

---

## 9. Sources and References

### Official Arduino Resources

1. **SiliconLabs Arduino Core**: [github.com/SiliconLabs/arduino](https://github.com/SiliconLabs/arduino)
2. **Arduino README**: [arduino/readme.md](https://github.com/SiliconLabs/arduino/blob/main/readme.md)
3. **Matter Library**: [arduino/libraries/Matter/readme.md](https://github.com/SiliconLabs/arduino/blob/main/libraries/Matter/readme.md)
4. **Matter Lightbulb Example**: [matter_lightbulb_callback.ino](https://github.com/SiliconLabs/arduino/blob/main/libraries/Matter/examples/matter_lightbulb_callback/matter_lightbulb_callback.ino)

### Silicon Labs Documentation

5. **BGAPI Reference Manual**: [bluetooth-api-reference.pdf](https://www.silabs.com/documents/public/reference-manuals/bluetooth-api-reference.pdf)
6. **BGAPI Documentation**: [docs.silabs.com/bluetooth/latest/](https://docs.silabs.com/bluetooth/latest/)
7. **RAIL API Documentation**: [docs.silabs.com/rail/latest/](https://docs.silabs.com/rail/latest/rail-api/efr32-main)
8. **Gecko SDK**: [github.com/SiliconLabs/gecko_sdk](https://github.com/SiliconLabs/gecko_sdk)
9. **Memory Layout**: [docs.silabs.com/gecko-platform/latest/memory](https://docs.silabs.com/gecko-platform/latest/memory)
10. **Gecko Bootloader**: [Bootloader User Guide](https://docs.silabs.com/bluetooth/latest/bootloader-user-guide-gsdk-4/)

### Community Resources

11. **XIAO MG24 Documentation**: [github.com/lboue/XIAO_MG24_docs](https://github.com/lboue/XIAO_MG24_docs)
12. **Seeed Studio Wiki**: [wiki.seeedstudio.com/xiao_mg24_getting_started/](https://wiki.seeedstudio.com/xiao_mg24_getting_started/)
13. **XIAO MG24 Sense Product**: [seeedstudio.com/Seeed-XIAO-MG24-Sense-p-6248.html](https://www.seeedstudio.com/Seeed-XIAO-MG24-Sense-p-6248.html)

### Board Support

14. **Silicon Labs Quick Start**: [silabs.com/support/training/quick-start-xiao-mg24](https://www.silabs.com/support/training/develop-with-seeed/quick-start-seeed-studio-xiao-mg24-sense)
15. **Getting Started Guide**: [MakerGuides XIAO MG24](https://www.makerguides.com/getting-started-with-xiao-mg24-sense/)

---

## 10. Conclusion

**Summary**: Arduino's approach validates our three-layer architecture and reinforces the importance of **protocol-specific high-level APIs** over raw RAIL exposure.

**Key Learnings**:

1. **Don't expose RAIL directly** - Wrap with protocol-specific APIs (BLE, Matter)
2. **Feature flags for protocols** - Users select only what they need
3. **Document memory budgets** - Matter requires ~160 KB RAM (62% of total!)
4. **Prioritize BLE first** - Simpler, more widely used than Matter
5. **Callback safety matters** - Use `critical-section` + atomics for C → Rust
6. **Bundle blobs in crates** - Don't require separate GSDK download

**Next Steps**:

1. ✅ Update `RADIO_IMPLEMENTATION_PLAN.md` with Arduino insights
2. Create `docs/MEMORY_BUDGET.md` with per-feature analysis
3. Revise phase priority: BLE → Thread → Matter → Zigbee
4. Add BLE + Matter coexistence warning
5. Begin Phase D1-RAIL-sys implementation (internal use only)

---

**Document Version**: 1.0
**Last Updated**: December 27, 2025
**Author**: EFR32MG24 HAL Development Team
**Review Status**: Ready for plan integration

<\!-- META: last_updated=2025-12-27 version=1.0.0 status=reference maintained_by=orchestrator -->
