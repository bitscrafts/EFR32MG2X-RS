#!/usr/bin/env bash
# Sample a GPIO port's DOUT register over SWD and report one bit per sample.
#
# READ ONLY: the core is halted for ~50 ms per sample, the register is read,
# then the core is resumed. Nothing is written to flash, and nothing is written
# to the target at all.
#
# Why this exists: "the LED blinks" is a claim that otherwise needs a human
# watching the board. Sampling PORTA_DOUT bit 7 (PA7, the XIAO user LED) over
# SWD turns it into data: if the bit takes both values across samples, the
# firmware is executing the toggle loop.
#
# Usage: read-pin.sh [reg] [bit] [samples] [host] [user]
#   reg     default 0x5003C040 (GPIO non-secure alias, PORTA_DOUT)
#   bit     default 7 (PA7 user LED)
set -uo pipefail

REG="${1:-0x5003C040}"
BIT="${2:-7}"
SAMPLES="${3:-6}"
HOST="${4:-${FLASH_HOST:-localhost}}"
USER_NAME="${5:-${FLASH_USER:-$(id -un)}}"

CFG=/tmp/openocd-mg24-mini.cfg
if ! ssh -o ConnectTimeout=10 "${USER_NAME}@${HOST}" "test -f ${CFG}"; then
  echo "missing ${CFG} on ${HOST}; copy scripts/openocd-mg24-mini.cfg there first" >&2
  exit 2
fi

values=()
for i in $(seq 1 "${SAMPLES}"); do
  out="$(ssh -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
    "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
       -c 'init; halt' -c 'mdw ${REG} 1' -c 'resume; shutdown'" 2>&1)"
  word="$(printf '%s\n' "${out}" | sed -n "s/^$(printf '%s' "${REG}" | tr 'A-F' 'a-f'): \([0-9a-fA-F]*\).*/\1/p" | head -1)"
  if [ -z "${word}" ]; then
    echo "sample ${i}: read failed: $(printf '%s\n' "${out}" | grep -m1 -E 'Error|Failed')" >&2
    continue
  fi
  bit_val=$(( (16#${word} >> BIT) & 1 ))
  values+=("${bit_val}")
  printf 'sample %d: dout=0x%08x bit%d=%d\n' "${i}" "$((16#${word}))" "${BIT}" "${bit_val}"
  sleep 0.4
done

if [ ${#values[@]} -eq 0 ]; then
  echo "no samples" >&2
  exit 3
fi

uniq_count=$(printf '%s\n' "${values[@]}" | sort -u | wc -l)
echo "distinct values: ${uniq_count} of ${#values[@]} samples"
if [ "${uniq_count}" -ge 2 ]; then
  echo "VERDICT: bit ${BIT} toggles -> firmware is executing the toggle loop"
  exit 0
fi
echo "VERDICT: bit ${BIT} constant at ${values[0]} -> NOT proven to toggle (sampling or firmware)"
exit 1
