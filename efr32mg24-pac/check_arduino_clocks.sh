#!/bin/bash
# Check CMU state when Arduino starts

~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "sleep 3000" \
  -c "halt" \
  -c "echo === CMU Registers ===" \
  -c "mdw 0x50008300 8" \
  -c "echo === GPIO Registers ===" \
  -c "mdw 0x50000000 8" \
  -c "exit"
