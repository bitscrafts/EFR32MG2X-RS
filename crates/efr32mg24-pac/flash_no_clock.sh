#!/bin/bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "program blink_no_clock_patched.hex verify" \
  -c "reset run" \
  -c "exit"
