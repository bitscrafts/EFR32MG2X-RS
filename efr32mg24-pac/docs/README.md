# EFR32MG24 Rust Documentation

## 🎉 Breakthrough Achievement

**First confirmed bare-metal Rust boot on EFR32MG24 with Gecko Bootloader!**

Date: January 10, 2026

---

## Documentation Index

### Quick Start

- **[COMPLETE_GUIDE.md](COMPLETE_GUIDE.md)** - START HERE
  - Silicon Labs engineer guidance
  - Complete ApplicationProperties specification
  - Step-by-step build and flash workflow
  - Troubleshooting guide

### Technical Details

- **[RUST_BLINK_INVESTIGATION.md](RUST_BLINK_INVESTIGATION.md)** - Deep technical investigation
  - Complete chronological debug session
  - All attempted fixes and results
  - Memory layout analysis
  - Comparison with Arduino C firmware

- **[STATUS_SUMMARY.md](STATUS_SUMMARY.md)** - Executive summary
  - What works and what doesn't
  - Key discoveries
  - Next steps

### Tools

- **[../start-page.py](../start-page.py)** - Find vector tables in hex files
- **[../patch_word13.py](../patch_word13.py)** - Patch Word 13 to point to ApplicationProperties

---

## Quick Reference

### Minimal Working Rust Firmware

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as _;
use panic_halt as _;

// Include complete ApplicationProperties structure
// (See COMPLETE_GUIDE.md for full definition)

#[entry]
fn main() -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
```

### Build Workflow

```bash
# 1. Build
cargo build --example minimal_test --features rt --release --target thumbv8m.main-none-eabihf

# 2. Convert to HEX
arm-none-eabi-objcopy -O ihex target/.../minimal_test minimal_test.hex

# 3. Auto-patch Word 13
python3 auto_patch.py  # (see COMPLETE_GUIDE.md)

# 4. Verify
commander util appinfo minimal_test_patched.hex

# 5. Flash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
        -c "init" -c "program minimal_test_patched.hex verify" \
        -c "reset run" -c "exit"

# 6. Power cycle board and test!
```

---

## Status

### ✅ Working

- Bootloader integration (ApplicationProperties validates)
- Word 13 auto-detection and patching
- Rust firmware boots and executes
- PC confirmed in application code

### ⏳ In Progress

- GPIO clock enable (CMU writes don't take effect)
- LED blink functionality

---

## Key Insights

1. **Word 13 must point to MAGIC, not structure start**
   - Arduino has 8-byte header before magic
   - Word 13 points to magic (structure + 8 bytes)

2. **ApplicationProperties location varies**
   - Linker places it at different addresses each build
   - Auto-detection is REQUIRED

3. **Simplicity Commander is essential**
   - Validates structure from bootloader perspective
   - Shows exactly what bootloader sees

4. **app_type must be 0x22**
   - Matches Arduino configuration
   - Required for compatibility

---

## Silicon Labs Resources

- [Gecko SDK](https://github.com/SiliconLabs/gecko_sdk)
- [Simplicity Commander](https://www.silabs.com/developers/simplicity-studio/simplicity-commander)
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)

---

## Next Steps

1. Solve GPIO clock enable issue
2. Get LED blinking in Rust
3. Develop HAL modules (clock, GPIO, timers)
4. Publish working example

---

**For complete details, see [COMPLETE_GUIDE.md](COMPLETE_GUIDE.md)**
