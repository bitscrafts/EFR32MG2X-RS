# EFR32MG24 Rust HAL Development Plan

## Project Overview

**Goal**: Create production-ready Peripheral Access Crate (PAC), Hardware Abstraction Layer (HAL), and radio/Bluetooth stack for Silicon Labs EFR32MG24 wireless SoC in Rust.

**Status**: Phase D2 verified on hardware -- a phone advertises, connects, reads and writes, and the BME680 answers as
the standard Environmental Sensing service. Open inside D2: **D2f pairing/bonding** (the PSA layer is written and
verified on the board; the security manager fails before the key exchange, so it stays out of the product build) and
**D2g BME680 gas resistance**. Phase D3 (Zigbee 3.0) and Phase E (Security) are next. The state of the tree, counted,
and the open items are in [`STATUS.md`](STATUS.md) and [`BACKLOG.md`](BACKLOG.md); this file is the plan and the
decisions behind it.

**Status 2026-11-26**: the HAL has 29 public modules and 28 examples, and every example of every crate compiles under
`scripts/check-examples-build.sh`; `scripts/gate.sh` reports `VERDICT: PASS`. What each peripheral does, and what has a
record read back off the part, is measured rather than asserted: `LETIMER0`, the IADC's PRS trigger and a timer feeding
the converter through the PRS were all closed on 2026-11-26, each with a document in `docs/04-reference/`. The text
below is the plan as it was written in December 2025 and January 2026; where a phase marker disagrees with the table in
"Where the plan stands", the table is the current one.

**Repository Structure**:
```
EFR32MG24/
├── Cargo.toml                     # Workspace configuration (32 members: 10 crates, 22 libraries)
├── crates/efr32mg2x-pac/                 # Peripheral Access Crate (B220, 2,633 files, 103 instances)
├── crates/efr32mg2x-hal/                 # Hardware Abstraction Layer (29 public modules, 28 examples)
├── crates/efr32mg24-rail-sys/            # RAIL radio FFI bindings
├── crates/efr32mg24-bluetooth-sys/       # Bluetooth 5.3 LE (BGAPI FFI + safe API)
├── crates/xiao-mg24-bsp/                 # Board Support Package (XIAO MG24 Sense)
├── docs/                          # Documentation (SSOT-organized)
├── PLAN.md                        # This file
├── HANDOFF.md                     # Project handoff
└── CLAUDE.md                      # Project-specific instructions
```

---

## Where the plan stands (2026-11-26)

| phase | state | evidence |
|---|---|---|
| P -- preparation | complete | PAC generated from the B220 SVD: 2,633 files, 103 peripheral instances in 46 families |
| A -- essential | complete | `gpio`, `clock`, `delay`; the LED blink measured at 1.02 Hz |
| B -- communication | complete | `usart`, `i2c`, `spi`, `timer`, `pwm`; USART loopback with a control, SPI with a jumper and a control, I2C on the IMU |
| C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |
| D0-D1 -- radio foundation | complete | RAIL FFI bindings, board crate |
| D2 -- Bluetooth 5.3 LE | complete to D2e, on hardware | advertising, scanning, GATT read/write/notify, the BME680 on a phone |
| D2f -- pairing and bonding | open | PSA layer verified; the security manager fails before the key exchange |
| D2g -- BME680 gas resistance | open | heater, `res_heat` and `par_g*` missing |
| D3 -- Zigbee 3.0 | not started | -- |
| D4 -- Thread/Matter | not started | -- |
| E -- security | not started | `radioaes` is in the PAC and no driver opens it |

The peripheral inventory behind that table -- 29 public modules, and `pcnt0`, `burtc`, `acmp`, `vdac`, `keyscan`,
`radioaes`, `dcdc`, `dpll0`, `ulfrco`, `fsrco` and `hfrcoem23` still absent -- is
[`COVERAGE-BOUNDARY-2026-11-25.md`](../04-reference/COVERAGE-BOUNDARY-2026-11-25.md).

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
   - `efr32mg2x-pac` crate
   - `efr32mg2x-hal` crate
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
cd efr32mg2x-pac
svd2rust --locked -i svd/EFR32MG24B220F1536IM48.svd --target cortex-m -o src
```

**Key Deliverables**:
- B220 PAC source code in crates/efr32mg2x-pac/src/lib.rs
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

### ✅ Phase B: Extended HAL Development - Communication Peripherals (COMPLETE)

Delivered: `usart`, `i2c`, `spi`, `timer`, plus the ones this section numbered without status in December --
`adc` (`hal_adc_read`, and the PRS trigger of 2026-11-26), `dma` (stages 1 and 2), `rtc`/`sysrtc` and `wdog`. `prs` is
delivered as one channel with named producers. Still absent, and named as such rather than marked done: `pcnt0`,
`burtc`, `acmp`, `vdac`, `keyscan`, `radioaes`.

**Tier 2 Peripherals** (Important):

6. **I2C** - ✅ TIER 2 COMPLETE
   - ✅ Master mode (I2C0, I2C1)
   - ✅ 7-bit addressing
   - ✅ Standard (100 kHz) and Fast (400 kHz) modes
   - ✅ embedded-hal I2C traits
   - ✅ Example: `hal_i2c_hardware_imu`, `hal_i2c_recovery`, `hal_i2c_async`
   - ⏳ Slave mode (deferred)
   - ⏳ 10-bit addressing (deferred)

7. **IADC** (ADC) - ✅ COMPLETE for single-shot
   - ✅ Single-shot conversion, 12-bit
   - ✅ Started by the CPU, by a PRS edge, or by a timer's overflow through the PRS
   - ⏳ Continuous conversion and multi-channel scan (deferred; the microphone path uses LDMA instead)
   - ✅ Examples: `hal_adc_read`, `hal_adc_prs_trigger`, `hal_adc_prs_timer`

8. **LDMA** (DMA Controller) - ✅ STAGES 1-2
   - ✅ Memory-to-memory transfers
   - ✅ Peripheral-to-memory transfers with a circular buffer
   - ⏳ Linked descriptors and interrupt-driven async (stages 3-4, deferred)
   - ✅ Example: `hal_dma_stop`

9. **PRS** (Peripheral Reflex System) - ✅ ONE CHANNEL, MEASURED
   - ✅ `Prs::connect` writes `ASYNC_CHn_CTRL` whole; `Prs::channel_ctrl` reads it back
   - ✅ Producers named: `Off`, `Timer0Overflow`, `Timer0Compare0`, `Timer1Overflow`, `GpioPin`
   - ⏳ `GpioPin` encoded and never driven (open item); synchronous channels and other consumers untouched
   - ✅ Example: `hal_adc_prs_timer`

10. **SYSRTC/LETIMER** (Low-Power Timers) - ✅ COMPLETE
    - ✅ SYSRTC0: 32.768 kHz counter, compare, 64-bit extended
    - ✅ LETIMER0: 24-bit counter, `TOP` with `CNTTOPEN`, two compares, runs in EM2
    - ✅ Examples: `hal_sysrtc_wrap`, `hal_sysrtc_wide`, `hal_letimer_compare`

**Estimated Duration**: 4-6 weeks

---

### ◐ Phase C: Advanced Features (PARTLY DELIVERED)

11. **MVP** (Math Vector Processor) - ✅ MEASURED
    - `hal_mvp_bench`: 256 additions in 14.66 µs against 34.33 µs on the CPU, `ratio_x100=234`

12. **VDAC** (DAC) - ⏳ NOT STARTED
    - Analog output, waveform generation

13. **ACMP** (Analog Comparator) - ⏳ NOT STARTED
    - Voltage comparison, interrupt on threshold

14. **Security Peripherals** - ⏳ NOT STARTED in the HAL
    - `radioaes` is in the PAC and no driver opens it; the PSA layer used by Bluetooth lives in the radio crate
    - AES acceleration, SHA acceleration, ECC, Secure Vault integration

15. **Radio Support** - ✅ DELIVERED, AND IT RUNS
    - `efr32mg24-rail-sys` (FFI) and `efr32mg24-bluetooth-sys` (BGAPI plus a safe blocking API)
    - The stack advertises, connects, and serves GATT; the BME680 is published as the standard Environmental Sensing
      service and a phone reads it
    - Pure Rust, no vendor externs: not started, ordered after Bluetooth is closed

**Estimated Duration**: 2-3 months

---

### ◐ Phase 8: Testing & Examples (ONGOING, AND IT IS THE POINT)

**Parallel with Development**:

1. **Unit Tests** - ◐ PARTLY
   - Host tests live in `libraries/` (`record-core`, `bt-heap`, `aes-modes`, `motion-detection`) and run in the gate
   - Per-peripheral HAL tests cannot be host tests: a driver that touches registers has to be measured on the part

2. **Hardware-in-Loop Tests** - ✅ UNDERWAY ON THE REAL BOARD
   - The XIAO MG24 Sense is on the bench at `lnx`, flashed with `scripts/flash.sh --confirm` and read back with
     `scripts/read-mailbox.sh`; no dev kit was needed
   - 19 of the 28 HAL examples carry a `VERIFIED ON HARDWARE (date)` marker, and every claim in this plan that has a
     number behind it came from that loop

3. **Examples Repository** - ✅ DELIVERED
   - One example per peripheral, named after what it proves, plus a runnable examples repository at
     <https://github.com/bitscrafts/efr32mg2x-examples>

4. **Documentation** - ✅ CONTINUOUS
   - Crate READMEs generated from the tree and gate-checked; `docs/04-reference/` holds one document per measurement

**Estimated Duration**: Ongoing throughout development

---

### ◐ Phase 9: Community & Ecosystem (PARTLY DELIVERED)

**Long-term Goals**:

1. **Embassy Support** (Async Framework) - ✅ DRIVER, EXECUTOR AND TIME DRIVER PROVEN
   - A time driver and executor on the SYSRTC, including the 32-bit wrap, measured
   - `embedded_hal_async::i2c::I2c` measured on the board (`WHO_AM_I = 0x6a`)
   - What it does not do: the wait returns `Pending` and self-wakes, so the core does not sleep during a transfer

2. **Board Support Packages** - ✅ TWO, AND ON HARDWARE
   - `xiao-mg24-bsp` (28 examples) and `xiao-expansion-base-bsp` (10 examples) for the XIAO MG24 Sense and its carrier

3. **Device Variant Support** - ⏳ ONE VARIANT
   - B220 only; the A-series and the MG21/MG26 families are not generated

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
crates/efr32mg2x-hal/
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

The PAC generation problem this section used to describe -- svd2rust failing until it was installed with `--locked` --
closed in December 2025 and the resolution is in `CLAUDE.md`, not here. What is open now, in order:

1. **D2f pairing.** Reproduce the DHKey failure with the security manager enabled and the PSA layer's entry points
   instrumented, so that the state machine's last step before key generation is named rather than inferred.
2. **D2g BME680 gas resistance.** Implement the heater, `res_heat` and the `par_g*` curves, and measure the resistance
   against a known gas event rather than against the datasheet's typical value.
3. **PRS with a GPIO producer.** `Source::GpioPin` is encoded and never driven; the measurement is N edges producing N
   conversions, with a control in which the pin does not move.
4. **Phase D3 (Zigbee 3.0)**, following the `-sys` pattern that RAIL and Bluetooth already use, then **Phase D4**
   (Thread/Matter) and **Phase E** (security).

Detail, with what each one needs to count as measured, is in [`BACKLOG.md`](BACKLOG.md).

---

**Last Updated**: 2026-11-26
**Current Phase**: Phase D2 verified on hardware; D2f pairing and D2g gas resistance open
**Progress**: phases P, A, B, C (to DMA stage 2), D0, D1 and D2 complete -- 29 public HAL modules, 28 HAL examples,
RAIL plus Bluetooth plus both board crates
**Next Milestone**: D2f and D2g, then Phase D3 (Zigbee 3.0) or Phase E (Security features)
