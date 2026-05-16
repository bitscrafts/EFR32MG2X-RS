#!/bin/bash
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "program test_blink_c/build/SiliconLabs.silabs.xiao_mg24/test_blink_c.ino.hex verify" \
  -c "reset run" \
  -c "exit"
