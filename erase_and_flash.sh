#!/bin/bash
# Erase application area and flash Rust firmware

~/Library/Arduino15/packages/SiliconLabs/tools/openocd/0.12.0-arduino1-static/bin/openocd \
  -f interface/cmsis-dap.cfg \
  -f target/efm32s2_g23.cfg \
  -c "init" \
  -c "halt" \
  -c "flash erase_address 0x08006000 0x178000" \
  -c "program blink_bare_patched.hex verify" \
  -c "reset run" \
  -c "exit"
