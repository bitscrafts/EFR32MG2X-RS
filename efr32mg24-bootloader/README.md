# EFR32MG24 Custom Bootloader

Minimal custom bootloader for EFR32MG24 written in Rust.

## Features

- **Minimal size**: ~8-12 KB (vs. Gecko bootloader's 24 KB)
- **Simple boot**: Validates and boots application
- **No ApplicationProperties_t required**: Clean build process
- **Memory safe**: Written in Rust
- **Fast boot**: Minimal initialization

## Memory Layout

```
0x08000000 ┌──────────────────────────┐
           │ Bootloader (16 KB)       │  <- This crate
0x08004000 ├──────────────────────────┤
           │ Application (1520 KB)    │  <- Your application
0x08180000 └──────────────────────────┘

RAM:
0x20000000 ┌──────────────────────────┐
           │ Application RAM (256 KB) │
0x20040000 └──────────────────────────┘
```

## Boot Sequence

1. **Reset**: CPU starts executing bootloader at 0x08000000
2. **RAM Init**: Initialize .bss and .data sections
3. **Check Mode**: Check if bootloader mode requested (magic value in RAM)
4. **Validate App**: Read application vector table at 0x08004000
5. **Boot App**:
   - Set VTOR (Vector Table Offset Register) to 0x08004000
   - Set stack pointer from application's vector table
   - Jump to application's reset handler

## Building

### Build Bootloader

```bash
cargo build --release
```

### Build Test Application

```bash
# Use test-app-memory.x for linker script
RUSTFLAGS="-C link-arg=-Ttest-app-memory.x" \
  cargo build --example test-blink-app --release
```

### Convert to Intel HEX

```bash
# Bootloader
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader \
  bootloader.hex

# Test application
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/examples/test-blink-app \
  test-app.hex
```

### Combine and Flash

```bash
# Combine bootloader + application
srec_cat bootloader.hex -Intel test-app.hex -Intel -o combined.hex -Intel

# Flash with OpenOCD
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg \
  -c "init" \
  -c "program combined.hex verify" \
  -c "reset run" \
  -c "exit"
```

## Application Requirements

Applications loaded by this bootloader must:

1. **Start at 0x08004000**: Use custom linker script (see `test-app-memory.x`)
2. **Have valid vector table**: First word = stack pointer, second word = reset handler
3. **Use standard cortex-m-rt**: No special ApplicationProperties_t needed

### Application Linker Script

```ld
MEMORY
{
  FLASH : ORIGIN = 0x08004000, LENGTH = 1520K
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
```

### Application Build Configuration

In your application's `.cargo/config.toml`:

```toml
[target.thumbv8m.main-none-eabihf]
rustflags = [
  "-C", "link-arg=-Ttest-app-memory.x",
]
```

## Entering Bootloader Mode

To enter bootloader mode from application (for future firmware updates):

```rust
// Write magic value to last word of RAM
const BOOTLOADER_MAGIC: u32 = 0xB00710AD;
const BOOTLOADER_FLAG: *mut u32 = 0x2003FFFC as *mut u32;

unsafe {
    core::ptr::write_volatile(BOOTLOADER_FLAG, BOOTLOADER_MAGIC);
}

// Reset the system
cortex_m::peripheral::SCB::sys_reset();
```

## LED Indicators

- **Normal boot**: No LED indication (fast boot to application)
- **Bootloader mode**: Slow blink (1 second ON, 1 second OFF)
- **HardFault**: Fast blink (100 ms ON, 100 ms OFF)

## Code Size

Current build sizes (release, opt-level="z"):

- **Bootloader**: ~4-6 KB
- **Test application**: ~2-3 KB
- **Total**: ~6-9 KB (well under 16 KB budget)

## Future Enhancements

Planned features for Phase 2:

- [ ] UART firmware update (XMODEM-CRC)
- [ ] CRC32 validation
- [ ] Button-triggered bootloader mode
- [ ] Application signature verification
- [ ] Rollback protection
- [ ] Power-fail-safe updates

## Troubleshooting

### Application Won't Boot

Check:
1. Application built with correct memory.x (ORIGIN = 0x08004000)
2. Vector table valid (stack pointer in RAM range, reset vector in flash)
3. Thumb mode bit set in reset vector (bit 0 = 1)

### HardFault on Boot

- Check VTOR is set correctly (0xE000ED08 = 0x08004000)
- Verify application's stack pointer is valid
- Check application's reset handler address

### LED Doesn't Blink

- Verify application initializes CMU and GPIO
- Check PA7 configured as push-pull output
- Verify delay function works (timing might need adjustment)

## Development Status

**Phase 1: Minimal Bootloader** - IN PROGRESS

- [x] Crate structure
- [x] Minimal runtime (no cortex-m-rt)
- [x] Boot sequence implementation
- [ ] Build and test on hardware
- [ ] Verify application boot
- [ ] Verify LED blink

**Next**: Test on XIAO MG24 Sense hardware

## References

- [embassy-boot](https://docs.embassy.dev/embassy-boot/)
- [From Zero to main()](https://interrupt.memfault.com/blog/how-to-write-a-bootloader-from-scratch)
- [EFR32MG24 Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efr32xg24-rm.pdf)

## License

MIT OR Apache-2.0

## Documentation

All project documentation has been organized in the `docs/` directory following Single Source of Truth (SSOT) principles.

**Start here:** [Documentation Index](docs/INDEX.md)

Quick links:
- [Getting Started & Overview](docs/01-overview/)
- [Guides & Tutorials](docs/02-guides/)
- [Reference Documentation](docs/03-reference/)
- [Development](docs/04-development/)
- [Operations](docs/05-operations/)

For complete documentation structure, see [docs/TOC.md](docs/TOC.md).
