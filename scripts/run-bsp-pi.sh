#!/usr/bin/env bash
# Dispatch the BSP-finishing job to pi with GLM-5.3 (ollama-cloud).
#
#   scripts/run-bsp-pi.sh [-r|--resume [SESSION_FILE]]
#
# Why a script (HARD RULE 0d): a dispatch is the most expensive action this project takes. It has to
# be reviewable before it runs, re-runnable identically after it fails, and it has to record the session
# id at launch -- that id is the only handle for a resume, and it is unrecoverable once the process exits.
#
# Everything the run needs is set here, not inherited from an interactive shell: PATH, the cargo env,
# the target dir, and the provider credentials check.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROVIDER="ollama-cloud"
MODEL="glm-5.3:cloud"
BRIEF="$ROOT/briefs/bsp-finish.md"
LOGDIR="$ROOT/_tmp/pi-bsp"
mkdir -p "$LOGDIR"
LOG="$LOGDIR/run-$(date +%Y%m%d-%H%M%S).log"
SESSION_FILE="$LOGDIR/session.txt"

export PATH="/root/.local/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin"
export CARGO_TARGET_DIR="/projects/_cache/targets"

[[ -f "$BRIEF" ]] || { echo "no brief at $BRIEF -- refusing to spend a run with nothing to do"; exit 1; }
command -v pi >/dev/null || { echo "pi not on PATH -- refusing"; exit 1; }

# A background shell does not inherit the interactive environment, and the first dispatch failed
# exactly here: `No API key found for provider ollama-cloud`. Load the key from the file that defines
# it, without ever printing it. Fail loudly rather than launching a run that cannot authenticate.
if [[ -z "${OLLAMA_API_KEY:-}" ]]; then
  if grep -q "export OLLAMA_API_KEY" /root/.bashrc 2>/dev/null; then
    # shellcheck disable=SC1090
    source /root/.bashrc >/dev/null 2>&1 || true
  fi
fi
if [[ -z "${OLLAMA_API_KEY:-}" ]]; then
  echo "OLLAMA_API_KEY is not set and could not be loaded from /root/.bashrc -- refusing to launch"
  exit 1
fi

resume=0
session=""
if [[ "${1:-}" == "-r" || "${1:-}" == "--resume" ]]; then
  resume=1
  session="${2:-$(cat "$SESSION_FILE" 2>/dev/null || true)}"
  [[ -n "$session" ]] || { echo "resume asked for, but no session id recorded"; exit 1; }
  [[ -f "$session" ]] || { echo "resume: session file not found: $session"; exit 1; }
fi

echo "=== dispatch plan, before spending anything ==="
echo "provider      : $PROVIDER"
echo "model         : $MODEL"
echo "brief         : $BRIEF ($(wc -c < "$BRIEF") bytes)"
echo "log           : $LOG"
echo "target dir    : $CARGO_TARGET_DIR"
echo "head          : $(git log --oneline -1)"
echo "tree          : $(git status --porcelain | wc -l) change(s) uncommitted"
if [[ $resume -eq 1 ]]; then
  echo "mode          : RESUME $session"
else
  echo "mode          : new session"
fi
echo "============================================="

if [[ $resume -eq 1 ]]; then
  exec pi --yolo --provider "$PROVIDER" --model "$MODEL" --session "$session" -p "$(cat "$BRIEF")" 2>&1 | tee "$LOG"
else
  exec pi --yolo --provider "$PROVIDER" --model "$MODEL" -p "$(cat "$BRIEF")" 2>&1 | tee "$LOG"
fi
