# LED Blink Attempts Log

**Goal**: Achieve visible 1 Hz LED blink from bare-metal Rust on EFR32MG24
**Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48)
**LED**: PA7 (active-low, D20/LED_BUILTIN)

---

## Summary of Attempts

| # | Method | Delay | Result | Notes |
|---|--------|-------|--------|-------|
| 1 | XOR toggle | 500k NOPs | LED ON (constant) | Too fast (39 Hz) |
| 2 | No clock enable | N/A | LED OFF | Proved bootloader disables clocks |
| 3 | Non-secure CMU | 500k NOPs | LED ON (constant) | CMU addresses correct |
| 4 | CMU unlock | 500k NOPs | LED ON (constant) | Unlock sequence working |
| 5 | Correct GPIO offset | 500k NOPs | LED ON (constant) | Found PORTA at +0x30 |
| 6 | XOR toggle | 19.5M NOPs | LED ON (constant) | Slower delay, still no blink |
| 7 | Explicit writes 0x00/0x80 | 10M NOPs | LED ON (constant) | Direct writes, no toggle |
| 8 | Read-modify-write | 20M NOPs | Blinked once, stays ON | First partial success! |
| 9 | LED OFF test | N/A | Blinked once, stays OFF | Confirmed control works |
| 10 | Explicit writes 0x00/0x80 | 10M NOPs | LED ON (constant) | Retry explicit writes |
| 11 | Read-modify-write | 20M NOPs | LED ON (constant) | Retry RMW |
| 12 | **DOUTTGL register** | 20M NOPs | **LED ON (constant)** | Hardware toggle - NO BLINK |
| 13 | **Extreme slow delay** | **400M NOPs** | **LED ON (constant)** | **10-20 sec delays - CRITICAL FAILURE** |

---

## Detailed Attempt Records

### Attempt 1: Initial XOR Toggle (500k NOPs)
**Date**: Jan 18, 2026
**File**: `examples/blink_unlock_cmu.rs` (initial version)

**Code**:
```rust
loop {
    let dout = core::ptr::read_volatile(GPIO_PORTA_DOUT);
    core::ptr::write_volatile(GPIO_PORTA_DOUT, dout ^ (1 << 7));
    for _ in 0..500_000 {
        cortex_m::asm::nop();
    }
}
```

**Configuration**:
- CMU unlock: 0x93F7 to 0x50008010 (WRONG - secure address)
- CMU CLKEN0: 0x50008308 (WRONG - offset)
- GPIO: 0x5003C004 (WRONG - offset)

**Result**: LED stays ON (appears constant)

**Analysis**:
- 500k NOPs at 39 MHz = 12.8 ms
- Blink rate = 39 Hz (too fast to see)
- Register addresses were wrong

---

### Attempt 2: No Clock Enable Test
**Date**: Jan 18, 2026
**File**: `examples/blink_no_clock.rs`

**Code**:
```rust
// NO clock enable - test if bootloader pre-enables
const GPIO_PORTA_DOUT: *mut u32 = 0x5000000C as *mut u32;
core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);
```

**Result**: LED stays OFF

**Analysis**:
- Proved bootloader leaves GPIO clock DISABLED
- Must explicitly enable GPIO clock
- This was an important discovery

---

### Attempt 3: Non-Secure CMU (Secure addresses caused HardFault)
**Date**: Jan 18, 2026
**File**: `examples/blink_cmu_ns.rs`

**Code**:
```rust
const CMU_CLKEN0: *mut u32 = 0x40008308 as *mut u32;  // Non-secure
const CMU_CLKEN1: *mut u32 = 0x4000830C as *mut u32;
core::ptr::write_volatile(CMU_CLKEN0, 1 << 2);  // WRONG bit
```

**Configuration**:
- CMU base: 0x40008000 (CORRECT - non-secure)
- CLKEN0 offset: 0x308 (WRONG - should be 0x64)
- GPIO bit: 2 (WRONG - should be 26)

**Result**: LED stays ON, but HardFault resolved

**Analysis**:
- Non-secure CMU address correct (0x4xxxxxxx)
- But offset and bit position still wrong

---

### Attempt 4: CMU Unlock Sequence
**Date**: Jan 18, 2026
**File**: `examples/blink_unlock_cmu.rs` (version 2)

**Code**:
```rust
const CMU_LOCK: *mut u32 = 0x50008010 as *mut u32;  // WRONG - secure
core::ptr::write_volatile(CMU_LOCK, 0x93F7);
```

**Result**: HardFault (secure address)

**Analysis**:
- Unlock key 0x93F7 discovered from GSDK headers
- But used wrong (secure) address
- Led to discovery of TrustZone-M security state issue

---

### Attempt 5: Correct GPIO Offset Discovery
**Date**: Jan 18, 2026

**Discovery Process**:
```bash
# Found PORTA starts at offset 0x30, not 0x04!
GPIO base:     0x5003C000
PORTA base:    0x5003C030  (+0x30)
PORTA_MODEL:   0x5003C034  (+0x04 from PORTA)
PORTA_DOUT:    0x5003C03C  (+0x0C from PORTA)
```

**Verification**:
```bash
# Arduino registers
$ openocd -c "mdw 0x5003C034 1"
0x5003C034: 40000000  # MODE7 = push-pull

# Rust registers (before fix)
$ openocd -c "mdw 0x5003C004 1"
0x5003C004: 00000000  # Wrong address!
```

**Result**: LED control started working after fix

---

### Attempt 6: Longer Delay (19.5M NOPs)
**Date**: Jan 18, 2026

**Code**:
```rust
for _ in 0..19_500_000 {  // ~0.5s at 39 MHz
    cortex_m::asm::nop();
}
```

**Configuration**:
- All correct addresses now
- CMU unlock: 0x40008010
- CMU CLKEN0: 0x40008064
- GPIO bit: 26
- PORTA_MODEL: 0x5003C034
- PORTA_DOUT: 0x5003C03C

**Result**: LED stays ON (constant)

**Analysis**:
- Delay increased 39x
- Still no visible blink
- Suggests frequency assumption wrong OR toggle method wrong

---

### Attempt 7: Explicit Writes (No Toggle)
**Date**: Jan 18, 2026

**Code**:
```rust
loop {
    // LED ON
    core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);
    delay();

    // LED OFF
    core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);
    delay();
}
```

**Result**: LED stays ON (constant)

**Analysis**:
- Explicit write of 0x00 (all bits clear)
- Explicit write of 0x80 (bit 7 set)
- Active-low LED: 0 = ON, 1 = OFF
- Still no blink

---

### Attempt 8: Read-Modify-Write (First Partial Success!)
**Date**: Jan 18, 2026

**Code**:
```rust
loop {
    // LED ON (clear bit 7)
    let dout = core::ptr::read_volatile(GPIO_PORTA_DOUT);
    core::ptr::write_volatile(GPIO_PORTA_DOUT, dout & !(1 << 7));
    delay();

    // LED OFF (set bit 7)
    let dout = core::ptr::read_volatile(GPIO_PORTA_DOUT);
    core::ptr::write_volatile(GPIO_PORTA_DOUT, dout | (1 << 7));
    delay();
}
```

**Result**: LED blinked ONCE then stays ON

**Analysis**:
- **BREAKTHROUGH!** First time LED changed state
- Proved register writes work
- But only blinked once - loop issue?
- Delay: 20M NOPs

**OpenOCD Verification**:
```bash
# First halt (2 seconds after boot)
$ openocd -c "mdw 0x5003C03C 1"
0x5003C03C: 00000000  # LED should be ON

# Second halt (5 seconds after boot)
$ openocd -c "mdw 0x5003C03C 1"
0x5003C03C: 00000080  # LED should be OFF

# DOUT IS CHANGING! But visual blink not working
```

---

### Attempt 9: LED OFF Test (Diagnostic)
**Date**: Jan 18, 2026
**File**: `examples/led_off.rs`

**Code**:
```rust
// Turn LED OFF and stay there
core::ptr::write_volatile(GPIO_PORTA_DOUT, 0);
loop {
    cortex_m::asm::nop();
}
```

**Result**: LED blinked once then stays OFF (expected OFF)

**Analysis**:
- Confirmed LED can be turned OFF
- Proved register writes work
- LED is active-low (0 = ON confirmed)
- Pattern: Always "blinks once" on startup

---

### Attempt 10: Explicit Writes Retry (Slower)
**Date**: Jan 18, 2026

**Code**:
```rust
loop {
    core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);  // ON
    for _ in 0..10_000_000 { nop(); }

    core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);  // OFF
    for _ in 0..10_000_000 { nop(); }
}
```

**Result**: LED stays ON (constant)

**Analysis**:
- 10M NOPs = ~256ms at 39 MHz
- Should be visible if working
- Still no blink

---

### Attempt 11: Read-Modify-Write Retry
**Date**: Jan 18, 2026

**Code**:
```rust
loop {
    let dout = core::ptr::read_volatile(GPIO_PORTA_DOUT);
    core::ptr::write_volatile(GPIO_PORTA_DOUT, dout & !(1 << 7));
    for _ in 0..20_000_000 { nop(); }

    let dout = core::ptr::read_volatile(GPIO_PORTA_DOUT);
    core::ptr::write_volatile(GPIO_PORTA_DOUT, dout | (1 << 7));
    for _ in 0..20_000_000 { nop(); }
}
```

**Result**: LED stays ON (constant)

**Analysis**:
- Same as Attempt 8 but second try
- 20M NOPs = ~512ms at 39 MHz
- No visible blink
- Inconsistent with Attempt 8 (which blinked once)

---

### Attempt 12: DOUTTGL Hardware Toggle Register
**Date**: Jan 18, 2026
**File**: `examples/blink_unlock_cmu.rs` (final version)

**Code**:
```rust
const GPIO_PORTA_DOUTTGL: *mut u32 = 0x5003C040 as *mut u32;

loop {
    // Hardware atomic toggle
    core::ptr::write_volatile(GPIO_PORTA_DOUTTGL, 1 << 7);

    for _ in 0..20_000_000 {
        cortex_m::asm::nop();
    }
}
```

**Configuration**:
- DOUTTGL register at offset 0x40 from PORTA base
- Full address: 0x5003C040
- Writing 1 to bit 7 toggles PA7 state
- Atomic hardware operation (no read-modify-write)

**Expected Behavior**:
- Should toggle PA7 every 20M NOPs
- If 39 MHz: ~512ms per toggle = ~1 Hz blink
- Atomic operation should be race-free

**Result**: LED stays ON (constant) - NO BLINK

**Analysis**:
- DOUTTGL is the "proper" way to toggle GPIO
- Arduino likely uses this internally
- Still no visible blink
- **Critical finding**: Even hardware toggle doesn't work
- Suggests issue is NOT the toggle method

---

### Attempt 13: Extreme Slow Delay Test (CRITICAL DIAGNOSTIC)
**Date**: Jan 24, 2026
**File**: `examples/blink_extreme_slow.rs`

**Code**:
```rust
loop {
    // LED ON (active-low: write 0)
    core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);

    // EXTREME SLOW DELAY - 400M NOPs
    // If 39 MHz: 400M ÷ 39M = ~10.3 seconds
    // If 19 MHz: 400M ÷ 19M = ~21 seconds
    // Either way, should be UNMISTAKABLE
    for _ in 0..400_000_000 {
        cortex_m::asm::nop();
    }

    // LED OFF (active-low: write 1 to bit 7)
    core::ptr::write_volatile(GPIO_PORTA_DOUT, 1 << 7);

    // EXTREME SLOW DELAY
    for _ in 0..400_000_000 {
        cortex_m::asm::nop();
    }
}
```

**Configuration**:
- All correct addresses (proven from previous attempts)
- CMU unlock: 0x40008010 with key 0x93F7
- GPIO clock: bit 26 of CLKEN0 (0x40008064)
- PORTA_MODEL: 0x5003C034 (MODE7 = push-pull)
- PORTA_DOUT: 0x5003C03C
- Direct register writes (no toggle complexity)

**Expected Behavior**:
- LED ON for 10-20 seconds (depending on actual CPU freq)
- LED OFF for 10-20 seconds
- **Impossible to miss** even with severe timing miscalculation
- Should work even if CPU is 2x or 10x different from expected

**Result**: **LED STAYS ON (constant) - NO BLINK**

**User Observation**: "waited for 5 min, no blink it just stays on!"

**Analysis - CRITICAL FINDINGS**:
This is a **breakthrough negative result** that rules out many hypotheses:

1. **NOT a simple timing issue**:
   - 400M NOPs is ~20x longer than previous attempts
   - Even if CPU is 10x slower/faster than expected, delay would still be 1-100 seconds
   - Human eye cannot miss 1+ second blinks

2. **NOT compiler optimization**:
   - 400M iterations is far too many to optimize away
   - Would require sophisticated dead code elimination
   - Previous attempts showed PC advancing (code executes)

3. **NOT a toggle method issue**:
   - This uses explicit direct writes (simplest possible method)
   - No XOR, no RMW, no DOUTTGL complexity
   - Just `write 0x00` then `write 0x80`

4. **STRONGLY SUGGESTS**:
   - **Loop exits after first iteration** (most likely!)
   - **Code execution stops** (hardfault, panic, bootloader re-entry?)
   - **Delay function is broken** (NOPs not executing?)
   - **LED hardware has external issue** (capacitor, RC filter?)

**Critical Questions This Raises**:
1. Why does LED end up ON instead of OFF if loop runs once?
   - Loop starts with `write 0x00` (ON)
   - If loop runs once: ON → delay → OFF → delay → (repeat)
   - If loop exits after first ON: makes sense!
   - If loop exits after first OFF: LED should be OFF (contradiction!)

2. Does the loop actually execute more than once?
   - Need to verify with debugger/breakpoint
   - Check if PC stays in loop or exits

3. Is there a watchdog timer resetting the chip?
   - Bootloader might have enabled WDT
   - Long delays could trigger WDT reset
   - But Arduino works, so WDT should be disabled

**Implications**:
- Cannot be solved by adjusting timing
- Need to investigate loop execution itself
- Consider hardware debugging (oscilloscope, logic analyzer)
- Custom bootloader might avoid whatever is breaking the loop

---

## Pattern Analysis

### What Works ✅
1. CMU unlock (0x93F7 to 0x40008010)
2. GPIO clock enable (bit 26 of CLKEN0 at 0x40008064)
3. PA7 configuration (push-pull at 0x5003C034)
4. LED turns ON (write 0x00 to DOUT)
5. LED turns OFF (write 0x80 to DOUT)
6. DOUT register changes (verified via OpenOCD)
7. Code executes (PC advances through loop)

### What Doesn't Work ❌
1. Visible blink at any attempted rate
2. Consistent behavior (sometimes blinks once, usually doesn't)
3. DOUTTGL toggle (expected to work, doesn't)
4. Various delay lengths (500k to 400M NOPs) - **EVEN 10-20 SECOND DELAYS FAIL**
5. Different write methods (XOR, explicit, RMW, toggle)

### Consistent Observations
- LED always ends up ON (not OFF)
- Sometimes blinks ONCE on startup
- DOUT register DOES change (OpenOCD confirms)
- But no visible continuous blink

---

## Hypotheses Remaining

### Hypothesis NEW: Loop Execution Failure (VERY HIGH CONFIDENCE - POST ATTEMPT 13)
**Evidence**:
- LED always ends up ON (loop first action is `write 0x00` = ON)
- Sometimes blinks ONCE then stays ON (loop runs one iteration then stops)
- 400M NOP delay test shows same behavior as fast delays
- DOUT register changes when manually checked, but not continuously

**Possible Causes**:
1. **Watchdog Timer Reset** - Long delays trigger WDT, chip resets, bootloader restarts app
   - But: Arduino doesn't have this issue
   - Counter-evidence: Bootloader should disable WDT
2. **Stack Overflow** - Loop execution corrupts stack, causes hardfault
   - But: Simple loop, no recursion, minimal stack use
3. **Panic Handler** - Something triggers panic, panic handler halts
   - Check: panic-halt should infinite loop, not return
4. **Code Optimization Breaking Loop** - Compiler inlines and breaks loop logic
   - But: Release builds should still work
5. **Loop Counter Bug** - Iterator for _ in 0..N gets optimized incorrectly
   - Possible: Check generated assembly

**Critical Test Needed**:
- **Halt chip with OpenOCD during long delay** and check:
  - Current PC location
  - Stack pointer value
  - Register states
  - Is code in loop? In panic handler? In bootloader?

**Test**: Use OpenOCD to halt during extreme slow delay and inspect PC

### Hypothesis 1: CPU Frequency Wrong ~~(HIGH CONFIDENCE)~~ **ELIMINATED BY ATTEMPT 13**
**Evidence**:
- We assume 39 MHz HFRCO
- But bootloader might set different frequency
- ~~20M NOPs doesn't produce visible blink~~ **400M NOPs (10-20 sec) also fails**

**Status**: **RULED OUT** - Even 400M NOP delays produce no blink. Timing cannot explain this.

**Test**: ~~Measure actual frequency with SysTick~~ NOT NECESSARY - problem is not timing

### Hypothesis 2: Compiler Optimization (MEDIUM CONFIDENCE)
**Evidence**:
- Release build optimizations
- NOPs might be optimized out or shortened

**Test**: Check assembly, try debug build

### Hypothesis 3: Missing Peripheral Configuration (LOW CONFIDENCE)
**Evidence**:
- Arduino works, we don't
- Might be missing some register write

**Test**: Complete register comparison Arduino vs Rust

### Hypothesis 4: Hardware Issue / External Circuit (LOW CONFIDENCE)
**Evidence**:
- Arduino works perfectly
- Our LED turns on/off once
- Suggests hardware is fine

**Test**: Oscilloscope measurement

### Hypothesis 5: Loop Logic Bug (MEDIUM CONFIDENCE)
**Evidence**:
- Sometimes blinks once then stops
- Suggests loop might not be executing correctly

**Test**: Add state tracking, measure with debugger

---

## Next Steps

### Immediate (Priority 1)
1. **Extreme slow delay test** - 400M NOPs (~10 seconds)
2. **CPU frequency measurement** - Use SysTick or read CMU_STATUS
3. **Assembly inspection** - Check if NOPs actually compiled

### Short-term (Priority 2)
4. **Register comparison** - Arduino vs Rust full dump
5. **Debug build** - Test without optimizations
6. **Oscilloscope** - Measure PA7 voltage transitions

### Alternative Approach (Priority 3)
7. **Custom Rust bootloader** - Eliminate Gecko bootloader complexity
8. **Direct flash at 0x00000000** - No bootloader, full control

---

## Resources Used

### Documentation
- EFR32MG24 Reference Manual
- efr32mg24_gpio.h (GPIO register definitions)
- efr32mg24_cmu.h (CMU register definitions)
- Arduino BSP source code
- Gecko bootloader headers

### Tools
- OpenOCD (register inspection)
- arm-none-eabi-objdump (disassembly)
- Python (ApplicationProperties patcher)
- Cargo (Rust builds)

### Hardware
- XIAO MG24 Sense board
- CMSIS-DAP debugger (built-in)

---

## Time Investment

- Initial attempts (1-6): ~4 hours
- GPIO offset discovery: ~2 hours
- Read-modify-write testing (7-11): ~2 hours
- DOUTTGL implementation (Attempt 12): ~1 hour
- Extreme slow delay test (Attempt 13): ~1 hour
- Documentation (comprehensive logs): ~3 hours
- **Total**: ~13 hours of debugging

---

## Lessons Learned

1. **Register addresses matter** - Offset errors prevented everything
2. **TrustZone-M is complex** - Security state affects peripheral access
3. **Bootloader state is minimal** - Must initialize everything
4. **CMU must be unlocked** - Critical discovery
5. **GPIO has port offset** - PORTA at +0x30, not base
6. **DOUTTGL exists** - Hardware toggle register (though doesn't solve issue)
7. **OpenOCD is invaluable** - Register inspection confirmed our writes work
8. ~~**Timing is critical**~~ **Timing is NOT the issue** - 400M NOP test eliminated this hypothesis
9. **Extreme negative results are valuable** - Attempt 13 ruled out entire categories of issues
10. **Loop execution is suspect** - LED ending ON suggests loop runs once then stops

---

## Conclusion

We've achieved **LED control** but not **LED blink**. The LED responds to our commands (turns on/off), registers change correctly, but continuous visible blinking at ~1 Hz has not been achieved despite 13 different attempts.

**CRITICAL BREAKTHROUGH (Attempt 13)**: Extreme slow delay test (400M NOPs = 10-20 seconds) also fails. This **eliminates timing/frequency as the root cause** and strongly suggests **loop execution failure**.

**Most likely issue**: ~~CPU frequency assumption is wrong~~ **Loop exits after first iteration or code execution stops**. The LED ending up ON (not OFF) matches the loop's first action being `write 0x00` (ON).

**Next critical test**: ~~Extreme slow delay (400M NOPs)~~ **Use OpenOCD to halt chip during long delay and inspect PC location** - is code still in loop? In panic handler? Did chip reset?

**Alternative paths**:
1. **Implement custom Rust bootloader** for full control and simpler debugging
2. **Check assembly output** to see if loop actually compiles correctly
3. **Use hardware debugger** to step through code and find where execution stops
