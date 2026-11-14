#!/usr/bin/env bash
# Read the fall detector's state from the board over SWD and decode it.
#
# What is read, and why the addresses are looked up instead of baked in:
#
#   DTS      4000 x u8   interval before each stored sample, 250 us units
#   LOG      64 x 12 B   events: uptime u32, freefall u16, peak u16, kind u8
#   counters u32 each    FALLS, IMPACTS, WRITTEN, AIRBORNE
#   CAPTURE  4000 x 6 B  the waveform ring
#   TOTAL    u32         samples written since boot
#
# Every one of these moves when a static is added or removed, so the script reads them
# from the ELF it is pointed at. Baked-in addresses would silently read the wrong thing.
#
# Usage: scripts/read-fall-log.sh [host] [elf]
set -euo pipefail

HOST="${1:-${FLASH_HOST:-localhost}}"
ELF="${2:-/projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/base_fall_detector}"
ODD=/tmp/openocd-mg24-mini.cfg
OUT=/tmp/fall-read
mkdir -p "$OUT"

addr() { arm-none-eabi-nm "$ELF" | grep -E "$1" | head -1 | cut -d' ' -f1; }
DTS=$(addr recording3DTS)
LOG=$(addr events3LOG)
FALLS=$(addr events5FALLS)
CAPTURE=$(addr recording7CAPTURE)
TOTAL=$(addr recording5TOTAL)

echo "elf      $ELF"
echo "DTS      0x$DTS"
echo "LOG      0x$LOG"
echo "FALLS    0x$FALLS"
echo "CAPTURE  0x$CAPTURE"
echo "TOTAL    0x$TOTAL"
echo

WORDS=$(
  ssh "$HOST" "sudo -n openocd -f interface/cmsis-dap.cfg -f $ODD \
    -c 'init; halt' \
    -c 'dump_image /tmp/fall_dts.bin 0x$DTS 4000' \
    -c 'dump_image /tmp/fall_log.bin 0x$LOG 768' \
    -c 'dump_image /tmp/fall_cap.bin 0x$CAPTURE 24000' \
    -c 'mdw 0x$FALLS 4' \
    -c 'mdw 0x$TOTAL 1' \
    -c 'resume; shutdown' 2>&1 | grep -E '^0x|Error'"
)
echo "$WORDS"

scp -q "$HOST:/tmp/fall_dts.bin" "$OUT/dts.bin"
scp -q "$HOST:/tmp/fall_log.bin" "$OUT/log.bin"
scp -q "$HOST:/tmp/fall_cap.bin" "$OUT/cap.bin"

TOTAL_HEX=$(printf '%s\n' "$WORDS" | awk -v a="0x$TOTAL:" '$1 == a {print $2; exit}')
TOTAL_N=$(printf '%d' "0x${TOTAL_HEX:-0}")

uv run python - "$OUT" "$TOTAL_N" <<'PY'
import struct
import sys
from pathlib import Path

out = Path(sys.argv[1])
written = int(sys.argv[2]) if len(sys.argv) > 2 else 0

print()
print("=== event log ===")
raw = (out / "log.bin").read_bytes()
KIND = {1: "AIRBORNE", 2: "impact", 3: "FALL"}
entries = [struct.unpack_from("<IHHB", raw, i * 12) for i in range(64)]
live = [e for e in entries if e[3]]
if not live:
    print("  (empty: nothing classified since boot)")
prev = None
for up, ff, pk, kind in sorted(live, key=lambda e: e[0]):
    gap = f"{(up - prev) / 1000:6.2f}s" if prev is not None else "   ---  "
    print(f"  {up / 1000:9.2f}s {gap}  free {ff:4d}ms  peak {pk:5d}mg  {KIND.get(kind, kind)}")
    prev = up

print()
print("=== recording ===")
cap = (out / "cap.bin").read_bytes()
dts = (out / "dts.bin").read_bytes()
# Only the slots that have been written: the rest are zero, and a zero sample is
# indistinguishable from free fall — the first version of this script counted 937
# "free-fall samples" that were merely unwritten memory.
count = min(4000, written) if written else 4000
first = (written - count) % 4000 if written else 0
idx = [(first + i) % 4000 for i in range(count)]
samples = [struct.unpack_from("<3h", cap, i * 6) for i in idx]
intervals = [dts[i] * 250 for i in idx]

print(f"  written since boot: {written} (analysing the last {count})")
live_dt = [v for v in intervals if v]
if live_dt:
    mean = sum(live_dt) / len(live_dt)
    print(f"  interval: mean {mean:.0f} us ({1e6 / mean:.0f} Hz), max {max(live_dt)} us")
    print("  (63.75 ms means the display blocked the loop)")
else:
    print("  (no timing recorded: this image predates per-sample intervals)")

mags = [round((s[0] ** 2 + s[1] ** 2 + s[2] ** 2) ** 0.5) for s in samples]
print(f"  magnitude: min {min(mags)} mg, max {max(mags)} mg")
print(f"  below 400 mg: {sum(1 for m in mags if m < 400)} samples")
print(f"  above 2000 mg: {sum(1 for m in mags if m > 2000)} samples")
PY
