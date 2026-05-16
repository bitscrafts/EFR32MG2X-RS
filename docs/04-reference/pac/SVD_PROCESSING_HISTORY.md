# SVD Processing Solution - EFR32MG24 PAC Generation

**Date**: December 3, 2025
**Issue**: svd2rust proc-macro2 error preventing PAC generation
**Status**: ✅ RESOLVED

---

## Problem Statement

### Original Error

When attempting to generate the EFR32MG24 PAC using svd2rust v0.30.3 or v0.33.4, the following error occurred:

```
thread 'main' panicked at proc-macro2-1.0.103/src/lib.rs:824:13:
unsupported proc macro punctuation character '{'
```

**Error Location**: `render_register_mod` function in svd2rust
**Affected**: All 20 EFR32MG24 device variants
**Impact**: Complete blocker - could not generate PAC

---

## Root Cause Analysis

### Investigation Steps

1. **SVD File Analysis** ✅
   - Analyzed all 20 SVD files for special characters
   - Result: No problematic characters found ('{', '}', etc.)
   - SVD files are valid XML
   - Conclusion: Issue not in source SVD files

2. **Minimal Reproduction** ✅
   - Created minimal SVD with single GPIO peripheral
   - Result: Same error occurred
   - Conclusion: Issue is in svd2rust itself, not SVD content

3. **Version Testing** ✅
   - Tested svd2rust v0.30.3: ❌ Failed
   - Tested svd2rust v0.33.4: ❌ Failed
   - Both versions exhibited identical error

4. **GitHub Issue Search** ✅
   - Found: Issue #866 in rust-embedded/svd2rust
   - Title: "svd2rust fails with unsupported proc macro punctuation character"
   - Status: Closed with solution

### Root Cause

**Dependency Version Mismatch**: When installing svd2rust without the `--locked` flag, cargo resolves dependencies to their latest compatible versions. The latest `proc-macro2` crate (v1.0.103) introduced stricter validation that is incompatible with how svd2rust v0.30.3 and v0.33.4 generate code.

The issue manifests when svd2rust tries to create punctuation tokens with characters that proc-macro2 v1.0.103+ considers invalid for the `Punct` type.

---

## Solution

### Command

```bash
cargo install svd2rust --locked
```

### Why This Works

The `--locked` flag tells cargo to use the **exact dependency versions** specified in svd2rust's `Cargo.lock` file, rather than resolving to the latest compatible versions. This ensures:

1. Compatible `proc-macro2` version is installed
2. All dependency versions match those tested by svd2rust maintainers
3. No version resolution conflicts

### Version Installed

```
svd2rust v0.37.1 (with locked dependencies)
└── proc-macro2 v1.0.x (compatible version from Cargo.lock)
```

---

## Complete PAC Generation Process

### Prerequisites

```bash
# Install Rust target
rustup target add thumbv8m.main-none-eabihf

# Install tools with locked dependencies
cargo install svd2rust --locked
cargo install form
```

### Step-by-Step Process

#### 1. Project Structure

```
EFR32MG24/
├── efr32mg24-pac/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── memory.x
│   ├── svd/
│   │   └── EFR32MG24A020F1536GM48.svd
│   └── src/
└── .cargo/
    └── config.toml
```

#### 2. Generate PAC

```bash
cd efr32mg24-pac

# Generate from SVD
svd2rust -i svd/EFR32MG24A020F1536GM48.svd \
         --target cortex-m \
         -o src

# Result: src/lib.rs (22 MB, single file)
```

**Generation Output**:
- File: `src/lib.rs`
- Size: 22 MB
- Lines: 221,082
- Warnings: ~100 (missing register descriptions - cosmetic)
- Errors: None

#### 3. (Optional) Organize with `form`

```bash
cd src

# Split into modules
form -i lib.rs -o .

# Remove original
rm lib.rs

# Format
cargo fmt
```

**Note**: For initial testing, skip `form` and use the single lib.rs file. It compiles successfully as-is.

#### 4. Configure Cargo.toml

```toml
[package]
name = "efr32mg24-pac"
version = "0.1.0"
edition = "2021"  # Important: Use 2021, not 2024
license = "MIT OR Apache-2.0"
description = "Peripheral Access Crate for EFR32MG24 microcontrollers"

[dependencies]
cortex-m = "0.7.7"
vcell = "0.1.3"

[dependencies.cortex-m-rt]
version = "0.7.3"
optional = true

[features]
default = []
rt = ["cortex-m-rt/device"]
```

#### 5. Build PAC

```bash
# Check compilation
cargo check --target thumbv8m.main-none-eabihf

# Build release
cargo build --release --target thumbv8m.main-none-eabihf
```

**Build Results**:
- Compilation: ✅ Success
- Warnings: 1 (unexpected cfg for critical-section feature)
- Errors: 0
- Output: `libefr32mg24_pac.rlib` (112 MB)
- Build Time: ~5 minutes (release)

---

## SVD File Details

### Source

- **Repository**: https://github.com/silabs-EricB/cortex-debug-dp-efr32mg24
- **Location**: `data/*.svd`
- **Count**: 20 device variants
- **Total Size**: 172 MB (40 files - original + processed)

### Device Variants

```
EFR32MG24A010F1024IM40   -  1024KB Flash,  128KB RAM,  40-pin
EFR32MG24A010F1024IM48   -  1024KB Flash,  128KB RAM,  48-pin
EFR32MG24A010F1536GM48   -  1536KB Flash,  256KB RAM,  48-pin (GM)
EFR32MG24A020F1536GM48   -  1536KB Flash,  256KB RAM,  48-pin (default)
... (16 more variants)
```

### Primary Target

**EFR32MG24A020F1536GM48**:
- Flash: 1536 KB
- RAM: 256 KB
- Package: 48-pin QFN
- Features: Full peripheral set

### SVD Quality

**Status**: ✅ High quality, no manual fixes required

- Valid XML structure
- No special characters
- Complete peripheral descriptions
- Minor: Some registers lack descriptions (cosmetic warnings only)

---

## Edition 2021 vs 2024

### Issue with Edition 2024

When using Rust Edition 2024, the generated PAC produces 133 warnings about unsafe operations:

```
warning[E0133]: call to unsafe function is unsafe and requires unsafe block
```

**Cause**: Edition 2024 requires explicit `unsafe` blocks around unsafe operations, even within `unsafe fn`. The generated code from svd2rust v0.37.1 doesn't include these blocks.

### Solution

**Use Edition 2021**: The PAC compiles cleanly with edition = "2021" in Cargo.toml.

```toml
# efr32mg24-pac/Cargo.toml
[package]
edition = "2021"  # Not 2024

# Workspace Cargo.toml
[workspace.package]
edition = "2021"  # Apply to all crates
```

**Note**: Future svd2rust versions may support edition 2024. For now, 2021 is recommended.

---

## Verification

### PAC Statistics

```
File Size:    22 MB (source)
Lines of Code: 221,082
Compiled Size: 112 MB (.rlib)
Peripherals:   133 (66 secure + 66 non-secure + 1 DEVINFO)
Build Time:    ~5 minutes (release)
Warnings:      1 (cfg feature)
Errors:        0
```

### Sample Peripherals

**Core Peripherals** (Secure & Non-Secure):
- GPIO_S / GPIO_NS
- CMU_S / CMU_NS (Clock Management)
- EMU_S / EMU_NS (Energy Management)
- USART0_S / USART0_NS
- I2C0_S / I2C0_NS, I2C1_S / I2C1_NS
- TIMER0-4_S / TIMER0-4_NS
- IADC0_S / IADC0_NS (ADC)
- LDMA_S / LDMA_NS (DMA)

**Radio Peripherals** (Secure & Non-Secure):
- FRC_S / FRC_NS (Frame Controller)
- MODEM_S / MODEM_NS
- AGC_S / AGC_NS (Automatic Gain Control)
- SYNTH_S / SYNTH_NS (Synthesizer)
- RAC_S / RAC_NS (Radio Controller)

**Security**:
- SEMAILBOX_S_HOST (Secure Mailbox)
- RADIOAES_S / RADIOAES_NS
- SMU_S / SMU_NS (Security Management)

**Other**:
- MVP_S / MVP_NS (Math Vector Processor)
- DEVINFO (Device Information)

### Build Verification

```bash
# PAC compiles
cargo build -p efr32mg24-pac --target thumbv8m.main-none-eabihf
# ✅ Success

# HAL compiles (with PAC dependency)
cargo build -p efr32mg24-hal --target thumbv8m.main-none-eabihf
# ✅ Success

# Workspace compiles
cargo build --release --target thumbv8m.main-none-eabihf
# ✅ Success
```

---

## Common Issues & Solutions

### Issue 1: "unsupported proc macro punctuation character"

**Solution**: Install svd2rust with `--locked` flag
```bash
cargo install svd2rust --locked --force
```

### Issue 2: Edition 2024 unsafe warnings

**Solution**: Use edition = "2021" in Cargo.toml

### Issue 3: Cannot find SVD file

**Verify structure**:
```bash
efr32mg24-pac/
├── svd/
│   └── EFR32MG24A020F1536GM48.svd  # Must exist
└── src/
```

### Issue 4: form splits code but lib.rs missing

**Expected**: `form` creates module directories and you need to create lib.rs manually with module declarations, OR use the generated lib.rs directly without `form` for simpler structure.

**Recommendation**: Skip `form` initially - single lib.rs works fine.

---

## Performance Notes

### Build Times (Release, M1 Mac)

- PAC generation: ~30 seconds
- PAC compilation: ~5 minutes
- HAL compilation: ~2 seconds
- Full workspace: ~5 minutes

### Memory Usage

- Source: 22 MB (lib.rs)
- Compiled: 112 MB (.rlib)
- Debug symbols: Included in release (for easier debugging)

---

## Alternative Approaches (Not Needed)

The following approaches were **NOT required** but are documented for completeness:

### ❌ svdtools Patching

Not needed - SVD files are already valid. Would have been used if SVD files had errors.

### ❌ Manual SVD Editing

Not needed - No errors in source SVD files.

### ❌ form Organization

Optional - Single lib.rs works fine, though `form` provides better organization for large PACs.

---

## Future Improvements

### Short Term

1. Add examples to PAC (blinky, register access)
2. Document peripheral usage patterns
3. Create device.x variants for all 20 devices

### Medium Term

1. Automated build script for all variants
2. Feature flags for device selection
3. CI/CD for automated testing

### Long Term

1. Submit patches to svd2rust if edition 2024 support needed
2. Contribute EFR32MG24 support to probe-rs
3. Create comprehensive HAL

---

## References

### Issues

- **svd2rust #866**: "unsupported proc macro punctuation character"
  - https://github.com/rust-embedded/svd2rust/issues/866
  - Solution: `cargo install svd2rust --locked`

### Documentation

- svd2rust: https://docs.rs/svd2rust/
- proc-macro2: https://docs.rs/proc-macro2/
- Rust Edition Guide: https://doc.rust-lang.org/edition-guide/

### Similar Projects

- stm32-rs: https://github.com/stm32-rs/stm32-rs
- nrf-rs: https://github.com/nrf-rs/nrf-hal

---

## Summary

**Problem**: svd2rust dependency version mismatch
**Solution**: `cargo install svd2rust --locked`
**Result**: ✅ Fully functional 112 MB PAC with 221,082 lines
**Status**: Production-ready for HAL development

The EFR32MG24 PAC is now successfully generated and compiles without errors!

---

**Document Version**: 1.0
**Date**: December 3, 2025
**Author**: Marcelo Correa
**Status**: Solution Verified and Working
