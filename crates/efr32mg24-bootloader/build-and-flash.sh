#!/bin/bash
set -e

echo "=== Building Custom Bootloader + Test Application ==="
echo ""

# Build bootloader
echo "Building bootloader..."
cd /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/efr32mg24-bootloader
cargo build --release --target thumbv8m.main-none-eabihf
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader \
  bootloader.hex

echo ""
echo "Bootloader sections:"
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/efr32mg24-bootloader
echo "✓ Bootloader: $(ls -lh bootloader.hex | awk '{print $5}')"

# Build test app
echo ""
echo "Building test app..."
cd /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/bootloader-test-app
cargo build --release --target thumbv8m.main-none-eabihf
arm-none-eabi-objcopy -O ihex \
  target/thumbv8m.main-none-eabihf/release/bootloader-test-app \
  test-app.hex

echo ""
echo "Test app sections:"
arm-none-eabi-size target/thumbv8m.main-none-eabihf/release/bootloader-test-app
echo "✓ Test app: $(ls -lh test-app.hex | awk '{print $5}')"

echo ""
echo "Hex file lines:"
echo "  Bootloader: $(wc -l < /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/efr32mg24-bootloader/bootloader.hex) lines"
echo "  Test app:   $(wc -l < test-app.hex) lines"

echo ""
echo "=== Flashing to Hardware ==="
echo ""

# Flash bootloader
echo "Flashing bootloader..."
cd /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/efr32mg24-bootloader
./openocd/bin/openocd -s ./openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program bootloader.hex verify; exit" \
  2>&1 | grep -E "(Programming|Verified|Error)" || true

# Flash test app
echo ""
echo "Flashing test app..."
cd /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/bootloader-test-app
/Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/efr32mg24-bootloader/openocd/bin/openocd \
  -s /Users/${FLASH_USER:-$(id -un)}/Private/20250626_rust_projects/2025-10-29/EFR32MG24/efr32mg24-bootloader/openocd/scripts \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init; reset halt; program test-app.hex verify; reset run; exit" \
  2>&1 | grep -E "(Programming|Verified|Error)" || true

echo ""
echo "=== Flash Complete! ==="
echo ""
echo "OBSERVE THE LED:"
echo "  - Blinking at 1 Hz (500ms ON/OFF) = SUCCESS!"
echo "  - Blinking FAST (~100ms) = HardFault"
echo "  - Blinking SLOW (~1 second) = Bootloader mode"
echo "  - Solid ON = Code stuck (check with debugger)"
echo "  - Not blinking = Check connection"
echo ""
