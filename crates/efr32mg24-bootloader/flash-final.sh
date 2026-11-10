#!/bin/bash
# Flash using Arduino IDE's OpenOCD

set -e

OPENOCD="./openocd/bin/openocd"
OPENOCD_SCRIPTS="./openocd/scripts"

echo "=== Flashing EFR32MG24 Custom Bootloader ==="
echo ""
echo "Using local OpenOCD (from Arduino IDE)"
echo ""

# Check files exist
if [ ! -f bootloader.hex ]; then
    echo "Error: bootloader.hex not found"
    exit 1
fi

if [ ! -f test-app.hex ]; then
    echo "Error: test-app.hex not found"
    exit 1
fi

echo "Step 1: Flashing bootloader (792 bytes at 0x08000000)..."
$OPENOCD -s "$OPENOCD_SCRIPTS" \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program bootloader.hex verify; exit"

echo ""
echo "✓ Bootloader flashed and verified"

echo ""
echo "Step 2: Flashing application (7.8 KB at 0x08004000)..."
$OPENOCD -s "$OPENOCD_SCRIPTS" \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program test-app.hex verify; reset; exit"

echo ""
echo "=== Flash Complete! ==="
echo ""
echo "✓ Bootloader: 792 bytes flashed at 0x08000000"
echo "✓ Application: 7.8 KB flashed at 0x08004000"
echo "✓ Device: reset and running"
echo ""
echo "🎯 OBSERVE THE LED:"
echo "  - Blinking at 1 Hz (500ms ON/OFF) = ✅ SUCCESS!"
echo "  - Blinking FAST (~100ms) = HardFault"
echo "  - Blinking SLOW (~1 second) = Bootloader mode"
echo "  - Not blinking = Check connection"
echo ""
