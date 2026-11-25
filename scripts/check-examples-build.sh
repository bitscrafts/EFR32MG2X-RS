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

# Every example of every crate in the workspace, with the feature union its examples need. `*` means "all of them", and
# the completeness check below refuses a tree where an example exists that no line here reaches.
#
# package              example  features
EXAMPLES="
efr32mg2x-pac                 *  rt
efr32mg2x-hal                 *  rt
efr32mg2x-bluetooth-sys       *
efr32mg2x-rail-sys            *
efr32mg2x-radio               *  ble,stack
efr32mg2x-emlib-sys           *
embassy-efr32                 *
efr32mg2x-xiao-bsp            *  sense,stack
efr32mg2x-expansion-base-bsp  *  stack
"

printf '%-46s %-6s\n' "example" "build"
echo "$EXAMPLES" | while read -r PKG EX FEAT; do
    [ -z "${PKG:-}" ] && continue
    LABEL="$PKG/$EX"
    if [ -n "$FILTER" ] && ! printf '%s' "$LABEL" | grep -qE "$FILTER"; then
        continue
    fi
    printf '%-46s ' "$LABEL"
    # `*` builds every example the crate has. The list below used to name them one by one, and it had drifted: the HAL
    # shipped twenty-two examples while this script compiled four, so eighteen of them -- every instrument added since
    # 2026-11-15 -- were never built here at all.
    # shellcheck disable=SC2086
    if [ "$EX" = "*" ]; then
        cargo build --release --target "$TARGET" -p "$PKG" --examples --features "$FEAT" \
            > "/tmp/build-all-$PKG.log" 2>&1
    else
        cargo build --release --target "$TARGET" -p "$PKG" --example "$EX" --features "$FEAT" \
            > "/tmp/build-$EX.log" 2>&1
    fi
    if [ $? -eq 0 ]; then
        printf 'ok\n'
    else
        printf 'FAIL\n'
        grep -m3 -E "^error" "/tmp/build-$EX.log" | sed 's/^/    /'
        FAILED=1
    fi
    BUILT=$((BUILT + 1))
done | tee /tmp/examples-build.log

# Every example that exists has to be reachable from this list, or the green verdict below is about a subset of the
# tree. `*` covers a crate entirely; anything else has to be named.
MISSING=0
for dir in crates/*/examples; do
    [ -d "$dir" ] || continue
    pkg=$(grep -m1 '^name' "$(dirname "$dir")/Cargo.toml" 2>/dev/null | cut -d'"' -f2)
    # These three are not workspace members: they are built by their own scripts, against their own targets.
    case "$pkg" in efr32mg2x-bootloader|efr32mg2x-bootloader-test-app|efr32mg2x-cmsis-stubs) continue;; esac
    [ -n "$pkg" ] || continue
    for f in "$dir"/*.rs; do
        [ -e "$f" ] || continue
        nome=$(basename "$f" .rs)
        if ! echo "$EXAMPLES" | grep -qE "^$pkg[[:space:]]+(\*|$nome)([[:space:]]|$)"; then
            echo "  $pkg/$nome is not in this script's list"
            MISSING=1
        fi
    done
done
if [ "$MISSING" = "1" ]; then
    echo "VERDICT: FAIL (an example exists that this script would never build)"
    exit 1
fi

# The subshell above cannot set FAILED for us; the log is the evidence and this is the verdict.
if grep -q "FAIL" /tmp/examples-build.log; then
    echo "VERDICT: FAIL ($(grep -c FAIL /tmp/examples-build.log) example(s) do not build)"
    exit 1
fi
CRATES=$(grep -c " ok$" /tmp/examples-build.log || true)
# The verdict counts examples, not rows: a `*` row is twenty-two of them. Counting rows is how the old version of this
# script could say "26 examples build" while the HAL alone shipped twenty-two that it never compiled.
COUNT=0
for dir in crates/*/examples; do
    [ -d "$dir" ] || continue
    pkg=$(grep -m1 '^name' "$(dirname "$dir")/Cargo.toml" 2>/dev/null | cut -d'"' -f2)
    case "$pkg" in efr32mg2x-bootloader|efr32mg2x-bootloader-test-app|efr32mg2x-cmsis-stubs) continue;; esac
    n=$(find "$dir" -maxdepth 1 -name '*.rs' | wc -l)
    COUNT=$((COUNT + n))
done
echo "VERDICT: PASS ($COUNT example(s) build, across $CRATES crate(s))"
