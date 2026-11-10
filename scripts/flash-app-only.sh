#!/bin/bash
# Flash only test application (bootloader already present)

OPENOCD="crates/efr32mg24-bootloader/openocd/bin/openocd"
SCRIPTS="crates/efr32mg24-bootloader/openocd/scripts"

$OPENOCD -s $SCRIPTS \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "halt" \
  -c "flash erase_sector 0 2 2" \
  -c "program crates/bootloader-test-app/test-app.hex verify" \
  -c "reset run" \
  -c "exit"
