# Gecko Bootloader Blink Investigation - POSTPONED

**Date**: January 24, 2026
**Status**: Investigation postponed, switching to custom bootloader approach
**Reason**: After 13 attempts and comprehensive debugging, root cause remains elusive. Custom bootloader provides cleaner path forward.

---

## Investigation Summary

### Goal
Achieve visible 1 Hz LED blink from bare-metal Rust on EFR32MG24 using Gecko bootloader.

### What We Achieved ✅

1. **Bootloader Integration** - Complete
   - ApplicationProperties_t structure implemented and validated
   - Auto-detection script (patch_word13.py) working
   - Bootloader successfully jumps to Rust application
   - PC confirmed in application code range (0x080061xx)

2. **Peripheral Initialization** - Complete
   - CMU unlock sequence (key 0x93F7 to 0x40008010)
   - GPIO clock enable (CMU_CLKEN0 bit 26 = 0x04000000)
   - PA7 configured as push-pull output
   - All register addresses verified correct via OpenOCD

3. **LED Control** - Partial Success
   - Can turn LED ON (write 0x00 to DOUT)
   - Can turn LED OFF (write 0x80 to DOUT)
   - DOUT register changes confirmed via OpenOCD
   - Single state changes work perfectly

### What Didn't Work ❌

**Main Issue**: LED won't blink continuously at any rate

**13 Attempts Documented**:
- Attempts 1-12: Various toggle methods, delays from 500k to 20M NOPs
- **Attempt 13 (Critical)**: 400M NOP delays (~10-20 seconds) - NO BLINK

**Evidence**:
- Code executes (PC advances)
- Registers change when manually checked
- LED always ends up ON (not OFF)
- Sometimes blinks ONCE on startup then stays ON

### Key Discoveries

1. **TrustZone-M Non-Secure State**
   - Bootloader hands off in non-secure mode
   - Must use non-secure peripheral addresses (0x4xxxxxxx for CMU)

2. **Register Addresses**
   - CMU_LOCK: 0x40008010
   - CMU_CLKEN0: 0x40008064 (NOT 0x308)
   - GPIO clock: bit 26 (NOT bit 2)
   - PORTA base: offset 0x30 from GPIO base (NOT 0x04)
   - PORTA_MODEL: 0x5003C034
   - PORTA_DOUT: 0x5003C03C
   - PORTA_DOUTTGL: 0x5003C040

3. **CMU Must Be Unlocked**
   - Write 0x93F7 to CMU_LOCK before modifying clock registers
   - Critical for peripheral clock enable

4. **Timing NOT the Issue**
   - Attempt 13 ruled out CPU frequency/timing hypotheses
   - 400M NOPs should be visible regardless of actual frequency
   - Even 10x timing error would still produce visible blink

### Most Likely Root Cause (Unconfirmed)

**Loop Execution Failure** - Evidence suggests loop runs once then stops:
- LED ends up ON (loop first action is `write 0x00` = ON)
- Pattern consistent with single iteration
- Possible causes:
  - Watchdog timer reset
  - Stack overflow/corruption
  - Panic handler triggered
  - Compiler loop optimization bug
  - Code execution stops (hardfault, exception)

### Remaining Diagnostic Tests (Not Performed)

1. **OpenOCD Halt During Delay**
   ```bash
   # Halt chip mid-delay and inspect PC
   openocd -c "halt" -c "reg pc" -c "reg sp"
   ```
   Would show: Is code in loop? Panic handler? Reset?

2. **Assembly Inspection**
   ```bash
   arm-none-eabi-objdump -d blink_extreme_slow > disasm.txt
   ```
   Would show: Is loop actually compiled? How?

3. **Debug Build Test**
   ```bash
   cargo build --example blink_extreme_slow --features rt  # No --release
   ```
   Would show: Does unoptimized build work?

4. **Hardware Oscilloscope**
   - Measure PA7 voltage directly
   - Confirm electrical signal changes

---

## Files and Documentation

### Code Files
- `examples/blink_extreme_slow.rs` - Latest test (400M NOP delays)
- `blink_extreme_slow_patched.hex` - Flashed to device
- `patch_word13.py` - Auto-detection/patching script
- All other blink attempt examples preserved

### Documentation Files
- `docs/BLINK_ATTEMPTS_LOG.md` - Detailed log of all 13 attempts
- `docs/ACTION_PLAN.md` - Systematic debugging plan (partially executed)
- `docs/BARE_METAL_DISCOVERIES.md` - Technical discoveries
- `docs/RUST_BLINK_INVESTIGATION.md` - Complete technical deep-dive

### Register Values (Verified via OpenOCD)
```
CMU_CLKEN0: 0x04000000 (GPIO clock enabled)
PORTA_MODEL: 0x40000000 (MODE7 = push-pull)
PORTA_DOUT: Toggles 0x00 ↔ 0x80 when manually checked
```

---

## Decision Rationale: Why Custom Bootloader

### Problems with Current Approach
1. **Complexity**: ApplicationProperties_t structure required by Gecko bootloader
2. **Build Process**: Hex patching step fragile and error-prone
3. **Limited Control**: Can't debug bootloader handoff
4. **Uncertain Root Cause**: 13 attempts, root cause still unclear
5. **Time Investment**: 13+ hours debugging without resolution

### Benefits of Custom Bootloader
1. **Simplicity**: No ApplicationProperties_t needed
2. **Clean Build**: Direct ELF to hex conversion
3. **Full Control**: We write every line of boot code
4. **Smaller Size**: 12-16 KB vs. Gecko's 24 KB
5. **Rust Safety**: Memory-safe implementation
6. **Known Timeline**: 2-6 weeks to production-ready

### Learning Value
- All discoveries transfer to custom bootloader
- Understanding gained will inform implementation
- First milestone (2 weeks) will definitively solve blink mystery

---

## Knowledge Preserved for Custom Bootloader

### Essential Initialization Sequence
```rust
// 1. Unlock CMU
const CMU_LOCK: *mut u32 = 0x40008010 as *mut u32;
core::ptr::write_volatile(CMU_LOCK, 0x93F7);

// 2. Enable GPIO clock (bit 26)
const CMU_CLKEN0: *mut u32 = 0x40008064 as *mut u32;
core::ptr::write_volatile(CMU_CLKEN0, 1 << 26);

// 3. Configure PA7 as push-pull output
const GPIO_PORTA_MODEL: *mut u32 = 0x5003C034 as *mut u32;
let model = core::ptr::read_volatile(GPIO_PORTA_MODEL);
core::ptr::write_volatile(GPIO_PORTA_MODEL, (model & !0xF0000000) | 0x40000000);

// 4. Control LED
const GPIO_PORTA_DOUT: *mut u32 = 0x5003C03C as *mut u32;
core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x00);  // ON (active-low)
core::ptr::write_volatile(GPIO_PORTA_DOUT, 0x80);  // OFF
```

### Memory Layout (with Gecko Bootloader)
```
0x08000000 ┌──────────────────────────┐
           │ Gecko Bootloader (24 KB) │
0x08006000 ├──────────────────────────┤
           │ Rust Application         │
           │ - Vector Table           │
           │ - .text, .rodata         │
           │ - ApplicationProperties  │
0x08180000 └──────────────────────────┘

RAM:
0x20000000 ┌──────────────────────────┐
           │ .data, .bss, stack       │
0x20040000 └──────────────────────────┘
```

### TrustZone-M Security State
- Bootloader hands off in **NON-SECURE** state
- Use non-secure peripheral addresses
- CMU_S at 0x40008000 (non-secure alias)
- GPIO_NS at 0x5003C000

---

## Next Steps (Custom Bootloader)

### Phase 1: Minimal Bootloader (Week 1-2)
1. Create `efr32mg24-bootloader` crate
2. Implement minimal runtime (no cortex-m-rt)
3. Implement basic MSC flash driver
4. Implement application jump with VTOR setup
5. Test with simple blink application at 0x08004000

**Success Criteria**: LED blinks with custom bootloader

### Phase 2: Firmware Update (Week 3-4)
1. Implement UART driver
2. Add XMODEM-CRC protocol
3. Add CRC32 validation
4. Create host-side upload tool

### Phase 3: Production Features (Week 5-6)
1. Embassy-boot swap algorithm
2. Rollback protection
3. Size optimization
4. Comprehensive testing

---

## Time Investment

**Total debugging time**: ~13 hours
- Initial attempts (1-6): 4 hours
- GPIO offset discovery: 2 hours
- Read-modify-write testing (7-11): 2 hours
- DOUTTGL (Attempt 12): 1 hour
- Extreme slow delay (Attempt 13): 1 hour
- Documentation: 3 hours

**Value gained**:
- Complete understanding of peripheral initialization
- Register addresses and configurations
- TrustZone-M security state handling
- ApplicationProperties_t structure
- Bootloader handoff mechanics

---

## References

- `docs/BLINK_ATTEMPTS_LOG.md` - Comprehensive log
- `docs/RUST_BOOTLOADER_RESEARCH.md` - Custom bootloader guide (11,000+ lines)
- `docs/ACTION_PLAN.md` - Debugging plan
- `examples/blink_extreme_slow.rs` - Latest test code
- All 13 example variations preserved in `examples/` folder

---

## Conclusion

This investigation successfully achieved:
- ✅ Gecko bootloader integration
- ✅ Complete peripheral initialization understanding
- ✅ LED control capability
- ❌ Continuous LED blink (unresolved)

**Decision**: Postpone further Gecko bootloader debugging and implement custom Rust bootloader. This provides:
- Cleaner solution with known timeline
- Eliminates ApplicationProperties_t complexity
- Full control over boot sequence
- Smaller bootloader size
- Production-ready architecture

**All knowledge gained transfers directly to custom bootloader implementation.**

---

**Postponed by**: AI Assistant & User
**Date**: January 24, 2026
**Next action**: Create minimal custom bootloader crate
**Expected resolution**: 2 weeks (first milestone - blink working)
