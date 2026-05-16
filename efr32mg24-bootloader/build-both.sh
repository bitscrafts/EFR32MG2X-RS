#!/bin/bash
set -e

echo "=== Building Custom Bootloader + Test Application ==="
echo ""

# Build bootloader first (uses bootloader.x)
echo "Step 1: Building bootloader..."
sed -i.bak 's/test-app-memory\.x/bootloader.x/' .cargo/config.toml
cargo build --release --target thumbv8m.main-none-eabihf
arm-none-eabi-objcopy -O ihex ../target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader bootloader.hex
echo "✓ Bootloader built: $(ls -lh bootloader.hex | awk '{print $5}')"

# Build test application (uses test-app-memory.x)
echo ""
echo "Step 2: Building test application..."
sed -i.bak 's/bootloader\.x/test-app-memory.x/' .cargo/config.toml
touch examples/test-blink-app.rs
cargo build --release --example test-blink-app --target thumbv8m.main-none-eabihf
arm-none-eabi-objcopy -O ihex ../target/thumbv8m.main-none-eabihf/release/examples/test-blink-app test-app.hex
echo "✓ Test app built: $(ls -lh test-app.hex | awk '{print $5}')"

# Restore original config (bootloader.x for main binary)
mv .cargo/config.toml.bak .cargo/config.toml

echo ""
echo "=== Build Complete! ==="
echo ""
echo "Files ready for flashing:"
echo "  - bootloader.hex (792 bytes at 0x08000000)"
echo "  - test-app.hex (7.8 KB at 0x08004000)"
echo ""
