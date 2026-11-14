# EFR32MG24 Rust Development Status Summary

**Date**: January 10, 2026
**Hardware**: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)

---

## 🎉 MAJOR ACHIEVEMENT: First Rust Boot on EFR32MG24!

We successfully achieved the **first confirmed bare-metal Rust boot** on EFR32MG24 with Gecko Bootloader integration!

### What Works ✅

1. **Bootloader Integration**
   - ApplicationProperties_t structure validated by Gecko Bootloader
   - Word 13 dynamic patching working (auto-detects structure location)
   - Silicon Labs Simplicity Commander confirms structure validity
   - Bootloader successfully jumps to Rust application code

2. **Rust Firmware Execution**
   - PC confirmed in application code (NOT stuck in bootloader!)
   - Minimal Rust firmware runs infinite loop successfully
   - Vector table correctly configured (SP, PC, Word 13)
   - Memory layout correct (bootloader + application)

3. **Development Workflow**
   - Automated build→convert→patch→flash pipeline
   - Simplicity Commander integration for validation
   - OpenOCD CMSIS-DAP flashing working
   - No brick risk (recovery procedure documented)

### What Doesn't Work Yet ❌

1. **GPIO Clock Enable**
   - CMU_S.CLKEN0 writes have no effect
   - GPIO clock remains disabled (verified via OpenOCD)
   - LED doesn't blink (clock issue, not GPIO issue)
   - Reason: Unknown (possibly CMU lock, bootloader setup, or security config)

2. **Hardware Control**
   - Cannot control LED due to clock issue
   - GPIO configuration writes work, but have no effect without clock
   - Delay loops execute correctly (code runs)

---

## Key Technical Discoveries

### 1. ApplicationProperties Structure (Complete)

**Size**: 88 bytes total (with Arduino 8-byte header)

```rust
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],                // Arduino-specific header
    magic: [u8; 16],                // Silicon Labs magic signature
    struct_version: u32,             // 0x0201
    signature_type: u32,             // 0 (no signature)
    signature_location: u32,         // 0xFFFFFFFF
    app_info: ApplicationData,       // 28 bytes
    cert: u32,                       // NULL
    long_token_section_address: u32, // NULL
    decrypt_key: [u8; 16],          // All zeros
}
```

**Critical Fields**:
- `app_type`: Must be `0x22` (34 decimal) to match Arduino
- `version`: `0x00000001` for rollback protection
- `magic`: Exact 16-byte Silicon Labs signature

### 2. Word 13 Must Point to Magic

**NOT** to structure start! Arduino has 8-byte header before magic.

```
Structure:  0x080061E0  ┌─────────────────┐
                        │  8-byte header  │
Magic:      0x080061E8  ├─────────────────┤ ← Word 13 points HERE!
                        │  16-byte magic  │
                        ├─────────────────┤
                        │  metadata...    │
                        └─────────────────┘
```

### 3. Auto-Detection is Required

Linker places `.app_properties` at variable addresses:
- Build 1: 0x08006220
- Build 2: 0x080061E0
- Build 3: 0x08006200

**Solution**: Search hex file for magic signature, calculate structure address.

### 4. Simplicity Commander Validation

```bash
$ commander util appinfo firmware.hex
Found application properties in image.
Application properties location : 0x080061e8  # ← Must match Word 13!
App type                        : Unknown application type 00000022
App version                     : 0x00000001
```

If this passes, bootloader will accept the firmware.

---

## Silicon Labs Engineer Guidance

Key insights from official support:

1. **Bootloader Requirement**:
   > "the bootloader requires an application to have a **valid** Applications Properties structure or it will not jump to the application's RESET vector"

2. **Source Code References**:
   - [application_properties.h](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.5/platform/bootloader/api/application_properties.h)
   - [application_properties.c](https://github.com/SiliconLabs/gecko_sdk/blob/gsdk_4.5/platform/bootloader/app_properties/app_properties.c)

3. **Validation Tool**:
   - Use Simplicity Commander `util appinfo` to verify structure
   - This shows exactly what the bootloader sees

---

## Working Example: Minimal Rust Firmware

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as _;
use panic_halt as _;

// ApplicationProperties (88 bytes total)
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],
    magic: [u8; 16],
    struct_version: u32,
    signature_type: u32,
    signature_location: u32,
    app_info: ApplicationData,
    cert: u32,
    long_token_section_address: u32,
    decrypt_key: [u8; 16],
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7,
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    header: [0x02, 0x01, 0x02, 0x02, 0x02, 0x03, 0x02, 0x04],
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0201,
    signature_type: 0,
    signature_location: 0xFFFFFFFF,
    app_info: ApplicationData {
        app_type: 0x22,              // Match Arduino!
        version: 0x00000001,
        capabilities: 0,
        product_id: [0u8; 16],
    },
    cert: 0,
    long_token_section_address: 0,
    decrypt_key: [0u8; 16],
};

#[entry]
fn main() -> ! {
    // Minimal test - just infinite loop
    loop {
        cortex_m::asm::nop();
    }
}
```

**Proof of Execution**:
```bash
$ openocd -c "init" -c "halt" -c "reg pc"
pc (/32): 0x080061bc  # ← In application code! ✅
```

---

## Build and Flash Workflow

### 1. Build

```bash
cargo build --example minimal_test --features rt --release \
            --target thumbv8m.main-none-eabihf
```

### 2. Convert to HEX

```bash
arm-none-eabi-objcopy -O ihex \
    target/thumbv8m.main-none-eabihf/release/examples/minimal_test \
    minimal_test.hex
```

### 3. Auto-Patch Word 13

```python
def find_app_properties(hex_file):
    magic_hex = "13B779FAC925DDB7"
    with open(hex_file, 'r') as f:
        for line in f:
            if magic_hex in line.upper():
                addr_offset = int(line[3:7], 16)
                data_hex = line[9:-3]
                magic_pos = data_hex.upper().find(magic_hex)
                if magic_pos >= 0:
                    byte_offset = magic_pos // 2
                    magic_addr = 0x08000000 + addr_offset + byte_offset
                    struct_addr = magic_addr - 8
                    return struct_addr, magic_addr
    return None, None

struct_addr, magic_addr = find_app_properties('minimal_test.hex')
subprocess.run(['python3', 'patch_word13.py', 'minimal_test.hex',
                'minimal_test_patched.hex', f'0x{struct_addr:08X}'])
```

### 4. Verify

```bash
commander util appinfo minimal_test_patched.hex
# Should show: "Found application properties in image"
```

### 5. Flash

```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
        -c "init" \
        -c "program minimal_test_patched.hex verify" \
        -c "reset run" \
        -c "exit"
```

### 6. Test

Power cycle board and check PC:
```bash
openocd -c "init" -c "halt" -c "reg pc" -c "exit"
# Should show PC in application (0x08006xxx), NOT bootloader (0x08000170)
```

---

## Next Steps

### Immediate (GPIO Clock Issue)

1. **Investigate CMU Lock Mechanism**
   - Check if CMU registers require unlock sequence
   - Look for LOCK register in CMU peripheral
   - Compare with Arduino initialization code

2. **Reverse-Engineer Arduino Clock Setup**
   - Disassemble Arduino setup() function
   - Identify exact clock configuration sequence
   - Replicate in Rust

3. **Try Different Approaches**
   - Use PAC register accessors instead of raw pointers
   - Check bootloader-provided clock state
   - Test different peripheral base addresses (secure vs non-secure)

### Medium-Term (HAL Development)

1. **Clock Module**
   - CMU (Clock Management Unit) driver
   - HFXO/HFRCO/LFXO/LFRCO support
   - Proper enable/disable sequences

2. **GPIO Module**
   - Pin configuration
   - Input/output control
   - Interrupt support

3. **Timer Module**
   - Delay implementation
   - PWM generation
   - Proper microsecond/millisecond timing

### Long-Term (Ecosystem)

1. **Embassy Async Runtime**
   - Async GPIO
   - Async timers
   - DMA support

2. **embedded-hal Traits**
   - Complete trait implementations
   - Hardware abstraction
   - Driver ecosystem compatibility

3. **Community**
   - Publish to crates.io
   - Documentation
   - Examples and tutorials

---

## Files and Documentation

### Created Files

1. **docs/RUST_BLINK_INVESTIGATION.md** - Complete technical investigation
2. **docs/STATUS_SUMMARY.md** - This file (executive summary)
3. **examples/minimal_test.rs** - Minimal working Rust firmware
4. **examples/blink_bare.rs** - Blink attempt (boots but GPIO issue)
5. **patch_word13.py** - Auto-patch Word 13 in hex files

### Key Scripts

1. **flash_blink.sh** - Flash firmware via OpenOCD
2. **Auto-patch script** - Find and patch ApplicationProperties

### External Resources

- [Silicon Labs Gecko SDK](https://github.com/SiliconLabs/gecko_sdk)
- [Simplicity Commander](https://www.silabs.com/developers/simplicity-studio/simplicity-commander)
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)

---

## Conclusion

**We achieved the primary goal**: Rust firmware successfully boots and runs on EFR32MG24!

The Gecko Bootloader integration is **complete and working**. The ApplicationProperties structure is validated, Word 13 pointing works, and the bootloader jumps to Rust code as expected.

The remaining GPIO clock issue is a **separate problem** related to clock initialization, not bootloader integration. This can be solved through further investigation of the CMU peripheral and Arduino initialization sequence.

**This establishes EFR32MG24 as a viable Rust embedded target!** 🎉

---

**Status**: Bootloader integration ✅ COMPLETE | GPIO control ⏳ IN PROGRESS
**Last Updated**: January 10, 2026
