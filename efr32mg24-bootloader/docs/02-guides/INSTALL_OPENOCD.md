# OpenOCD for XIAO MG24 Flashing

## Local OpenOCD (Included)

This project includes a local copy of OpenOCD in the `openocd/` directory.

No installation required - the flash scripts use the local OpenOCD binary.

### Source

Copied from Arduino IDE installation:
- OpenOCD 0.12.0+dev-01514-g21fa2de70 (2024-02-07-19:03)
- Location: `./openocd/bin/openocd`
- Config files: `./openocd/scripts/`

### Verify Local OpenOCD

```bash
./openocd/bin/openocd --version
```

Should show:
```
Open On-Chip Debugger 0.12.0+dev-01514-g21fa2de70 (2024-02-07-19:03)
```

## Alternative: System-wide Install (Optional)

If you want to install OpenOCD system-wide (not required):

```bash
brew install openocd
```

## Test Connection

```bash
cd efr32mg24-bootloader
openocd -f interface/cmsis-dap.cfg -f ../efr32mg24.cfg -c "init" -c "exit"
```

Expected output should include:
```
Info : CMSIS-DAP: ...
Info : clock speed 10000 kHz
Info : SWD DPIDR 0x6ba02477
```

## After Installation

Once OpenOCD is installed, run:

```bash
./flash-now.sh
```

This will flash the bootloader and application to your device.

## Alternative: probe-rs

If you prefer, you can also use probe-rs (modern Rust-based tool):

```bash
cargo install probe-rs --locked
```

Then flash with:
```bash
probe-rs download --chip EFR32MG24B220F1536IM48 bootloader.hex
probe-rs download --chip EFR32MG24B220F1536IM48 test-app.hex
probe-rs reset
```
