#!/bin/bash
# Flash bootloader + Arduino blink at 0x08006000

OPENOCD="crates/efr32mg24-bootloader/openocd/bin/openocd"
SCRIPTS="crates/efr32mg24-bootloader/openocd/scripts"

$OPENOCD -s $SCRIPTS \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "program crates/efr32mg24-bootloader/bootloader.hex verify" \
  -c "program crates/efr32mg24-pac/arduino_blink.hex verify" \
  -c "reset run" \
  -c "exit"
