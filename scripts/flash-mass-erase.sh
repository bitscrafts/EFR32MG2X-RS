#!/bin/bash
# Mass erase chip and flash bootloader + application

OPENOCD="crates/efr32mg24-bootloader/openocd/bin/openocd"
SCRIPTS="crates/efr32mg24-bootloader/openocd/scripts"

$OPENOCD -s $SCRIPTS \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "halt" \
  -c "flash erase_address 0x08000000 0x180000" \
  -c "program crates/efr32mg24-bootloader/bootloader.hex verify" \
  -c "program crates/bootloader-test-app/test-app.hex verify" \
  -c "reset run" \
  -c "exit"
