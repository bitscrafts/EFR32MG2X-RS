#!/usr/bin/env bash
# Measure every HAL example on the board, one at a time, and record what each one reported.
#
# The same instrument as the PAC's: per example, clear the mailbox (it survives a reset, so a stale record would read as a
# pass), flash with --run, wait, read the record back. Since the mailbox module now also logs the same text over RTT, a
# person watching `probe-rs run` sees it live; this script is the batch path that produces a table.
#
#   bash scripts/measure-hal-examples.sh             # all of them
#   bash scripts/measure-hal-examples.sh hal_imu_read # one
set -uo pipefail

REPO="/projects/efr32mg24-rs"
OUT="${OUT:-/projects/_tmp/efr32mg24-rs/HAL-HARDWARE.tsv}"
TARGET_DIR="${CARGO_TARGET_DIR:-/projects/_cache/targets}"
ELFDIR="${TARGET_DIR}/thumbv8m.main-none-eabihf/release/examples"
HOST="${FLASH_HOST:-10.10.10.217}"
USER_NAME="${FLASH_USER:-mvcorrea}"
PKG="efr32mg2x-hal"
SRC="crates/efr32mg2x-hal/examples"

cd "$REPO"

if [[ $# -gt 0 ]]; then
  EXAMPLES=("$@")
else
  mapfile -t EXAMPLES < <(ls "$SRC"/*.rs | xargs -n1 basename | sed 's/\.rs$//' | sort)
fi

[[ -f "$OUT" ]] || printf 'example\tresult\tstatus\trecord\n' > "$OUT"

for ex in "${EXAMPLES[@]}"; do
  elf="${ELFDIR}/${ex}"
  printf '%-24s ' "$ex"

  # Build first, always: measuring a cached ELF reports the previous binary's behaviour under the new name.
  if ! CARGO_TARGET_DIR="$TARGET_DIR" PATH="/usr/local/cargo/bin:$PATH" \
       cargo build --release --target thumbv8m.main-none-eabihf \
       -p "$PKG" --features rt --example "$ex" >/tmp/measure-build-"$ex".log 2>&1; then
    printf 'NAO COMPILA (ver /tmp/measure-build-%s.log)\n' "$ex"
    printf '%s\tbuild-failed\t\t\n' "$ex" >> "$OUT"; continue
  fi
  if [[ ! -f "$elf" ]]; then
    printf 'SEM ELF\n'; printf '%s\tno-elf\t\t\n' "$ex" >> "$OUT"; continue
  fi
  if ! python3 scripts/elf-load-range.py "$elf" >/dev/null 2>&1; then
    printf 'RECUSADO (endereco)\n'; printf '%s\trefused\t\t\n' "$ex" >> "$OUT"; continue
  fi

  FLASH_HOST="$HOST" FLASH_USER="$USER_NAME" bash scripts/clear-mailbox.sh "$HOST" "$USER_NAME" >/dev/null 2>&1

  # The clear is only trusted if it can be seen to have worked: openocd cannot always halt a target whose clocks are
  # already configured, and the clear then fails silently while the previous record stays in RAM.
  if [[ -n "$(bash scripts/read-mailbox.sh "$HOST" "$USER_NAME" 2>/dev/null | tail -1)" ]]; then
    printf 'MAILBOX NAO LIMPOU (registo antigo)\n'
    printf '%s\tstale-mailbox\t\t\n' "$ex" >> "$OUT"; continue
  fi

  if ! FLASH_HOST="$HOST" FLASH_USER="$USER_NAME" bash scripts/flash.sh "$elf" --confirm --run >"/tmp/flash-$ex.log" 2>&1; then
    printf 'FALHA DE GRAVACAO\n'; printf '%s\tflash-failed\t\t%s\n' "$ex" "$(tail -1 "/tmp/flash-$ex.log")" >> "$OUT"; continue
  fi

  sleep 3
  out=$(bash scripts/read-mailbox.sh "$HOST" "$USER_NAME" 2>/dev/null)
  status=$(printf '%s\n' "$out" | sed -n 's/^seq=[^ ]* status=\([a-zA-Z]*\) len=.*/\1/p' | tail -1)
  record=$(printf '%s\n' "$out" | tail -1)
  if [[ -z "$record" ]]; then
    printf 'SEM REGISTO\n'; printf '%s\tno-record\t\t\n' "$ex" >> "$OUT"
  else
    printf '%s [%s]\n' "${record:0:52}" "${status:-?}"
    printf '%s\trecord\t%s\t%s\n' "$ex" "${status:-unknown}" "$record" >> "$OUT"
  fi
done

printf '\nresultados em %s\n' "$OUT"
