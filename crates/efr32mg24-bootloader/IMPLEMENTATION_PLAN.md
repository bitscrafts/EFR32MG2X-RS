# Custom Bootloader Implementation Plan

**Date**: 2026-01-25
**Goal**: Fix test application build and achieve 1 Hz LED blink with custom Rust bootloader

---

## Research Summary

### Key Findings from Silicon Labs Gecko Bootloader

Based on analysis of the [Gecko SDK](https://github.com/SiliconLabs/gecko_sdk) and [bootloader documentation](https://docs.silabs.com/mcu-bootloader/latest/):

1. **VTOR Configuration** (Cortex-M33 requirement):
   - Vector Table Offset Register at 0xE000ED08
   - Must point to application vector table before jump
   - Requires DSB/ISB memory barriers

2. **Application Validation** ([btl_bootload.c](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.1/platform/bootloader/core/btl_bootload.c)):
   - Stack pointer must point to valid RAM (0x20000000-0x20040000 for EFR32MG24)
   - Reset vector must be in application flash with Thumb bit set
   - Optional: CRC32 or ECDSA signature verification

3. **ApplicationProperties Structure**:
   - Required by Gecko bootloader for Series 2 devices
   - Contains magic bytes, version, capabilities
   - Our minimal bootloader may not need this

4. **Jump Sequence** (standard Cortex-M pattern):
   ```c
   // Disable interrupts
   __disable_irq();

   // Set VTOR to application
   SCB->VTOR = app_address;
   __DSB();
   __ISB();

   // Load stack pointer and reset vector
   uint32_t sp = *(uint32_t*)app_address;
   uint32_t pc = *(uint32_t*)(app_address + 4);

   // Set stack pointer
   __set_MSP(sp);

   // Jump to application
   ((void (*)(void))pc)();
   ```

---

## Current Status

### What Works ✅

1. **Bootloader Binary**:
   - Builds successfully (792 bytes)
   - Fixed Thumb bit issue (`|1` instead of `+1`)
   - Flashes without errors
   - Validation logic is correct
   - VTOR setup and jump code is correct

2. **Hardware**:
   - XIAO MG24 Sense verified functional
   - Arduino examples work (volume, blink)
   - OpenOCD flash procedure reliable
   - CMSIS-DAP debugger working

3. **Infrastructure**:
   - Local OpenOCD setup complete
   - Recovery scripts for Arduino examples
   - Documentation organized with SSOT methodology

### What's Broken ❌

**CRITICAL ISSUE**: Test Application Build

```bash
$ arm-none-eabi-size test-blink-app
   text    data     bss     dec     hex filename
      0       0       0       0       0 test-blink-app
```

**Symptoms**:
- Binary file exists (7.8 KB)
- ELF sections are empty
- `objcopy` produces 13-byte hex (just header)
- LED turns ON when flashed but doesn't blink (stuck)

**Root Cause**: Build system confusion

The build.rs script cannot reliably distinguish between:
- Building bootloader (needs `bootloader.x` linker script)
- Building examples (need cortex-m-rt's `link.x` + `memory.x`)

**Why It Fails**:
```toml
# .cargo/config.toml applies to ALL builds in workspace
[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=-Tbootloader.x"]  # Wrong for examples!
```

When building examples:
1. Cargo sees `-Tbootloader.x` in rustflags
2. cortex-m-rt tries to use its own `link.x`
3. Linker gets conflicting scripts
4. Result: empty sections

---

## Solution Strategies

### Option 1: Separate Workspace Crates (RECOMMENDED)

**Approach**: Create independent crates for bootloader and test app

**Structure**:
```
EFR32MG24/
├── efr32mg24-bootloader/           # Bootloader crate
│   ├── .cargo/config.toml          # Uses bootloader.x
│   ├── Cargo.toml
│   ├── build.rs                    # Copies bootloader.x
│   ├── bootloader.x
│   └── src/main.rs
│
└── bootloader-test-app/            # Test application crate
    ├── .cargo/config.toml          # Standard cortex-m-rt
    ├── Cargo.toml
    ├── build.rs                    # Standard cortex-m-rt build
    ├── memory.x                    # App memory: 0x08004000
    └── src/main.rs
```

**Benefits**:
- Clean separation of concerns
- No build system conflicts
- Standard cortex-m-rt for test app
- Easy to maintain and extend

**Drawbacks**:
- Requires restructuring project
- Two separate build commands

**Implementation Steps**:
1. Create `../bootloader-test-app/` directory
2. Copy test-blink-app.rs to new crate
3. Create standard cortex-m-rt Cargo.toml
4. Create memory.x for application space (0x08004000)
5. Update build-and-flash.sh to build from both directories

---

### Option 2: Manual Build Commands

**Approach**: Use explicit RUSTFLAGS for each build

**Build Commands**:
```bash
# Build bootloader
cd efr32mg24-bootloader
RUSTFLAGS="-C link-arg=-Tbootloader.x" \
  cargo build --release --target thumbv8m.main-none-eabihf --bin efr32mg24-bootloader

# Build test app (in same or different directory)
cd ../bootloader-test-app  # or stay in same dir
RUSTFLAGS="-C link-arg=-Tmemory.x" \
  cargo build --release --target thumbv8m.main-none-eabihf --bin test-blink-app
```

**Benefits**:
- No project restructuring needed
- Full control over linker flags
- Can keep everything in one crate

**Drawbacks**:
- Manual commands required
- Can't use `cargo build --examples`
- Error-prone (easy to forget flags)

**Implementation Steps**:
1. Remove rustflags from .cargo/config.toml
2. Update build-and-flash.sh with explicit RUSTFLAGS
3. Ensure memory.x exists for test app
4. Build and test

---

### Option 3: Custom Startup Without cortex-m-rt

**Approach**: Write manual startup code like bootloader does

**Application Structure**:
```rust
#![no_std]
#![no_main]

use core::arch::asm;

// Custom vector table in test-app-linker.x
#[link_section = ".vector_table"]
#[no_mangle]
pub static VECTOR_TABLE: [u32; 16] = [
    0x20040000,              // Initial stack pointer
    Reset as u32 | 1,        // Reset handler with Thumb bit
    // ... other handlers
];

#[no_mangle]
pub unsafe extern "C" fn Reset() -> ! {
    // Zero .bss
    extern "C" {
        static mut _sbss: u32;
        static mut _ebss: u32;
    }
    let mut bss = &mut _sbss as *mut u32;
    let bss_end = &mut _ebss as *mut u32;
    while bss < bss_end {
        core::ptr::write_volatile(bss, 0);
        bss = bss.offset(1);
    }

    main()
}

fn main() -> ! {
    // LED blink code
    loop { /* ... */ }
}
```

**Benefits**:
- No cortex-m-rt dependency
- Full control over startup
- Matches bootloader implementation style

**Drawbacks**:
- More code to maintain
- Reinventing cortex-m-rt functionality
- More error-prone

**Implementation Steps**:
1. Remove cortex-m-rt dependency from test app
2. Write custom Reset handler
3. Create test-app-linker.x with proper vector table
4. Implement .bss initialization
5. Test and verify

---

### Option 4: Environment Variable Detection in build.rs

**Approach**: Use environment variables to detect build target

**build.rs**:
```rust
fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Check if building an example
    if let Ok(cargo_crate_name) = env::var("CARGO_CRATE_NAME") {
        if cargo_crate_name.starts_with("test_") {
            // Building test app - use cortex-m-rt
            fs::copy("test-app-memory.x", out_dir.join("memory.x")).unwrap();
            println!("cargo:rustc-link-arg=-Tmemory.x");
        } else {
            // Building bootloader
            fs::copy("bootloader.x", out_dir.join("bootloader.x")).unwrap();
            println!("cargo:rustc-link-arg=-Tbootloader.x");
        }
    }
}
```

**Benefits**:
- Single crate
- Automatic detection
- Can still use `cargo build --examples`

**Drawbacks**:
- Fragile (relies on naming conventions)
- CARGO_CRATE_NAME may not work as expected
- Hard to debug when it fails

**Status**: Already tried this approach - didn't work reliably

---

## Recommended Implementation: Option 1 (Separate Crates)

### Detailed Steps

#### Step 1: Create Test Application Crate

```bash
cd /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/
mkdir bootloader-test-app
cd bootloader-test-app
```

**Cargo.toml**:
```toml
[package]
name = "bootloader-test-app"
version = "0.1.0"
edition = "2021"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"

[profile.release]
debug = true
lto = true
opt-level = "z"  # Optimize for size
codegen-units = 1
```

**memory.x** (application memory layout):
```ld
MEMORY
{
  /* Application starts after 16 KB bootloader */
  FLASH : ORIGIN = 0x08004000, LENGTH = 1520K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}
```

**.cargo/config.toml**:
```toml
[build]
target = "thumbv8m.main-none-eabihf"

[target.thumbv8m.main-none-eabihf]
runner = "probe-rs run --chip EFR32MG24B220F1536IM48"
rustflags = [
  "-C", "link-arg=-Tlink.x",  # cortex-m-rt linker script
]
```

**src/main.rs**:
```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

fn delay_ms(ms: u32) {
    let cycles = ms * 10000;
    for _ in 0..cycles {
        cortex_m::asm::nop();
    }
}

#[entry]
fn main() -> ! {
    unsafe {
        // Unlock CMU
        const CMU_LOCK: *mut u32 = 0x40008010 as *mut u32;
        core::ptr::write_volatile(CMU_LOCK, 0x93F7);

        // Enable GPIO clock (bit 26)
        const CMU_CLKEN0: *mut u32 = 0x40008064 as *mut u32;
        core::ptr::write_volatile(CMU_CLKEN0, 1 << 26);

        // Configure PA7 as push-pull output (MODE = 0x4)
        const GPIO_PORTA_MODEL: *mut u32 = 0x5003C034 as *mut u32;
        let model = core::ptr::read_volatile(GPIO_PORTA_MODEL);
        core::ptr::write_volatile(GPIO_PORTA_MODEL, (model & !0xF0000000) | 0x40000000);

        const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;

        loop {
            // LED ON (active-low: write 0)
            core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);
            delay_ms(500);

            // LED OFF (active-low: write 1 to bit 7)
            core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);
            delay_ms(500);
        }
    }
}
```

#### Step 2: Update Bootloader Crate

**efr32mg24-bootloader/.cargo/config.toml**:
```toml
[build]
target = "thumbv8m.main-none-eabihf"

[target.thumbv8m.main-none-eabihf]
# Bootloader-specific linker flags
rustflags = [
  "-C", "link-arg=-Tbootloader.x",
]
```

**efr32mg24-bootloader/build.rs**:
```rust
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Copy bootloader linker script
    fs::copy("bootloader.x", out_dir.join("bootloader.x"))
        .expect("Failed to copy bootloader.x");

    println!("cargo:rerun-if-changed=bootloader.x");
    println!("cargo:rustc-link-search={}", out_dir.display());
}
```

#### Step 3: Update Build Script

**build-and-flash.sh**:
```bash
#!/bin/bash
set -e

echo "=== Building Custom Bootloader + Test Application ==="
echo ""

# Build bootloader
echo "Building bootloader..."
cd efr32mg24-bootloader
cargo build --release --target thumbv8m.main-none-eabihf
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader \
  bootloader.hex
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader
echo "✓ Bootloader: $(ls -lh bootloader.hex | awk '{print $5}')"

# Build test app
echo ""
echo "Building test app..."
cd ../bootloader-test-app
cargo build --release --target thumbv8m.main-none-eabihf
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/bootloader-test-app \
  test-app.hex
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/bootloader-test-app
echo "✓ Test app: $(ls -lh test-app.hex | awk '{print $5}')"

echo ""
echo "=== Flashing to Hardware ==="
echo ""

# Flash bootloader
echo "Flashing bootloader..."
cd ../efr32mg24-bootloader
./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program bootloader.hex verify; exit" \
  2>&1 | grep -E "(Programming|Verified|Error)" || true

# Flash test app
echo ""
echo "Flashing test app..."
cd ../bootloader-test-app
../efr32mg24-bootloader/openocd/bin/openocd \
  -s ../efr32mg24-bootloader/openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program test-app.hex verify; reset run; exit" \
  2>&1 | grep -E "(Programming|Verified|Error)" || true

echo ""
echo "=== Flash Complete! ==="
echo ""
echo "OBSERVE THE LED:"
echo "  - Blinking at 1 Hz (500ms ON/OFF) = SUCCESS!"
echo "  - Blinking FAST (~100ms) = HardFault"
echo "  - Blinking SLOW (~1 second) = Bootloader mode"
echo "  - Solid ON = Check with debugger"
echo "  - Not blinking = Check connection"
echo ""
```

#### Step 4: Verification Steps

1. **Build bootloader**:
   ```bash
   cd efr32mg24-bootloader
   cargo build --release
   arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader
   ```
   Expected: text ≈ 792, data ≈ 0, bss ≈ 0

2. **Build test app**:
   ```bash
   cd ../bootloader-test-app
   cargo build --release
   arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/bootloader-test-app
   ```
   Expected: text > 0, data ≥ 0, bss ≥ 0 (NOT all zeros!)

3. **Generate hex files**:
   ```bash
   arm-none-eabi-objcopy -O ihex \
     target/thumbv8m.main-none-eabihf/release/bootloader-test-app \
     test-app.hex
   wc -l test-app.hex
   ```
   Expected: More than 1 line (not just header)

4. **Inspect vector table**:
   ```bash
   arm-none-eabi-objdump -d \
     target/thumbv8m.main-none-eabihf/release/bootloader-test-app \
     | head -50
   ```
   Expected: Vector table at 0x08004000 with valid entries

5. **Flash and test**:
   ```bash
   cd ../efr32mg24-bootloader
   ./build-and-flash.sh
   ```
   Expected: LED blinking at 1 Hz

---

## Debugging Plan

### If LED Still Doesn't Blink

1. **Use OpenOCD to read memory**:
   ```bash
   # Read bootloader vector table
   openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
     -c "init; halt; mdw 0x08000000 8; exit"

   # Read application vector table
   openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
     -c "init; halt; mdw 0x08004000 8; exit"

   # Read VTOR register
   openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
     -c "init; halt; mdw 0xE000ED08 1; exit"
   ```

2. **Check program counter**:
   ```bash
   openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
     -c "init; halt; reg pc; exit"
   ```
   Expected: PC should be in application range (0x08004000-0x0817FFFF)

3. **Single-step through bootloader**:
   ```bash
   openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
     -c "init; reset halt; bp 0x080002C0 2 hw; resume; halt; reg; exit"
   ```

4. **Add debug output to bootloader**:
   - Toggle LED at different points in boot sequence
   - 1 blink = entering bootloader
   - 2 blinks = app validated
   - 3 blinks = about to jump
   - Continuous slow blink = bootloader mode (app invalid)

---

## Success Criteria

1. **Build Success**:
   - Bootloader builds to ~792 bytes
   - Test app builds with non-zero sections
   - Hex files contain actual code

2. **Flash Success**:
   - Both binaries flash without errors
   - OpenOCD verify passes

3. **Runtime Success**:
   - LED blinks at 1 Hz (500ms ON, 500ms OFF)
   - Consistent timing (not erratic)
   - Survives power cycle

4. **Validation Success**:
   - VTOR set to 0x08004000
   - PC executing in application range
   - Stack pointer in RAM range

---

## Timeline Estimate

**Implementation**: 2-4 hours
- Create new crate: 30 minutes
- Update build scripts: 30 minutes
- Testing and debugging: 1-3 hours

**Total**: Should achieve success within one work session

---

## Rollback Plan

If Option 1 fails, fallback sequence:

1. Try Option 2 (manual build commands)
2. Try Option 3 (custom startup without cortex-m-rt)
3. Abandon custom bootloader, use Gecko bootloader

Recovery command:
```bash
cd efr32mg24-bootloader
./flash-arduino-volume.sh
```

---

## References

- [Gecko SDK bootloader source](https://github.com/SiliconLabs/gecko_sdk/tree/gsdk_4.1/platform/bootloader)
- [Cortex-M33 Technical Reference Manual](https://developer.arm.com/documentation/100230/latest/)
- [rustBoot bootloader](https://github.com/nihalpasham/rustBoot)
- [STM32 bootloader example](https://github.com/karthickai/bootloader)
- [Silicon Labs Gecko Bootloader Documentation](https://docs.silabs.com/mcu-bootloader/latest/)
- [Bootloaders on Cortex-M CPUs](https://pyjamabrah.com/posts/bootloader/)

---

**Next Action**: Implement Option 1 (Separate Workspace Crates)
