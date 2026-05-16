# Blink Debug Plan - Systematic Approach

**Problem**: Rust firmware compiles, flashes successfully, but doesn't execute (LED never blinks)

**Date**: 2026-01-10
**Status**: Planning systematic debug approach

## Current Situation

### What Works ✅
- OpenOCD CMSIS-DAP flashing
- Gecko Bootloader (microphone example runs)
- Rust compilation and hex generation
- ApplicationProperties_t structure created
- Vector table word 13 patched correctly

### What Doesn't Work ❌
- Rust firmware never executes (no LED blink)
- Even simplest possible code doesn't run

## Root Cause Hypotheses

1. **Bootloader validation failing** - Something in our binary is rejected
2. **Reset handler not executing** - Startup code issue
3. **Missing critical initialization** - Silicon Labs specific requirement
4. **Binary format incompatibility** - Cortex-M-RT vs expected format

## Systematic Debug Plan

### Phase 1: C Baseline (Proven Working Code)
**Goal**: Create minimal C blink that works with Gecko Bootloader

**Steps**:
1. Create minimal C application (blink.c):
   - Just GPIO toggle
   - Use Silicon Labs SDK minimal template
   - Include ApplicationProperties_t structure
   - No complex initialization

2. Build with Silicon Labs toolchain:
   - Use same linker script concepts (0x08006000 start)
   - Generate hex file
   - Verify ApplicationProperties_t in binary

3. Flash and verify:
   - Use OpenOCD (same method as Rust)
   - Confirm LED blinks
   - **Baseline established** ✅

### Phase 2: Binary Analysis (Find Differences)
**Goal**: Identify exact differences between working C and non-working Rust

**Compare**:
1. **Vector tables**:
   - Stack pointer values
   - Reset handler addresses
   - Exception handler table
   - Word 13 (ApplicationProperties pointer)

2. **ApplicationProperties_t structure**:
   - Magic signature
   - Structure version
   - Signature type/location
   - Application data fields
   - Exact byte-by-byte comparison

3. **Startup code**:
   - Entry point behavior
   - Stack setup
   - BSS initialization
   - Data copying from flash to RAM

4. **Memory layout**:
   - Section placement (.text, .data, .bss, .rodata)
   - Alignment requirements
   - Section sizes

**Tools**:
```bash
# Disassemble both binaries
arm-none-eabi-objdump -d c_blink.elf > c_blink.asm
arm-none-eabi-objdump -d rust_blink.elf > rust_blink.asm

# Compare sections
arm-none-eabi-readelf -S c_blink.elf
arm-none-eabi-readelf -S rust_blink.elf

# Hex dump comparison
xxd c_blink.hex > c_blink.hexdump
xxd rust_blink.hex > rust_blink.hexdump
diff c_blink.hexdump rust_blink.hexdump
```

### Phase 3: Rust Port (Replicate Working C)
**Goal**: Make Rust binary identical to working C binary

**Approach**:
1. **Match startup sequence**:
   - Ensure cortex-m-rt does same initialization as C startup
   - Check BSS clearing
   - Check data section copying

2. **Match ApplicationProperties**:
   - Use exact same structure as C
   - Same field values
   - Same section placement

3. **Match memory layout**:
   - Adjust linker script to match C layout
   - Same section alignments
   - Same section order

4. **Match GPIO code**:
   - Use exact same register addresses
   - Same register values
   - Same sequence

### Phase 4: Incremental Testing
**Goal**: Test each component isolation

**Test Sequence**:
1. **Test 1**: Minimal main that does nothing
   ```rust
   #[entry]
   fn main() -> ! {
       loop { cortex_m::asm::nop(); }
   }
   ```
   - Flash and check with debugger if main() is reached

2. **Test 2**: Just GPIO init, no toggle
   ```rust
   fn main() -> ! {
       // Init GPIO
       // Set PA7 to known state
       loop { cortex_m::asm::nop(); }
   }
   ```

3. **Test 3**: Single toggle (no loop)
   ```rust
   fn main() -> ! {
       // Init GPIO
       // Toggle once
       loop { cortex_m::asm::nop(); }
   }
   ```

4. **Test 4**: Slow blink (10s delay)
   ```rust
   fn main() -> ! {
       loop {
           // Toggle
           // 10 second delay
       }
   }
   ```

### Phase 5: Debugger Verification (Optional)
**Goal**: Use OpenOCD/GDB to verify execution

**Commands**:
```bash
# Start OpenOCD
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg

# In another terminal, connect GDB
arm-none-eabi-gdb target/.../rust_blink.elf
(gdb) target remote :3333
(gdb) monitor reset halt
(gdb) break main
(gdb) continue
# Check if breakpoint hits
(gdb) info registers
(gdb) x/32x 0x08006000  # Examine vector table
```

## Alternative Approaches

### Option A: Use Arduino Bootloader Source
- Get Arduino bootloader source code
- Understand exact validation logic
- Replicate in Rust binary

### Option B: Simplicity Studio Project
- Create minimal blink in Simplicity Studio
- Export as C project
- Use that as C baseline

### Option C: Skip Bootloader Entirely
- Flash Rust firmware directly to 0x08000000
- Erase bootloader (we can restore it)
- Verify Rust code itself works
- Then add bootloader compatibility

### Option D: Community Help
- Post on Silicon Labs forums
- Post on Rust embedded forum
- Share exact issue with binaries

## Decision Tree

```
Start
  │
  ├─→ Create C baseline → Works?
  │                          ├─→ Yes: Compare C vs Rust binaries
  │                          └─→ No: Issue with bootloader itself
  │
  ├─→ Binary comparison → Found differences?
  │                          ├─→ Yes: Fix Rust to match C
  │                          └─→ No: Deeper investigation needed
  │
  ├─→ Rust matches C → Works now?
  │                          ├─→ Yes: ✅ SOLVED!
  │                          └─→ No: Use debugger/Option C
  │
  └─→ Still failing → Try Option C (skip bootloader)
                          ├─→ Works: Bootloader validation issue
                          └─→ No: Rust code issue
```

## Success Criteria

- [ ] C baseline blinks successfully
- [ ] Differences between C and Rust identified
- [ ] Rust modified to match C
- [ ] Rust firmware blinks successfully
- [ ] Root cause documented
- [ ] Reproducible solution created

## Expected Timeline

- Phase 1 (C baseline): 30 minutes
- Phase 2 (Analysis): 1 hour
- Phase 3 (Rust port): 1 hour
- Phase 4 (Testing): 30 minutes
- **Total**: ~3 hours to definitive solution

## Next Immediate Actions

1. **Create minimal C blink application** using Silicon Labs SDK
2. **Build with proper toolchain** (arm-none-eabi-gcc + linker script)
3. **Flash and verify working**
4. **Compare binaries** to find the difference

---

**Rationale**: This systematic approach ensures we:
- Have a known-working baseline (C code)
- Can identify exact differences (binary comparison)
- Can fix specific issues (targeted changes)
- Can verify solution (incremental testing)

**This will definitively solve the issue.**
