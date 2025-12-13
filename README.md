# EFR32MG24 Rust Support

Rust Peripheral Access Crate (PAC) and Hardware Abstraction Layer (HAL) for Silicon Labs EFR32MG24 wireless SoCs.

> ✅ **Status**: HAL Phase 5 Tier 2 - USART and I2C Complete, SPI/Timers Next

## Overview

This project aims to provide first-class Rust support for the Silicon Labs EFR32MG24 family of wireless microcontrollers. The EFR32MG24 is a powerful ARM Cortex-M33 based SoC with integrated 2.4 GHz radio supporting Zigbee, Thread, Matter, and Bluetooth LE protocols.

### Target Device

**EFR32MG24A020F1536GM48**:
- **Core**: ARM Cortex-M33 @ 78 MHz with FPU, DSP, and TrustZone
- **Memory**: 1536 KB Flash, 256 KB RAM
- **Radio**: 2.4 GHz multiprotocol (Matter, Zigbee 3.0, Thread, BLE 5.3)
- **Security**: Secure Vault with hardware crypto acceleration
- **Package**: 48-pin QFN

## Project Structure

```
EFR32MG24/
├── efr32mg24-pac/        # Peripheral Access Crate (register-level)
├── efr32mg24-hal/        # Hardware Abstraction Layer (driver-level)
├── docs/                 # Documentation, plans, and findings
├── CLAUDE.md             # Project instructions for Claude Code
└── README.md             # This file
```

## Current Status

### ✅ Completed

- [x] Comprehensive research on EFR32 Rust ecosystem
- [x] Project structure and workspace configuration
- [x] SVD file acquisition (41 device variants from official DFP pack)
- [x] Tool installation (svd2rust v0.37.1 with --locked)
- [x] B220 PAC generation for XIAO MG24 (138,448 lines, 14 MB source)
- [x] PAC compilation verified (compiles in ~2.6 minutes)
- [x] Workspace build verification
- [x] XIAO MG24 hardware identification (EFR32MG24B220F1536IM48)
- [x] Complete documentation suite

### ✅ Completed - HAL Tier 1

- [x] **GPIO** - Hardware register access (MODEL/MODEH, DOUT, DIN), type-safe pin modes, embedded-hal v1.0 traits
- [x] **CMU** - Clock configuration (HFXO/HFRCO/LFXO/LFRCO), SYSCLKCTRL register, peripheral consumption
- [x] **Delay** - SysTick-based delays (ms/us/ns), embedded-hal v1.0 DelayNs trait
- [x] **Documentation** - Module READMEs, phase 2 completion docs

### ✅ Completed - HAL Tier 2 (Partial)

- [x] **USART0** - Serial communication with hardware register access, embedded-hal-nb v1.0 traits
- [x] **I2C0/I2C1** - I2C master mode with 7-bit addressing, embedded-hal v1.0 I2C traits
- [x] **Examples** - 5 working examples (01_clock, 02_delay, 03_gpio, 04_usart, 05_i2c)

### 🚧 In Progress - HAL Tier 2

- [ ] **SPI** - SPI master mode (USART in SPI mode)
- [ ] **Timer** - Timers and PWM (TIMER0-4)

### ⏳ Planned - HAL Tier 3

- [ ] **ADC (IADC)** - Analog-to-digital conversion
- [ ] **DMA (LDMA)** - Direct memory access
- [ ] **EMU** - Energy management and low-power modes
- [ ] **Board Support Package** - XIAO MG24 Sense BSP
- [ ] **Embassy Support** - Async runtime integration
- [ ] **Hardware Testing** - Validation on actual hardware

## PAC (Peripheral Access Crate)

### Generation Success

The PAC has been successfully generated from Silicon Labs official SVD files for the Seeed Studio XIAO MG24 Sense hardware.

**Target Device**: EFR32MG24B220F1536IM48-B (B-series, high-power radio variant)

**SVD Source**:
- Silicon Labs Gecko Platform DFP 2025.6.2
- 41 device variants available (27 A-series + 14 B-series)
- Official distribution from Silicon Labs CMSIS-Pack

**Generated PAC**:
- 138,448 lines of Rust code (14 MB source)
- 133 peripherals accessible
- Compilation time: ~2.6 minutes
- Output: 112 MB .rlib
- Edition 2021

**Critical Issue Resolved**:
- Initial SVD files from community repo (A020) didn't match hardware (B220)
- Obtained official DFP pack with all 41 EFR32MG24 variants
- Selected correct B220 SVD for XIAO MG24 Sense hardware
- Previous A020 PAC preserved in `efr32mg24-pac/src_backup_a020/` for reference

### PAC Generation Commands

```bash
# Install svd2rust with locked dependencies (required)
cargo install svd2rust --locked  # v0.37.1

# Generate PAC from SVD
cd efr32mg24-pac
svd2rust -i svd/EFR32MG24B220F1536IM48.svd \
         --target cortex-m \
         -o src/

# Format and build
cargo fmt
cargo build --target thumbv8m.main-none-eabihf
```

**Important**: Use `--locked` flag to pin transitive dependencies. Without it, version conflicts cause proc-macro2 errors.

### Documentation

See detailed PAC documentation:
- [docs/pac/SVD_PACK_EXTRACTION.md](docs/pac/SVD_PACK_EXTRACTION.md) - How we obtained all 41 SVD files from official DFP pack
- [docs/pac/B220_VS_A020_COMPARISON.md](docs/pac/B220_VS_A020_COMPARISON.md) - B220 vs A020 peripheral comparison
- [docs/hardware/XIAO_MG24_HARDWARE.md](docs/hardware/XIAO_MG24_HARDWARE.md) - Target hardware specifications

## Development Plan

See [docs/PLAN.md](docs/PLAN.md) for the complete development roadmap.

**Current Progress: Phase 5 Tier 2 Partial (62%)**

1. ✅ **Phase 1-4**: Project Setup, PAC Generation, PAC Verification
2. ✅ **Phase 5 Tier 1**: HAL Core - GPIO, CMU, Delay with hardware register access
3. 🚧 **Phase 5 Tier 2**: HAL Communication - USART ✅, I2C ✅, SPI/Timers (Next)
4. ⏳ **Phase 6-7**: HAL Advanced - ADC, DMA, Power Management
5. ⏳ **Phase 8-9**: Testing, Examples, Ecosystem (Embassy, BSPs)

## Documentation

**Complete documentation index**: [docs/README.md](docs/README.md)

### Quick Links

- **[docs/STATUS.md](docs/STATUS.md)** - Current project status (what's done, what's next)
- **[docs/PLAN.md](docs/PLAN.md)** - Development roadmap (9-phase plan)
- **[docs/hardware/XIAO_MG24_HARDWARE.md](docs/hardware/XIAO_MG24_HARDWARE.md)** - Target hardware details
- **[docs/pac/B220_VS_A020_COMPARISON.md](docs/pac/B220_VS_A020_COMPARISON.md)** - Current PAC analysis
- **[docs/hal/HAL_STRUCTURE_PLAN.md](docs/hal/HAL_STRUCTURE_PLAN.md)** - HAL architecture plan
- **[CLAUDE.md](CLAUDE.md)** - Project instructions for Claude Code

## Why EFR32MG24?

The EFR32MG24 is particularly interesting for Rust embedded development:

- **Modern Architecture**: Cortex-M33 with TrustZone (memory safety + Rust safety = 🎯)
- **Wireless Protocols**: Native support for Matter, Thread, Zigbee (growing IoT markets)
- **Low Power**: Sub-µA sleep modes ideal for battery applications
- **Security Features**: Hardware crypto + Secure Vault + Rust = maximum security
- **Untapped Market**: No existing Rust support for Series 2 devices (pioneer opportunity)

## Related Projects

### Similar Chip Families

- [stm32-rs](https://github.com/stm32-rs/stm32-rs) - STM32 family (most mature)
- [nrf-rs](https://github.com/nrf-rs/nrf-hal) - Nordic nRF52/53 (excellent HAL)
- [rp-rs](https://github.com/rp-rs/rp-hal) - Raspberry Pi RP2040 (official support)
- [esp-rs](https://github.com/esp-rs/esp-hal) - Espressif ESP32 (vendor-backed)

### Silicon Labs Rust (Legacy)

- [efm32-rs](https://github.com/efm32-rs) - EFM32 Series 0/1 only (last update 2023)
- [em32-rs](https://github.com/em32-rs/wg) - Working group (largely inactive)
- [chrysn/efm32gg-hal](https://github.com/chrysn/efm32gg-hal) - Minimal GPIO-only HAL

**Note**: None of the above support EFR32 Series 2 devices (MG21/MG24/MG26).

## Prerequisites

### Software

- Rust 1.70+ with `thumbv8m.main-none-eabihf` target
- svd2rust (PAC generation)
- form (code organization)
- cargo-binutils (optional, for binary inspection)

### Hardware (for testing)

- xG24-DK2601B EFR32MG24 Development Kit (~$50)
- J-Link debug probe or Silicon Labs WSTK board
- Silicon Labs Simplicity Commander (for flashing)

## Building

```bash
# Install Rust target for Cortex-M33
rustup target add thumbv8m.main-none-eabihf

# Install svd2rust with locked dependencies (if regenerating PAC)
cargo install svd2rust --locked

# Build workspace
cargo build --release --target thumbv8m.main-none-eabihf
```

**Build Results**:
- PAC: 112 MB .rlib with 133 peripherals
- HAL: ~25 KB .rlib with GPIO, CMU, Delay implementations
- Build Time: ~3 minutes (release mode)
- Clean Compilation: ✅ No errors or warnings

## Contributing

This project is in early development. Contributions are welcome!

Areas where help is needed:
- HAL peripheral implementations (GPIO, USART, I2C, SPI, Timers, ADC, etc.)
- Testing on hardware (xG24-DK2601B dev kit)
- Documentation and examples
- embedded-hal v1.0 trait implementations
- Embassy async support

## Resources

### Official Silicon Labs

- [EFR32MG24 Product Page](https://www.silabs.com/wireless/zigbee/efr32mg24-series-2-socs)
- [Silicon Labs Community](https://community.silabs.com/)
- [Simplicity SDK](https://github.com/SiliconLabs/simplicity_sdk) (C/C++)

### Rust Embedded

- [Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [embedded-hal](https://github.com/rust-embedded/embedded-hal)
- [awesome-embedded-rust](https://github.com/rust-embedded/awesome-embedded-rust)
- [Rust Embedded Matrix Chat](https://matrix.to/#/#rust-embedded:matrix.org)

### SVD & Code Generation

- [CMSIS-SVD Specification](https://www.keil.com/pack/doc/CMSIS/SVD/html/)
- [svd2rust](https://github.com/rust-embedded/svd2rust)
- [svdtools](https://github.com/stm32-rs/svdtools) (SVD patching)

### SVD Files Source

- [cortex-debug-dp-efr32mg24](https://github.com/silabs-EricB/cortex-debug-dp-efr32mg24)
- ARM Keil Device Family Packs

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

- Silicon Labs for the EFR32MG24 SoC and documentation
- Rust Embedded Working Group for tooling and guidance
- stm32-rs project for organizational patterns
- All contributors to the Rust embedded ecosystem

---

**Project Status**: Phase 5 Tier 2 Partial - USART and I2C Implemented
**Last Updated**: December 12, 2025
**Maintainer**: Marcelo Correa <mvcorrea+github@gmail.com>
**Repository**: https://github.com/bitscrafts/efr32-rs (planned)
