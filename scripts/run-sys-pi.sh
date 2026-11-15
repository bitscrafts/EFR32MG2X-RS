#!/usr/bin/env bash
# Dispatch the vendor-fetch job to pi with GLM-5.3 (ollama-cloud).
#
#   scripts/run-sys-pi.sh [-r|--resume [SESSION_FILE]]
#
# `--yolo` is not a shortcut taken lightly: the default approval mode is `always-ask`, and an
# unattended run has nobody to ask, so every write is denied and the model correctly concludes it is
# in read-only planning mode. Measured 2026-11-15: a full dispatch produced zero edits for that reason.
#
# Same shape as scripts/run-bsp-pi.sh, different brief and log directory. A dispatch is the most
# expensive action this project takes: it has to be reviewable before it runs, re-runnable identically
# after it fails, and it has to record the session id at launch -- that id is the only resume handle.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PROVIDER="ollama-cloud"
MODEL="glm-5.3:cloud"
BRIEF="$ROOT/briefs/sys-vendor-fetch.md"
LOGDIR="$ROOT/_tmp/pi-sys"
mkdir -p "$LOGDIR"
LOG="$LOGDIR/run-$(date +%Y%m%d-%H%M%S).log"
SESSION_FILE="$LOGDIR/session.txt"

export PATH="/root/.local/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin"
export CARGO_TARGET_DIR="/projects/_cache/targets"

[[ -f "$BRIEF" ]] || { echo "no brief at $BRIEF -- refusing to spend a run with nothing to do"; exit 1; }
command -v pi >/dev/null || { echo "pi not on PATH -- refusing"; exit 1; }

# A background shell does not inherit the interactive environment; load the provider key without
# printing it, and refuse to launch a run that cannot authenticate.
if [[ -z "${OLLAMA_API_KEY:-}" ]]; then
  if grep -q "export OLLAMA_API_KEY" /root/.bashrc 2>/dev/null; then
    # shellcheck disable=SC1090
    source /root/.bashrc >/dev/null 2>&1 || true
  fi
fi
[[ -n "${OLLAMA_API_KEY:-}" ]] || { echo "OLLAMA_API_KEY missing -- refusing"; exit 1; }

resume=0
session=""
if [[ "${1:-}" == "-r" || "${1:-}" == "--resume" ]]; then
  resume=1
  session="${2:-$(cat "$SESSION_FILE" 2>/dev/null || true)}"
  [[ -n "$session" ]] || { echo "resume asked for, but no session recorded"; exit 1; }
  [[ -f "$session" ]] || { echo "resume: session file not found: $session"; exit 1; }
fi

echo "=== dispatch plan, before spending anything ==="
echo "provider      : $PROVIDER"
echo "model         : $MODEL"
echo "brief         : $BRIEF ($(wc -c < "$BRIEF") bytes)"
echo "log           : $LOG"
echo "repo size     : $(du -sm crates | cut -f1) MB in crates/   (the criterion is >=40 MB smaller at the end)"
echo "head          : $(git log --oneline -1)"
echo "tree          : $(git status --porcelain | wc -l) change(s) uncommitted"
if [[ $resume -eq 1 ]]; then echo "mode          : RESUME $session"; else echo "mode          : new session"; fi
echo "============================================="

if [[ $resume -eq 1 ]]; then
  pi --yolo --provider "$PROVIDER" --model "$MODEL" --session "$session" -p "$(cat "$BRIEF")" 2>&1 | tee "$LOG"
else
  pi --yolo --provider "$PROVIDER" --model "$MODEL" -p "$(cat "$BRIEF")" 2>&1 | tee "$LOG"
fi
