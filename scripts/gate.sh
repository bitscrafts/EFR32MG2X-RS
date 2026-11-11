#!/usr/bin/env bash
# The gate that can run without the board.
#
# Every check here has a reason to exist, and each one has failed for real at least once
# in this workspace:
#
#   fmt        the tree drifts into unformatted code that later hides a real diff
#   clippy     zero warnings is the standing rule; 16 accumulated in one afternoon
#   examples   a crate whose examples do not build is a crate nobody can try
#   tests      the classifier's thresholds are only as good as the tests that pin them
#
# What it deliberately does not do:
#
#   - it does not touch hardware. Flashing belongs to scripts/flash-lnx.sh, which refuses
#     an image that would overwrite the bootloader.
#   - it does not build the Bluetooth sys crate. That crate has 17 errors from its own
#     bindgen output (`E0609 no field ... &__BindgenUnionField`) and the headers it needs
#     are gitignored, so it cannot be built from a clean checkout by anyone. It is named
#     here rather than silently skipped: a gate that hides a broken crate is worse than no
#     gate.
#
# Usage: scripts/gate.sh
set -uo pipefail
cd "$(dirname "$0")/.."

TARGET=thumbv8m.main-none-eabihf
HOST=x86_64-unknown-linux-gnu
FAILED=0
pass() { printf '  PASS  %s\n' "$1"; }
fail() { printf '  FAIL  %s\n' "$1"; FAILED=1; }

echo "=== format ==="
if cargo fmt --all --check >/dev/null 2>&1; then
  pass "cargo fmt --all --check"
else
  fail "cargo fmt --all --check (run: cargo fmt --all)"
fi

echo "=== clippy, zero warnings ==="
for spec in \
  "efr32mg24-pac:--features rt" \
  "efr32mg24-hal:--features rt" \
  "motion-detection:" \
  "xiao-mg24-bsp:"
do
  pkg=${spec%%:*}
  extra=${spec#*:}
  out=$(cargo clippy --release --target "$TARGET" -p "$pkg" $extra 2>&1)
  n=$(printf '%s\n' "$out" | grep -cE '^warning: [a-z]' || true)
  if printf '%s\n' "$out" | grep -qE '^error'; then
    fail "$pkg clippy (does not compile)"
  elif [ "$n" -eq 0 ]; then
    pass "$pkg clippy, 0 warnings"
  else
    fail "$pkg clippy, $n warnings"
  fi
done

echo "=== examples build ==="
for spec in \
  "efr32mg24-pac:--features rt" \
  "efr32mg24-hal:--features rt" \
  "xiao-mg24-bsp:"
do
  pkg=${spec%%:*}
  extra=${spec#*:}
  count=$(ls "crates/$pkg/examples/"*.rs 2>/dev/null | wc -l)
  if cargo build --release --target "$TARGET" -p "$pkg" $extra --examples >/dev/null 2>&1; then
    pass "$pkg: $count examples"
  else
    fail "$pkg: examples do not build"
  fi
done
# An example that lives in a directory is only built by `--examples` if its [[example]]
# block declares it. A directory nobody declared compiles never and fails silently, so the
# check is on the declaration, from the name the manifest gives it.
for dir in crates/*/examples/*/; do
  [ -f "${dir}main.rs" ] || continue
  crate=$(basename "$(dirname "$(dirname "$dir")")")
  rel="examples/$(basename "$dir")/main.rs"
  name=$(grep -B4 "path = \"$rel\"" "crates/$crate/Cargo.toml" | grep -m1 '^name = ' | cut -d'"' -f2)
  if [ -n "$name" ]; then
    pass "$crate: $rel declared as example \"$name\""
  else
    fail "$crate: $rel is not declared in Cargo.toml (it would never build)"
  fi
done

echo "=== host tests ==="
if cargo test -p motion-detection --target "$HOST" >/tmp/gate-tests.log 2>&1; then
  n=$(grep -oE '[0-9]+ passed' /tmp/gate-tests.log | head -1)
  pass "motion-detection: ${n:-passed}"
else
  fail "motion-detection tests"
fi

echo
echo "not gated on purpose:"
echo "  efr32mg24-bluetooth-sys  -- 17 errors from its own bindgen output; headers gitignored"
echo "  efr32mg24-rail-sys       -- needs the SDK headers, same reason"
echo
if [ "$FAILED" -eq 0 ]; then
  echo "VERDICT: PASS"
else
  echo "VERDICT: FAIL"
fi
exit "$FAILED"
