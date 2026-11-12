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

# Every crate in the workspace gets checked, discovered rather than listed: a hardcoded list
# silently stops covering the crate that was added last, which is the one that needs it most.
CRATES=$(ls -d crates/*/ libraries/*/ 2>/dev/null | sed 's:/*$::')
# Named here so the gap is visible instead of silent:
#   efr32mg24-bluetooth-sys, efr32mg24-rail-sys  bindgen output does not compile from a clean
#                                                checkout; their headers are gitignored
#   efr32mg24-bootloader, bootloader-test-app     separate workspaces, excluded from this
#                                                one on purpose
SKIP="efr32mg24-bluetooth-sys efr32mg24-rail-sys efr32mg24-bootloader bootloader-test-app"
features_for() {
  case "$1" in
    efr32mg24-pac|efr32mg24-hal) echo "--features rt" ;;
    *) echo "" ;;
  esac
}

echo "=== clippy, zero warnings ==="
for dir in $CRATES; do
  pkg=$(basename "$dir")
  case " $SKIP " in *" $pkg "*) continue ;; esac
  [ -f "$dir/Cargo.toml" ] || continue
  extra=$(features_for "$pkg")
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
for dir in $CRATES; do
  pkg=$(basename "$dir")
  case " $SKIP " in *" $pkg "*) continue ;; esac
  [ -f "$dir/Cargo.toml" ] || continue
  extra=$(features_for "$pkg")
  count=$(ls "$dir/examples/"*.rs 2>/dev/null | wc -l)
  if cargo build --release --target "$TARGET" -p "$pkg" $extra --examples >/dev/null 2>&1; then
    pass "$pkg: $count examples"
  else
    fail "$pkg: examples do not build"
  fi
  # A hardware crate whose API nothing builds is a crate whose API is not tested. Only the
  # crates that own pins are held to this; the PAC is exercised by its own 19 examples and
  # the domain crates by their host tests.
  case "$pkg" in
    *-bsp)
      if [ "$count" -eq 0 ]; then
        fail "$pkg: exposes hardware but has no examples"
      fi
      ;;
  esac
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

echo "=== host tests: domain crates ==="
# Every crate under libraries/ must build and test on the host, and must not drag in
# anything from the hardware stack. That second half is the rule that keeps them testable:
# one dependency on a HAL turns `cargo test` into a cross-compilation.
for dir in libraries/*/; do
  [ -f "${dir}Cargo.toml" ] || continue
  name=$(basename "$dir")

  if cargo test -p "$name" --target "$HOST" >"/tmp/gate-$name.log" 2>&1; then
    n=$(grep -oE '[0-9]+ passed' "/tmp/gate-$name.log" | head -1)
    pass "$name: host tests (${n:-passed})"
  else
    fail "$name: host tests"
  fi

  # The hardware stack, as a dependency, in any kind: normal, build or dev.
  if cargo tree -p "$name" --edges normal,build -e no-dev --prefix none 2>/dev/null \
      | grep -qE '^(efr32mg24|motion-detection|cortex-m)'; then
    fail "$name: depends on the hardware stack"
  else
    pass "$name: no hardware dependency"
  fi
done

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
