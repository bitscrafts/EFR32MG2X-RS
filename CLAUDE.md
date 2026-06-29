# EFR32MG24 Project Instructions

## Project Context

This project provides Rust support (PAC + HAL + radio + Bluetooth) for the Silicon Labs EFR32MG24 wireless MCU.

**Status**: Phase D2 Complete - Bluetooth 5.3 LE Integration done. Next: Phase E (Application Development) / Phase D3 (Zigbee).

**Target Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)

**Key Documentation** (SSOT-organized under `docs/`):
- [docs/03-development/PLAN.md](docs/03-development/PLAN.md) - Development roadmap (9 phases)
- [docs/03-development/STATUS.md](docs/03-development/STATUS.md) - Current implementation status
- [docs/03-development/BACKLOG.md](docs/03-development/BACKLOG.md) - Feature backlog and TODOs
- [docs/02-architecture/IDENTIFIER_SCHEME.md](docs/02-architecture/IDENTIFIER_SCHEME.md) - Hierarchical identifier specification
- [docs/02-architecture/TERMINOLOGY.md](docs/02-architecture/TERMINOLOGY.md) - Phase/Stage terminology reference
- [docs/INDEX.md](docs/INDEX.md) - Complete documentation index
- [README.md](README.md) - Project overview and quick start

## Terminology and Identifiers

**IMPORTANT**: This project uses a two-level development hierarchy with unique identifiers:

### Hierarchy Levels

1. **Project Phases** (A, B, C, D...) - Overall HAL development roadmap milestones
   - Scope: Multiple peripherals across months
   - Example: "Phase C - Advanced Peripherals"

2. **Module Stages** (1, 2, 3...) - Incremental rollout within a single peripheral
   - Scope: Single module features across days/weeks
   - Example: "DMA Stage 1 - Memory-to-memory transfers"

**Always use**: "Phase C: DMA Stage 1 Complete" (NOT "Phase C: DMA Phase 1")

### Hierarchical Identifiers

Every work item has a unique identifier: **`[PHASE][STAGE]-[MODULE].[SUBSTAGE]`**

**Examples**:
```
A1-GPIO      = Phase A, Stage 1: GPIO module
B4-Timer     = Phase B, Stage 4: Timer/PWM
C2-DMA.1     = Phase C, Stage 2 (DMA), Substage 1 (memory-to-memory)
C3-EMU       = Phase C, Stage 3: Energy Management Unit
```

**Benefits**: Globally unique, self-documenting, sortable, searchable

**Usage**:
- Commits: `[C2-DMA.1] feat(hal): Add memory-to-memory DMA`
- Tasks: `@identifier(C3-EMU)`
- Docs: Module headers include identifier

**References**:
- [docs/TERMINOLOGY.md](docs/TERMINOLOGY.md) - Phase/Stage terminology
- [docs/IDENTIFIER_SCHEME.md](docs/IDENTIFIER_SCHEME.md) - Complete identifier specification

## Project Structure

```
EFR32MG24/
├── Cargo.toml                      # Workspace root (5 member crates)
├── efr32mg24-pac/                  # Peripheral Access Crate (register-level)
│   ├── Cargo.toml
│   └── src/lib.rs                  # 138,448 lines, 133 peripherals (B220 variant)
├── efr32mg24-hal/                  # Hardware Abstraction Layer (driver-level)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                  # 12 public peripheral modules
│   │   ├── adc/                    # IADC - 12-bit single-shot conversion
│   │   ├── clock/                  # CMU - HFXO/HFRCO/LFXO/LFRCO configuration
│   │   ├── delay/                  # SysTick delays (ms/us/ns)
│   │   ├── dma/                    # LDMA - memory-to-memory + peripheral-to-memory
│   │   ├── emu/                    # Energy Management Unit (EM0-EM4)
│   │   ├── gpio/                   # GPIO - type-safe pin modes
│   │   ├── i2c/                    # I2C master mode (I2C0/I2C1)
│   │   ├── rtc/                    # SYSRTC0 real-time counter/calendar
│   │   ├── spi/                    # SPI master mode (Spi0/Spi1/Spi2)
│   │   ├── timer/                  # TIMER0-4 with PWM
│   │   ├── usart/                  # USART0 serial communication
│   │   ├── wdog/                   # WDOG0 watchdog with lock mechanism
│   │   └── prelude.rs
│   ├── examples/                   # 14 HAL examples (00_recovery through 12_wdog)
│   ├── docs/                       # HAL-specific documentation
│   ├── memory.x                    # Linker memory layout
│   ├── device.x                    # Interrupt vector table
│   └── build.rs                    # Build script
├── efr32mg24-rail-sys/             # RAIL radio FFI bindings (3,974 lines)
├── efr32mg24-bluetooth-sys/        # Bluetooth 5.3 LE stack (BGAPI FFI + safe blocking API)
│   └── examples/                   # 4 BLE FFI examples
├── xiao-mg24-bsp/                  # Board Support Package (XIAO MG24 Sense)
│   └── examples/                   # 9 BSP examples (blink + BLE + microphone streaming)
├── efr32mg24-bootloader/           # Gecko bootloader integration (workspace-excluded)
├── bootloader-test-app/            # Bootloader test application (workspace-excluded)
├── book/                           # mdBook-generated documentation
├── docs/                           # Workspace documentation (SSOT-organized)
│   ├── INDEX.md                    # Complete documentation index
│   ├── 01-getting-started/         # Flashing, recovery, safety procedures
│   ├── 02-architecture/            # Radio, HAL, identifier scheme, terminology
│   ├── 03-development/             # STATUS, PLAN, BACKLOG, LOG, CI/CD
│   ├── 04-reference/               # Bootloader/radio/hardware research
│   └── 99-archive/                 # Historical/resolved documents
├── scripts/                        # Build and flashing scripts
├── .archive/                       # Timestamped documentation backups
├── HANDOFF.md                      # Comprehensive project handoff
├── CHANGELOG.md                    # Version history
└── CLAUDE.md                       # This file
```

## Current Status

### Phase P - Preparation (Complete)
- Research on existing Rust support for EFR32 family (no Series 2 support found)
- EFR32MG24 MCU specification analysis (Cortex-M33, memory, peripherals)
- Cargo workspace created (PAC + HAL + radio + Bluetooth + BSP)
- XIAO MG24 hardware identified (EFR32MG24B220F1536IM48-B)
- Silicon Labs Gecko Platform DFP pack obtained (2025.6.2), 41 SVD files extracted
- B220 PAC generated with svd2rust v0.37.1 (`--locked`): 138,448 lines, 133 peripherals

### Phase A - Essential Peripherals (Complete)

**PAC (Peripheral Access Crate)**:
- Generated from B220 SVD file
- 138,448 lines of Rust code (14 MB source)
- 133 peripherals accessible
- Compiles in ~2.6 minutes
- Output: 112 MB .rlib

**HAL Phase A Peripherals** (hardware register access):
1. **GPIO (A1)** - Complete
   - Hardware register manipulation (MODEL/MODEH, DOUT, DIN)
   - Type-safe pin modes (compile-time enforcement)
   - Drive strength / pull resistor configuration
   - embedded-hal v1.0 OutputPin/InputPin traits
   - GPIO clock enable via CMU

2. **CMU (A2)** - Complete
   - HFXO/HFRCO/LFXO/LFRCO configuration
   - SYSCLKCTRL register access
   - Peripheral consumption pattern
   - FrozenClocks for frequency tracking
   - Critical section protection

3. **Delay (A3)** - Complete
   - SysTick-based blocking delays (ms/us/ns)
   - embedded-hal v1.0 DelayNs trait
   - Integration with CMU clock frequencies

### Phase B - Communication Peripherals (Complete)

4. **USART0 (B1)** - Complete
   - Hardware register access (CTRL, FRAME, CLKDIV, STATUS)
   - Configurable baud rate, parity, stop bits
   - embedded-hal-nb v1.0 Read/Write traits
   - Automatic clock enable via CMU

5. **I2C (B2)** - Complete (I2C0/I2C1)
   - 7-bit addressing, Standard (100 kHz) / Fast (400 kHz) modes
   - Write, read, write-read operations
   - embedded-hal v1.0 I2c trait

6. **SPI (B3)** - Complete (Spi0/Spi1/Spi2)
   - Spi0 = USART0, Spi1 = EUSART0, Spi2 = EUSART1 (all in SPI master mode)
   - All 4 SPI modes (Mode 0-3), MSB-first / LSB-first
   - embedded-hal v1.0 SpiBus trait (all instances)

7. **Timer/PWM (B4)** - Complete (TIMER0-4)
   - 5 timer instances, 3 PWM channels each
   - Edge-aligned and center-aligned PWM
   - Counter modes (up/down/up-down) with automatic prescaler
   - Interrupt support (overflow/compare)
   - embedded-hal PWM trait

### Phase C - Advanced Peripherals (Complete - Stage 1)

8. **ADC (C1)** - Complete (IADC)
   - 12-bit single-shot conversion, configurable reference (VBGR 1.21V, VDD)
   - 6 input channels + ground, timeout-protected, thread-safe

9. **DMA (C2)** - Complete (LDMA)
   - C2-DMA.1: Memory-to-memory transfers (blocking, type-safe channels)
   - C2-DMA.2: Peripheral-to-memory transfers with circular buffer mode
   - Deferred: C2-DMA.3 (linked descriptors), C2-DMA.4 (interrupt-driven async)

10. **EMU (C3)** - Complete
    - EM0-EM4 energy mode transitions, type-safe states, builder config
    - Voltage scaling, BOD, EM4 hibernate

11. **RTC (C4)** - Complete (SYSRTC0)
    - Calendar + counter modes, alarms, 3 capture/compare channels
    - 32.768 kHz LFXO clock source

12. **WDOG (C5)** - Complete (WDOG0)
    - 16 timeout periods, lock mechanism (key 0xABE8)
    - EM1-EM3 support, debug freeze, warning mode

### Phase D0-D1 - Radio Foundation (Complete)

- **D0**: -sys crate template, workspace configuration, xiao-mg24-bsp skeleton
- **D1-RAIL.1**: efr32mg24-rail-sys crate - 3,974 lines of FFI bindings, SHA256 blob
  verification, platform-independent build, Grade A- review

### Phase D2 - Bluetooth 5.3 LE (Complete)

- **efr32mg24-bluetooth-sys**: BGAPI FFI bindings (4,307 lines) + safe blocking API
  (Bluetooth, BleEvent, GATT server, event polling)
- **XiaoBle BSP**: advertiser + GATT server + event system with automatic connection
  management (80% code reduction vs raw FFI)
- **D2-MIC.1**: MSM381ACT001 analog microphone driver (16 kHz audio streaming via LDMA)
- **Examples**: 4 BLE FFI examples + 6 BSP BLE/sensor examples + microphone streaming

### Build Statistics
- Total HAL code: ~4,772 lines (12 modules, all with READMEs)
- HAL examples: 14 (00_recovery through 12_wdog), all build to flashable binaries
- BLE examples: 4 (bluetooth-sys) + 6 (BSP integration)
- BSP examples: 9 (blink variants + BLE + microphone streaming)
- Zero compilation warnings or errors

## Development Guidelines

### Code Style (User Preferences)
- No emojis in code or commit messages
- All modules in src/ must have their own README.md file
- Use mermaid for diagrams
- Keep files around 400 lines (split into submodules if larger)
- Avoid repeating information (single source of truth)

### Commit Messages
- Avoid mentioning Claude in commit messages
- Use conventional commit format

### Documentation Standards
- Module README in src/module_name/README.md for each peripheral
- Status and planning docs in docs/ folders
- Only README.md in root folders (all other docs in docs/)
- Update status docs when completing phases
- Use mermaid diagrams for architecture
- **Always backup documentation before modification** (see backup workflow below)

### Documentation Backup Workflow

**Critical Requirement**: Before modifying any documentation file, create a timestamped backup in `.archive/`:

```bash
# Backup workflow (MUST be done before any documentation modification)
cp docs/PLAN.md .archive/PLAN_$(date +%Y%m%d_%H%M%S).md
# Then proceed with modifications

# For any .md file modification:
cp path/to/file.md .archive/filename_$(date +%Y%m%d_%H%M%S).md
```

**What gets archived**:
- Planning documents (PLAN.md, STATUS.md)
- Module documentation (README.md files when doing major rewrites)
- Project documentation (CLAUDE.md, README.md)
- Changelogs (CHANGELOG.md)

**Archive location**: `.archive/` folder at project root
**Retention policy**: Archives kept indefinitely for history
**Purpose**: Quick rollback capability without git operations

### File Organization
- Files >400 lines should be split into submodules
- Use module folders (e.g., src/gpio/ with mod.rs, types.rs, pin.rs, traits.rs)
- Coordinator module (mod.rs) should be small, mostly re-exports
- Implementation details in separate files

## Key Technical Patterns

### Hardware Register Access
```rust
// Critical section for atomic operations
critical_section::with(|_cs| {
    let gpio = unsafe { &(*crate::pac::GpioS::ptr()) };
    gpio.porta_model().modify(|r, w| unsafe {
        w.bits((r.bits() & !(0xF << (PIN * 4))) | (0x4 << (PIN * 4)))
    });
});
```

### Peripheral Consumption
```rust
pub fn new(cmu: crate::pac::CmuS, config: ClockConfig) -> Self {
    // Takes ownership of CMU peripheral to ensure exclusive access
}
```

### Type-State Pattern (GPIO)
```rust
// Compile-time pin mode enforcement
Pin<PORT, PIN, Input<Pull>> -> Pin<PORT, PIN, Output<DRIVE>>
```

### USART/EUSART Register Differences (SPI)
```rust
// USART (Spi0) - uses CTRL and FRAME registers
usart.ctrl().write(|w| {
    w.sync().set_bit()  // Synchronous mode
        .clkpol().bit(cpol)
        .clkpha().bit(cpha)
});

// EUSART (Spi1/Spi2) - uses CFG0 and CFG2 registers
eusart.cfg0().write(|w| w.sync().sync());  // Different API
eusart.cfg2().write(|w| {
    w.clkpol().bit(cpol)
        .clkpha().bit(cpha)
        .master().set_bit()
});
```

### I2C State Machine Pattern
```rust
// Wait for specific state transitions
while !i2c.state().read().state().is_idle() {}

// Issue commands and wait for completion
i2c.cmd().write(|w| w.start().set_bit());
while i2c.state().read().state().is_idle() {}
```

## PAC Generation (Reference)

### How the PAC Was Generated

The PAC (Peripheral Access Crate) was successfully generated from Silicon Labs official SVD files.

**Critical Steps**:

1. **Obtain SVD Files**:
```bash
# Download Silicon Labs Gecko Platform DFP pack
wget https://www.silabs.com/documents/public/cmsis-packs/SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack

# Extract SVD files (41 variants)
unzip SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack \
  "SVD/EFR32MG24/*.svd" -d svd/
```

2. **Install svd2rust with --locked** (Critical!):
```bash
# MUST use --locked to pin dependencies
cargo install svd2rust --locked  # v0.37.1
```

**Why --locked is required**: Without it, transitive dependency version conflicts cause proc-macro2 errors that look like SVD file problems but are actually build system issues.

3. **Generate PAC**:
```bash
cd efr32mg24-pac

# Use correct SVD for hardware (B220 for XIAO MG24 Sense)
svd2rust -i svd/EFR32MG24B220F1536IM48.svd \
         --target cortex-m \
         -o src/

# Format and verify
cargo fmt
cargo build --target thumbv8m.main-none-eabihf
```

**Result**:
- 138,448 lines of Rust code (14 MB)
- 133 peripherals accessible
- Compiles in ~2.6 minutes
- Output: 112 MB .rlib

**Important**: The A020 vs B220 mismatch was a critical discovery. Always verify the exact device variant matches your hardware.

## Common Commands

### Build Commands
```bash
# Check library
cargo check --features rt

# Build all examples
cargo build --examples --features rt --release

# Build specific examples
cargo build --example 03_gpio --features rt --release
cargo build --example 07_timer_pwm --features rt --release
cargo build --example 08_adc --features rt --release
cargo build --example 09_dma --features rt --release

# Flash to device (requires probe-rs)
cargo run --example 08_adc --features rt --release
```

### Development
```bash
# Add new peripheral module
mkdir src/usart
touch src/usart/{mod.rs,types.rs,README.md}

# Check file sizes
find src -name "*.rs" -exec wc -l {} + | sort -n

# Generate docs
cargo doc --no-deps --features rt --open
```

## MCU Specifications

**EFR32MG24B220F1536IM48-B** (XIAO MG24 Sense):
- ARM Cortex-M33 @ 78 MHz with FPU and TrustZone-M
- 1536 KB Flash memory
- 256 KB RAM
- 48-pin package (IM48)
- B-series (high-power radio variant)
- 2.4 GHz radio (Bluetooth 5.3, Zigbee, Thread, Matter)
- Hardware crypto acceleration

**Rust Target**: `thumbv8m.main-none-eabihf`

## Phase D2 Completion Summary

**Bluetooth 5.3 LE Integration Completed**:

1. **efr32mg24-rail-sys (D1-RAIL.1)**:
   - 3,974 lines of auto-generated Rust FFI bindings from RAIL C headers
   - Platform-independent build (macOS/Linux/Windows)
   - SHA256 blob integrity verification
   - Graceful degradation (compiles without blobs for development)
   - Grade A- review

2. **efr32mg24-bluetooth-sys (D2-BLE.2)**:
   - 4,307 lines of BGAPI FFI bindings + safe blocking API
   - `blocking` module: Bluetooth, BleEvent, GATT server, event polling
   - `ffi` module: raw unsafe BGAPI bindings for advanced users
   - 4 BLE FFI examples (beacon, advertiser, GATT heart rate, custom service)

3. **XiaoBle BSP (D2-BLE.3)**:
   - Advertiser + GATT server + event system
   - Automatic connection management
   - Safe BLE wrapper: 80% code reduction vs raw FFI
   - 6 BSP BLE/sensor examples + microphone streaming

4. **DMA Stage 2 (C2-DMA.2)**:
   - LDMA peripheral-to-memory transfers with circular buffer mode
   - Enables continuous ADC/IADC capture

5. **Microphone Driver (D2-MIC.1)**:
   - MSM381ACT001 analog microphone driver (373 lines)
   - 16 kHz audio streaming via LDMA

**Key Achievements**:
- All 12 HAL modules have comprehensive READMEs and embedded-hal v1.0 traits
- All modules follow the multi-file pattern (mod.rs, types.rs, traits.rs, README.md)
- Zero compilation warnings or errors
- 14 HAL examples + 4 BLE FFI examples + 9 BSP examples
- Production-grade: SAFETY comments, critical sections, timeout protection
- Radio foundation enables BLE, Zigbee, Thread, Matter

## Known Issues & Limitations

### Current Limitations
1. **GPIO**: Interrupts not yet implemented
2. **USART**: Only USART0 supported (EUSART0/1 for async in future)
3. **I2C**: Only master mode (slave mode not implemented)
4. **SPI**: Only master mode, blocking operations only
5. **DMA**: Stages 1-2 complete; linked descriptors (Stage 3) and interrupt-driven async (Stage 4) deferred
6. **General**: No async/await support (Embassy planned)
7. **Testing**: No hardware testing yet (requires XIAO MG24 Sense board)
8. **Radio**: RAIL/Bluetooth bindings require Silicon Labs binary blobs (MSLA licensing)

### Future Work
1. **Phase D3**: Zigbee 3.0 protocol stack integration
2. **Phase D4**: Thread/Matter support
3. **Phase E**: Security features (AES, SHA, Secure Vault)
4. **C2-DMA.3/4**: Linked descriptors and interrupt-driven async DMA
5. **C6-Test**: Hardware validation on XIAO MG24 Sense
6. Embassy async runtime integration

## Resources

**Official Documentation**:
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)
- [EFR32MG24 Datasheet](https://www.silabs.com/documents/public/data-sheets/efr32mg24-datasheet.pdf)

**Rust Resources**:
- [Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [embedded-hal Documentation](https://docs.rs/embedded-hal/)
- [cortex-m Documentation](https://docs.rs/cortex-m/)

**Repository References**:
- [stm32-rs](https://github.com/stm32-rs/stm32-rs) - Similar project organization
- [nrf-rs](https://github.com/nrf-rs/nrf-hal) - Excellent HAL example
- [efm32-rs](https://github.com/efm32-rs) - EFM32 Series 0/1 (legacy)

## Success Metrics

**Phase A - Essential Peripherals** (Complete):
- [x] GPIO hardware register access working
- [x] CMU clock configuration working
- [x] Delay implementation integrated
- [x] embedded-hal v1.0 traits implemented for GPIO/Delay

**Phase B - Communication Peripherals** (Complete):
- [x] USART0, I2C0/I2C1, SPI0/SPI1/SPI2, TIMER0-4 all production-ready
- [x] All communication peripherals have embedded-hal v1.0 traits
- [x] All modules have comprehensive README documentation

**Phase C - Advanced Peripherals** (Complete - Stage 1):
- [x] ADC, DMA (Stages 1-2), EMU, RTC, WDOG all production-ready
- [x] 14 HAL examples (all build successfully)
- [ ] DMA Stages 3-4 (linked descriptors, interrupt-driven async) deferred

**Phase D0-D2 - Radio + Bluetooth** (Complete):
- [x] RAIL FFI bindings (efr32mg24-rail-sys)
- [x] Bluetooth 5.3 LE stack (efr32mg24-bluetooth-sys)
- [x] XiaoBle BSP with safe BLE wrapper
- [x] Microphone driver with LDMA streaming
- [x] 10+ BLE/BSP examples

**Long Term Goals**:
- [ ] Published to crates.io
- [ ] Hardware tested on XIAO MG24 Sense
- [ ] Phase D3: Zigbee 3.0 integration
- [ ] Phase D4: Thread/Matter support
- [ ] Phase E: Security features (AES, SHA, Secure Vault)
- [ ] Embassy async support
- [ ] Community contributors

## Contact & Repository

**Author**: Marcelo Correa <mvcorrea+github@gmail.com>
**Repository**: https://github.com/bitscrafts/EFR32MG2X-RS
**License**: MIT OR Apache-2.0

---

**Last Updated**: June 28, 2026
**Current Phase**: Phase D2 Complete - Bluetooth 5.3 LE Integration done
**Current Status**: 12 HAL modules + RAIL FFI + Bluetooth stack + XiaoBle BSP + microphone driver
**Next Milestone**: Phase D3 (Zigbee 3.0) or Phase E (Application Development)
