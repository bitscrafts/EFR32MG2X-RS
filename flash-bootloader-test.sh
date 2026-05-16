#!/bin/bash
# Flash custom bootloader and test application

OPENOCD="efr32mg24-bootloader/openocd/bin/openocd"
SCRIPTS="efr32mg24-bootloader/openocd/scripts"

$OPENOCD -s $SCRIPTS \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "program efr32mg24-bootloader/bootloader.hex verify" \
  -c "program bootloader-test-app/test-app.hex verify" \
  -c "reset run" \
  -c "exit"
