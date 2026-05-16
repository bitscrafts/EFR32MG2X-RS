#!/bin/bash
# Flash working Arduino blink example
# Use this to restore verified working firmware

set -e

OPENOCD="./openocd/bin/openocd"
OPENOCD_SCRIPTS="./openocd/scripts"
BLINK_HEX="arduino-working-examples/arduino_blink.hex"

echo "=== Flashing Arduino Blink Example (Verified Working) ==="
echo ""

if [ ! -f "$BLINK_HEX" ]; then
    echo "Error: $BLINK_HEX not found"
    exit 1
fi

echo "Flashing Arduino blink example..."
$OPENOCD -s "$OPENOCD_SCRIPTS" \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program $BLINK_HEX verify; reset; exit"

echo ""
echo "=== Flash Complete! ==="
echo ""
echo "LED should blink at 1 Hz"
echo ""
