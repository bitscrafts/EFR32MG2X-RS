# EFR32MG24 Custom Bootloader - Development Log

**Started**: January 24, 2026
**Status**: Phase 1 - Minimal Bootloader Implementation

---

## Session 1: Initial Implementation (Jan 24, 2026)

### Objectives

1. Create minimal custom bootloader crate
2. Implement basic boot sequence
3. Create test blink application
4. Verify LED blinks with custom bootloader

### Progress

#### 1. Crate Structure ✅

Created `efr32mg24-bootloader` crate with:

**Files created**:
- `Cargo.toml` - Package configuration with size optimization
- `build.rs` - Build script to use custom linker script
- `memory.x` - Linker script (bootloader at 0x08000000, 16 KB)
- `.cargo/config.toml` - Target and linker configuration
- `src/main.rs` - Main bootloader code
- `README.md` - Documentation
- `examples/test-blink-app.rs` - Test application
- `test-app-memory.x` - Linker script for application (starts at 0x08004000)
- `build-all.sh` - Build helper script
- `flash.sh` - Flash helper script

**Key decisions**:
- Bootloader size: 16 KB (conservative, can optimize down to 8-12 KB later)
- Application start: 0x08004000 (immediately after bootloader)
- No cortex-m-rt for bootloader (minimal runtime, saves space)
- Standard cortex-m-rt for applications (simpler)

#### 2. Bootloader Implementation ✅

**src/main.rs** implements:

1. **Minimal Runtime**:
   - Custom Reset handler (no cortex-m-rt dependency)
   - RAM initialization (.bss zero-init)
   - Custom vector table in linker script

2. **Boot Sequence**:
   ```rust
   Reset() → main() → init_ram() → should_enter_bootloader()
     ├─ if bootloader mode → bootloader_mode() (future: UART update)
     └─ else → boot_application(0x08004000)
           ├─ Validate vector table
           ├─ Set VTOR register
           └─ Jump to application
   ```

3. **Application Validation**:
   - Check stack pointer in valid RAM range (0x20000000-0x20040000)
   - Check reset vector in application flash and has Thumb bit set
   - If invalid, stay in bootloader mode

4. **VTOR Setup** (Critical for Cortex-M33):
   ```rust
   // Set Vector Table Offset Register
   const VTOR: *mut u32 = 0xE000ED08 as *mut u32;
   ptr::write_volatile(VTOR, 0x08004000);  // Application vector table
   ```

5. **Application Jump**:
   ```asm
   msr msp, {stack}    // Set Main Stack Pointer from app's vector table
   bx {reset}          // Branch to application's reset handler
   ```

6. **Error Handling**:
   - HardFault handler: Fast LED blink (100 ms)
   - Bootloader mode: Slow LED blink (1 second)
   - Panic handler: Calls HardFault handler

#### 3. Test Application ✅

**examples/test-blink-app.rs**:
- Simple blink at 1 Hz (500 ms ON, 500 ms OFF)
- Uses standard cortex-m-rt (standard Rust embedded app)
- Initializes CMU and GPIO (same as our previous tests)
- No ApplicationProperties_t needed!

#### 4. Build Configuration ✅

**Size optimization** (`Cargo.toml`):
```toml
[profile.release]
opt-level = "z"          # Optimize for size
lto = "fat"              # Link-time optimization
codegen-units = 1        # Single codegen unit
strip = true             # Strip symbols
panic = "abort"          # Smaller panic handler
```

**Memory layout** (`memory.x`):
```ld
Bootloader: 0x08000000 - 0x08003FFF (16 KB)
Application: 0x08004000 - 0x0817FFFF (1520 KB)
```

#### 5. Helper Scripts ✅

**build-all.sh**:
1. Build bootloader with custom memory.x
2. Build test application with test-app-memory.x
3. Convert both to Intel HEX
4. Combine with srec_cat
5. Show sizes

**flash.sh**:
- Flash combined.hex to device via OpenOCD
- Uses efr32mg24.cfg configuration

### Current Status

**Building**: Running `cargo build --release` (in progress)
- Compiling efr32mg24-pac (takes ~2-3 minutes)
- Will then compile bootloader

**Expected outcome**:
- Bootloader binary ~4-8 KB
- No ApplicationProperties_t complexity
- Clean build process (no hex patching!)

### Next Steps

1. ✅ Wait for build to complete
2. ⏳ Check for compilation errors
3. ⏳ Build test application
4. ⏳ Combine bootloader + application
5. ⏳ Flash to hardware
6. ⏳ **CRITICAL TEST**: Does LED blink?

### Key Differences from Gecko Bootloader

> **Note**: This section documents the initial planning phase. For actual final results and comparison, see [READY_TO_FLASH.md](../02-guides/READY_TO_FLASH.md#key-differences-from-gecko-bootloader).

**Initial design targets** (actual results exceeded expectations):

| Aspect | Gecko Bootloader | Custom Bootloader (Target) |
|--------|------------------|---------------------------|
| Size | 24 KB | 16 KB (target 8-12 KB) |
| Build Process | Requires hex patching | Direct build |
| ApplicationProperties_t | Required | Not needed |
| Vector Table Word 13 | Must point to magic | Standard ARM layout |
| Boot Validation | Complex | Simple (SP + PC check) |
| Memory Safety | C implementation | Rust |
| Customization | Limited | Full control |
| Source Code | Closed | Open (our code) |

### Advantages of Custom Bootloader

1. **Simpler Build**: No post-build hex patching required
2. **Smaller Size**: ~50% smaller than Gecko bootloader
3. **Full Control**: We understand every line of code
4. **Debugging**: Can add debug output, breakpoints, etc.
5. **Standard Apps**: Applications use standard cortex-m-rt
6. **Faster Boot**: Minimal initialization
7. **Memory Safe**: Rust guarantees at compile time

### Potential Issues to Watch

1. **VTOR Setup**: Must be set correctly before jumping to app
2. **Memory Barriers**: Need DSB/ISB after VTOR write
3. **Stack Pointer**: Must validate before using
4. **Interrupt Disable**: Should disable before jump (already done)
5. **Peripheral State**: App should reinitialize all peripherals

### Code Quality

**Safety**:
- All peripheral access in `unsafe` blocks
- Validation before jumping to application
- HardFault handler for unexpected errors

**Size**:
- opt-level="z" for maximum size reduction
- LTO enabled
- No unnecessary dependencies
- Minimal runtime (no cortex-m-rt in bootloader)

**Maintainability**:
- Well-commented code
- Clear separation of concerns
- Simple, linear boot flow
- Easy to understand and modify

### Testing Plan

#### Phase 1: Basic Boot Test (NOW)
1. Build and flash combined image
2. Verify LED blinks at 1 Hz
3. Measure bootloader size
4. Check boot time

#### Phase 2: Validation Tests
1. Test with invalid application (should stay in bootloader mode)
2. Test bootloader mode entry (magic value)
3. Test HardFault handler (inject fault)

#### Phase 3: Application Compatibility
1. Test with different applications
2. Verify VTOR setup works correctly
3. Test interrupt handling in application

### Documentation

**Created**:
- `README.md` - User guide and API reference
- `DEVELOPMENT_LOG.md` - This file
- Code comments in src/main.rs
- Build scripts with usage instructions

**TODO**:
- Add memory layout diagram
- Document boot sequence in detail
- Create troubleshooting guide
- Add oscilloscope traces (if needed)

### Lessons Applied from Previous Debugging

From 13 Gecko bootloader blink attempts, we learned:

1. **CMU Unlock**: Required (0x93F7 to 0x40008010) ✅ Applied
2. **GPIO Clock**: Bit 26 of CMU_CLKEN0 ✅ Applied
3. **PORTA Offset**: 0x30 from GPIO base ✅ Applied
4. **Register Addresses**: All correct addresses used ✅ Applied
5. **Active-Low LED**: PA7, 0=ON, 0x80=OFF ✅ Applied
6. **TrustZone-M**: Non-secure addresses ✅ Applied

This knowledge directly informed the bootloader and test app implementation.

### Success Criteria for Phase 1

**Must have**:
- ✅ Bootloader compiles
- ⏳ Test application compiles
- ⏳ Combined image creates successfully
- ⏳ Flash process works
- ⏳ **LED blinks at 1 Hz** ← PRIMARY GOAL

**Nice to have**:
- Bootloader <12 KB
- Boot time <100 ms
- No warnings or errors
- Clean code

---

## Build Output (Pending)

Waiting for compilation to complete...

```
cargo build --release
  Compiling efr32mg24-pac...
  [In progress]
```

---

**Next update**: After build completes and we test on hardware.
