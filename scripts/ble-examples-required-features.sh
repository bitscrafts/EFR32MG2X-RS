#!/usr/bin/env bash
# Build the three BLE examples that declare required-features, with those features, and report the load range.
set -u
cd /projects/efr32mg24-rs || exit 1
TARGET=thumbv8m.main-none-eabihf
OUT=/projects/_cache/targets/$TARGET/release/examples

build_one() {
  local pkg="$1" ex="$2" feats="$3"
  printf "%-24s " "$ex"
  if cargo build --release --target "$TARGET" -p "$pkg" --example "$ex" --features "$feats" >/tmp/b.log 2>&1; then
    if [ -f "$OUT/$ex" ]; then
      echo "OK   $(python3 scripts/elf-load-range.py "$OUT/$ex" | head -1)"
    else
      echo "OK   (no ELF produced)"
    fi
  else
    echo "FAIL $(grep -m1 -oE 'error(\[E[0-9]+\])?:.{0,70}' /tmp/b.log)"
  fi
}

build_one efr32mg24-radio ble_gatt "ble,gatt,stack"
build_one xiao-mg24-bsp module_ble_motion "sense,stack"
build_one xiao-mg24-bsp module_ble_fall "sense,stack"
