#!/bin/bash
# OpenOCD debug session to check if Rust code runs

~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "reset halt" \
  -c "reg" \
  -c "mdw 0x08000000 4" \
  -c "mdw 0x08006000 4" \
  -c "mdw 0x08006220 8" \
  -c "reg pc" \
  -c "exit"
