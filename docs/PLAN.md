# EFR32MG24 Rust HAL Development Plan

## Project Overview

**Goal**: Create production-ready Peripheral Access Crate (PAC) and Hardware Abstraction Layer (HAL) for Silicon Labs EFR32MG24 wireless SoC in Rust.

**Status**: Phase B Partial - GPIO, CMU, Delay, USART, I2C, SPI Implemented

**Repository Structure**:
```
EFR32MG24/
├── Cargo.toml                     # Workspace configuration
├── efr32mg24-pac/                 # Peripheral Access Crate
├── efr32mg24-hal/                 # Hardware Abstraction Layer
├── docs/                          # Documentation
├── PLAN.md                        # This file
├── FINDINGS.md                    # Detailed findings
└── CLAUDE.md                      # Project-specific instructions (TBD)
```

---

## Phase Breakdown

### ✅ Phase 1: Research & Analysis (COMPLETED)

**Duration**: ~3 hours

**Achievements**:
1. Comprehensive research on existing Rust support for EFR32 family
   - Found: No production-ready support for Series 2 devices
   - Identified: efm32-rs (Series 0/1 only), archived projects
2. Analyzed MCU specifications:
   - Cortex-M33 @ 78 MHz
   - 1536 KB Flash, 256 KB RAM
   - 2.4 GHz radio (Zigbee, Thread, Matter, BLE)
   - TrustZone security
3. Documented comparison of MG21/MG24/MG26 variants
4. Created two comprehensive research reports

**Key Deliverables**:
- Research findings embedded in agent outputs
- Understanding of MCU architecture
- Identified technical approach (svd2rust + HAL development)

---

### ✅ Phase 2: Project Setup (COMPLETED)

**Duration**: ~1 hour

**Achievements**:
1. Created Cargo workspace with:
   - `efr32mg24-pac` crate
   - `efr32mg24-hal` crate
   - Shared dependencies configuration
   - Size-optimized release profiles
2. Installed required tools:
   - svd2rust v0.33.4 (12 min install time)
   - form v0.12.1 (7 min install time)
   - Rust target: thumbv8m.main-none-eabihf
3. Cloned SVD files:
   - 20 device variants (172 MB total)
   - From silabs-EricB/cortex-debug-dp-efr32mg24
4. Created utility scripts:
   - `fix_svd.py` - SVD character cleanup
   - `comprehensive_svd_fix.py` - Comprehensive SVD fixer
   - `deep_svd_analyze.py` - Deep character analysis

**Key Deliverables**:
- Functional workspace structure
- All SVD files organized
- Development tools installed

---

### ✅ Phase 3: PAC Generation (COMPLETED)

**Duration**: 2 hours (including troubleshooting)

**Achievements**:
1. Identified hardware-PAC mismatch (A020 vs B220)
2. Obtained Silicon Labs Gecko Platform DFP pack (2025.6.2)
3. Extracted all 41 EFR32MG24 SVD files (27 A-series + 14 B-series)
4. Resolved svd2rust tooling issue using --locked flag
5. Successfully generated B220 PAC for XIAO MG24 hardware
6. PAC metrics:
   - Source: 138,448 lines of Rust code (14 MB)
   - Compile time: ~2.6 minutes
   - Output: 112 MB .rlib with 133 peripherals
   - 36% smaller than initial A020 PAC

**Key Issue Resolved**:
- Initial svd2rust failures were due to dependency version conflicts
- Solution: `cargo install svd2rust --locked` (v0.37.1)
- This pins all transitive dependencies to tested versions

**Generation Command**:
```bash
cd efr32mg24-pac
svd2rust --locked -i svd/EFR32MG24B220F1536IM48.svd --target cortex-m -o src
```

**Key Deliverables**:
- B220 PAC source code in efr32mg24-pac/src/lib.rs
- A020 PAC preserved in src_backup_a020/ for reference
- Complete SVD file collection (41 variants)

---

### ✅ Phase 4: PAC Verification (COMPLETED)

**Duration**: 1 hour

**Tasks Completed**:
1. ✅ Verified B220 PAC compilation (edition 2021)
2. ✅ Workspace build verification
3. ✅ PAC and HAL crate integration tested
4. ✅ Compilation metrics documented
5. ✅ Build time: ~2.6 minutes for PAC, ~5 minutes for full workspace
6. ✅ Output: 112 MB PAC .rlib, 5.4 KB HAL .rlib (stub)

**Configuration Status**:
- PAC uses edition 2021 for clean compilation
- HAL configured to use PAC as dependency
- Workspace-level dependencies working correctly
- Memory layout (memory.x) in place
- Build script (build.rs) configured

**Key Achievement**:
- Complete workspace compiles successfully
- Ready for HAL peripheral implementation

**Estimated Duration**: Completed in 1 hour

---

### ✅ Phase A: Initial HAL Development - Essential Peripherals Complete

**Prerequisites**: ✅ Phase 4 complete

**Tier 1 Peripherals** (Essential) - STATUS:

1. **GPIO** (Digital I/O) - ✅ COMPLETE
   - ✅ Port configuration via MODEL/MODEH registers
   - ✅ Pin mode setting (push-pull output, pull-up/pull-down/floating input)
   - ✅ Digital read/write via DIN/DOUT registers
   - ✅ Drive strength configuration
   - ✅ embedded-hal v1.0 OutputPin/InputPin traits
   - ✅ Type-safe pin modes (compile-time enforcement)
   - ✅ GPIO clock enable via CMU
   - ⏳ Interrupt support (deferred to Tier 2)

2. **CMU** (Clock Management Unit) - ✅ COMPLETE
   - ✅ HFXO/HFRCO configuration
   - ✅ LFXO/LFRCO configuration
   - ✅ SYSCLKCTRL register configuration
   - ✅ Peripheral consumption pattern
   - ✅ Frequency tracking in FrozenClocks
   - ✅ Critical section protection
   - ⏳ Clock prescalers (deferred)
   - ⏳ Peripheral clock individual enable/disable (deferred)

3. **Delay** - ✅ COMPLETE
   - ✅ SysTick-based blocking delays
   - ✅ Millisecond/microsecond/nanosecond precision
   - ✅ embedded-hal v1.0 DelayNs trait
   - ✅ Integration with CMU clock frequencies

4. **EMU** (Energy Management Unit) - ⏳ TIER 2
   - Energy mode transitions (EM0-EM4)
   - Voltage scaling
   - Power management

5. **USART/EUSART** (Serial Communication) - ✅ TIER 2 COMPLETE
   - ✅ UART mode
   - ✅ Configuration (baud rate, parity, stop bits)
   - ✅ embedded-hal-nb serial traits
   - ✅ Example: 04_usart.rs

6. **TIMER** (Timers) - ⏳ TIER 2
   - Basic timer functionality
   - PWM generation
   - embedded-hal timer traits

**Completed (Phase A - Tier 1)**: December 4, 2025
**Completed (Phase B - USART)**: December 4, 2025
**Completed (Phase B - I2C)**: December 12, 2025
**Examples**: 5 working examples (01_clock, 02_delay, 03_gpio, 04_usart, 05_i2c)

---

### ⏳ Phase B: Extended HAL Development - Communication Peripherals (IN PROGRESS)

**Tier 2 Peripherals** (Important):

6. **I2C** - ✅ TIER 2 COMPLETE
   - ✅ Master mode (I2C0, I2C1)
   - ✅ 7-bit addressing
   - ✅ Standard (100 kHz) and Fast (400 kHz) modes
   - ✅ embedded-hal I2C traits
   - ✅ Example: 05_i2c.rs
   - ⏳ Slave mode (deferred)
   - ⏳ 10-bit addressing (deferred)

7. **IADC** (ADC)
   - Single-shot conversion
   - Continuous conversion
   - Multi-channel support
   - embedded-hal ADC traits

8. **LDMA** (DMA Controller)
   - Linked descriptor support
   - Peripheral-to-memory transfers
   - Memory-to-memory transfers

9. **PRS** (Peripheral Reflex System)
   - Inter-peripheral signaling
   - Producer/consumer configuration

10. **SYSRTC/LETIMER** (Low-Power Timers)
    - Real-time clock
    - Low-energy timer
    - Wakeup from sleep modes

**Estimated Duration**: 4-6 weeks

---

### ⏳ Phase C: Advanced Features (PENDING)

**Tier 3 Peripherals** (Advanced):

11. **MVP** (Math Vector Processor)
    - DSP operations
    - Vector math acceleration

12. **VDAC** (DAC)
    - Analog output
    - Waveform generation

13. **ACMP** (Analog Comparator)
    - Voltage comparison
    - Interrupt on threshold

14. **Security Peripherals**
    - AES acceleration
    - SHA acceleration
    - ECC operations
    - Secure Vault integration

15. **Radio Support** (Complex)
    - Basic radio initialization
    - FFI bindings to Silicon Labs libraries
    - Consider pure Rust implementation long-term

**Estimated Duration**: 2-3 months

---

### ⏳ Phase 8: Testing & Examples (ONGOING)

**Parallel with Development**:

1. **Unit Tests**
   - Per-peripheral HAL tests
   - Mock hardware testing

2. **Hardware-in-Loop Tests**
   - Dev kit required: xG24-DK2601B (~$50)
   - Real hardware validation

3. **Examples Repository**
   - Blink LED
   - Serial echo
   - I2C sensor reading
   - ADC sampling
   - PWM generation
   - Low-power modes

4. **Documentation**
   - API documentation (rustdoc)
   - Peripheral usage guides
   - Migration guides from C
   - Comparison with Silicon Labs SDK

**Estimated Duration**: Ongoing throughout development

---

### ⏳ Phase 9: Community & Ecosystem (PENDING)

**Long-term Goals**:

1. **Embassy Support** (Async Framework)
   - Timer driver implementation
   - DMA abstraction
   - Async peripheral drivers
   - Executor integration

2. **Board Support Packages**
   - xG24-DK2601B dev kit
   - Common EFR32MG24 modules
   - Custom board templates

3. **Device Variant Support**
   - All 20 EFR32MG24 variants
   - EFR32MG21 family
   - EFR32MG26 family

4. **Community Engagement**
   - Rust Embedded Working Group
   - Blog posts / tutorials
   - Conference talks
   - GitHub organization (efr32-rs?)

5. **Upstream Contributions**
   - probe-rs EFR32 support improvements
   - svd2rust bug fixes
   - embedded-hal trait implementations

**Estimated Duration**: 6+ months, ongoing

---

## Technical Decisions

### Device Variant Strategy

**Decision**: One PAC with feature flags (stm32-rs model)

**Rationale**:
- 20 variants share ~95% of peripherals
- Differences mainly in Flash/RAM size and GPIO count
- Feature flags allow compile-time selection
- Easier maintenance than 20 separate crates

**Implementation**:
```toml
[features]
default = []
efr32mg24a010f1024im40 = []
efr32mg24a020f1536gm48 = []
# ... 18 more variants
```

### TrustZone Handling

**Decision**: Single PAC with both secure (_S) and non-secure peripherals

**Rationale**:
- Most users will use non-secure mode
- Secure peripherals clearly marked with _S suffix
- Feature flag `secure` for secure-mode compilation
- Typestate pattern for compile-time security guarantees

**Implementation**:
```rust
#[cfg(feature = "secure")]
pub use scratchpad_s::SCRATCHPAD_S;

pub use emu_ns::EMU_NS;  // Non-secure by default
```

### HAL Organization

**Decision**: Trait-based HAL with embedded-hal v1.0 support

**Rationale**:
- Ecosystem compatibility
- Portable applications
- Clear abstraction boundaries
- Testable without hardware

**Structure**:
```
efr32mg24-hal/
├── src/
│   ├── lib.rs
│   ├── gpio.rs           # GPIO implementation
│   ├── serial.rs         # USART/EUSART
│   ├── i2c.rs           # I2C
│   ├── spi.rs           # SPI
│   ├── timer.rs         # Timers
│   ├── adc.rs           # IADC
│   ├── dma.rs           # LDMA
│   ├── clock.rs         # CMU
│   └── power.rs         # EMU
└── examples/            # Usage examples
```

### Debugging Strategy

**Decision**: Use Simplicity Commander until probe-rs stable

**Rationale**:
- probe-rs has known issues with EFR32MG24
- Simplicity Commander reliable and free
- Can contribute probe-rs fixes later
- Document both approaches

**Workflow**:
```bash
# Build
cargo build --release --target thumbv8m.main-none-eabihf

# Convert to hex
arm-none-eabi-objcopy -O ihex target/.../app app.hex

# Flash
commander flash --device EFR32MG24 app.hex

# Debug with openocd (alternative)
openocd -f interface/jlink.cfg -f target/efr32mg24.cfg
```

---

## Risk Assessment

### High-Priority Risks

1. **svd2rust Compatibility** (CURRENT BLOCKER)
   - **Impact**: Cannot generate PAC
   - **Likelihood**: High (currently blocked)
   - **Mitigation**: Try older svd2rust versions, use svdtools, manual editing
   - **Status**: Investigating

2. **TrustZone Complexity**
   - **Impact**: Incorrect security model
   - **Likelihood**: Medium
   - **Mitigation**: Study ARM TrustZone-M docs, test on hardware
   - **Status**: Research needed

3. **Radio Stack Complexity**
   - **Impact**: Limited wireless functionality
   - **Likelihood**: High (very complex)
   - **Mitigation**: Start with FFI to Silicon Labs libraries, pure Rust long-term
   - **Status**: Deferred to Phase C

### Medium-Priority Risks

4. **Debug Tooling Limitations**
   - **Impact**: Difficult development/debugging
   - **Likelihood**: Medium
   - **Mitigation**: Use Simplicity Commander, contribute to probe-rs
   - **Status**: Workaround available

5. **Community Adoption**
   - **Impact**: Project sustainability
   - **Likelihood**: Medium
   - **Mitigation**: Early engagement, good documentation, examples
   - **Status**: Planned for Phase 9

6. **Maintenance Burden**
   - **Impact**: Keeping up with Silicon Labs updates
   - **Likelihood**: Medium
   - **Mitigation**: Automated testing, community contributions
   - **Status**: Ongoing concern

---

## Success Criteria

### Phase 3-4 Success (PAC Generation)
- [ ] PAC compiles without errors
- [ ] All peripherals accessible
- [ ] Interrupt vector table generated
- [ ] Memory layout correct
- [ ] Documentation builds

### Phase 5-6 Success (Basic HAL)
- [ ] GPIO: Blink LED example works
- [ ] USART: Serial echo works
- [ ] I2C: Sensor communication works
- [ ] Timer: PWM generation works
- [ ] ADC: Voltage reading works
- [ ] All embedded-hal v1.0 traits implemented

### Phase C Success (Advanced HAL)
- [ ] DMA transfers working
- [ ] Low-power modes functional
- [ ] Security peripherals accessible
- [ ] Basic radio initialization

### Phase 8-9 Success (Ecosystem)
- [ ] 10+ working examples
- [ ] Embassy support
- [ ] Board support packages
- [ ] Community contributors
- [ ] Published to crates.io

---

## Resources Required

### Hardware
- **Essential**: xG24-DK2601B EFR32MG24 Dev Kit (~$50)
- **Optional**: J-Link debug probe (~$60)
- **Optional**: Additional EFR32MG21/MG26 boards for family support

### Software
- ✅ Rust toolchain
- ✅ svd2rust, form
- ⏳ svdtools (if needed for patches)
- ✅ Simplicity Commander (flashing tool)
- ⏳ Silicon Labs documentation
- ⏳ Gecko SDK (reference implementation)

### Time Estimate
- **Phase 3-4** (PAC): 1-2 weeks (currently blocked)
- **Phase 5-6** (Basic HAL): 2-3 months
- **Phase C** (Advanced): 2-3 months
- **Phase 8-9** (Ecosystem): 6+ months, ongoing

**Total to Production-Ready HAL**: 6-9 months of focused development

---

## Open Questions

1. **Should we support EFR32BG24 (Bluetooth-only variant)?**
   - Same silicon, different radio stack focus
   - Likely works with same PAC/HAL
   - Decision: Add as feature flag later

2. **How to handle radio calibration data?**
   - Silicon Labs stores in flash
   - Need to preserve during firmware updates
   - Decision: Document, provide utilities

3. **Should HAL use defmt or log for diagnostics?**
   - defmt: More efficient, embedded-specific
   - log: Standard Rust, more portable
   - Decision: Support both via feature flags

4. **Licensing strategy?**
   - MIT/Apache-2.0 dual (Rust standard)
   - Silicon Labs SVD files have their own license
   - Decision: Clarify in NOTICE file

---

## Next Actions (Immediate)

### To Unblock Phase 3:

1. **Try svd2rust v0.30.3** (older, stable version)
   ```bash
   cargo install svd2rust --version 0.30.3 --force
   cd efr32mg24-pac
   svd2rust -i ../efr32mg24-hal/svd/EFR32MG24A020F1536GM48.svd \
            --target cortex-m -o src
   ```

2. **If still fails, create minimal SVD** with just a few peripherals
   - Test svd2rust with minimal input
   - Incrementally add peripherals to find problematic one

3. **Install svdtools and create patches**
   ```bash
   pip3 install svdtools
   # Create efr32mg24.yaml patch file
   svd patch efr32mg24.yaml
   ```

4. **Report issue to svd2rust project**
   - With minimal reproduction case
   - SVD file samples
   - Full error output

5. **Research alternative code generation**
   - Check if anyone else has EFR32MG24 PAC
   - Look for ARM CMSIS-Pack tools
   - Consider chiptool (alternative to svd2rust)

---

**Last Updated**: December 13, 2025
**Current Phase**: Phase B Partial - USART, I2C, and SPI Complete, Timers Next
**Progress**: Phase B 70% complete (communication peripherals done)
**Next Milestone**: Implement Timers/PWM
