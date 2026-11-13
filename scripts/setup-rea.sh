#!/usr/bin/env bash
# setup-rea.sh — install the reverse-engineering toolchain this project uses to read vendor blobs.
#
# Idempotent: every step checks before it acts, so running it twice costs nothing. The versions are pinned
# because REA rejects the others (measured 2026-11-13):
#
#   Node     22.23.4+ / 24.22.1+ / 26.12.1+   the container ships 26.7.0, which rea-agents REFUSES
#   JDK      25                               Ghidra 12.2 refuses older: "Select a 64-bit full JDK 25"
#   Ghidra   12.2 PUBLIC                      REA's bridge is verified against 12.1.4, accepts 12.x >= 12.1
#
# Usage: bash scripts/setup-rea.sh [--check]     (--check runs only the verification)
#
# After it succeeds, `docs/03-development/BLOB-DISASSEMBLY.md` §Level 2 has the commands that use it.
set -u

GHIDRA_DIR=${GHIDRA_DIR:-/projects/_tmp/tools}
GHIDRA_VER=ghidra_12.2_PUBLIC
GHIDRA_ZIP=ghidra_12.2_PUBLIC_20261016.zip
GHIDRA_URL="https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.2_build/$GHIDRA_ZIP"
JAVA_HOME_DIR=/usr/lib/jvm/java-25-openjdk-amd64
REA=/root/.local/lib/node_modules/rea-agents/scripts/rea.mjs
# The system node here is 26.7.0, which rea-agents REFUSES (it wants >=26.12.1, or 22/24). So every call goes
# through a pinned Node 22 fetched by npx -- slower on the first call, and the only version that works.
NODE22="npx -y node@22.23.4"

check_only=0
[ "${1:-}" = "--check" ] && check_only=1

verify() {
  export JAVA_HOME=$JAVA_HOME_DIR
  export PATH="/root/.local/bin:$JAVA_HOME/bin:/usr/local/bin:/usr/bin:/bin"
  export GHIDRA_INSTALL_DIR=$GHIDRA_DIR/$GHIDRA_VER
  [ -f "$REA" ] || { echo "FAIL: rea not installed"; return 1; }
  "$JAVA_HOME/bin/java" -version 2>&1 | head -1
  $NODE22 "$REA" capabilities 2>&1 | grep -A5 "id: ghidra" | grep -E "status:" | head -1
}

if [ "$check_only" = "1" ]; then
  verify
  exit $?
fi

echo "== 1/4 JDK 25 (Ghidra 12.2 refuses 21)"
if [ -x "$JAVA_HOME_DIR/bin/java" ]; then
  echo "   already present: $($JAVA_HOME_DIR/bin/java -version 2>&1 | head -1)"
else
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq && apt-get install -y -qq openjdk-25-jdk-headless || { echo "FAIL: apt"; exit 1; }
fi

echo "== 2/4 Ghidra 12.2"
if [ -x "$GHIDRA_DIR/$GHIDRA_VER/support/analyzeHeadless" ]; then
  echo "   already present: $GHIDRA_DIR/$GHIDRA_VER"
else
  mkdir -p "$GHIDRA_DIR" && cd "$GHIDRA_DIR" || exit 1
  [ -f "$GHIDRA_ZIP" ] || curl -sL -o "$GHIDRA_ZIP" "$GHIDRA_URL" || { echo "FAIL: download"; exit 1; }
  # python3 -m zipfile preserves the executable bit; zipfile.extractall does NOT, and the result is a Ghidra
  # whose analyzeHeadless exists but cannot run (REA then says "executable_missing" about a file that is there).
  python3 -m zipfile -e "$GHIDRA_ZIP" . || { echo "FAIL: extract"; exit 1; }
  chmod +x "$GHIDRA_VER"/support/analyzeHeadless "$GHIDRA_VER"/support/*.sh \
           "$GHIDRA_VER"/ghidraRun "$GHIDRA_VER"/Ghidra/Features/Decompiler/os/linux_x86_64/* 2>/dev/null
fi

echo "== 3/4 rea-agents"
if [ -f "$REA" ]; then
  echo "   already present: $(dirname "$(dirname "$REA")")"
else
  npm i -g rea-agents@latest || { echo "FAIL: npm"; exit 1; }
  # Warm the pinned Node so the first real call is not a download.
  $NODE22 --version
fi

echo "== 4/4 verify (the only step that proves the whole thing)"
verify
