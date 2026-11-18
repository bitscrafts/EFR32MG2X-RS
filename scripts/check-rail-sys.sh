#!/usr/bin/env bash
# The gate skips efr32mg24-rail-sys on purpose: its bindings are generated at build time and need
# libclang plus the vendor blob, which the gate's environment does not always have. The skip is
# legitimate; the silence that came with it was not. Twice in one evening an example was written in
# that crate against an API nobody had read, and the gate said PASS because the gate does not look
# there.
#
# So this script is the gate's eye on that crate: it builds it for the target and runs clippy with
# warnings denied. A crate excluded from the gate has to have its own check, or the work done in it
# is invisible.
#
# Usage: bash scripts/check-rail-sys.sh [--quiet]
set -uo pipefail

cd "$(dirname "$0")/.." || exit 2

PKG="efr32mg2x-rail-sys"
TARGET="thumbv8m.main-none-eabihf"
QUIET="${1:-}"
LOG=/tmp/check-rail-sys.log
: > "$LOG"
falhas=0

pass() { [ "$QUIET" = "--quiet" ] || echo "  PASS  rail-sys: $1"; }
fail() { echo "  FAIL  rail-sys: $1"; falhas=$((falhas + 1)); }

# 1. The library compiles for the part.
if cargo build -p "$PKG" --target "$TARGET" >>"$LOG" 2>&1; then
  pass "builds for $TARGET"
else
  fail "does not build for $TARGET ($(grep -m1 -E '^error' "$LOG"))"
fi

# 2. Warnings are errors here too. A -sys crate with warnings is one nobody has read.
if cargo clippy -p "$PKG" --target "$TARGET" -- -D warnings >>"$LOG" 2>&1; then
  pass "clippy clean"
else
  fail "clippy reports warnings ($(grep -m1 -E '^error' "$LOG"))"
fi

# 3. The 802.15.4 entry points are actually generated. The header is in build.rs, but a comment
#    saying so is not a measurement: if the include path breaks, the symbols vanish and the
#    wrappers become the only thing that fails, far from the cause.
BINDINGS=$(find target/"$TARGET" -name bindings.rs -path "*rail-sys*" 2>/dev/null | head -1)
if [ -n "$BINDINGS" ]; then
  if grep -q "RAIL_IEEE802154_Config2p4GHzRadio" "$BINDINGS"; then
    pass "802.15.4 bindings are generated"
  else
    fail "802.15.4 bindings are missing from the generated file"
  fi
else
  fail "no generated bindings file to inspect"
fi

if [ "$falhas" -eq 0 ]; then
  echo "  VERDICT: PASS (rail-sys, which the gate skips, is checked here)"
  exit 0
fi
echo "  VERDICT: FAIL ($falhas of 3 checks)"
exit 1
