#!/bin/bash
# Flash working Arduino volume example
# Use this to restore verified working firmware

set -e

OPENOCD="./openocd/bin/openocd"
OPENOCD_SCRIPTS="./openocd/scripts"
VOLUME_HEX="arduino-working-examples/microphone_volume_with_bootloader.hex"

echo "=== Flashing Arduino Volume Example (Verified Working) ==="
echo ""

if [ ! -f "$VOLUME_HEX" ]; then
    echo "Error: $VOLUME_HEX not found"
    exit 1
fi

echo "Flashing Arduino volume example..."
$OPENOCD -s "$OPENOCD_SCRIPTS" \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program $VOLUME_HEX verify; reset; exit"

echo ""
echo "=== Flash Complete! ==="
echo ""
echo "LED should respond to microphone input"
echo ""
