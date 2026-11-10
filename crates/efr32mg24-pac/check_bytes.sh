#!/bin/bash
# Check ApplicationProperties byte-by-byte
~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "reset halt" \
  -c "mdb 0x08006220 16" \
  -c "exit"
