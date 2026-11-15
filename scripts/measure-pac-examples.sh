#!/usr/bin/env bash
# Measure every PAC example on the board, one at a time, and record what each one reported.
#
# Per example: clear the mailbox (it survives a reset, so a stale record would read as a pass), flash with --run,
# wait, read the record back. The record is the only instrument that shows the program reached its own main loop;
# "flashed successfully" has lied here before (a blink that flashed and did not blink).
#
#   bash scripts/measure-pac-examples.sh            # all of them
#   bash scripts/measure-pac-examples.sh pac_bme680 # one
set -uo pipefail

REPO="/projects/efr32mg24-rs"
OUT="${OUT:-/projects/_tmp/efr32mg24-rs/PAC-HARDWARE.tsv}"
TARGET_DIR="${CARGO_TARGET_DIR:-/projects/_cache/targets}"
ELFDIR="${TARGET_DIR}/thumbv8m.main-none-eabihf/release/examples"
HOST="${FLASH_HOST:-10.10.10.217}"
USER_NAME="${FLASH_USER:-mvcorrea}"

cd "$REPO"

if [[ $# -gt 0 ]]; then
  EXAMPLES=("$@")
else
  mapfile -t EXAMPLES < <(ls crates/efr32mg2x-pac/examples/*.rs | xargs -n1 basename | sed 's/\.rs$//' | sort)
fi

[[ -f "$OUT" ]] || printf 'example\tresult\trecord\n' > "$OUT"

for ex in "${EXAMPLES[@]}"; do
  elf="${ELFDIR}/${ex}"
  printf '%-28s ' "$ex"

  if [[ ! -f "$elf" ]]; then
    printf 'SEM ELF\n'; printf '%s\tno-elf\t\n' "$ex" >> "$OUT"; continue
  fi
  if ! python3 scripts/elf-load-range.py "$elf" >/dev/null 2>&1; then
    printf 'RECUSADO (endereco)\n'; printf '%s\trefused\t\n' "$ex" >> "$OUT"; continue
  fi

  FLASH_HOST="$HOST" FLASH_USER="$USER_NAME" bash scripts/clear-mailbox.sh "$HOST" "$USER_NAME" >/dev/null 2>&1

  if ! FLASH_HOST="$HOST" FLASH_USER="$USER_NAME" bash scripts/flash.sh "$elf" --confirm --run >"/tmp/flash-$ex.log" 2>&1; then
    printf 'FALHA DE GRAVACAO\n'; printf '%s\tflash-failed\t%s\n' "$ex" "$(tail -1 "/tmp/flash-$ex.log")" >> "$OUT"; continue
  fi

  sleep 3
  record=$(bash scripts/read-mailbox.sh "$HOST" "$USER_NAME" 2>/dev/null | tail -1)
  if [[ -z "$record" ]]; then
    printf 'SEM REGISTO\n'; printf '%s\tno-record\t\n' "$ex" >> "$OUT"
  else
    printf '%s\n' "${record:0:60}"
    printf '%s\trecord\t%s\n' "$ex" "$record" >> "$OUT"
  fi
done

printf '\nresultados em %s\n' "$OUT"
