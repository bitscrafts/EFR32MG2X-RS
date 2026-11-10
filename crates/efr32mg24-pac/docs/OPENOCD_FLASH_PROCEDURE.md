# OpenOCD Flash Procedure for XIAO MG24 Sense

## Hardware Setup

- **Board**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
- **Debugger**: CMSIS-DAP interface (built into XIAO board)
- **Connection**: USB-C cable

## Prerequisites

- OpenOCD installed (Arduino Silicon Labs package includes it)
- CMSIS-DAP driver working
- Board connected via USB

## OpenOCD Paths

**OpenOCD Binary**:
```
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd
```

**Config Files**:
```
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/share/openocd/scripts/interface/cmsis-dap.cfg
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/share/openocd/scripts/target/efm32s2_g23.cfg
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/share/openocd/scripts/target/efm32s2.cfg
```

## OpenOCD Configuration Files Explained

### interface/cmsis-dap.cfg
```tcl
# ARM CMSIS-DAP compliant adapter
adapter driver cmsis-dap

# Optionally specify the serial number of CMSIS-DAP usb device.
# adapter serial 02200201E6661E601B98E3B9
```

**Purpose**: Configures OpenOCD to use the CMSIS-DAP debug adapter protocol (built into XIAO MG24).

### target/efm32s2_g23.cfg
```tcl
# Silicon Labs (formerly Energy Micro) EFM32 target
# Family group 22 has flash base address 0x00000000
# Family group 23 has flash base address 0x08000000
set FLASHBASE 0x08000000
source [find target/efm32s2.cfg]
```

**Purpose**: Sets the flash base address to **0x08000000** for EFR32 family group 23 (includes EFR32MG24).

**Critical Detail**: This is why our flash starts at 0x08000000, NOT 0x00000000 (like older EFM32 devices).

### target/efm32s2.cfg

**Purpose**: Main EFM32 Series 2 target configuration:
- Sets up SWD/JTAG debugging interface
- Configures Cortex-M33 target (`cortex_m`)
- Defines flash bank at `$_FLASHBASE` (0x08000000)
- Implements DCI (Debug Communication Interface) for Series 2
- Sets adapter speed to 1000 kHz
- Configures work area in RAM at 0x20000000 (2 KB default)

**Key Parameters**:
- `adapter speed 1000` - 1 MHz SWD clock
- `target create efm32s2.cpu cortex_m` - Cortex-M target
- `flash bank efm32s2.flash efm32s2 0x08000000` - Main flash at 0x08000000
- `flash bank userdata.flash efm32s2 0x0FE00000` - User data flash

**DCI Commands Available**:
- `efm32s2_dci_read_se_status` - Read Secure Element status
- `efm32s2_dci_device_erase` - Full chip erase (if debug locked)
- `efm32s2_dci_device_lock` - Enable debug lock

## Flash Commands

### Flash Arduino Sketch (MicrophoneVolume Example)

**CRITICAL: Use Arduino IDE, NOT arduino-cli**

Arduino IDE builds work correctly, but arduino-cli builds produce non-functional binaries.

**Evidence**:
- Arduino IDE build: 245,276 bytes (15% flash usage) - **WORKS**
- arduino-cli build: 915,300 bytes (58% flash usage) - **DOES NOT WORK**

The size difference indicates arduino-cli is compiling with different flags, libraries, or optimization settings. **Under investigation**.

**Step 1: Build in Arduino IDE**
1. Open Arduino IDE (GUI application)
2. `File → Examples → SilabsMicrophoneAnalog → MicrophoneVolume`
3. `Tools → Board → Silicon Labs → XIAO MG24 Sense`
4. Enable verbose output: `File → Preferences → Show verbose output during: compilation` (check box)
5. Click **Verify** (checkmark button) to compile
6. In the output, find the build path (look for lines like):
   ```
   /var/folders/.../arduino_build_.../MicrophoneVolume.ino.hex
   ```
7. Copy this path for use in Step 2

**Step 2: Flash with OpenOCD**
```bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "program /path/to/MicrophoneVolume.ino.hex verify reset exit"
```

**Alternative: Use Arduino IDE Upload**
The simplest method is to just click **Upload** in Arduino IDE, which handles everything automatically.

### Flash Rust Bare-Metal Example (blink_pac)

**Step 1: Build the Rust example**
```bash
cargo build --release --example blink_pac --features rt
```

**Step 2: Convert to hex**
```bash
cargo objcopy --release --example blink_pac --features rt -- -O ihex blink.hex
```

**Step 3: Flash with OpenOCD**
```bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "program blink.hex verify reset exit"
```

## Memory Layout

### Arduino Application Memory Map

Based on the hex file analysis:
- **Bootloader**: 0x08000000 - 0x08005FFF (24 KB)
- **Application**: 0x08006000 - 0x0817FFFF (1512 KB)

### Rust Bare-Metal Memory Map

For Rust bare-metal applications (see `memory.x`):
- **Bootloader**: 0x08000000 - 0x08003FFF (16 KB) - Reserved for Gecko bootloader
- **Application**: 0x08004000 - 0x0817FFFF (1520 KB)

**Note**: Arduino uses a 24 KB bootloader, Rust uses 16 KB reservation. This may need adjustment.

## Troubleshooting

### USB Transfer Errors

You may see these errors during flashing:
```
Error: busy command USB transfer at 0
Error: USB write: late transfer competed
```

**These are usually harmless** - if you see `** Verified OK **`, the flash was successful.

### Double Fault / Lockup

If you see:
```
Error: [efm32s2.cpu] clearing lockup after double fault
xPSR: 0xf9000000 pc: 0xfffffffe msp: 0xfffffffc
```

**Recovery**: Flash a known-good Arduino sketch to restore the bootloader, then try again.

### Complete Bootloader Recovery Procedure

**TESTED AND VERIFIED - Use this if chip is in fault state**

If you see `pc: 0xfffffffe` (corrupted/fault state), follow these steps:

**Step 1: Flash the Gecko Bootloader**
```bash
# Using local copy in project root
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "program xiao_mg24_bootloader.hex verify reset exit"
```

Or using full path:
```bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "program /Users/mvcorrea/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/bootloaders/seeed-studio-xiao-mg24-bootloader-storage-internal-single-512k.hex verify reset exit"
```

**Expected output**: `** Verified OK **` (ignore pthread mutex assertion error at the end)

**Step 2: Build Arduino Sketch in IDE**
1. Open Arduino IDE
2. Load MicrophoneVolume example
3. Select XIAO MG24 board
4. Click **Verify** to build (or use **Upload** to flash directly)
5. Note the hex file path from verbose output

**Step 3: Flash the Application**
```bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "program /path/to/MicrophoneVolume.ino.hex verify reset exit"
```

**Step 4: Verify Recovery**
- After reset, check that PC is at valid address (e.g., `pc: 0x08000170`)
- **NOT** at fault address `pc: 0xfffffffe`
- Test that volume example works (LED responds to microphone)

**Bootloader File Location**:
```
/Users/mvcorrea/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/bootloaders/
seeed-studio-xiao-mg24-bootloader-storage-internal-single-512k.hex
```
(30 KB file)

**Local copies in project root** (for convenience):
- `xiao_mg24_bootloader.hex` - Gecko bootloader (30 KB)
- `microphone_volume_with_bootloader.hex` - Working Arduino volume example (2.5 MB, arduino-cli build)
- `microphone_volume_arduinocli.hex` - App-only hex without bootloader (2.5 MB)
- `interface/cmsis-dap.cfg` - CMSIS-DAP interface config
- `target/efm32s2_g23.cfg` - EFR32MG24 target config (sets FLASHBASE)
- `target/efm32s2.cfg` - Main Series 2 target config

## Simplified OpenOCD Command

The key OpenOCD command pattern:
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "program <hexfile> verify reset exit"
```

Where:
- `program` - Programs the flash
- `verify` - Verifies the flash contents
- `reset` - Resets the target after programming
- `exit` - Exits OpenOCD after completion

## Testing

### Arduino MicrophoneVolume

**Expected behavior**: LED dims when quiet, brightens (amber) with noise/sound.

### Rust Blink

**Expected behavior**: LED blinks at approximately 1 Hz (on/off every ~0.5 seconds).

## Notes

- The XIAO MG24 has built-in CMSIS-DAP debugger (no external debugger needed)
- OpenOCD works reliably despite occasional USB transfer warnings
- Always verify the `** Verified OK **` message to confirm successful flash
- Memory layout differs between Arduino (24 KB bootloader) and Rust (16 KB reservation)

---

**Last Updated**: 2026-01-04
**Tested With**: OpenOCD 0.12.0+dev-01514-g21fa2de70
