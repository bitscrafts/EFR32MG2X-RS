#!/bin/bash
# Test: Flash Rust firmware directly to 0x08000000 (no bootloader)
# This will ERASE the bootloader - for testing only!

echo "⚠️  WARNING: This will erase the Arduino bootloader!"
echo "Press Ctrl+C to cancel, or Enter to continue..."
read

~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; flash erase_sector 0 0 last; program blink_pac.hex verify; reset; exit"
