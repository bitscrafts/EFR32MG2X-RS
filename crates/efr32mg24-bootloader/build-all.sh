#!/bin/bash
# Build bootloader and test application, combine and flash

set -e

echo "=== Building EFR32MG24 Custom Bootloader ==="

# Build bootloader
echo "Building bootloader..."
cargo build --release

# Build test application with custom memory layout
echo "Building test application..."
RUSTFLAGS="-C link-arg=-Ttest-app-memory.x" \
  cargo build --example test-blink-app --release

# Convert to Intel HEX
echo "Converting to Intel HEX..."
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader \
  bootloader.hex

arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/examples/test-blink-app \
  test-app.hex

# Combine bootloader + application
echo "Combining bootloader and application..."
if command -v srec_cat &> /dev/null; then
    srec_cat bootloader.hex -Intel test-app.hex -Intel -o combined.hex -Intel
    echo "Created combined.hex"
else
    echo "Warning: srec_cat not found, skipping combine step"
    echo "Install with: brew install srecord"
fi

# Show sizes
echo ""
echo "=== Build Sizes ==="
echo "Bootloader:"
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader

echo ""
echo "Test Application:"
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/examples/test-blink-app

echo ""
echo "=== Build Complete ==="
echo "Bootloader: bootloader.hex"
echo "Application: test-app.hex"
echo "Combined: combined.hex"
echo ""
echo "To flash, run: ./flash.sh"
