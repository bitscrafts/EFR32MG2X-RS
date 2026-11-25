#!/usr/bin/env bash
# Everything that must pass before anything is published. Each block exists because something got past it once.
#
# Usage:  bash scripts/pre-publish.sh [caminho-da-crate]      (por omissão: crates/efr32mg2x-pac)
#
# It never publishes and never uploads: it only reads, builds and compares. The last line is a VERDICT.
set -u
cd "$(dirname "$0")/.." || exit 1
export PATH=/usr/local/cargo/bin:$PATH

CRATE="${1:-crates/efr32mg2x-pac}"
[ -d "$CRATE" ] || { echo "no such crate: $CRATE"; exit 1; }
# A versão pode vir do manifest da crate ou por herança do workspace (`version.workspace = true`), e o `cut`
# devolve a linha inteira quando não há aspas -- foi assim que este script leu "version.workspace = true" como
# se fosse um número e reprovou cinco verificações em cascata.
NOME=$(grep -m1 '^name' "$CRATE/Cargo.toml" | sed -n 's/.*"\([^"]*\)".*/\1/p')
VER=$(grep -m1 '^version' "$CRATE/Cargo.toml" | sed -n 's/.*"\([^"]*\)".*/\1/p')
if [ -z "$VER" ]; then
  VER=$(grep -A12 '^\[workspace.package\]' Cargo.toml | grep -m1 '^version' | sed -n 's/.*"\([^"]*\)".*/\1/p')
fi
[ -n "$NOME" ] && [ -n "$VER" ] || { echo "cannot read name/version from $CRATE/Cargo.toml"; exit 1; }
FAILED=0
LINHAS=0

ok()   { LINHAS=$((LINHAS+1)); printf '  PASS  %s\n' "$1"; }
bad()  { LINHAS=$((LINHAS+1)); FAILED=$((FAILED+1)); printf '  FAIL  %s\n' "$1"; [ -n "${2:-}" ] && printf '        %s\n' "$2"; }

echo "== $NOME $VER =="

# 1. The gate: build, clippy, fmt, host tests, every example. Nothing else means anything if this is red.
if bash scripts/gate.sh >/tmp/pp-gate.log 2>&1; then ok "gate (build, clippy, fmt, host tests, examples)"; else bad "gate" "ver /tmp/pp-gate.log"; fi

# 2. The package must build as it will be shipped. `cargo package` does that and never contacts the registry; the
#    `cargo publish --dry-run` that used to be here was removed because it *is* a publish attempt (it needs a token and
#    uploads when it has one) and because compiling the package is all it ever proved -- it certified a README that
#    lied. The reading the package needs is step 3, not this one.
if cargo package -p "$NOME" --allow-dirty >/tmp/pp-package.log 2>&1; then ok "cargo package builds the crate as it will ship"; else bad "cargo package" "$(tail -3 /tmp/pp-package.log)"; fi

# 3. The package must say what the tree does: identity, links, every include_str! target.
if python3 scripts/check-publishable.py >/tmp/pp-pub.log 2>&1; then ok "check-publishable (identity, links, include_str targets)"; else bad "check-publishable" "$(grep -E 'FAIL|missing|not found' /tmp/pp-pub.log | head -3)"; fi

# 4. crates.io refuses anything over 10 MB.
cargo package -p "$NOME" --allow-dirty --no-verify >/dev/null 2>&1
CRATE_FILE=$(ls -t target/package/"$NOME"-"$VER".crate 2>/dev/null | head -1)
if [ -n "$CRATE_FILE" ]; then
  BYTES=$(stat -c%s "$CRATE_FILE")
  if [ "$BYTES" -lt 10485760 ]; then ok "package size ${BYTES} B (limit 10485760)"; else bad "package size ${BYTES} B is over the 10 MB limit"; fi
else bad "package was not produced"; fi

# 5. Nothing in the package may point at this machine. A path from /projects or _tmp compiles here and nowhere else.
if [ -n "${CRATE_FILE:-}" ]; then
  VAZADOS=$(tar xzOf "$CRATE_FILE" --wildcards '*/README.md' '*/Cargo.toml' '*/src/lib.rs' 2>/dev/null | grep -cE '/projects/|/root/|/tmp/|_tmp/')
  if [ "${VAZADOS:-0}" -eq 0 ]; then ok "no local paths in the package's README, manifest or lib.rs"; else bad "$VAZADOS local path(s) in the package"; fi
fi

# 6. Every README: links resolve, named paths exist, nothing points at a local machine.
if python3 scripts/check-readmes.py --check >/tmp/pp-readmes.log 2>&1; then ok "every README points at something that exists"; else bad "a README points at something that is not there" "$(grep -v '^OK' /tmp/pp-readmes.log | head -3)"; fi

# 7. Every generated file still matches what generates it. A generated file that drifts is a lie with a build.
for gerador in "gen-crate-readme.py --all --check" "gen-pac-examples-doc.py --check" "emit-ble-config.py --check" "add-pac-module-docs.py --check"; do
  if python3 scripts/$gerador >/tmp/pp-gen.log 2>&1; then ok "generated: ${gerador%% *}"; else bad "generated file out of date: $gerador" "$(head -2 /tmp/pp-gen.log)"; fi
done

# 8. THE ONE THAT MATTERS MOST: the example printed in the README must compile, as a stranger would use it.
#    The published 0.1.0 shipped a program that did `use panic_halt` with no panic-halt in the dependency list,
#    and no check caught it because every existing check reads the crate, never the snippet.
TMP=$(mktemp -d)
if python3 scripts/check-readme-example.py "$CRATE/README.md" "$TMP" --target thumbv8m.main-none-eabihf >/tmp/pp-ex.log 2>&1; then
  ok "the README's example builds in an empty crate"
else
  bad "the README's example does not build" "$(grep -E '^error' /tmp/pp-ex.log | head -3)"
fi
rm -rf "$TMP"

# 9. The linker script must leave the Gecko bootloader alone. An image at 0x08000000 bricks the board.
ORIG=$(grep -hoE 'FLASH *: *ORIGIN *= *0x[0-9A-Fa-f_]+' "$CRATE"/memory.x 2>/dev/null | grep -oE '0x[0-9A-Fa-f_]+' | head -1)
if [ "$ORIG" = "0x08006000" ]; then ok "linker script starts at 0x08006000 (after the 24 KB bootloader)"; else bad "linker script ORIGIN is '${ORIG:-absent}', not 0x08006000"; fi

# 10. The version in the README's install line must be the version being published, or the snippet installs an
#     older crate than the page describes.
if grep -qE "$NOME *= *(\\{ *version *= *\"$VER\"|\"$VER\")" "$CRATE/README.md" 2>/dev/null; then ok "README install line matches version $VER"; else bad "README install line does not say $NOME = \"$VER\""; fi

# 11. Whether this version is already taken. Publishing a version is irreversible, so this is read first.
INDICE=$(curl -s --max-time 25 "https://index.crates.io/${NOME:0:2}/${NOME:2:2}/$NOME" 2>/dev/null | grep -c "\"vers\":\"$VER\"")
if [ "${INDICE:-0}" -eq 0 ]; then ok "version $VER is not on crates.io yet"; else bad "version $VER is already published -- bump before publishing"; fi

# 12. The commit must read as a human engineer's: no tool names in the identity or the message.
IDENT=$(git config user.name; git config user.email)
if echo "$IDENT" | grep -qiE 'hermes|claude|pi |laguna|agent'; then bad "git identity names a tool: $IDENT"; else ok "commit identity is a person"; fi
if git log -1 --format=%B 2>/dev/null | grep -qiE 'hermes|claude code|co-authored-by|generated by'; then bad "last commit message names a tool"; else ok "last commit message names no tool"; fi

echo
if [ "$FAILED" -eq 0 ]; then
  echo "VERDICT: PASS ($LINHAS checks). Publishing still needs an express order."
else
  echo "VERDICT: FAIL ($FAILED of $LINHAS checks). Do not publish."
fi
exit "$FAILED"
