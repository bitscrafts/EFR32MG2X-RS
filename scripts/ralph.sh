#!/usr/bin/env bash
# A Ralph loop: feed one prompt, let the agent finish one task, throw the context away, repeat.
#
# WHY THIS SHAPE
#
#   Measured on this project: continuing a long session is not cheap. Round 1 cost $5.23 over 286 turns inside a single
#   run ($0.018/turn), but resuming that same session later cost $0.72/turn on Sonnet and $1.18/turn on Opus, because
#   every new turn re-sends ~300 KB of history. A loop that never resumes pays the brief once per iteration instead.
#
#   The technique is Geoffrey Huntley's "Ralph Wiggum": `while :; do cat PROMPT.md | claude-code; done`. Its whole
#   point is the opposite of a long session -- the memory lives on disk (the plan, the report, git), not in a context
#   window. Each iteration gets a fresh window, reads the plan, does ONE task, writes down what it measured, commits,
#   and exits. If it gets the wrong idea, the fix is to sharpen the plan on disk, not to argue with it in a session.
#
# GUARDS, because a loop that spends money needs a brake
#
#   * An iteration cap (default 6) -- `./ralph.sh 3` for three.
#   * A usage brake: before each iteration it reads the subscription usage and stops if the weekly figure is at or
#     above WEEKLY_MAX (default 93%). Running out mid-night with no one watching is the failure this prevents.
#   * A stop file: `touch _tmp/ralph/STOP` and the loop exits after the current iteration.
#   * Every iteration is logged separately, so a later reader can see which iteration produced which change.
#
# USAGE
#
#   bash scripts/ralph.sh [iterations]
#
#   PROMPT=path/to/prompt.md   the prompt fed to every iteration (default: _tmp/efr32mg24-rs/RALPH-PROMPT.md)
#   MODEL / PERM / AGENTS_FILE as in run-claude-finish.sh
set -u

PROJECT=/projects/efr32mg24-rs
STATE=/projects/_tmp/efr32mg24-rs/ralph
PROMPT="${PROMPT:-/projects/_tmp/efr32mg24-rs/RALPH-PROMPT.md}"
MAX_ITERS="${1:-6}"
WEEKLY_MAX="${WEEKLY_MAX:-93}"
MODEL="${MODEL:-opusplan}"
PERM="${PERM:-bypassPermissions}"
AGENTS_FILE="${AGENTS_FILE:-$PROJECT/.claude/agents/opus-reviewer.json}"
CLAUDE_BIN="${CLAUDE_BIN:-$HOME/.local/bin/claude}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin"
export IS_SANDBOX="${IS_SANDBOX:-1}"

mkdir -p "$STATE"

if [ ! -f "$PROMPT" ]; then
  echo "prompt not found: $PROMPT -- a loop without a mission is just a bill" >&2
  exit 1
fi

# The integer percent of the weekly subscription figure, or empty if it cannot be read.
#
# This first shipped as a two-group capture concatenated into one number, which turned "90.0%" into 900 and made the
# brake compare against a scale it could never reach -- a brake that never fires is worse than no brake. Kept as one
# integer percent so the comparison in the loop is the same scale as the number a human reads.
weekly_used() {
  bash /root/.hermes/scripts/usage-check.sh 2>/dev/null \
    | sed -n 's/.*7-day (weekly): *\([0-9][0-9]*\)\..*/\1/p' | head -1
}

echo "== ralph loop"
echo "   prompt     : $PROMPT ($(wc -c < "$PROMPT") bytes)"
echo "   iterations : $MAX_ITERS (stop file: $STATE/STOP)"
echo "   brake      : weekly usage >= ${WEEKLY_MAX}% stops the loop"
echo "   model/perm : $MODEL / $PERM"
echo "   subagents  : ${AGENTS_FILE:-none}"
echo "   HEAD       : $(git -C "$PROJECT" log --oneline -1)"
echo

for i in $(seq 1 "$MAX_ITERS"); do
  if [ -f "$STATE/STOP" ]; then
    echo "== stop file present, leaving before iteration $i"
    break
  fi

  used="$(weekly_used)"
  if [ -n "$used" ] && [ "$used" -ge "$WEEKLY_MAX" ]; then
    echo "== weekly usage at ${used}%, at or above the brake of ${WEEKLY_MAX}%; stopping before iteration $i"
    break
  fi

  echo "== iteration $i/$MAX_ITERS  (weekly ${used:-?}%, HEAD $(git -C "$PROJECT" rev-parse --short HEAD))"
  LOG="$STATE/iter-$i-$(date -u +%Y%m%dT%H%M%SZ).jsonl"

  # A fresh session every time: no --resume, so nothing from the previous window is re-sent.
  ARGS=(--model "$MODEL" --permission-mode "$PERM" --max-turns 120
        --output-format stream-json --verbose -p "$(cat "$PROMPT")")
  [ -f "$AGENTS_FILE" ] && ARGS=(--agents "$AGENTS_FILE" "${ARGS[@]}")

  ( cd "$PROJECT" && "$CLAUDE_BIN" "${ARGS[@]}" 2>&1 | tee "$LOG" | tail -4 )

  # What the iteration left behind, in one line each -- the only memory that survives it.
  echo "   HEAD now : $(git -C "$PROJECT" log --oneline -1)"
  echo "   changed  : $(git -C "$PROJECT" status --porcelain | wc -l) path(s)"
  echo "   log      : $LOG"

  # The gate is the objective part: a loop that keeps going with a red gate is making the tree worse.
  if ! bash "$PROJECT/scripts/gate.sh" >"$STATE/gate-$i.log" 2>&1; then
    echo "   gate     : FAIL (see $STATE/gate-$i.log) -- continuing, the next iteration's job is to fix it"
  else
    echo "   gate     : PASS"
  fi
  echo
done

echo "== ralph loop finished"
git -C "$PROJECT" log --oneline -5
