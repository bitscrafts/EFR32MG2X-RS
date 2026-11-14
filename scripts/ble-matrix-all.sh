#!/usr/bin/env bash
# Run every BLE example on the board and record, per example, what the radio actually did.
#
#   bash scripts/ble-matrix-all.sh [results-file]
#
# For each example: build, flash through scripts/flash.sh (which refuses a load address below 0x08006000 and verifies
# by read-back), read the board's mailbox over SWD, then capture the air for a fixed window with btmon while the host
# scans. The board's own record is printed but is **not** the criterion: an example whose job is to transmit passes
# only if its address is seen on the air.
#
# The results file is append-per-example, so a timeout in the middle leaves the finished rows intact.
set -u

HOST="${FLASH_HOST:-10.10.10.217}"
USER_="${FLASH_USER:-mvcorrea}"
RESULTS="${1:-/projects/_tmp/efr32mg24-rs/MATRIX-RESULTS.md}"
FILTER="${2:-}"
ADDR_E4="E4:0A:75:13:3E:E0"     # the board's own advertising address
WINDOW="${WINDOW:-12}"

# package example features
# Features are **space separated**, and `read` hands the remainder of the line to the last variable. `ble+stack` is
# not feature syntax: cargo reads it as one unknown feature and every example "fails to build for a missing feature".
EXAMPLES="
efr32mg24-radio ble_beacon ble stack
efr32mg24-radio ble_scanner ble stack
efr32mg24-radio ble_advertise_custom ble stack
efr32mg24-radio ble_gatt ble gatt stack
efr32mg24-radio ble_diag ble stack diag
efr32mg24-radio ble_boot_probe stack
efr32mg24-radio ble_rail_then_stack stack
xiao-mg24-bsp module_ble_beacon sense stack
xiao-mg24-bsp module_ble_motion sense stack
xiao-mg24-bsp module_ble_fall sense stack
xiao-expansion-base-bsp base_ble_env_gatt stack
xiao-expansion-base-bsp base_ble_env stack
efr32mg24-hal hal_led_blink rt
"

APPEND="${APPEND:-0}"
if [ "$APPEND" = 0 ]; then
  echo "# BLE example matrix" > "$RESULTS"
fi
echo "" >> "$RESULTS"
echo "Run $(date -u +%Y-%m-%dT%H:%M:%SZ) at $(git -C /projects/efr32mg24-rs rev-parse --short HEAD)" >> "$RESULTS"
echo "" >> "$RESULTS"
printf '%-46s %-6s %-6s %-9s %s\n' "example" "build" "flash" "board" "air (12 s)" | tee -a "$RESULTS"

echo "$EXAMPLES" | while read -r PKG EX FEAT; do
    [ -z "${PKG:-}" ] && continue
    LABEL="$PKG/$EX"
    # A filter, so the matrix can be run in halves without a 600-second tool timeout.
    if [ -n "${FILTER:-}" ] && ! printf '%s' "$LABEL" | grep -qE "$FILTER"; then continue; fi
    printf '%-46s ' "$LABEL"

    BUILD=FAIL
    if cargo build --release --target thumbv8m.main-none-eabihf -p "$PKG" --example "$EX" --features "$FEAT" \
        > "/tmp/mx-$EX-build.log" 2>&1; then
        BUILD=ok
    fi
    printf '%-6s ' "$BUILD"
    if [ "$BUILD" != ok ]; then
        printf '%-6s %-9s %s\n' "-" "-" "build failed: $(grep -m1 -E '^error' "/tmp/mx-$EX-build.log" | cut -c1-60)" | tee -a "$RESULTS"
        continue
    fi

    ELF="/projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/$EX"
    # Clear before the flash, not after: the mailbox survives a reset, so an example that only writes it on an event
    # would otherwise be shown reporting the *previous* image's numbers.
    bash scripts/clear-mailbox.sh "$HOST" "$USER_" < /dev/null >/dev/null 2>&1
    FLASH=FAIL
    # `< /dev/null` on everything that can read stdin: a command inside `while read` that consumes stdin ends the
    # loop after one iteration, which is exactly what happened on the first run of this script.
    if FLASH_HOST="$HOST" FLASH_USER="$USER_" bash scripts/flash.sh "$ELF" --confirm --run < /dev/null > "/tmp/mx-$EX-flash.log" 2>&1; then
        if grep -q "Verification successful" "/tmp/mx-$EX-flash.log"; then FLASH=ok; fi
    fi
    printf '%-6s ' "$FLASH"
    sleep 4

    BOARD="$(bash scripts/read-mailbox.sh "$HOST" "$USER_" < /dev/null 2>/dev/null | tail -1 | cut -c1-46)"
    # An empty mailbox after a fresh flash is a fact, not a failure: several examples report only when something
    # happens, and some transmit without saying anything at all.
    [ -z "$BOARD" ] && BOARD="(silent)"

    # The air, measured from the host: btmon records every report the controller passes up, during a real scan.
    ssh -n -o ConnectTimeout=10 "$USER_@$HOST" "sudo -n sh /tmp/host-aircheck.sh" > "/tmp/mx-$EX-air.log" 2>&1
    AIR="$(sed -n 's/^board=\([0-9]*\).*/\1/p' "/tmp/mx-$EX-air.log")"
    [ -z "$AIR" ] && AIR="?"

    printf '%-9s %s\n' "$BOARD" "board=$AIR" | tee -a "$RESULTS"
done

echo "" >> "$RESULTS"
echo "## Reading the table" >> "$RESULTS"
echo "" >> "$RESULTS"
echo "- \`build\`/\`flash\`: the example compiled and the image was flashed and verified by read-back." >> "$RESULTS"
echo "- \`board\`: the firmware's own record, truncated. It is a self-report and not the criterion." >> "$RESULTS"
echo "- \`air\`: how many times the board's address was seen by the host in the window. An example whose job is to" >> "$RESULTS"
echo "  transmit passes only when this is non-zero." >> "$RESULTS"
echo "" >> "$RESULTS"
echo "Done: $RESULTS"
