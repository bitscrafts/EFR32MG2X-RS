#!/usr/bin/env bash
# Build **every** example, with the features it declares, and fail if one does not compile.
#
#   bash scripts/check-examples-build.sh [filter]
#
# Why this exists next to `cargo build --examples`: an example with `required-features` is silently *skipped* when those
# features are not enabled, so a workspace-wide `--examples` build reports success while the Bluetooth examples -- the
# ones that matter most here -- were never compiled at all. That is a check that passes while the thing it checks is
# broken, which is worse than no check.
#
# The list is explicit for the same reason: each entry names the features its example needs, and adding an example means
# adding a line here.
set -u

TARGET=thumbv8m.main-none-eabihf
FILTER="${1:-}"
FAILED=0
BUILT=0

# package example features...
EXAMPLES="
efr32mg24-pac          pac_led_blink            rt
efr32mg24-pac          pac_bme680               rt
efr32mg24-pac          pac_env_display          rt
efr32mg24-pac          pac_rtc_read             rt
efr32mg24-pac          pac_oled_hello           rt
efr32mg24-hal          hal_led_blink            rt
efr32mg24-hal          hal_mic_level            rt
efr32mg24-hal          hal_imu_read             rt
efr32mg24-hal          hal_timer0_pwm           rt
efr32mg24-radio        ble_beacon               ble stack
efr32mg24-radio        ble_scanner              ble stack
efr32mg24-radio        ble_advertise_custom     ble stack
efr32mg24-radio        ble_gatt                 ble gatt stack
efr32mg24-radio        ble_diag                 ble stack diag
efr32mg24-radio        ble_boot_probe           stack
efr32mg24-radio        ble_rail_then_stack      stack
xiao-mg24-bsp          module_led_blink         sense
xiao-mg24-bsp          module_ble_beacon        sense stack
xiao-mg24-bsp          module_ble_motion        sense stack
xiao-mg24-bsp          module_ble_fall          sense stack
xiao-mg24-bsp          module_mic_level         sense
xiao-mg24-bsp          module_imu               sense
xiao-expansion-base-bsp base_led_blink
xiao-expansion-base-bsp base_ble_env            stack
xiao-expansion-base-bsp base_ble_env_gatt       stack
xiao-expansion-base-bsp base_env_display
"

printf '%-46s %-6s\n' "example" "build"
echo "$EXAMPLES" | while read -r PKG EX FEAT; do
    [ -z "${PKG:-}" ] && continue
    LABEL="$PKG/$EX"
    if [ -n "$FILTER" ] && ! printf '%s' "$LABEL" | grep -qE "$FILTER"; then
        continue
    fi
    printf '%-46s ' "$LABEL"
    # shellcheck disable=SC2086
    if cargo build --release --target "$TARGET" -p "$PKG" --example "$EX" --features "$FEAT" \
        > "/tmp/build-$EX.log" 2>&1; then
        printf 'ok\n'
    else
        printf 'FAIL\n'
        grep -m3 -E "^error" "/tmp/build-$EX.log" | sed 's/^/    /'
        FAILED=1
    fi
    BUILT=$((BUILT + 1))
done | tee /tmp/examples-build.log

# The subshell above cannot set FAILED for us; the log is the evidence and this is the verdict.
if grep -q "FAIL" /tmp/examples-build.log; then
    echo "VERDICT: FAIL ($(grep -c FAIL /tmp/examples-build.log) example(s) do not build)"
    exit 1
fi
COUNT=$(grep -c " ok$" /tmp/examples-build.log || true)
echo "VERDICT: PASS ($COUNT example(s) build)"
