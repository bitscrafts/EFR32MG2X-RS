#!/usr/bin/env bash
# Which BLE examples build, and which need the vendored stack?
#
# Every one is compiled for the real target, because an example that only builds for the host proves nothing about
# this board. The feature that pulls in the Bluetooth host blobs is `stack`; an example that fails without it is
# tried with it, and that is recorded as the reason rather than hidden.
set -u

ROOT=/projects/efr32mg24-rs
cd "$ROOT" || exit 1
TARGET=thumbv8m.main-none-eabihf
OUT=/projects/_cache/targets/$TARGET/release/examples

EXAMPLES="
efr32mg24-rail-sys:rail_ble_tx
efr32mg24-rail-sys:rail_ble_rx
efr32mg24-rail-sys:rail_caps
efr32mg24-radio:ble_advertise_custom
efr32mg24-radio:ble_beacon
efr32mg24-radio:ble_scanner
efr32mg24-radio:ble_gatt
efr32mg24-radio:ble_boot_probe
efr32mg24-bluetooth-sys:bt_version
xiao-mg24-bsp:module_ble_beacon
xiao-mg24-bsp:module_ble_motion
xiao-mg24-bsp:module_ble_fall
xiao-expansion-base-bsp:base_ble_env
"

printf "%-22s %-22s %-8s %-9s %s\n" PACKAGE EXAMPLE FEATURES "SIZE" "LOAD RANGE"
for entry in $EXAMPLES; do
  pkg=${entry%%:*}
  ex=${entry##*:}
  built=""
  used=""
  for feat in "" "stack"; do
    if [ -z "$feat" ]; then
      args=(--release --target "$TARGET" -p "$pkg" --example "$ex")
    else
      args=(--release --target "$TARGET" -p "$pkg" --example "$ex" --features stack)
    fi
    if cargo build "${args[@]}" >/tmp/blebuild.log 2>&1; then
      built="$OUT/$ex"
      used=${feat:-none}
      break
    fi
  done

  if [ -z "$built" ] || [ ! -f "$built" ]; then
    printf "%-22s %-22s %-8s %-9s %s\n" "$pkg" "$ex" "-" "FAIL" "$(grep -m1 -oE 'error(\[E[0-9]+\])?:.*' /tmp/blebuild.log | cut -c1-60)"
    continue
  fi

  size=$(stat -c%s "$built")
  range=$(python3 "$ROOT/scripts/elf-load-range.py" "$built" 2>/dev/null | head -1)
  printf "%-22s %-22s %-8s %-9s %s\n" "$pkg" "$ex" "$used" "$size" "${range:-no range}"
done
