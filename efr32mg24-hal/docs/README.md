# EFR32MG24 HAL Documentation

Documentation for the EFR32MG24 Hardware Abstraction Layer.

## Getting Started

- **[../README.md](../README.md)** - Main HAL README with quick start
- **[BUILD_SYSTEM.md](BUILD_SYSTEM.md)** - Build system and toolchain setup
- **[LINKER_SETUP.md](LINKER_SETUP.md)** - Linker script configuration

## Development

- **[STATUS.md](STATUS.md)** - Current implementation status
- **[PHASE2_PLAN.md](PHASE2_PLAN.md)** - Phase 2 implementation plan and progress

## Module Documentation

Module-specific documentation is located with the source code:

- **[src/clock/README.md](../src/clock/README.md)** - Clock Management Unit (CMU)
- **[src/gpio/README.md](../src/gpio/README.md)** - GPIO digital I/O
- **[src/delay/README.md](../src/delay/README.md)** - SysTick-based delays

## API Documentation

Generate complete API documentation with:

```bash
cargo doc --no-deps --features rt --open
```

---

**Last Updated**: December 4, 2025
