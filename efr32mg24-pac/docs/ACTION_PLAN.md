# Action Plan: Solving the EFR32MG24 Rust Blink Issue

**Date**: January 18, 2026
**Status**: LED control works, blink timing issue remains
**Goal**: Achieve visible 1 Hz LED blink from bare-metal Rust

---

## Current State Summary

### ✅ What's Working

1. **Bootloader Integration**
   - ApplicationProperties structure validated
   - Bootloader successfully jumps to Rust application
   - PC confirmed in application code (0x080061xx)

2. **Clock Management**
   - CMU unlock with key 0x93F7 working
   - GPIO clock enabled (CMU_CLKEN0 bit 26 = 0x04000000)
   - Verified via OpenOCD: `mdw 0x40008064` shows `0x04000000`

3. **GPIO Control**
   - PA7 configured as push-pull output
   - PORTA_MODEL = 0x40000000 (MODE7 = push-pull)
   - DOUT register writes work
   - LED can be turned ON (DOUT = 0x00)
   - LED can be turned OFF (DOUT = 0x80)

4. **Register Access**
   - All register addresses confirmed correct:
     - CMU_LOCK: 0x40008010
     - CMU_CLKEN0: 0x40008064
     - PORTA_MODEL: 0x5003C034
     - PORTA_DOUT: 0x5003C03C
     - PORTA_DOUTTGL: 0x5003C040

### ❌ What's Not Working

**LED doesn't blink visibly at 1 Hz**
- DOUT register toggles correctly (0x00 ↔ 0x80 verified via OpenOCD)
- Code is running (PC advances through delay loop)
- But NO visible blink observed

---

## Root Cause Hypotheses

### Hypothesis 1: CPU Frequency Mismatch (MOST LIKELY)

**Evidence:**
- We assume 39 MHz HFRCO after bootloader handoff
- Delay calculation: 20M NOPs ÷ 39 MHz = ~0.5 seconds
- But if actual frequency is different, timing is wrong

**Possible scenarios:**
1. Bootloader sets 19 MHz (default HFRCO) → our delays are 2x too long
2. Bootloader sets 38 MHz → our delays are close but slightly off
3. Bootloader sets 78 MHz → our delays are 2x too short
4. NOPs are optimized out or take multiple cycles

**Test:**
```rust
// Try MUCH longer delay (100M NOPs = ~2.5s at 39MHz)
for _ in 0..100_000_000 {
    cortex_m::asm::nop();
}
```

**Verification:**
- Use SysTick to measure actual CPU frequency
- Read CMU_STATUS register for current clock source
- Compare with Arduino's clock configuration

### Hypothesis 2: Compiler Optimization Issues

**Evidence:**
- Release builds optimize aggressively
- NOP loops might be optimized out or shortened

**Test:**
```rust
// Use volatile reads to prevent optimization
loop {
    core::ptr::write_volatile(GPIO_PORTA_DOUTTGL, 1 << 7);

    // Volatile delay
    let mut count = 0;
    while count < 20_000_000 {
        unsafe { core::ptr::read_volatile(&count); }
        count += 1;
    }
}
```

**Verification:**
- Check assembly output for actual NOP count
- Use debug build instead of release
- Add `#[inline(never)]` to delay function

### Hypothesis 3: GPIO Configuration Missing

**Evidence:**
- Arduino works perfectly
- Our code turns LED on/off but maybe missing something

**Possible issues:**
1. Drive strength not configured
2. Slew rate not set
3. Pin muxing not correct
4. Port CTRL register needs configuration

**Test:**
```rust
// Configure PORTA CTRL register
const GPIO_PORTA_CTRL: *mut u32 = 0x5003C030 as *mut u32;
core::ptr::write_volatile(GPIO_PORTA_CTRL, 0x00400040);  // Match Arduino

// Or try different drive modes
core::ptr::write_volatile(GPIO_PORTA_MODEL,
    (model & !0xF0000000) | 0x60000000);  // Strong drive
```

**Verification:**
- Compare all GPIO registers between Arduino and Rust
- Check CTRL, MODEL, MODEH registers
- Verify no other configuration needed

### Hypothesis 4: Peripheral Locking or Security

**Evidence:**
- Some writes work, others might be ignored
- TrustZone-M security could block certain operations

**Possible issues:**
1. GPIO has its own LOCK register
2. PPUSATD (peripheral security) blocking writes
3. Some registers require unlock sequence

**Test:**
```rust
// Check and unlock GPIO if needed
const GPIO_LOCK: *mut u32 = 0x5003C230 as *mut u32;  // Offset 0x230
const GPIO_UNLOCK_KEY: u32 = 0xA534;
core::ptr::write_volatile(GPIO_LOCK, GPIO_UNLOCK_KEY);
```

**Verification:**
- Read GPIO_LOCK register
- Check GPIO_GPIOLOCKSTATUS
- Review peripheral security attributes

### Hypothesis 5: LED Hardware Issue

**Evidence:**
- Arduino blinks perfectly
- Our code turns LED on solidly
- But blink not visible

**Possible issues:**
1. LED has external circuitry (RC filter, capacitor)
2. Multiple pins controlling LED
3. Active-low/active-high confusion
4. LED requires specific drive current

**Test:**
```rust
// Try very slow blink (visible to human eye even if glitching)
for _ in 0..200_000_000 {  // ~5 seconds
    cortex_m::asm::nop();
}

// Or use DOUTTGL exclusively
core::ptr::write_volatile(GPIO_PORTA_DOUTTGL, 1 << 7);
```

**Verification:**
- Measure PA7 with oscilloscope
- Check if voltage actually changes
- Verify LED circuit on schematic

---

## Systematic Debug Plan

### Phase 1: Verify Register States (IMMEDIATE)

**Goal**: Confirm what Arduino actually sets

**Steps:**
1. Flash Arduino blink
2. Halt after setup() completes
3. Dump ALL relevant registers:
   ```bash
   # CMU registers
   openocd -c "mdw 0x40008000 20"  # CMU base

   # GPIO PORTA registers
   openocd -c "mdw 0x5003C030 10"  # PORTA base

   # Clock status
   openocd -c "mdw 0x50008008 1"   # CMU_STATUS
   ```
4. Flash Rust blink
5. Halt after initialization
6. Dump SAME registers
7. **Compare byte-by-byte**

**Expected outcome**: Find missing register writes

### Phase 2: Measure Actual CPU Frequency (HIGH PRIORITY)

**Goal**: Determine actual clock frequency

**Method 1: SysTick Timer**
```rust
// Configure SysTick for 1 second
const SYSTICK_LOAD: *mut u32 = 0xE000E014 as *mut u32;
const SYSTICK_CTRL: *mut u32 = 0xE000E010 as *mut u32;

// Assume 39 MHz, request 1 second tick
core::ptr::write_volatile(SYSTICK_LOAD, 39_000_000 - 1);
core::ptr::write_volatile(SYSTICK_CTRL, 0x7);  // Enable, interrupt, CPU clock

// Blink on SysTick interrupt
```

**Method 2: Read CMU_STATUS**
```rust
const CMU_STATUS: *mut u32 = 0x40008008 as *mut u32;
let status = core::ptr::read_volatile(CMU_STATUS);
// Check which oscillator is selected
```

**Method 3: Compare with Arduino**
```c
// In Arduino setup():
Serial.begin(115200);
Serial.print("CPU Freq: ");
Serial.println(SystemCoreClock);
```

**Expected outcome**: Know exact CPU frequency

### Phase 3: Test DOUTTGL Register (NEXT TEST)

**Goal**: Use hardware toggle instead of manual writes

**Code:**
```rust
const GPIO_PORTA_DOUTTGL: *mut u32 = 0x5003C040 as *mut u32;

loop {
    // Hardware atomic toggle
    core::ptr::write_volatile(GPIO_PORTA_DOUTTGL, 1 << 7);

    // Known delay
    for _ in 0..20_000_000 {
        cortex_m::asm::nop();
    }
}
```

**Why this might work:**
- DOUTTGL is atomic (no read-modify-write race)
- Might avoid any timing issues with manual toggle
- Arduino might use this internally

**Expected outcome**: Visible blink if toggle method was the issue

### Phase 4: Extreme Delay Test (DIAGNOSTIC)

**Goal**: Rule out timing issues completely

**Code:**
```rust
loop {
    // LED ON
    core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);

    // VERY long delay (10 seconds if 39 MHz)
    for _ in 0..400_000_000 {
        cortex_m::asm::nop();
    }

    // LED OFF
    core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);

    // VERY long delay
    for _ in 0..400_000_000 {
        cortex_m::asm::nop();
    }
}
```

**Expected outcome:**
- If this blinks → timing issue confirmed
- If this doesn't blink → hardware/config issue

### Phase 5: Oscilloscope Measurement (HARDWARE VERIFICATION)

**Goal**: Verify PA7 electrical signal

**Measurements:**
1. Probe PA7 directly
2. Measure voltage swing (should be 0V to 3.3V)
3. Measure frequency (should match delay calculation)
4. Check for glitches or ringing

**Expected outcome**: Confirm electrical signal is correct

### Phase 6: Arduino Source Analysis (REFERENCE)

**Goal**: Find exact Arduino initialization sequence

**Steps:**
1. Locate Arduino core source
2. Find `pinMode()` implementation
3. Find `digitalWrite()` implementation
4. Extract exact register writes
5. Replicate in Rust

**Files to check:**
```bash
~/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/cores/gecko/
~/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/variants/xiao_mg24/
```

**Expected outcome**: Missing initialization steps identified

---

## Implementation Priority

### Priority 1: IMMEDIATE (Do First)

1. **Test DOUTTGL register** (1 hour)
   - Simple code change
   - Might immediately solve the problem
   - File: `examples/blink_unlock_cmu.rs`

2. **Extreme delay test** (30 minutes)
   - Rule out timing issues
   - Quick verification
   - File: `examples/blink_slow.rs`

### Priority 2: HIGH (Do Next)

3. **Measure CPU frequency** (2 hours)
   - Use SysTick timer
   - Read CMU registers
   - Compare with Arduino
   - File: `examples/cpu_freq_test.rs`

4. **Register comparison** (1 hour)
   - Arduino vs Rust register dump
   - Find missing writes
   - Document differences
   - File: `docs/REGISTER_COMPARISON.md`

### Priority 3: MEDIUM (If Above Fails)

5. **Check GPIO LOCK** (1 hour)
   - Test unlock sequence
   - Verify security attributes
   - File: `examples/gpio_unlock_test.rs`

6. **Oscilloscope verification** (2 hours)
   - Requires hardware setup
   - Definitive electrical verification
   - Document waveforms

### Priority 4: LOW (Last Resort)

7. **Disassemble Arduino firmware** (3 hours)
   - Binary analysis of working code
   - Extract exact sequence
   - File: `docs/ARDUINO_DISASSEMBLY.md`

8. **Contact Silicon Labs support** (varies)
   - Forum post with all findings
   - Request bare-metal example
   - File: `docs/SILABS_FORUM_POST.md`

---

## Success Criteria

### Minimum Success (Phase 1)
- [ ] DOUTTGL blink works at any visible rate

### Partial Success (Phase 2)
- [ ] Know exact CPU frequency
- [ ] Can calculate correct delay
- [ ] LED blinks at 1 Hz ± 10%

### Complete Success (Phase 3)
- [ ] LED blinks reliably at 1 Hz
- [ ] Understand all initialization requirements
- [ ] Documented working bare-metal example
- [ ] Can replicate on any EFR32MG24

### Ideal Success (Phase 4)
- [ ] Create `efr32mg24-hal` crate with working GPIO
- [ ] SysTick-based delay implementation
- [ ] All peripherals initialized correctly
- [ ] Published documentation and examples

---

## Next Immediate Actions

### Action 1: Test DOUTTGL (NOW)

Build and test the version with DOUTTGL register:

```bash
cd efr32mg24-pac
cargo build --example blink_unlock_cmu --features rt --release
arm-none-eabi-objcopy -O ihex \
  ../target/thumbv8m.main-none-eabihf/release/examples/blink_unlock_cmu \
  blink_unlock_cmu.hex
python3 patch_word13.py blink_unlock_cmu.hex blink_unlock_cmu_patched.hex 0x08006228
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init" -c "program blink_unlock_cmu_patched.hex verify" \
  -c "reset run" -c "exit"
```

**Expected time**: 15 minutes
**Expected outcome**: Visible LED blink OR rule out toggle method

### Action 2: Create Extreme Delay Test

```bash
# Create new example with 10-second delays
cp examples/blink_unlock_cmu.rs examples/blink_extreme_slow.rs
# Edit to use 400M NOP delays
cargo build --example blink_extreme_slow --features rt --release
# Flash and test
```

**Expected time**: 30 minutes
**Expected outcome**: Visible blink OR confirm it's not timing

### Action 3: Register Comparison

```bash
# Flash Arduino
openocd -c "init" -c "program arduino_blink.hex" -c "reset halt" \
  -c "mdw 0x40008000 32" -c "mdw 0x5003C030 16" > arduino_regs.txt

# Flash Rust
openocd -c "init" -c "program blink_unlock_cmu_patched.hex" -c "reset halt" \
  -c "mdw 0x40008000 32" -c "mdw 0x5003C030 16" > rust_regs.txt

# Compare
diff -u arduino_regs.txt rust_regs.txt
```

**Expected time**: 45 minutes
**Expected outcome**: Find missing register initialization

---

## Timeline Estimate

**Optimistic**: 2-4 hours
- DOUTTGL works immediately
- Document solution
- Create clean examples

**Realistic**: 1-2 days
- Need to measure CPU frequency
- Adjust timing calculations
- Test and verify
- Document findings

**Pessimistic**: 3-5 days
- Hardware issues found
- Need Silicon Labs support
- Wait for response
- Implement complex solution

---

## Resources Needed

### Hardware
- [x] XIAO MG24 Sense board
- [ ] Oscilloscope (optional but helpful)
- [ ] Logic analyzer (optional)

### Software
- [x] Rust toolchain
- [x] arm-none-eabi-gcc
- [x] OpenOCD
- [x] Python 3
- [x] All examples and scripts

### Documentation
- [x] EFR32MG24 Reference Manual
- [x] Gecko Bootloader documentation
- [x] Arduino BSP source code
- [ ] Silicon Labs forum access
- [ ] GitHub issues/examples

### Knowledge
- [x] Bootloader integration
- [x] CMU unlock sequence
- [x] GPIO register addresses
- [x] TrustZone-M basics
- [ ] Exact clock configuration
- [ ] Complete peripheral init sequence

---

## Risk Assessment

### Low Risk (Can Handle)
- Timing calculation errors → Adjust based on measurement
- Missing GPIO config → Copy from Arduino
- Compiler optimization → Use volatile or debug build

### Medium Risk (Might Need Help)
- Complex clock tree → Need Silicon Labs docs
- Security configuration → Might need bootloader modification
- Hardware issues → Need oscilloscope

### High Risk (Blockers)
- Bootloader bug → Need Silicon Labs fix
- Hardware defect → Need replacement board
- Undocumented requirements → Need Silicon Labs support

**Mitigation**: Comprehensive documentation to share with Silicon Labs if needed

---

## Rollback Plan

If we can't solve the blink issue:

1. **Document everything** we learned (already done)
2. **Post to Silicon Labs forum** with full details
3. **Request bare-metal example** from Silicon Labs
4. **Continue with Arduino** for now
5. **Return to Rust** when we have answers

**But**: We're very close! The LED IS responding, we just need to find the right timing/toggle method.

---

## Conclusion

We have:
- ✅ Bootloader working
- ✅ Clock enable working
- ✅ GPIO control working
- ✅ LED responding to writes

We're 95% there! The remaining 5% is likely:
- Wrong timing calculation (most likely)
- Wrong toggle method (test DOUTTGL next)
- Missing one register write (compare with Arduino)

**Recommended next step**: Test DOUTTGL register immediately - it's already coded and ready to flash!
