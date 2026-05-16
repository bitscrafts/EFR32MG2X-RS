# Scripts Documentation

This directory contains utility scripts for building, flashing, and managing the EFR32MG24 firmware.

## Build Scripts

### build-container.sh

Container-based build script for EFR32MG24 firmware using Podman.

**Purpose:** Builds Rust firmware in a containerized environment to ensure consistent build conditions across different development machines.

**Usage:**
```bash
# Build all targets
./scripts/build-container.sh

# Build specific example
./scripts/build-container.sh --example 00_minimal_blink

# Clean build artifacts
./scripts/build-container.sh --clean

# Release build (default)
./scripts/build-container.sh --release
```

**Dependencies:**
- Podman (install with `brew install podman` on macOS)
- Containerfile in project root

**Features:**
- Automatically builds container image if not present
- Supports workspace and package-specific builds
- Provides binary size and location after successful build
- Color-coded output for easy debugging

## Flashing Scripts

All flashing scripts use OpenOCD with CMSIS-DAP interface for the EFR32MG24 chip.

**IMPORTANT SAFETY WARNINGS:**
- Always verify target device before flashing
- Incorrect addresses or erasing wrong regions can brick the device
- Keep backup of working firmware
- Use mass erase only when you have confirmed bootloader available
- Test with Arduino blink first to verify hardware functionality

### erase_and_flash.sh

Erases application area and flashes Rust firmware.

**Purpose:** Used for flashing Rust firmware while preserving bootloader area.

**Usage:**
```bash
./scripts/erase_and_flash.sh
```

**What it does:**
- Erases flash from 0x08006000 to 0x17E000 (application area only)
- Programs `blink_bare_patched.hex`
- Verifies programming
- Resets and runs

**Dependencies:**
- OpenOCD (Arduino distribution)
- `blink_bare_patched.hex` in project root
- CMSIS-DAP compatible debugger connected

### flash-app-only.sh

Flashes only test application (assumes bootloader already present).

**Purpose:** Quick flashing of test application without touching bootloader.

**Usage:**
```bash
./scripts/flash-app-only.sh
```

**What it does:**
- Erases only sectors 2-2 (application start area)
- Programs `bootloader-test-app/test-app.hex`
- Verifies and resets

**Dependencies:**
- Custom OpenOCD build in `efr32mg24-bootloader/openocd/`
- Pre-flashed bootloader at 0x08000000
- `bootloader-test-app/test-app.hex`

### flash-arduino-only.sh

Flashes Arduino blink to test hardware functionality.

**Purpose:** Hardware validation and testing using known-good Arduino firmware.

**Usage:**
```bash
./scripts/flash-arduino-only.sh
```

**What it does:**
- Programs Arduino blink hex file
- No erase operation (relies on OpenOCD auto-erase)
- Useful for verifying hardware is working

**Dependencies:**
- Custom OpenOCD build in `efr32mg24-bootloader/openocd/`
- `efr32mg24-pac/arduino_blink.hex`

**When to use:**
- Testing new hardware
- Verifying debugger connection
- Recovering from failed Rust firmware flash

### flash-bootloader-arduino.sh

Flashes bootloader and Arduino blink application together.

**Purpose:** Complete firmware stack with known-good application for testing.

**Usage:**
```bash
./scripts/flash-bootloader-arduino.sh
```

**What it does:**
- Programs bootloader at 0x08000000
- Programs Arduino blink at 0x08006000
- Verifies both images
- Resets device

**Dependencies:**
- Custom OpenOCD build in `efr32mg24-bootloader/openocd/`
- `efr32mg24-bootloader/bootloader.hex`
- `efr32mg24-pac/arduino_blink.hex`

**When to use:**
- Initial device setup
- After mass erase
- Testing bootloader functionality

### flash-bootloader-test.sh

Flashes custom bootloader and test application.

**Purpose:** Development and testing of custom Rust bootloader with test app.

**Usage:**
```bash
./scripts/flash-bootloader-test.sh
```

**What it does:**
- Programs custom Rust bootloader
- Programs custom test application
- Verifies both images
- Resets and runs

**Dependencies:**
- Custom OpenOCD build in `efr32mg24-bootloader/openocd/`
- `efr32mg24-bootloader/bootloader.hex`
- `bootloader-test-app/test-app.hex`

**When to use:**
- Bootloader development
- Testing bootloader API
- Validating application handoff

### flash-mass-erase.sh

Mass erases entire chip and flashes bootloader + application.

**Purpose:** Complete chip erase and reprogramming. Nuclear option for recovery.

**Usage:**
```bash
./scripts/flash-mass-erase.sh
```

**WARNINGS:**
- Erases ENTIRE flash from 0x08000000 to 0x08180000
- Removes all firmware, bootloader, and configuration
- Cannot be undone
- Only use when necessary

**What it does:**
- Halts processor
- Erases full flash range (1.5MB)
- Programs bootloader
- Programs test application
- Verifies and resets

**Dependencies:**
- Custom OpenOCD build in `efr32mg24-bootloader/openocd/`
- `efr32mg24-bootloader/bootloader.hex`
- `bootloader-test-app/test-app.hex`

**When to use:**
- Device completely bricked
- Flash corruption suspected
- Starting fresh development
- DO NOT use for routine flashing

## Common Issues and Troubleshooting

### OpenOCD Connection Failed

**Problem:** OpenOCD cannot connect to target

**Solutions:**
- Check USB connection
- Verify debugger is detected: `ls /dev/cu.*`
- Try different USB cable or port
- Check target power
- Verify CMSIS-DAP firmware on debugger

### Flash Verification Failed

**Problem:** Programming succeeds but verification fails

**Solutions:**
- Check power supply stability
- Reduce SWD clock speed in OpenOCD config
- Verify hex file integrity
- Try mass erase and reprogram

### Device Not Responding After Flash

**Problem:** Device does not run after flashing

**Solutions:**
- Check reset line connection
- Try manual power cycle
- Verify application entry point in hex file
- Check for stack overflow or memory corruption
- Flash Arduino blink to verify hardware

### SWD Connection Lost

**Problem:** Debug connection lost during operation

**Solutions:**
- See recovery documentation in `docs/01-getting-started/recovery/`
- Use recovery scripts in `xiao_mg24_flash_erase_macos/`
- May need to use Black Magic Probe for low-level recovery

## Flash Memory Layout

Understanding the memory layout is crucial for safe flashing:

```
0x08000000 - 0x08005FFF : Bootloader (24KB)
0x08006000 - 0x0817FFFF : Application (1496KB)
```

Always ensure your application hex files are configured for the correct base address (0x08006000 when using bootloader).

## Development Workflow

Typical development cycle:

1. Initial setup:
   ```bash
   ./scripts/flash-bootloader-arduino.sh
   ```

2. Verify hardware:
   - LED should blink
   - If not, check hardware connections

3. Build Rust application:
   ```bash
   ./scripts/build-container.sh
   ```

4. Flash your application:
   ```bash
   ./scripts/flash-app-only.sh
   ```

5. If issues occur, revert to known-good:
   ```bash
   ./scripts/flash-arduino-only.sh
   ```

## Related Documentation

- [Flashing Guide](../docs/01-getting-started/) - Comprehensive flashing procedures
- [Hardware Guide](../docs/04-reference/hardware/) - Hardware specifications
- [Recovery Guide](../docs/01-getting-started/recovery/) - Device recovery procedures
- [Bootloader Documentation](../docs/02-architecture/) - Bootloader architecture

## Script Maintenance

When modifying scripts:
- Test on actual hardware before committing
- Update this documentation
- Verify all paths are correct
- Check OpenOCD configuration compatibility
- Consider backward compatibility

---

*Last updated: 2026-05-16*
