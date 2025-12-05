# EFR32MG24 B220 vs A020 PAC Comparison

**Date**: December 4, 2025
**Status**: B220 PAC Generated and Modularized
**Purpose**: Document differences between B-series (B220) and A-series (A020) PACs

---

## Overview

We've generated PACs for both variants to compare:
- **A020**: EFR32MG24A020F1536GM48 (original, Silicon Labs dev kits)
- **B220**: EFR32MG24B220F1536IM48 (newer, Seeed Studio XIAO MG24 Sense) **← CURRENT**

## B220 PAC Structure (Current)

The B220 PAC was generated using svd2rust v0.37.1 with modular output:

| Metric | Value |
|--------|-------|
| **Source Size** | 19 MB (split across modules) |
| **Total Lines** | 138,448 lines across all files |
| **Module Folders** | 104 peripheral modules |
| **Rust Files** | 2,633 .rs files |
| **lib.rs Size** | 1,529 lines (peripheral definitions) |
| **Compile Time** | ~2 minutes 38 seconds |
| **Output .rlib** | 112 MB |

### Modular Structure

Each peripheral has its own module folder:
```
efr32mg24-pac/src/
├── lib.rs                 # Main crate (1,529 lines)
├── build.rs               # Build script
├── acmp0_ns/              # ACMP0 non-secure
│   └── [register files]
├── acmp0_s/               # ACMP0 secure
│   └── [register files]
├── gpio_s/                # GPIO secure (used by HAL)
│   └── [register files]
├── cmu_s/                 # CMU secure (used by HAL)
│   └── [register files]
└── ... (100+ more peripherals)
```

This modular approach:
- Reduces compilation memory usage
- Improves IDE performance
- Organizes code by peripheral
- Generated automatically by svd2rust

## Source Code Comparison

### File Sizes (Original Comparison)

| Metric | A020 | B220 | Difference |
|--------|------|------|------------|
| **Source Size** | 22 MB | 19 MB (modular) | -14% (B220 smaller) |
| **Lines of Code** | 221,082 | 138,448 | -37% (B220 smaller) |
| **Module Structure** | Monolithic | 104 modules | Modular |
| **Rust Files** | ~1 large file | 2,633 files | Split |

### Compilation

| Metric | A020 | B220 |
|--------|------|------|
| **Compile Time** | ~5 minutes | ~2m 38s |
| **Status** | ✅ Success | ✅ Success |
| **Warnings** | 1 (cfg) | 1 (cfg) |
| **Errors** | 0 | 0 |

## Key Observations

### 1. B220 is Significantly Smaller

The B220 variant has **~36% less code** than A020. Possible reasons:

**Hypothesis 1: Secure/Non-Secure Duplication**
- A020 SVD might have both secure and non-secure peripheral definitions
- B220 might only include one set (likely secure with TrustZone control)
- This would explain the ~2x difference (133 peripherals vs 66)

**Hypothesis 2: Documentation Differences**
- A020 might have more verbose register descriptions
- B220 SVD might be more concise

**Hypothesis 3: Peripheral Differences**
- B220 might have streamlined peripheral set
- Some A-series peripherals might be deprecated in B-series

### 2. Both Compile Successfully

Despite size differences, both PACs:
- Compile without errors
- Have similar structure (same svd2rust version)
- Generate same warning (critical-section feature)

### 3. Compilation Performance

B220 compiles **~50% faster**:
- A020: ~5 minutes
- B220: ~2 minutes 38 seconds

This is expected given the smaller code size.

## Peripheral Analysis

### Expected Peripherals (Both Should Have)

**Core Peripherals**:
- GPIO (Port A-D)
- CMU (Clock Management)
- EMU (Energy Management)
- USART0-1
- I2C0-1
- SPI (USART in SPI mode)
- TIMER0-4
- IADC0 (ADC)
- LDMA (DMA)
- RTC/RTCC

**Radio Peripherals**:
- MODEM
- FRC (Frame Controller)
- AGC (Automatic Gain Control)
- SYNTH (Synthesizer)
- RAC (Radio Controller)

**Security**:
- SEMAILBOX (Secure Mailbox)
- RADIOAES
- SMU (Security Management Unit)
- TrustZone support

**Special**:
- MVP (Math Vector Processor)
- DEVINFO (Device Information)

### To Investigate

Need to examine actual peripheral list in both PACs:
```bash
# Extract peripheral module names
grep "pub mod " src/lib.rs | head -50
grep "pub mod " src_backup_a020/lib.rs | head -50
```

Compare to identify:
- Missing peripherals in B220
- New peripherals in B220
- Renamed peripherals

## Package Differences

| Feature | A020 (GM48) | B220 (IM48) |
|---------|-------------|-------------|
| Package Type | QFN48 | QFN48 |
| Pin Variant | GM | IM |
| Memory | 1536KB/256KB | 1536KB/256KB |
| Radio Power | Standard | High (19.5 dBm) |

**GM vs IM Package**:
- Both are QFN48 (same physical package)
- Different pin mappings or capabilities
- IM = Industrial/standard mapping
- GM = Specific variant mapping

## Recommendations

### Current Approach: Use B220 for XIAO MG24

**Rationale**:
1. ✅ Matches actual hardware (XIAO MG24 Sense)
2. ✅ Smaller, faster to compile
3. ✅ Compiles successfully
4. ✅ Latest silicon revision

### Keep A020 as Reference

**Reasons**:
1. Compare peripheral availability
2. Reference for potential A-series board support
3. Understand silicon evolution
4. Debug unexpected differences

### Future Analysis Needed

1. **Peripheral List Comparison**
   - Extract complete module list from both
   - Identify additions/removals
   - Document incompatibilities

2. **Register-Level Comparison**
   - For common peripherals, compare register layouts
   - Check for address changes
   - Verify field compatibility

3. **HAL Impact Assessment**
   - Which HAL modules need variant-specific code
   - Can HAL abstract differences?
   - Need feature flags for variants?

## Migration Strategy

### For Users with A-Series Hardware

If someone has A-series hardware but we only support B220:

**Option 1**: Use B220 PAC anyway (risky)
- Most peripherals likely identical
- Basic GPIO/USART/I2C probably work
- Advanced features might differ

**Option 2**: Generate A020 PAC separately
- Keep both PACs available
- Use Cargo features to select
- More maintenance burden

**Option 3**: Wait for testing
- Once we have B220 hardware (XIAO)
- Test extensively
- Document any issues
- Then decide on A-series support

### Recommended: Start with B220 Only

1. **Now**: Use B220 PAC for all development
2. **When hardware arrives**: Test thoroughly on XIAO MG24
3. **If A-series support needed**: Add feature-gated A020 PAC later
4. **Long term**: Support both via features if demand exists

## Testing Plan

### When XIAO MG24 Arrives

1. **Basic PAC Test**
   - Read DEVINFO registers
   - Verify chip ID matches B220
   - Check peripheral addresses

2. **Peripheral Tests**
   - GPIO toggle
   - USART echo
   - I2C scan
   - Timer/PWM
   - ADC read

3. **Comparison with A020**
   - If tests fail, check A020 backup
   - Compare register definitions
   - Document actual differences

## Backup Strategy

### A020 PAC Preserved

Location: `efr32mg24-pac/src_backup_a020/`

Contents:
- `lib.rs` (22 MB, 221,082 lines)
- `device.x` (linker script)
- `build.rs` (build script)

**Purpose**:
- Reference for peripheral comparison
- Fallback if B220 has issues
- Historical record of initial PAC

**Do Not Delete** until:
- B220 verified working on hardware
- Peripheral comparison complete
- HAL stable on B220

## Conclusion

### Summary

The B220 PAC is **significantly smaller** than A020 (-36%), likely due to:
- Streamlined peripheral definitions
- Single secure/non-secure set
- Less documentation in SVD

Both compile successfully, but B220 is:
- ✅ **Correct for our target hardware** (XIAO MG24)
- ✅ **Faster to compile** (2.6 min vs 5 min)
- ✅ **More maintainable** (smaller codebase)

### Decision

**Proceed with B220 as primary PAC**, keep A020 as reference.

### Next Steps

1. ✅ B220 PAC generated and compiles
2. ⏳ Update documentation to reflect B220 as primary
3. ⏳ Extract and compare peripheral lists
4. ⏳ Begin HAL development using B220
5. ⏳ Test on XIAO MG24 hardware when it arrives

---

**Document Version**: 1.0
**Last Updated**: December 3, 2025
**Status**: B220 PAC Active, A020 Preserved for Reference
