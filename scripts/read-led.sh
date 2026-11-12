#!/usr/bin/env bash
# Read PA7's mode, output latch and pad input across several halts, and say
# exactly how far the evidence goes.
#
# THE POINT. Reading PORTA_DOUT proves the firmware executed the toggle
# instruction -- it does NOT prove the pin moved, because a pin left in
# "disabled" mode has a DOUT bit that toggles while the pad never does. That
# gap is precisely how a blink can "flash fine" and still not light anything.
#
# Three registers close most of the gap:
#   PORTA_MODEL (0x034) bit field mode7  -> 4 = push-pull output
#   PORTA_DOUT  (0x040) bit 7            -> what the CPU drives
#   PORTA_DIN   (0x044) bit 7            -> what the pad actually reads
#
# If mode7 == 4 and DIN bit 7 follows DOUT bit 7 across samples, the pad is
# being driven and the MCU side is proven. What is still NOT proven at that
# point: that the LED is on PA7 on this board, and that the LED itself works.
# Only light can prove light -- an external sensor, a photo, or an eye.
#
# READ ONLY. Nothing is written to the target.
#
# Usage: read-led.sh [samples] [host] [user]
set -uo pipefail

SAMPLES="${1:-8}"
HOST="${2:-10.10.10.217}"
USER_NAME="${3:-mvcorrea}"
CFG=/tmp/openocd-mg24-mini.cfg

BASE=0x5003C000          # GPIO non-secure alias
MODEL=$((BASE + 0x034))
DOUT=$((BASE + 0x040))
DIN=$((BASE + 0x044))

pair_values=()
mode7=""
for i in $(seq 1 "${SAMPLES}"); do
  out="$(ssh -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
    "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
       -c 'init; halt' \
       -c 'mdw $(printf '0x%08X' "$MODEL") 1' \
       -c 'mdw $(printf '0x%08X' "$DOUT") 1' \
       -c 'mdw $(printf '0x%08X' "$DIN") 1' \
       -c 'resume; shutdown'" 2>&1)"

  model_w="$(printf '%s\n' "${out}" | awk -v a="$(printf '0x%08X' "$MODEL" | tr 'A-F' 'a-f')" 'tolower($1)==a":" {print $2}' | head -1)"
  dout_w="$(printf '%s\n' "${out}" | awk -v a="$(printf '0x%08X' "$DOUT" | tr 'A-F' 'a-f')" 'tolower($1)==a":" {print $2}' | head -1)"
  din_w="$(printf '%s\n' "${out}" | awk -v a="$(printf '0x%08X' "$DIN" | tr 'A-F' 'a-f')" 'tolower($1)==a":" {print $2}' | head -1)"

  if [ -z "${model_w}" ] || [ -z "${dout_w}" ] || [ -z "${din_w}" ]; then
    echo "sample ${i}: read failed" >&2
    continue
  fi

  # PORTA_MODEL holds 4 bits per pin: mode0 in bits 3:0 ... mode7 in bits 31:28.
  mode7=$(( (16#${model_w} >> 28) & 0xF ))
  dout7=$(( (16#${dout_w} >> 7) & 1 ))
  din7=$(( (16#${din_w} >> 7) & 1 ))
  pair_values+=("${dout7}${din7}")
  printf 'sample %d: mode7=%d dout7=%d din7=%d  (model=0x%s dout=0x%s din=0x%s)\n' \
    "${i}" "${mode7}" "${dout7}" "${din7}" "${model_w}" "${dout_w}" "${din_w}"
  sleep 0.4
done

if [ ${#pair_values[@]} -eq 0 ]; then
  echo "no samples" >&2
  exit 3
fi

dout_states=$(printf '%s\n' "${pair_values[@]}" | cut -c1 | sort -u | wc -l)
din_states=$(printf '%s\n' "${pair_values[@]}" | cut -c2 | sort -u | wc -l)
mismatch=0
for p in "${pair_values[@]}"; do
  [ "${p:0:1}" != "${p:1:1}" ] && mismatch=$((mismatch + 1))
done

echo
echo "mode7=${mode7} (4 = push-pull output, 0 = disabled, 1 = input)"
echo "dout7 took ${dout_states} value(s); din7 took ${din_states} value(s); ${mismatch}/${#pair_values[@]} samples where din7 != dout7"
if [ "${mode7}" -ne 4 ]; then
  echo "VERDICT: pin 7 is NOT configured as a push-pull output -> the pad cannot follow DOUT."
  echo "         A blink here would flash, run, and light nothing. Real defect."
  exit 1
fi
if [ "${dout_states}" -lt 2 ]; then
  echo "VERDICT: DOUT bit 7 never changed -> the toggle loop is not running (or the pin is being driven elsewhere)."
  exit 1
fi
if [ "${din_states}" -ge 2 ] && [ "${mismatch}" -eq 0 ]; then
  echo "VERDICT: DOUT toggles AND the pad follows it (din7 == dout7, mode push-pull)."
  echo "         The MCU side is proven: the pad is electrically driven and changing."
  echo "         NOT proven: that the LED sits on PA7 and that the LED itself works."
  exit 0
fi
echo "VERDICT: DOUT toggles but the pad does not follow (din7 stuck or mismatched)."
echo "         Something between the register and the pad is wrong -- that is a real"
echo "         electrical/configuration defect, not a sampling artefact."
exit 1
