# Local OpenOCD for EFR32MG24

This directory contains a local copy of OpenOCD and configuration files for flashing the EFR32MG24 via CMSIS-DAP.

## Contents

```
openocd/
├── bin/
│   └── openocd                    # OpenOCD binary (v0.12.0+dev)
└── scripts/
    ├── interface/
    │   └── cmsis-dap.cfg         # CMSIS-DAP interface configuration
    └── target/
        ├── efm32s2_g23.cfg       # EFR32MG24 (family group 23) target config
        ├── efm32s2.cfg           # EFM32 Series 2 base configuration
        └── swj-dp.tcl            # SWD/JTAG Debug Port utility

```

## Source

Copied from Arduino IDE installation:
- `/Users/mvcorrea/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/`
- Version: OpenOCD 0.12.0+dev-01514-g21fa2de70 (2024-02-07-19:03)

## Usage

```bash
./openocd/bin/openocd -s openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program firmware.hex verify; reset; exit"
```

## Configuration Files

### interface/cmsis-dap.cfg
- Configures CMSIS-DAP adapter (SAMD11 on XIAO MG24)
- Standard ARM CMSIS-DAP protocol

### target/efm32s2_g23.cfg
- Sets flash base address to 0x08000000 (family group 23)
- Sources efm32s2.cfg for Series 2 configuration

### target/efm32s2.cfg
- EFM32 Series 2 target configuration
- Configures Cortex-M33 target
- Flash banking for main flash and userdata
- DCI (Debug Challenge Interface) support
- Device erase and lock procedures

### target/swj-dp.tcl
- SWD/JTAG Debug Access Point utility
- Handles transport selection (SWD for XIAO MG24)

## Hardware

- **Target**: EFR32MG24B220F1536IM48 (Cortex-M33)
- **Debugger**: SAMD11 CMSIS-DAPv2
- **Interface**: SWD (Serial Wire Debug)
- **Speed**: 1000 kHz

## License

OpenOCD is licensed under GNU GPL v2.
Configuration files are SPDX-License-Identifier: GPL-2.0-or-later
