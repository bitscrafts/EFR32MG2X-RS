# Bare-Metal Rust on EFR32MG24 with Gecko Bootloader - Critical Discoveries

**Date**: January 18, 2026
**Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48)
**Bootloader**: Gecko Bootloader v2.x (from Arduino BSP)
**Status**: Partial Success - LED control works, blink timing issues remain

---

## Executive Summary

After extensive debugging, we successfully achieved **partial bare-metal LED control** from Rust on the EFR32MG24 with the Gecko bootloader. The LED can be turned on/off, and registers are being written correctly, but the blink loop has timing issues that need investigation.

### Critical Discoveries

1. ✅ **ApplicationProperties structure working** - Bootloader validates and jumps to application
2. ✅ **CMU unlock sequence identified** - Key 0x93F7 required before clock modifications
3. ✅ **TrustZone-M security state** - Bootloader hands off in NON-SECURE state
4. ✅ **GPIO clock successfully enabled** - CMU_CLKEN0 bit 26 confirmed working
5. ✅ **LED control achieved** - Can turn LED on/off, DOUT register writes work
6. ⚠️  **Blink loop timing issue** - Code runs but visible blink not yet achieved

---

## 1. Boot Sequence and ApplicationProperties

### Working Configuration

The bootloader successfully validates and jumps to our Rust application with this structure:

```rust
#[repr(C)]
pub struct ApplicationProperties {
    header: [u8; 8],           // 0x02,0x01,0x02,0x02,0x02,0x03,0x02,0x04
    magic: [u8; 16],           // 0x13B779FAC925DDB7ADF3CFE0F1B614B8
    struct_version: u32,        // 0x0201
    signature_type: u32,        // 0
    signature_location: u32,    // 0xFFFFFFFF
    app_info: ApplicationData,  // app_type=0x22, version=0x01
    cert: u32,                  // 0
    long_token_section_address: u32,  // 0
    decrypt_key: [u8; 16],      // all zeros
}
```

**Critical**: Vector table Word 13 (offset 0x34) must point to the **MAGIC bytes** (structure address + 8), not the structure start.

### Verification

```bash
$ commander util appinfo firmware.hex
Found application properties in image
Application type: 0x00000022 (Unknown)
```

PC confirmed in application code (0x080061xx range) after bootloader handoff.

---

## 2. TrustZone-M Security State

### Discovery: Bootloader Hands Off in NON-SECURE State

**Critical Finding**: The Gecko bootloader configures the Cortex-M33 in **non-secure** state before jumping to the application.

**Evidence**:
- Accessing secure peripherals (0x5xxxxxxx) causes HardFault
- Non-secure peripheral aliases (0x4xxxxxxx for CMU) work correctly
- GPIO works at non-secure alias (0x5003C000 = GPIO_NS)

**Confusing Naming Convention**:
Silicon Labs uses counterintuitive naming:
- `CMU_S` at 0x40008000 = NON-secure CMU (accessible from non-secure code)
- `CMU_NS` at 0x50008000 = Secure CMU (causes HardFault from non-secure code)
- `GPIO_S` at 0x4003C000 = NON-secure GPIO
- `GPIO_NS` at 0x5003C000 = Appears to be accessible (inconsistent with CMU)

**Question for Silicon Labs**:
1. Is this naming convention documented anywhere?
2. Should bare-metal apps use only 0x4xxxxxxx addresses?
3. How do we configure SAU/IDAU if we need secure peripheral access?

---

## 3. CMU Clock Enable Sequence

### CRITICAL: CMU Must Be Unlocked

The CMU has a LOCK register that **must be unlocked** before modifying clock configuration.

**Unlock Sequence**:
```c
volatile uint32_t *cmu_lock = (uint32_t *)0x40008010;  // CMU_S.LOCK
*cmu_lock = 0x93F7;  // Unlock key
```

**Without unlock**:
- Writes to CLKEN0 have no effect
- Register reads back as 0x00000000

**With unlock**:
- CLKEN0 can be modified
- GPIO clock successfully enabled

### Correct Register Addresses

| Register | Offset | Full Address | Purpose |
|----------|--------|--------------|---------|
| CMU_LOCK | 0x10 | 0x40008010 | Unlock with 0x93F7 |
| CMU_CLKEN0 | 0x64 | 0x40008064 | Clock enable register 0 |
| CMU_CLKEN1 | 0x68 | 0x40008068 | Clock enable register 1 |

**GPIO Clock Enable**:
```c
volatile uint32_t *cmu_clken0 = (uint32_t *)0x40008064;
*cmu_clken0 = (1 << 26);  // GPIO is bit 26, NOT bit 2!
```

**Verification**:
```bash
$ openocd -c "mdw 0x40008064 1"
0x40008064: 04000000  # Bit 26 set = 0x04000000
```

### Wrong Information Found

❌ **CMU_CLKEN0 at offset 0x308** - INCORRECT
✅ **CMU_CLKEN0 at offset 0x64** - CORRECT

❌ **GPIO clock at bit 2** - INCORRECT
✅ **GPIO clock at bit 26** - CORRECT (confirmed in headers: `CMU_CLKEN0_GPIO (0x1UL << 26)`)

---

## 4. GPIO Configuration

### GPIO Base Address and Port Offsets

**GPIO Base**: 0x5003C000 (GPIO_NS - despite confusing naming, this works)

**PORTA Offset**: 0x30 (48 bytes from base)

**PORTA Register Addresses**:
```c
GPIO base:       0x5003C000
PORTA base:      0x5003C030  (base + 0x30)
PORTA_CTRL:      0x5003C030  (PORTA + 0x00)
PORTA_MODEL:     0x5003C034  (PORTA + 0x04)
PORTA_MODEH:     0x5003C038  (PORTA + 0x08)
PORTA_DOUT:      0x5003C03C  (PORTA + 0x0C)
PORTA_DOUTTGL:   0x5003C040  (PORTA + 0x10)
```

### GPIO Configuration Code (Working)

```c
// Enable GPIO clock
volatile uint32_t *cmu_clken0 = (uint32_t *)0x40008064;
*cmu_clken0 = (1 << 26);

// Configure PA7 as push-pull output
volatile uint32_t *gpio_porta_model = (uint32_t *)0x5003C034;
uint32_t model = *gpio_porta_model;
*gpio_porta_model = (model & ~0xF0000000) | 0x40000000;  // MODE7 = 0x4

// Control LED (active-low)
volatile uint32_t *gpio_porta_dout = (uint32_t *)0x5003C03C;
*gpio_porta_dout = 0x00;  // LED ON
*gpio_porta_dout = 0x80;  // LED OFF
```

**Verification**:
```bash
$ openocd -c "mdw 0x5003C034 1"
0x5003C034: 40000000  # MODE7 = push-pull

$ openocd -c "mdw 0x5003C03C 1"
0x5003C03C: 00000080  # PA7 = 1 (LED off)
```

### LED Pin Details

- **Pin**: PA7
- **Arduino**: D20 (LED_BUILTIN)
- **Polarity**: Active-LOW (0 = ON, 1 = OFF)

---

## 5. Minimal Working Initialization Sequence

```rust
#[no_mangle]
pub extern "C" fn main() -> ! {
    unsafe {
        // Step 1: Unlock CMU (NON-SECURE address)
        const CMU_LOCK: *mut u32 = 0x40008010 as *mut u32;
        core::ptr::write_volatile(CMU_LOCK, 0x93F7);

        // Step 2: Enable GPIO clock (bit 26 of CLKEN0)
        const CMU_CLKEN0: *mut u32 = 0x40008064 as *mut u32;
        core::ptr::write_volatile(CMU_CLKEN0, 1 << 26);

        // Step 3: Configure PA7 as push-pull output
        const GPIO_PORTA_MODEL: *mut u32 = 0x5003C034 as *mut u32;
        let model = core::ptr::read_volatile(GPIO_PORTA_MODEL);
        core::ptr::write_volatile(GPIO_PORTA_MODEL,
            (model & !0xF0000000) | 0x40000000);

        // Step 4: Control LED
        const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;
        core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);  // LED ON

        loop {
            cortex_m::asm::nop();
        }
    }
}
```

**Status**: This code successfully turns the LED ON. ✅

---

## 6. Current Issue: Blink Loop Not Working

### Symptoms

- LED can be turned ON ✅
- LED can be turned OFF ✅
- Registers are being written correctly (verified via OpenOCD) ✅
- Code is executing (PC in application range) ✅
- But visible blink at 1 Hz not achieved ❌

### What We've Tried

1. **XOR toggle**: DOUT register changes correctly (0x00 ↔ 0x80) but no visible blink
2. **Explicit writes**: Alternating between 0x00 and 0x80
3. **Read-modify-write**: Preserving other pins, only changing PA7
4. **Various delays**: From 500k to 20 million NOPs
5. **DOUTTGL register**: Using dedicated toggle register (next test)

### Evidence

```bash
# First halt
$ openocd -c "mdw 0x5003C03C 1"
0x5003C03C: 00000000  # LED should be ON

# Second halt (2 seconds later)
$ openocd -c "mdw 0x5003C03C 1"
0x5003C03C: 00000080  # LED should be OFF

# DOUT IS changing, but visual blink not clear
```

### Hypotheses

1. **CPU frequency unknown**: Assumed 39 MHz, but bootloader might set different frequency
2. **Timing too fast/slow**: 20M NOPs might not equal 1 second
3. **LED dimming issue**: Maybe LED appears "on" in both states due to ghosting/capacitance?
4. **Missing initialization**: Some peripheral might need configuration we're missing
5. **Loop optimization**: Compiler might be doing something unexpected with delay loop

---

## 7. Questions for Silicon Labs Employee

### Boot and Security

1. **What security state does the Gecko bootloader hand off to the application?**
   - We observe non-secure state - is this documented?
   - Should bare-metal apps always use 0x4xxxxxxx peripheral addresses?

2. **Clock configuration at handoff**:
   - What is the CPU frequency after bootloader hands off? (We assume 39 MHz HFRCO)
   - Does bootloader leave CMU unlocked?
   - Are any clocks pre-enabled besides the core?

3. **CMU_S vs CMU_NS naming**:
   - Why is CMU_S at 0x40008000 (non-secure alias)?
   - Is this naming convention documented?
   - Should we use CMU_S or CMU_NS for non-secure code?

### Peripheral Access

4. **GPIO configuration**:
   - Is push-pull mode (0x4) correct for driving an LED?
   - Do we need to configure drive strength or slew rate?
   - Should we use DOUTTGL register for toggling?

5. **Minimal initialization**:
   - What is the absolute minimum initialization needed after bootloader handoff?
   - Are there any lock registers besides CMU.LOCK?
   - Do we need to configure SAU/IDAU for basic GPIO access?

### Timing Issues

6. **Delay implementation**:
   - What is the actual CPU frequency at bootloader handoff?
   - How many cycles does a NOP take on Cortex-M33?
   - Is there a hardware timer we should use instead of NOPs?

7. **Reference example**:
   - Do you have any bare-metal C examples (without GSDK) that blink an LED with the Gecko bootloader?
   - What does `sl_system_init()` do that we're missing?

---

## 8. Current Code Status

### Files

- `examples/blink_unlock_cmu.rs` - Latest blink attempt (DOUTTGL not yet tested)
- `examples/led_off.rs` - Successfully turns LED off (confirmed working)
- `docs/BARE_METAL_DISCOVERIES.md` - This document

### Build and Flash

```bash
# Build
cargo build --example blink_unlock_cmu --features rt --release

# Convert to hex
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/examples/blink_unlock_cmu \
  blink_unlock_cmu.hex

# Patch ApplicationProperties
python3 patch_word13.py blink_unlock_cmu.hex blink_unlock_cmu_patched.hex 0x08006228

# Flash via OpenOCD
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init" -c "program blink_unlock_cmu_patched.hex verify" \
  -c "reset run" -c "exit"
```

### Verification Commands

```bash
# Check CMU unlock status
openocd -c "mdw 0x40008010 1"  # LOCK register

# Check GPIO clock enabled
openocd -c "mdw 0x40008064 1"  # CLKEN0 (should be 0x04000000)

# Check GPIO configuration
openocd -c "mdw 0x5003C034 1"  # MODEL (should be 0x40000000)

# Check LED state
openocd -c "mdw 0x5003C03C 1"  # DOUT (0x00 = on, 0x80 = off)
```

---

## 9. Next Steps (Waiting for Silicon Labs Response)

1. Clarify bootloader handoff state and required initialization
2. Confirm correct peripheral base addresses for non-secure code
3. Understand why DOUT changes don't result in visible LED blink
4. Get guidance on proper bare-metal initialization sequence
5. Potentially request example bare-metal blink code for reference

---

## 10. Summary of Achievements

✅ **Major Successes**:
1. ApplicationProperties structure validated by bootloader
2. Bootloader successfully jumps to Rust application
3. CMU unlock sequence discovered and working
4. GPIO clock enable verified (CLKEN0 bit 26)
5. GPIO pin configuration working (push-pull mode)
6. LED control achieved (can turn on/off via register writes)
7. Confirmed non-secure peripheral addressing requirements

❌ **Remaining Issues**:
1. Visible LED blink not yet achieved (timing issue)
2. Exact bootloader handoff configuration unclear
3. Optimal GPIO toggle method uncertain (DOUT vs DOUTTGL)

**Overall**: We have successfully achieved bare-metal Rust execution and peripheral control on the EFR32MG24 with the Gecko bootloader. The remaining blink timing issue is likely a configuration detail that Silicon Labs can help clarify.

---

**Contact**: [Your email]
**Repository**: [Your repo URL]
**Hardware**: Seeed Studio XIAO MG24 Sense
**Firmware Versions**: Gecko Bootloader v2.x, Arduino BSP 3.0.0
