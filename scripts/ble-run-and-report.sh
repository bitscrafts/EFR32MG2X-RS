#!/usr/bin/env bash
# Flash one BLE example and report what the mailbox says, plus RAIL's assert slot if it wrote there.
#
# Usage: bash scripts/ble-run-and-report.sh <package> <example> [features]
set -u
cd /projects/efr32mg24-rs || exit 1

PKG="$1"
EX="$2"
FEATS="${3:-}"
TARGET=thumbv8m.main-none-eabihf
ELF=/projects/_cache/targets/$TARGET/release/examples/$EX

if [ -n "$FEATS" ]; then
  cargo build --release --target "$TARGET" -p "$PKG" --example "$EX" --features "$FEATS" >/tmp/br.log 2>&1 || {
    echo "build failed:"; tail -5 /tmp/br.log; exit 1; }
else
  cargo build --release --target "$TARGET" -p "$PKG" --example "$EX" >/tmp/br.log 2>&1 || {
    echo "build failed:"; tail -5 /tmp/br.log; exit 1; }
fi

echo "== $PKG/$EX"
python3 scripts/elf-load-range.py "$ELF" | head -1
FLASH_HOST=10.10.10.217 FLASH_USER=mvcorrea bash scripts/flash.sh "$ELF" --confirm --run 2>&1 | grep -E "Verification|FAILED" || {
  echo "flash failed"; exit 1; }

sleep 8
echo "-- mailbox:"
bash scripts/read-mailbox.sh 2>/dev/null | head -3
echo "-- RAIL assert slot (magic, code, handle, count):"
ssh mvcorrea@10.10.10.217 'sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg -c "init; halt" -c "mdw 0x2003F500 4" -c "resume; shutdown" 2>&1 | grep 0x2003f500'
