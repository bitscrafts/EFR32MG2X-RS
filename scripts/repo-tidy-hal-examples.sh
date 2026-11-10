#!/usr/bin/env bash
# Retire the pre-existing HAL examples that were never verified on hardware.
#
# The 01..12 set was written in June against a HAL whose API has since drifted:
# 09_dma_adc_continuous no longer compiles (scantable, dmawufifosingle), and
# several others used a Board::split that returned three values when the
# signature promised four. Keeping them in the crate means `cargo build
# --examples` fails, which hides a broken example behind a broken example.
#
# They are moved, not deleted: the code is a record of what was attempted, and
# the pieces worth keeping are re-derived from the PAC examples that do run.
#
# Usage: repo-tidy-hal-examples.sh [--confirm]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES="$REPO_ROOT/crates/efr32mg24-hal/examples"
STAGE="${STAGE:-/projects/_tmp/efr32mg24-rs/retired-hal-examples}"
CONFIRM="${1:-}"

echo "source : $EXAMPLES"
echo "stage  : $STAGE"
echo

mapfile -t RETIRE < <(
  cd "$EXAMPLES"
  # Everything that is not part of the current hal_* ladder and not shared.
  for f in *.rs; do
    case "$f" in
      hal_*.rs) ;;
      *) echo "$f" ;;
    esac
  done
)

if [ "${#RETIRE[@]}" -eq 0 ]; then
  echo "nothing to retire"
  exit 0
fi

for f in "${RETIRE[@]}"; do
  echo "  retire $f"
done

if [ "$CONFIRM" != "--confirm" ]; then
  echo
  echo "DRY RUN: re-run with --confirm to move them to $STAGE"
  exit 0
fi

mkdir -p "$STAGE"
for f in "${RETIRE[@]}"; do
  [ -e "$STAGE/$f" ] && { echo "refusing: $STAGE/$f exists"; exit 1; }
  mv "$EXAMPLES/$f" "$STAGE/$f"
done

cat > "$STAGE/README.md" <<'EOF'
# Retired HAL examples

Moved out of `crates/efr32mg24-hal/examples` on 2026-11-11 because they no longer
compile against the current HAL, and an example that does not build cannot be a
test:

- `09_dma_adc_continuous.rs` — `scantable()` and `dmawufifosingle()` no longer
  exist in the PAC's IADC register set.
- the rest used `Board::split()` when it returned a three-tuple while the
  signature promised four, and configure clocks through `Clocks::new`, which on
  this board was implicated in the June bricking incident.

Nothing here was ever verified on hardware. The examples that replaced them
(`hal_bus_scan`, `hal_button`, `hal_oled_hello`, `hal_env_display`, `hal_imu` and
their PAC twins) each carry the measurement that backs them.
EOF

echo
echo "moved ${#RETIRE[@]} files to $STAGE"
