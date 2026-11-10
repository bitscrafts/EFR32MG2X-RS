# Bootloader Test Application

This is a standalone test application designed to verify the custom Rust bootloader.

## Purpose

Tests that the custom bootloader at 0x08000000-0x08003FFF can successfully:
1. Validate the application vector table
2. Set VTOR to 0x08004000
3. Jump to the application code
4. Execute GPIO initialization and LED blinking

## Memory Layout

```
0x08000000 - 0x08003FFF : Bootloader (16 KB, 792 bytes used)
0x08004000 - 0x0817FFFF : Application (this code)
0x20000000 - 0x2003FFFF : RAM (256 KB)
```

## Build

This is a **separate crate** from the bootloader, not part of the workspace:

```bash
cd bootloader-test-app
cargo build --release --target thumbv8m.main-none-eabihf
```

## Flash

Individual flash:
```bash
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/bootloader-test-app \
  test-app.hex

# From bootloader directory:
../efr32mg24-bootloader/openocd/bin/openocd \
  -s ../efr32mg24-bootloader/openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program test-app.hex verify; reset run; exit"
```

Or use the complete build-and-flash script from bootloader directory:
```bash
cd ../efr32mg24-bootloader
./build-and-flash.sh
```

## Expected Behavior

After flashing both bootloader and application:

**SUCCESS**: LED blinks at 1 Hz (500ms ON, 500ms OFF)
- Bootloader validated application
- VTOR set to 0x08004000
- Application running correctly

**FAILURE modes**:
- Fast blink (~100ms) = HardFault in application
- Slow blink (~1 second) = Bootloader mode (app validation failed)
- Solid ON = Application stuck (code running but not looping)
- OFF = No power or connection issue

## Code Sections

```bash
$ arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/bootloader-test-app
   text    data     bss     dec     hex filename
   2248       0       0    2248     8c8 bootloader-test-app
```

## Vector Table

Located at 0x08004000:
- Stack pointer: 0x20000400
- Reset vector: 0x08004801 (points to Reset handler with Thumb bit)
- Exception handlers: Standard Cortex-M33 layout

## Dependencies

- cortex-m 0.7 - Core ARM Cortex-M functions
- cortex-m-rt 0.7 - Cortex-M startup runtime
- panic-halt 0.2 - Halt on panic

## Technical Details

### Startup Sequence

1. Bootloader at 0x08000000 executes first
2. Bootloader validates application vector table:
   - Stack pointer is in RAM (0x20000000-0x20040000)
   - Reset vector is in flash with Thumb bit set
3. Bootloader sets VTOR = 0x08004000
4. Bootloader jumps to application reset handler
5. cortex-m-rt initializes .bss and .data sections
6. cortex-m-rt calls main()
7. main() initializes GPIO and enters blink loop

### GPIO Configuration

- CMU unlock: Write 0x93F7 to 0x40008010
- Enable GPIO clock: Set bit 26 of CMU_CLKEN0 (0x40008064)
- Configure PA7: Set bits 31:28 of GPIO_PORTA_MODEL (0x5003C034) to 0x4
- Toggle LED: Write to GPIO_PORTA_DOUT (0x5003C03C)
  - 0x00 = LED ON (active-low)
  - 0x80 = LED OFF (bit 7 high)

### Delay Implementation

Busy-wait loop with approximate timing:
- 39 MHz CPU / 4 cycles per loop ≈ 10k NOPs per millisecond
- 500ms delay = 5,000,000 NOP instructions

## Troubleshooting

### LED doesn't blink

1. Verify flash contents:
```bash
# Check application vector table
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; halt; mdw 0x08004000 8; exit"
```

2. Check PC register:
```bash
openocd -f interface/cmsis-dap.cfg -f target/efm32s2_g23.cfg \
  -c "init; halt; reg pc; exit"
```
Should be in range 0x08004000-0x08004FFF

3. Restore Arduino example:
```bash
cd ../efr32mg24-bootloader
./flash-arduino-volume.sh
```

### Build errors

If you see "region FLASH already defined":
- Ensure .cargo/config.toml does NOT specify `-Tlink.x`
- cortex-m-rt automatically adds it

If you see "workspace" errors:
- This crate must be excluded from workspace
- Check parent Cargo.toml has `exclude = ["bootloader-test-app"]`

## Success Criteria

Build verification:
```bash
$ arm-none-eabi-size target/.../bootloader-test-app
   text	   data	    bss
   2248	      0	      0  # NON-ZERO text section = SUCCESS
```

Hex file verification:
```bash
$ wc -l test-app.hex
     144  # More than 1 line = SUCCESS
```

Runtime verification:
- LED blinking at 1 Hz = COMPLETE SUCCESS
