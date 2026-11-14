#!/usr/bin/env bash
# Run one BLE example on the board and report what it did, from outside the board.
#
#   bash scripts/ble-example-matrix.sh <package> <example> <features>
#
# For each example this prints four things, and the point is that they are independent:
#
#   1. build rc and the load range (a flash below 0x08006000 is refused by flash.sh, not by this script);
#   2. the mailbox record, read over SWD -- what the firmware says about itself;
#   3. a host capture -- what the radio actually did;
#   4. the RSSI of the board's own address, if it appears, so a weak link is visible rather than assumed.
#
# The firmware's own report is not evidence. That is why (3) exists, and why a PASS here requires (3) to show
# something for an example whose job is to transmit.
set -u
PKG="$1"
EX="$2"
FEAT="${3:-stack}"
HOST="${FLASH_HOST:-localhost}"
USER_="${FLASH_USER:-$(id -un)}"
TARGET=thumbv8m.main-none-eabihf
OUT=/projects/_cache/targets/$TARGET/release/examples

echo "=== $PKG / $EX (features: ${FEAT:-none})"
if ! cargo build --release --target "$TARGET" -p "$PKG" --example "$EX" --features "$FEAT" > /tmp/matrix-build.log 2>&1; then
  echo "  build FAIL"
  grep -E "^error" -A2 /tmp/matrix-build.log | head -6
  exit 1
fi
ELF="$OUT/$EX"
echo "  $(python3 scripts/elf-load-range.py "$ELF" | head -1)"

FLASH_HOST="$HOST" FLASH_USER="$USER_" bash scripts/flash.sh "$ELF" --confirm --run 2>&1 | grep -qE "Verification successful" \
  || { echo "  flash FAIL"; exit 1; }
echo "  flashed and running"

sleep 7
echo -n "  mailbox: "
bash scripts/read-mailbox.sh 2>/dev/null | head -1

# The host capture, with its own self-test: a window in which nothing at all arrives proves nothing.
ssh "$USER_@$HOST" '( sudo -n timeout 12 btmon > /tmp/matrix-air.txt 2>&1 & ); sleep 2; sudo -n timeout 8 hcitool -i hci0 lescan --duplicates >/dev/null 2>&1; sleep 2' 2>/dev/null
REPORTS=$(ssh "$USER_@$HOST" 'grep -cE "LE Advertising Report" /tmp/matrix-air.txt' 2>/dev/null)
# **The board's own address and payload, not its name.** An example that advertises manufacturer data carries no
# name at all, so a name search reported "nothing on air" for three examples that were transmitting the whole time.
# That is the instrument lying, and it cost three flashes of chasing a firmware that was already correct.
BOARD=$(ssh "$USER_@$HOST" 'grep -c "E4:0A:75:13:3E:E0" /tmp/matrix-air.txt' 2>/dev/null)
NAME=$(ssh "$USER_@$HOST" 'grep -A12 "E4:0A:75:13:3E:E0" /tmp/matrix-air.txt | grep -oE "Name \([a-z]+\): .*" | head -1' 2>/dev/null)
PAYLOAD=$(ssh "$USER_@$HOST" 'grep -A12 "E4:0A:75:13:3E:E0" /tmp/matrix-air.txt | grep -oE "Data\[[0-9]+\]: [0-9a-f]+" | head -1' 2>/dev/null)
RSSI=$(ssh "$USER_@$HOST" 'grep -A6 "E4:0A:75:13:3E:E0" /tmp/matrix-air.txt | grep -oE "RSSI: -[0-9]+ dBm" | head -1' 2>/dev/null)
echo "  host: $REPORTS report(s); board $BOARD; ${NAME:-no name}; ${PAYLOAD:-no payload}; ${RSSI:-}"
