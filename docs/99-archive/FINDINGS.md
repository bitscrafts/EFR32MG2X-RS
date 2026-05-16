# EFR32MG24 Rust PAC/HAL Development - Findings and Progress

**Date Started**: December 2, 2025
**Project Goal**: Create Peripheral Access Crate (PAC) and Hardware Abstraction Layer (HAL) for Silicon Labs EFR32MG24

---

## Project Structure

```
EFR32MG24/
├── Cargo.toml                    # Workspace configuration
├── efr32mg24-pac/                # Peripheral Access Crate
│   ├── Cargo.toml
│   └── src/
├── efr32mg24-hal/                # Hardware Abstraction Layer
│   ├── Cargo.toml
│   ├── src/
│   └── svd/                      # SVD files from Silicon Labs
├── docs/                         # Documentation
├── fix_svd.py                    # SVD cleanup script
└── FINDINGS.md                   # This file
```

---

## Research Findings

### 1. Existing Rust Support

**Finding**: No production-ready Rust support exists for EFR32 Series 2 devices (MG21, MG24, MG26).

**Details**:
- **efm32-rs**: Only supports Series 0/1 (last update March 2023)
- **em32-rs**: Largely inactive since 2019
- **chrysn/efm32gg-hal**: Very immature, minimal GPIO only
- **Other projects**: Archived or abandoned

**Implication**: We're pioneering first-class Rust support for this MCU family.

### 2. SVD File Sources

**Primary Source**: GitHub repository `silabs-EricB/cortex-debug-dp-efr32mg24`
- Contains 20 SVD files for different EFR32MG24 variants
- File sizes: 8.3 MB (1024KB flash) to 9.0 MB (1536KB flash variants)
- All files successfully cloned to `efr32mg24-hal/svd/`

**Alternative Sources**:
1. ARM Keil Device Family Pack (GeckoPlatform_EFR32MG24_DFP)
2. SEGGER package system
3. Silicon Labs community website

**Selected Device**: EFR32MG24A020F1536GM48
- 1536 KB Flash
- 256 KB RAM
- 48-pin package
- Full-featured variant

### 3. MCU Specifications

**Core**:
- ARM Cortex-M33 with DSP, FPU, and TrustZone
- 78 MHz maximum frequency
- ARMv8-M Mainline architecture

**Memory Map**:
- Flash: 0x00000000 - 0x08180000
- RAM: 0x20000000 - 0x2003FFFF
- Secure/Non-secure peripheral separation

**Key Peripherals**:
- 1× USART, 2× EUSART
- 2× I2C (up to 1 Mbps)
- Multiple timers (TIMER, LETIMER, SYSRTC)
- 1× IADC (12/16-bit ADC)
- 8-channel LDMA (DMA controller)
- 16-channel PRS (Peripheral Reflex System)
- Hardware crypto accelerators (AES, SHA, ECC)

**Radio**:
- 2.4 GHz multiprotocol
- Zigbee 3.0, Thread, Matter, Bluetooth 5.3 LE
- +20 dBm TX power

### 4. Development Tools

**Installed**:
- `svd2rust` v0.33.4 - SVD to Rust code generation
- `form` v0.12.1 - Code formatting and organization
- Rust target: `thumbv8m.main-none-eabihf` (Cortex-M33 with FPU)

**Required but Not Yet Installed**:
- `svdtools` - For SVD patching (needed to fix vendor SVD issues)
- `cargo-binutils` - For binary inspection

---

## Technical Challenges Encountered

### Challenge 1: SVD File Parsing Error

**Issue**: svd2rust fails with error:
```
thread 'main' panicked at proc-macro2-1.0.103/src/lib.rs:824:13:
unsupported proc macro punctuation character '{'
```

**Root Cause**: Silicon Labs SVD files contain special characters (likely `{`, `}`) in register names, descriptions, or other fields that svd2rust/proc-macro2 cannot handle.

**Impact**: Cannot generate PAC directly from vendor SVD files.

**Solutions Attempted**:
1. Tried multiple device variants - same error
2. Created Python script `fix_svd.py` to remove curly braces - no braces found in obvious locations
3. Created `find_bad_chars.py` to locate problematic characters

**Next Steps**:
1. Install `svdtools` (industry-standard SVD patching tool)
2. Create YAML patches following stm32-rs model
3. Apply patches before running svd2rust

### Challenge 2: Project Organization

**Issue**: Initial confusion about folder structure - where to place SVD files and how to organize workspace.

**Resolution**:
- Created workspace with two members: `efr32mg24-pac` and `efr32mg24-hal`
- SVD files placed in `efr32mg24-hal/svd/` (20 files, 172 MB total)
- Workspace Cargo.toml created with shared dependencies

**Lesson Learned**: Follow established patterns (stm32-rs, nrf-rs) for multi-crate embedded projects.

---

## Development Process

### Phase 1: Research (Completed)

**Tasks Completed**:
- Comprehensive research on existing Rust support
- Analysis of EFR32MG24 specifications and documentation
- Comparison of MG21, MG24, MG26 variants
- Tool installation (svd2rust, form, Rust targets)

**Key Outputs**:
- Two comprehensive research reports (embedded in todo outputs)
- Clear understanding of MCU capabilities
- Identified gaps in ecosystem

### Phase 2: Project Setup (Completed)

**Tasks Completed**:
- Created workspace structure
- Initialized PAC and HAL crates
- Cloned and organized SVD files (20 variants)
- Configured workspace Cargo.toml with:
  - Profile optimizations (size-optimized release build)
  - Shared dependencies (cortex-m, embedded-hal)
  - Proper workspace resolver (v2)

**Project Configuration**:
```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
debug = true           # Keep debug info for easier debugging
```

### Phase 3: PAC Generation (In Progress - Blocked)

**Current Status**: Blocked by SVD parsing issues

**Attempted**:
- Direct svd2rust generation → Failed with punctuation error
- Tried multiple SVD file variants → All fail with same error
- Created diagnostic scripts → Identified need for SVD patching

**Remaining Steps**:
1. Install svdtools
2. Analyze SVD file structure in detail
3. Create YAML patches to fix:
   - Invalid register names with special characters
   - Description field formatting issues
   - Any TrustZone-related complexities
4. Apply patches and regenerate
5. Use `form` to organize generated code
6. Configure Cargo.toml for PAC crate

---

## Lessons Learned

### 1. Vendor SVD Quality

**Observation**: Vendor-provided SVD files often have issues that prevent direct use with svd2rust.

**Evidence**:
- Research showed ~85% success rate for svd2rust with vendor SVDs
- Silicon Labs SVDs contain characters that break proc-macro2
- STM32-RS project maintains extensive patch repository for this reason

**Best Practice**: Always plan for SVD patching phase in any new chip family port.

### 2. Tool Installation Time

**Observation**: Rust tool compilation can take significant time.

**Measurements**:
- svd2rust installation: 12 minutes 1 second
- form installation: 6 minutes 50 seconds

**Impact**: Factor in tool build time when planning development sessions.

### 3. Documentation Importance

**Observation**: Having vendor documentation is critical even when working from SVD files.

**Rationale**:
- SVD files may have errors or ambiguities
- Understanding peripheral interactions requires reference manual
- Errata sheets reveal silicon bugs that affect software design

**Action**: Should download and organize official Silicon Labs documentation.

---

## Next Actions

### Immediate (Next Session)

1. **Install svdtools**:
   ```bash
   pip3 install svdtools
   ```

2. **Analyze SVD structure**:
   - Parse XML to understand peripheral organization
   - Identify exact location of problematic characters
   - Document TrustZone peripheral naming (_S suffix)

3. **Create initial patch file** (YAML):
   ```yaml
   # efr32mg24.yaml
   _svd: efr32mg24-hal/svd/EFR32MG24A010F1536IM48.svd

   # Patches will go here
   ```

4. **Generate patched SVD**:
   ```bash
   svd patch efr32mg24.yaml
   ```

5. **Retry PAC generation**:
   ```bash
   svd2rust -i EFR32MG24A010F1536IM48.svd.patched --target cortex-m -o src
   form -i src/lib.rs -o src/
   rm src/lib.rs
   cargo fmt
   ```

### Short Term (This Week)

1. Get PAC compiling
2. Configure PAC Cargo.toml with:
   - cortex-m dependency
   - cortex-m-rt feature for interrupt vectors
   - vcell for register access
3. Test PAC on simple example (LED blink)
4. Document PAC generation process

### Medium Term (Next 2 Weeks)

1. Create patches for all 20 device variants
2. Automate patch generation/application
3. Begin HAL development:
   - GPIO implementation
   - Clock management (CMU)
   - USART driver
4. Create examples repository

### Long Term (1-3 Months)

1. Comprehensive HAL for major peripherals
2. embedded-hal v1.0 trait implementations
3. Documentation and tutorials
4. Community engagement (Rust Embedded WG)
5. Consider Embassy async support

---

## Open Questions

1. **TrustZone Handling**: How should we expose secure vs non-secure peripherals?
   - Option A: Separate PAC crates (efr32mg24-pac-s, efr32mg24-pac-ns)
   - Option B: Feature flags
   - Option C: Single PAC with typestate pattern

2. **Device Variant Strategy**: How to handle 20 different device variants?
   - Option A: One PAC per variant (20 crates)
   - Option B: One PAC with feature flags (stm32-rs model)
   - Option C: One PAC for common peripherals, separate for device-specific

3. **Radio Support**: How deep should HAL go into radio functionality?
   - Silicon Labs provides extensive C libraries
   - Radio stack is complex (Zigbee, Thread, Matter, BLE)
   - Consider FFI bindings vs pure Rust implementation

4. **Debug Probe Support**: probe-rs has issues with EFR32MG24
   - Use Silicon Labs Commander for now?
   - Contribute fixes to probe-rs?
   - Document alternative debugging methods?

---

## Resources Created

### Scripts

1. **fix_svd.py**: Attempts to fix common SVD issues
   - Removes curly braces from descriptions
   - Cleans up register names
   - (Needs enhancement for current issue)

2. **find_bad_chars.py**: Diagnostic tool to find problematic characters
   - Scans for `{`, `}`, backticks
   - Reports line numbers and context

### Configuration Files

1. **Workspace Cargo.toml**: Root workspace configuration
2. **efr32mg24-pac/Cargo.toml**: PAC crate (needs dependencies)
3. **efr32mg24-hal/Cargo.toml**: HAL crate (placeholder)

---

## References

### Documentation URLs

- Silicon Labs Community: https://community.silabs.com/
- ARM CMSIS-SVD Spec: https://www.keil.com/pack/doc/CMSIS/SVD/html/index.html
- svd2rust Documentation: https://docs.rs/svd2rust/
- Embedded Rust Book: https://docs.rust-embedded.org/book/

### Similar Projects (for Reference)

- stm32-rs: https://github.com/stm32-rs/stm32-rs
- nrf-rs: https://github.com/nrf-rs/nrf-hal
- rp-rs: https://github.com/rp-rs/rp-hal

---

## Statistics

- **Total SVD Files**: 20 variants (172 MB)
- **Research Documents Generated**: 2 comprehensive reports
- **Tools Installed**: 3 (svd2rust, form, Rust target)
- **Installation Time**: ~19 minutes
- **Lines of Configuration**: ~40 lines (Cargo.toml)
- **Current Blocker**: SVD parsing issue
- **Time to First PAC**: TBD (blocked)

---

**Last Updated**: December 2, 2025 23:56 UTC
**Status**: Phase 3 - PAC Generation (Blocked - SVD Issues)
**Next Milestone**: Successfully generate and compile PAC crate
