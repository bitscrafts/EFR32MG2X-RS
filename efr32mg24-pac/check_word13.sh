#!/bin/bash
# Check Word 13 value in device memory

~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "reset halt" \
  -c "echo === Vector Table Word 13 (offset 0x34) ===" \
  -c "mdw 0x08006034 1" \
  -c "echo === ApplicationProperties Structure ===" \
  -c "mdw 0x08006220 16" \
  -c "echo === Magic bytes (should be at Word 13 pointer) ===" \
  -c "mdw 0x08006228 4" \
  -c "exit"
