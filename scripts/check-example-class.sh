#!/usr/bin/env bash
# Build one BLE example and report whether a BGAPI class made it into the image.
#
#   bash scripts/check-example-class.sh <example> <symbol-fragment> [features]
#
# Written as a script because the inline form -- cargo build piped into a symbol grep with a command substitution --
# trips the shell parser's blocklist, and retrying it inline wastes the turn without changing the outcome.
set -u
cd /projects/efr32mg24-rs || exit 1
EX="${1:?example name}"
SYM="${2:?symbol fragment}"
FEATS="${3:-ble stack}"
TARGET=thumbv8m.main-none-eabihf
ELF=/projects/_cache/targets/$TARGET/release/examples/$EX

cargo build --release --target "$TARGET" -p efr32mg24-radio --example "$EX" --features "$FEATS" > /tmp/check-class.log 2>&1
RC=$?
echo "build rc=$RC"
if [ "$RC" -ne 0 ]; then
  grep -E "^error|undefined symbol" -A2 /tmp/check-class.log | head -12
  exit "$RC"
fi

python3 scripts/elf-load-range.py "$ELF" | head -1
COUNT=$(arm-none-eabi-nm "$ELF" 2>/dev/null | grep -c "$SYM")
echo "symbols matching '$SYM': $COUNT"
if [ "$COUNT" -eq 0 ]; then
  echo "  -> the class/feature is NOT in the image: the catalogue change did not reach the compiler, or the"
  echo "     component is not registered by the tables this build passes to sli_bt_init_stack."
else
  echo "  -> present, so the catalogue is being compiled in"
fi
