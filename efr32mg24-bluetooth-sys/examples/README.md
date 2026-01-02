# Bluetooth Stack Examples (BGAPI)

This directory contains examples for using the Silicon Labs Bluetooth stack (BGAPI) on EFR32MG24.

## Important: Layer Architecture

These examples demonstrate **protocol stack usage**, not HAL-level abstractions:

```
┌─────────────────────────────────────────┐
│  Application Code (Your App)           │
├─────────────────────────────────────────┤
│  efr32mg24-bluetooth-sys (This Crate)  │ ← BGAPI Protocol Stack
│  - BLE advertising, GATT, connections  │   (these examples)
├─────────────────────────────────────────┤
│  efr32mg24-hal                          │ ← Basic Peripherals Only
│  - Clocks, GPIO, SPI, I2C, timers      │   (no BLE)
├─────────────────────────────────────────┤
│  efr32mg24-pac                          │ ← Register Access
│  - Direct hardware registers           │
└─────────────────────────────────────────┘
```

**Key Points:**
- BLE functionality is NOT in the HAL layer (industry standard)
- These examples use BGAPI FFI directly (like nrf-softdevice, esp-radio)
- HAL is used only for basic initialization (clocks, GPIO)
- For board-specific examples, see `xiao-mg24-bsp/examples/`

## Example Structure

Each example follows this pattern:

```rust
#![no_std]
#![no_main]

use efr32mg24_hal as hal;           // For clocks, GPIO initialization
use efr32mg24_bluetooth_sys as bt;  // For BLE stack (BGAPI)

#[entry]
fn main() -> ! {
    // 1. Initialize HAL peripherals (clocks, GPIO)
    let dp = pac::Peripherals::take().unwrap();
    let clocks = hal::clock::ClockConfig::default().freeze(dp.CMU_S);

    // 2. Initialize BLE stack (bluetooth-sys FFI)
    unsafe {
        bt::sl_bt_system_start_bluetooth();
        // ... configure advertising, GATT, etc.
    }

    // 3. Event loop
    loop {
        // BLE event handling
    }
}
```

## Available Examples

### 01_ble_beacon.rs ✅ Complete

**Purpose**: Non-connectable BLE beacon (advertising only)

**What it shows**:
- Starting the Bluetooth stack
- Creating an advertising set
- Setting advertising data (device name, manufacturer data)
- Configuring advertising parameters (interval, TX power)
- Starting non-connectable advertising

**Hardware**: XIAO MG24 Sense
**Expected behavior**: Appears as "XIAO Beacon" in BLE scanner apps
**Build time**: ~30 seconds (after bluetooth-sys is built)

**Build**:
```bash
cargo build --example 01_ble_beacon --release
```

**Flash** (requires probe-rs):
```bash
cargo run --example 01_ble_beacon --release
```

---

### 02_ble_advertiser.rs 📋 Planned

**Purpose**: Connectable BLE peripheral (advertising + connections)

**What it will show**:
- Connectable advertising
- Connection event handling
- Connection parameter negotiation
- Disconnection handling

**Expected lines**: ~200-250

---

### 03_ble_gatt.rs 📋 Planned

**Purpose**: GATT server (Heart Rate Service)

**What it will show**:
- Creating GATT services
- Adding characteristics
- Reading/writing characteristic values
- Sending notifications to clients
- Standard Bluetooth SIG services

**Expected lines**: ~300-400

---

### 04_ble_peripheral.rs 📋 Planned

**Purpose**: Complete BLE peripheral (custom service)

**What it will show**:
- Custom 128-bit UUID service
- Multiple characteristics
- Read, write, notify operations
- CCCD (Client Characteristic Configuration Descriptor)
- Connection management

**Expected lines**: ~350-450

---

## Prerequisites

Before running these examples, ensure:

1. **Bluetooth stack is set up**:
   ```bash
   cd efr32mg24-bluetooth-sys
   ./scripts/setup-bluetooth.sh
   ```

2. **Blobs are extracted and verified**:
   - `blobs/efr32mg24b220/libbluetooth_controller_efr32xg24_gcc_release.a` (9.0 MB)
   - `blobs/efr32mg24b220/libbluetooth_host_efr32xg24_gcc_release.a` (1.4 MB)
   - `blobs/efr32mg24b220/libbgcommon_efr32xg24_gcc_release.a` (264 KB)
   - SHA256 verification passes in build

3. **BGAPI headers are available**:
   - `vendor/bluetooth/sl_bt_api.h` (815 KB)
   - `vendor/bluetooth/sl_status.h` (53 KB)

## Building Examples

### Check Compilation

Fast error checking (no binary generation):

```bash
cargo check --example 01_ble_beacon
```

**Build time**: ~5-10 seconds (incremental)

### Build for Hardware

Generate binary for flashing:

```bash
cargo build --example 01_ble_beacon --release
```

**Build time**: ~30 seconds (incremental after first build)

### Build All Examples

```bash
cargo build --examples --release
```

**Build time**: ~1-2 minutes (all examples)

### Flash to Device

Requires `probe-rs` and a debug probe:

```bash
# Flash and run with RTT logging
cargo run --example 01_ble_beacon --release
```

## Testing Examples

### Without Hardware (Compilation Check)

```bash
# Verify all examples compile
cargo build --examples --release
```

**Purpose**: Catch compilation errors, API misuse

### With Hardware (Integration Testing)

**Required**:
- XIAO MG24 Sense board
- USB-C cable (power + debug)
- Smartphone with BLE scanner app

**Procedure**:
1. Flash example to device
2. Open BLE scanner app (nRF Connect, LightBlue)
3. Scan for devices
4. Verify beacon appears with correct name
5. Check advertising data matches expected values

**Expected results**:
- Device visible in scanner
- Device name: "XIAO Beacon"
- Advertising interval: 100ms
- RSSI: -40 to -80 dBm (depending on distance)

## BLE Scanner Apps

Use these apps to verify beacon operation:

**iOS**:
- [LightBlue](https://apps.apple.com/app/lightblue/id557428110) (Free)
- Shows: device name, RSSI, advertising data

**Android**:
- [nRF Connect](https://play.google.com/store/apps/details?id=no.nordicsemi.android.mcp) (Free)
- Shows: device name, RSSI, advertising data, raw packet viewer

**Desktop**:
- [nRF Connect Desktop](https://www.nordicsemi.com/Products/Development-tools/nRF-Connect-for-desktop) (Free, Windows/macOS/Linux)
- Advanced features: packet sniffer, raw data viewer

## Example Development Workflow

### 1. Start with Beacon

Simplest example to verify Bluetooth stack works:

```bash
cargo build --example 01_ble_beacon --release
cargo run --example 01_ble_beacon --release
```

Scan with phone → See "XIAO Beacon" → Success!

### 2. Add GATT Server

Create custom service with characteristics:

```bash
cargo build --example 03_ble_gatt --release
cargo run --example 03_ble_gatt --release
```

Connect from phone → Read/write characteristics → See notifications

### 3. Build Complete Peripheral

Full-featured BLE device:

```bash
cargo build --example 04_ble_peripheral --release
cargo run --example 04_ble_peripheral --release
```

## BGAPI Reference

Key function categories used in examples:

### System Functions
- `sl_bt_system_start_bluetooth()` - Initialize BLE stack
- `sl_bt_system_get_version()` - Get stack version
- `sl_bt_system_set_tx_power()` - Set global TX power

### Advertiser Functions
- `sl_bt_advertiser_create_set()` - Create advertising set
- `sl_bt_advertiser_set_data()` - Set advertising data
- `sl_bt_advertiser_set_timing()` - Configure intervals
- `sl_bt_advertiser_start()` - Start advertising
- `sl_bt_advertiser_stop()` - Stop advertising

### GATT Server Functions
- `sl_bt_gatt_server_write_attribute_value()` - Update characteristic
- `sl_bt_gatt_server_send_notification()` - Notify clients
- `sl_bt_gatt_server_send_indication()` - Indicate clients

### Connection Functions
- `sl_bt_connection_set_parameters()` - Update connection params
- `sl_bt_connection_close()` - Disconnect

### Event System
- `sl_bt_wait_event()` - Blocking event wait
- `sl_bt_pop_event()` - Non-blocking event check

**Complete API**: See `vendor/bluetooth/sl_bt_api.h` (815 KB, 265 functions)

## Common Issues

### Build Fails: "bluetooth blob not found"

**Solution**: Run setup script first:
```bash
cd efr32mg24-bluetooth-sys
./scripts/setup-bluetooth.sh
```

### Build Fails: "SHA256 mismatch"

**Problem**: Corrupted blob download
**Solution**: Delete and re-download:
```bash
rm -rf blobs/efr32mg24b220/*.a
./scripts/setup-bluetooth.sh
```

### Example Compiles but Device Not Visible

**Troubleshooting**:
1. Verify antenna connected (XIAO MG24 has PCB antenna)
2. Check TX power set (0 dBm = moderate range)
3. Verify advertising started (check return codes)
4. Try different advertising interval (faster = more visible)
5. Check BLE scanner app settings (filters, RSSI threshold)

### Connection Immediately Drops

**Common causes**:
- GATT database not initialized
- Event loop not running
- Missing connection parameter response
- Insufficient memory for connection

**Solution**: Ensure event loop processes `connection` events

## Memory Requirements

### Flash (Code Size)
- Bluetooth stack blobs: ~11 MB (linked but not in application flash)
- Example code: ~50-100 KB per example
- Total application flash: ~150-200 KB

### RAM (Runtime)
- BLE stack heap: ~36 KB
- Connection buffers: ~4 KB per connection
- Application stack: ~4-8 KB
- Total RAM: ~50-60 KB for basic beacon

**XIAO MG24 has 256 KB RAM, 1536 KB Flash** - plenty for BLE applications

## Next Steps

**After mastering these examples**:

1. **Add sensors**: Combine BLE with I2C sensors (see `xiao-mg24-bsp/examples/`)
2. **Multi-protocol**: Use Thread or Zigbee alongside BLE
3. **Async runtime**: Integrate with Embassy for async/await BLE
4. **Production**: Add error handling, watchdog, low-power modes

## References

**Silicon Labs Documentation**:
- [Bluetooth API Reference](https://docs.silabs.com/bluetooth/latest/bluetooth-stack-api/) - Complete BGAPI documentation
- [AN1260: Bluetooth Advertising](https://www.silabs.com/documents/public/application-notes/an1260-bluetooth-advertising.pdf)
- [UG434: Bluetooth GATT Configurator](https://www.silabs.com/documents/public/user-guides/ug434-bluetooth-gatt-configurator.pdf)

**Bluetooth Specification**:
- [Bluetooth Core Spec 5.3](https://www.bluetooth.com/specifications/specs/core-specification-5-3/)
- [GATT Services](https://www.bluetooth.com/specifications/specs/)

**Example Projects**:
- [nrf-softdevice examples](https://github.com/embassy-rs/nrf-softdevice/tree/master/examples) - Similar architecture
- [TrouBLE examples](https://github.com/embassy-rs/trouble/tree/main/examples) - Pure Rust BLE stack

---

**Remember**: Bluetooth is complex. Start simple (beacon), then progress to more advanced features (GATT, connections, multi-protocol).

<!-- META: last_updated=2025-12-28 version=1.0.0 status=active maintained_by=orchestrator -->
