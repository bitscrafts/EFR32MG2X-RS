#!/usr/bin/env bash
# read-rate.sh — measure how fast a counter grows, in wall-clock seconds.
#
# The board publishes a counter into a fixed RAM word; this reads that word twice with the host's clock between
# reads. No assumption about the chip's core frequency enters the arithmetic, which is the whole point: every
# earlier rate measurement divided by an assumed core clock, and this project already has a known clock
# discrepancy (delay_ms(10) taking 4.35 ms).
#
# Usage: bash scripts/read-rate.sh [seconds]     (default 5)
set -u

SECS=${1:-5}
WORD=${WORD:-0x2003f300}

read_word() {
  ssh "$HOST" "sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg \
      -c 'init; halt' -c 'mdw $WORD 1' -c 'resume; shutdown' 2>&1" \
    | awk -v w="$WORD" 'index($0, w) == 1 { print strtonum("0x" $2) }' 2>/dev/null
}

HOST=${HOST:-mvcorrea@10.10.10.217}

w1=$(ssh "$HOST" "sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg -c 'init; halt' \
      -c 'mdw $WORD 1' -c 'resume; shutdown' 2>&1" | grep "^$WORD" | awk '{print $2}')
t1=$(date +%s.%N)
sleep "$SECS"
w2=$(ssh "$HOST" "sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg -c 'init; halt' \
      -c 'mdw $WORD 1' -c 'resume; shutdown' 2>&1" | grep "^$WORD" | awk '{print $2}')
t2=$(date +%s.%N)

python3 - "$w1" "$w2" "$t1" "$t2" <<'PY'
import sys
w1, w2, t1, t2 = sys.argv[1:5]
a = int(w1, 16)
b = int(w2, 16)
dt = float(t2) - float(t1)
d = (b - a) & 0xFFFFFFFF
print(f"word {a:#010x} -> {b:#010x}   delta={d} ticks in {dt:.2f} s")
print(f"rate = {d / dt:,.0f} Hz")
for name, nominal in (("LFRCO/LFXO 32768", 32768), ("ULFRCO 1000", 1000)):
    print(f"  ratio to {name}: {(d / dt) / nominal:,.1f}x")
PY
