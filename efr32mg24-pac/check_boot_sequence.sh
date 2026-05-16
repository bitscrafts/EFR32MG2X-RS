#!/bin/bash
# Connect without halting to see where PC naturally goes

~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "sleep 2000" \
  -c "halt" \
  -c "reg pc" \
  -c "bt" \
  -c "exit"
