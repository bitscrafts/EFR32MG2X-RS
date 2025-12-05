# EFR32MG24 Project Instructions

## Project Context

This project provides Rust support (PAC + HAL) for the Silicon Labs EFR32MG24 wireless MCU.

**Status**: Phase 5 Tier 1 Complete - GPIO, CMU, Delay with Hardware Register Access

**Target Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)

**Key Documentation**:
- [docs/PLAN.md](docs/PLAN.md) - Development roadmap (9 phases)
- [docs/STATUS.md](docs/STATUS.md) - Current implementation status
- [efr32mg24-hal/docs/PHASE2_PLAN.md](efr32mg24-hal/docs/PHASE2_PLAN.md) - Phase 2 completion details

## Project Structure

```
EFR32MG24/
├── Cargo.toml                      # Workspace root
├── efr32mg24-pac/                  # Peripheral Access Crate (register-level)
│   ├── Cargo.toml
│   └── src/lib.rs                  # 138,448 lines, 133 peripherals
├── efr32mg24-hal/                  # Hardware Abstraction Layer (driver-level)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── clock/                  # CMU - 4 files, hardware register access
│   │   ├── delay/                  # SysTick delays - 1 file
│   │   ├── gpio/                   # GPIO - 4 files, hardware register access
│   │   └── prelude.rs
│   ├── examples/
│   │   ├── 01_clock.rs            # Clock configuration example
│   │   ├── 02_delay.rs            # Delay usage example
│   │   └── 03_gpio.rs             # GPIO usage example (280+ lines)
│   ├── docs/                       # HAL-specific documentation
│   │   ├── README.md
│   │   ├── STATUS.md
│   │   ├── PHASE2_PLAN.md
│   │   ├── BUILD_SYSTEM.md
│   │   └── LINKER_SETUP.md
│   ├── memory.x                    # Linker memory layout
│   ├── device.x                    # Interrupt vector table
│   └── build.rs                    # Build script
├── docs/                           # Workspace documentation
│   ├── README.md
│   ├── PLAN.md                     # Overall project plan
│   └── STATUS.md                   # Workspace status
└── CLAUDE.md                       # This file
```

## Current Status

### Completed (Phase 5 Tier 1)

**PAC (Peripheral Access Crate)**:
- Successfully generated from B220 SVD file
- 138,448 lines of Rust code (14 MB source)
- 133 peripherals accessible
- Compiles in ~2.6 minutes
- Output: 112 MB .rlib

**HAL Tier 1 Peripherals** (Hardware Register Access):
1. **GPIO** - Complete
   - Hardware register manipulation (MODEL/MODEH, DOUT, DIN)
   - Type-safe pin modes (compile-time enforcement)
   - Drive strength configuration
   - Pull resistor configuration
   - embedded-hal v1.0 OutputPin/InputPin traits
   - GPIO clock enable via CMU

2. **CMU (Clock Management)** - Complete
   - HFXO/HFRCO configuration
   - LFXO/LFRCO configuration
   - SYSCLKCTRL register access
   - Peripheral consumption pattern
   - FrozenClocks for frequency tracking
   - Critical section protection

3. **Delay** - Complete
   - SysTick-based blocking delays
   - Millisecond/microsecond/nanosecond precision
   - embedded-hal v1.0 DelayNs trait
   - Integration with CMU clock frequencies

**Examples**: 3 working examples (all compile and build successfully)

**Build Results**:
- PAC: 112 MB .rlib
- HAL: ~25 KB .rlib
- Build time: ~3 minutes (release mode)
- All examples compile without errors

### In Progress (Phase 5 Tier 2)

Next implementation targets:
- USART/EUSART - Serial communication
- I2C - I2C master mode
- SPI - SPI master mode (USART in SPI mode)
- Timer - Timers and PWM generation

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

# Build examples
cargo build --examples --features rt --release

# Build specific example
cargo build --example 03_gpio --features rt --release

# Flash to device (requires probe-rs)
cargo run --example 03_gpio --features rt --release
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

## Phase 2 Completion Summary

**What Was Accomplished**:
- GPIO module split into 4 files (mod.rs, types.rs, pin.rs, traits.rs)
- Clock module split into 4 files (mod.rs, types.rs, clocks.rs, frozen.rs)
- Implemented hardware register access for GPIO (MODEL/MODEH, DOUT, DIN)
- Implemented hardware register access for CMU (SYSCLKCTRL)
- Verified Delay implementation with SysTick
- Fixed PAC field name issue (dp.cmu_s not dp.CMU_S)
- All 3 examples compile and build successfully
- Updated all module READMEs with Phase 2 status
- Documentation organized (only README.md in root, all others in docs/)

**Key Files Modified**:
- src/gpio/* - Hardware register manipulation
- src/clock/* - CMU SYSCLKCTRL access
- src/delay/* - SysTick integration
- examples/* - Fixed API usage
- docs/* - Status updates
- ../docs/* - workspace documentation

## Known Issues & Limitations

### Current Limitations
1. GPIO interrupts not yet implemented (deferred to Tier 2)
2. Clock prescalers not implemented
3. Individual peripheral clock enable/disable not implemented
4. No hardware testing yet (requires XIAO MG24 Sense board)

### Future Work
1. Implement USART/EUSART for serial communication
2. Add I2C master mode support
3. Implement SPI via USART
4. Add Timer/PWM support
5. Hardware testing and validation
6. Embassy async support

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

**Phase 5 Tier 1** (Complete):
- [x] GPIO hardware register access working
- [x] CMU clock configuration working
- [x] Delay implementation integrated
- [x] All examples compile and build
- [x] embedded-hal v1.0 traits implemented for GPIO/Delay
- [x] Documentation updated

**Phase 5 Tier 2** (In Progress):
- [ ] USART serial communication working
- [ ] I2C master mode working
- [ ] SPI master mode working
- [ ] Timer/PWM working

**Long Term Goals**:
- [ ] Published to crates.io
- [ ] Hardware tested on XIAO MG24 Sense
- [ ] 10+ working examples
- [ ] Embassy async support
- [ ] Community contributors

## Contact & Repository

**Author**: Marcelo Correa <mvcorrea+github@gmail.com>
**Repository**: https://github.com/bitscrafts/efr32-rs (planned)
**License**: MIT OR Apache-2.0

---

**Last Updated**: December 4, 2025
**Current Phase**: 5 Tier 1 Complete (56% of overall plan)
**Next Milestone**: Implement USART/EUSART for serial communication
