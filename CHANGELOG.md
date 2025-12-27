# Changelog

All notable changes to the EFR32MG24 HAL will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Phase C: Advanced Peripherals (2025-12-19 to 2025-12-27)

**C1-ADC: Analog-to-Digital Converter (IADC)**:
- Production-ready 12-bit ADC with VBGR/VDD reference selection
- Single-shot conversion with timeout protection
- 6 input channels (Ch0-Ch5) + Ground reference
- Thread-safe with critical sections (RTOS-ready)
- Comprehensive SAFETY documentation (5 unsafe blocks)
- Example 08_adc.rs with voltage calculation utilities
- Grade A expert review (SHIP IT)

**C2-DMA.1: DMA Controller (LDMA) - Stage 1**:
- Memory-to-memory transfers with type-safe channels
- 8-bit, 16-bit, 32-bit transfer sizes via TransferElement trait
- Blocking operation with timeout protection (~13ms)
- Critical sections for RTOS safety
- Comprehensive SAFETY documentation (6 unsafe blocks)
- Example 09_dma.rs with 5 test cases
- Grade B+ expert review (SHIP IT for Stage 1)
- Stage 2 planned: Peripheral transfers, linked descriptors, interrupts

#### Phase B: Communication Peripherals (2025-12-04 to 2025-12-18)

**B4-Timer: TIMER0-4 with PWM (2025-12-18)**:
- Hardware register access for all 5 timers
- PWM generation (3 channels per timer, edge/center-aligned)
- Interrupt support (overflow with listen/unlisten API)
- Raw duty cycle API for precision control
- Critical sections for RTOS safety
- SAFETY comments on all unsafe blocks
- Example 07_timer_pwm.rs
- Grade A expert review (SHIP IT)

**B3-SPI: SPI Master Mode (2025-12-12)**:
- Three instances: Spi0 (USART0), Spi1 (EUSART0), Spi2 (EUSART1)
- All 4 SPI modes (Mode 0-3) with CPOL/CPHA
- Configurable frequency (up to PCLK/2)
- MSB-first and LSB-first bit order
- Full-duplex, write-only, read-only operations
- embedded-hal v1.0 SpiBus trait (all instances)
- Example 06_spi.rs demonstrating all 3 instances

**B2-I2C: I2C Master Mode (2025-12-12)**:
- I2C0/I2C1 with 7-bit addressing
- Standard (100 kHz) and Fast (400 kHz) modes
- Hardware register access (CTRL, CMD, STATE, CLKDIV)
- Write, read, and write-read operations
- embedded-hal v1.0 I2c trait
- Example 05_i2c.rs

**B1-USART: Serial Communication (2025-12-04)**:
- USART0 register access with configurable baud rates
- 8-bit data, configurable parity (none, even, odd)
- Configurable stop bits (1 or 2)
- embedded-hal-nb v1.0 Read/Write traits
- Example 04_usart.rs

#### Phase A: Essential Peripherals (2025-12-04)

**A3-Delay: SysTick Delays**:
- SysTick timer integrated with CMU-configured clock frequencies
- Accurate millisecond/microsecond/nanosecond delays
- embedded-hal v1.0 DelayNs trait implementation verified
- Example 02_delay.rs

**A2-CMU: Clock Management Unit**:
- Implemented CMU SYSCLKCTRL register access for clock source selection (HFXO/HFRCO)
- Peripheral consumption pattern (takes ownership of CMU_S peripheral)
- Frequency tracking in FrozenClocks struct
- Critical section protection for clock configuration
- Split module into 4 files: mod.rs (73 lines), types.rs (66 lines), clocks.rs (138 lines), frozen.rs (40 lines)
- Example 01_clock.rs

**A1-GPIO: Hardware Register Access**:
- Implemented direct hardware register manipulation for GPIO operations
- PORTx_MODEL/MODEH registers for pin mode configuration (4 bits per pin)
- PORTx_DOUT, DOUTSET, DOUTCLR, DOUTTGL registers for digital output control
- PORTx_DIN register for digital input reading
- CMU_S CLKEN0 register GPIO bit for peripheral clock enable
- Critical sections for atomic read-modify-write operations
- Split module into 4 files: mod.rs (104 lines), types.rs (155 lines), pin.rs (248 lines), traits.rs (110 lines)
- Example 03_gpio.rs

#### Documentation System (2025-12-27)

**C7-Docs: Hierarchical Identifier System**:
- Created research-backed identifier scheme: [PHASE][STAGE]-[MODULE].[SUBSTAGE]
- Based on industry standards (WBS, JIRA, Military operations, SemVer)
- Created IDENTIFIER_SCHEME.md (535 lines) with complete specification
- Updated TERMINOLOGY.md (v2.0.0) with identifier reference
- Updated CLAUDE.md, BACKLOG.md, STATUS.md with identifiers
- Updated all 8 module READMEs with identifier headers
- Created .gitmessage git commit template
- Mapped all work: P1-P4 (Preparation), A1-A3 (Essential), B1-B4 (Communication), C1-C7 (Advanced)

#### HAL Phase 2: Hardware Register Access (2025-12-04)

**GPIO Module - Hardware Implementation**:
- Implemented direct hardware register manipulation for GPIO operations
- PORTx_MODEL/MODEH registers for pin mode configuration (4 bits per pin)
- PORTx_DOUT, DOUTSET, DOUTCLR, DOUTTGL registers for digital output control
- PORTx_DIN register for digital input reading
- CMU_S CLKEN0 register GPIO bit for peripheral clock enable
- Critical sections for atomic read-modify-write operations
- Split module into 4 files: mod.rs (104 lines), types.rs (155 lines), pin.rs (248 lines), traits.rs (110 lines)

**CMU Module - Clock Configuration**:
- Implemented CMU SYSCLKCTRL register access for clock source selection (HFXO/HFRCO)
- Peripheral consumption pattern (takes ownership of CMU_S peripheral)
- Frequency tracking in FrozenClocks struct
- Critical section protection for clock configuration
- Split module into 4 files: mod.rs (73 lines), types.rs (66 lines), clocks.rs (138 lines), frozen.rs (40 lines)

**Delay Module - Integration**:
- SysTick timer integrated with CMU-configured clock frequencies
- Accurate millisecond/microsecond/nanosecond delays
- embedded-hal v1.0 DelayNs trait implementation verified

**Examples**:
- Updated all 3 examples with hardware register access
- Fixed PAC API usage (dp.cmu_s instead of dp.CMU_S)
- All examples compile and build successfully to ARM Cortex-M33 binaries

**Documentation**:
- Updated all module READMEs with Phase 2 implementation status
- Created efr32mg24-hal/docs/PHASE2_PLAN.md documenting completion
- Reorganized documentation (only README.md in roots, all docs in docs/ folders)
- Updated workspace-level PLAN.md and STATUS.md

### Fixed

#### API Compatibility
- Fixed CMU peripheral field name from `dp.CMU_S` to `dp.cmu_s` across all files
- Removed circular import of `Pull` type in GPIO types.rs
- Fixed unused variable warning in GPIO clock enable (`|_r, w|` pattern)

#### Linker Script Configuration (2025-12-03)
- Added `memory.x` linker script with memory layout for EFR32MG24B220F1536IM48-B
  - Flash: 1536 KB @ 0x08000000
  - RAM: 256 KB @ 0x20000000
- Added `device.x` with weak aliases for all EFR32MG24 interrupt handlers
- Added `build.rs` to copy linker scripts to OUT_DIR
- Configured `.cargo/config.toml` with proper linker flags for Cortex-M33
- Added critical-section support via cortex-m `critical-section-single-core` feature

#### Documentation
- Added `LINKER_SETUP.md` - Complete linker script configuration guide
- Added `docs/BUILD_SYSTEM.md` - Build pipeline and optimization documentation
- Updated README with architecture notes and build information

#### Examples - Binary Builds
- All examples now build to flashable ARM Cortex-M33 binaries
  - `01_clock` - 707 KB
  - `02_delay` - 725 KB
  - `03_gpio` - 722 KB

### Fixed

#### Linker Issues
- Fixed "region 'FLASH' already defined" error by removing duplicate `-Tmemory.x` flag
- Fixed "undefined symbol: _critical_section_1_0_acquire" by enabling critical-section feature
- Fixed "undefined symbol" errors for device interrupts by adding complete device.x
- Fixed memory.x search path by ensuring build.rs adds OUT_DIR to linker search

### Changed
- Modified `.cargo/config.toml` to only use `-Tlink.x` (removed `-Tmemory.x`)
- Updated `efr32mg24-hal/Cargo.toml` to enable `critical-section-single-core` feature
- Removed memory.x from PAC crate (HAL now provides it)

## [0.1.0] - 2025-12-XX (Planned Initial Release)

### Added

#### Core HAL Modules
- **Clock Module**: System clock configuration
  - Clock source selection (FSRCO, HFRCODPLL, HFXO)
  - Frequency tracking and validation
  - Type-safe frequency constants

- **Delay Module**: Blocking delays using SysTick
  - Microsecond and millisecond delays
  - embedded-hal `DelayUs` and `DelayMs` traits
  - Clock-frequency aware timing

- **GPIO Module**: General Purpose I/O
  - Type-safe pin modes (Input, Output, Analog)
  - Pin mode conversion at compile time
  - embedded-hal GPIO traits
  - Pull-up/pull-down resistor configuration
  - Drive strength configuration

#### PAC (Peripheral Access Crate)
- SVD-generated peripheral register access
- TrustZone-aware secure/non-secure register access
- Type-safe register field access
- Critical-section protected peripheral singleton

#### Build System
- Cargo workspace configuration
- ARM Cortex-M33 target support (thumbv8m.main-none-eabihf)
- Size-optimized release profile (LTO, opt-level=z)
- probe-rs integration for flashing and debugging

#### Examples
- `01_clock` - Clock configuration example
- `02_delay` - Delay functionality demonstration
- `03_gpio` - GPIO pin control example

#### Project Infrastructure
- MIT OR Apache-2.0 dual license
- Comprehensive README files for each module
- STATUS.md tracking implementation progress
- Git repository initialization

### Design Decisions

#### Zero Unsafe Code in HAL
- All HAL code uses safe Rust abstractions
- Unsafe code confined to PAC layer only
- Peripherals accessed via safe `take()` instead of unsafe `steal()`

#### Type-Safe GPIO
- Pin modes enforced at compile time
- Mode conversions via consuming methods
- Prevents runtime errors from incorrect pin configuration

#### embedded-hal Compatibility
- Implements embedded-hal v1.0 traits
- Ensures compatibility with ecosystem drivers
- Future-proof for embedded-hal updates

#### Clock-Aware Delays
- Delays automatically adjust for system clock frequency
- Prevents timing errors when clock changes
- No manual recalibration needed

## Version History

- **[Unreleased]** - Current development
- **[0.1.0]** - Planned initial release

## Migration Guides

### Migrating from Unsafe to Safe Peripheral Access

**Before:**
```rust
let peripherals = unsafe { Peripherals::steal() };
```

**After:**
```rust
let peripherals = Peripherals::take().unwrap();
```

**Why:** The safe `take()` method enforces singleton pattern at runtime, preventing multiple mutable references to hardware peripherals.

### Enabling Binary Builds

**Required changes in your project:**

1. Add `.cargo/config.toml`:
```toml
[build]
target = "thumbv8m.main-none-eabihf"

[target.thumbv8m.main-none-eabihf]
rustflags = ["-C", "link-arg=-Tlink.x"]
```

2. Enable `rt` feature:
```toml
[dependencies]
efr32mg24-hal = { version = "0.1", features = ["rt"] }
cortex-m-rt = "0.7"
```

3. Use entry point macro:
```rust
#[cortex_m_rt::entry]
fn main() -> ! {
    // Your code here
    loop {}
}
```

## Breaking Changes

None yet (pre-release).

## Deprecations

None yet (pre-release).

## Known Issues

### Build Warnings
- `default-features` warning for cortex-m-rt in workspace
  - **Impact:** None (warning only)
  - **Workaround:** Will be fixed by specifying default-features in workspace dependencies
  - **Tracking:** Low priority

### Missing Features
- UART/USART drivers - Planned for v0.2.0
- I2C drivers - Planned for v0.2.0
- SPI drivers - Planned for v0.2.0
- ADC/DAC drivers - Planned for v0.2.0
- Timer/PWM drivers - Planned for v0.2.0
- Radio (Bluetooth/Zigbee) - Planned for v1.0.0

## Future Plans

### v0.2.0 (Planned)
- [ ] UART/USART driver with DMA support
- [ ] I2C master/slave drivers
- [ ] SPI master/slave drivers
- [ ] ADC driver with continuous conversion
- [ ] DAC driver with waveform generation
- [ ] Timer/PWM drivers
- [ ] Watchdog timer driver
- [ ] RTC driver

### v0.3.0 (Planned)
- [ ] Low power mode support
- [ ] Interrupt-driven peripherals
- [ ] DMA for all applicable peripherals
- [ ] Embassy async runtime support

### v1.0.0 (Planned)
- [ ] Radio drivers (Bluetooth, Zigbee, Thread, Matter)
- [ ] Comprehensive test suite
- [ ] Hardware-in-the-loop testing
- [ ] Stable API commitment
- [ ] Production ready

## Contributors

- Marcelo Correa (mvcorrea+github@gmail.com) - Initial implementation

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## References

### Hardware Documentation
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)
- [EFR32MG24 Datasheet](https://www.silabs.com/documents/public/data-sheets/efr32mg24-datasheet.pdf)
- [ARM Cortex-M33 Technical Reference Manual](https://developer.arm.com/documentation/100230/)

### Software Resources
- [cortex-m-rt](https://docs.rs/cortex-m-rt/) - Cortex-M runtime
- [embedded-hal](https://docs.rs/embedded-hal/) - Hardware abstraction traits
- [svd2rust](https://docs.rs/svd2rust/) - SVD to Rust code generator
- [probe-rs](https://probe.rs/) - Embedded debugging toolkit

### Community
- [Rust Embedded Book](https://docs.rust-embedded.org/book/)
- [Embedded Rust on Matrix](https://matrix.to/#/#rust-embedded:matrix.org)
- [Silicon Labs Community](https://community.silabs.com/)
