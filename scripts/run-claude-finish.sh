#!/usr/bin/env bash
# Dispatch a full implementation run to Claude Code, with Opus planning and Sonnet executing.
#
# WHY THIS SCRIPT EXISTS (and why it is not a one-liner)
#
#   * A dispatch is the most expensive action a project takes. It must be reviewable before it spends anything, and
#     re-runnable identically after it dies. An inline command exists only in a terminal history.
#   * The environment a background shell inherits is not the environment the tool needs: PATH, IS_SANDBOX and the
#     provider credentials have to be set here. A run that worked because the calling shell happened to have the
#     right PATH is not reproducible.
#   * The session id is the only handle for a resume after a turn cap, and it is unrecoverable once the process exits,
#     so it is captured to a file at dispatch time.
#
# MODEL
#
#   `opusplan` is the Claude Code alias for "Opus while planning, Sonnet while executing". It is the account tier of
#   this project's cost policy: the subscription (not the paid Messages API) does the judgement-heavy work. Verified
#   on this machine: a one-turn `--model opusplan` probe reports `modelUsage: claude-sonnet-5-5` for the execution
#   turn, which is what the alias means.
#
# USAGE
#
#   bash scripts/run-claude-finish.sh <brief-file> [--resume [session-id]]
#
#   Without --resume it starts a new session; with --resume and no id it continues the most recent session recorded in
#   _tmp; with an explicit id it continues that one. RESUME FIRST is the standing rule: a fresh start re-derives
#   context the previous run already paid for.
#
# ENV OVERRIDES
#
#   MODEL=opusplan    MAX_TURNS=200    BRIEF_LABEL=ble-finish
set -u

PROJECT=/projects/efr32mg24-rs
STATE_DIR=/projects/_tmp/efr32mg24-rs/claude-finish
BRIEF="${1:-/projects/_tmp/efr32mg24-rs/BLE-FINISH-BRIEF.md}"
shift || true

RESUME=0
SESSION=""
while [ $# -gt 0 ]; do
  case "$1" in
    --resume)
      RESUME=1
      shift
      if [ $# -gt 0 ] && [ "${1#-}" = "$1" ]; then
        SESSION="$1"
        shift
      fi
      ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

MODEL="${MODEL:-opusplan}"
# `opusplan` switches by MODE, not per turn: `plan` runs Opus, anything else runs Sonnet. Verified on this machine
# with one-turn probes: plan -> claude-opus-5-5, bypassPermissions -> claude-sonnet-5-5. Pinning the mode to
# execution for a whole session is therefore the same as pinning the model to Sonnet, which is what round 1 did
# without saying so.
PERM="${PERM:-bypassPermissions}"
MAX_TURNS="${MAX_TURNS:-200}"
LABEL="${BRIEF_LABEL:-ble-finish}"
CLAUDE_BIN="${CLAUDE_BIN:-$HOME/.local/bin/claude}"
# A subagent definition file, passed with `--agents`. This is how a cheap executor reports to an expensive reviewer
# WITHOUT resuming a long session: a subagent has its own context window, so the reviewer does not re-read the whole
# history (measured: a resumed session costs ~$0.7-1.2/turn, a subagent request costs a fraction of that). The file
# form of `--agents` requires Claude Code v2.1.281+; this machine runs 2.1.300.
AGENTS_FILE="${AGENTS_FILE:-}"

# **Resuming is the default rule in this project (AGENTS.md HARD RULE 0e) -- and it is affordable only if the history
# is compacted.** Measured on 2026-11-13: a 286-turn single run carried ~179k tokens of context per turn at
# $0.018/turn, while resuming that same session for 8 turns carried ~310k per turn at $0.72-1.18. The cost is the
# per-turn context, not the resume: keep the thread, shrink the window.
#
#   AUTOCOMPACT=auto     let Claude Code decide (default)
#   AUTOCOMPACT=120000   force a window, e.g. a quarter of a 1M-context model, which keeps recent work verbatim
#   AUTOCOMPACT=0        pass nothing, i.e. leave whatever the session already had
#
# **You can also compact by hand, before spending anything, and it works in `-p`.** Measured 2026-11-13 on a scratch
# session: `claude -p "/compact" --resume <sid>` returned `subtype: success` for $0.086 and wrote a
# "This session is being continued from a previous conversation" message of 3489 characters in place of the history --
# the request carried the extract, not the log. The transcript file on disk *grows* while this happens, because it is
# an append-only log; that is not evidence of failure, and judging compaction by file size is the wrong instrument.
#
#   bash scripts/run-claude-finish.sh <brief> --resume <sid>     # AUTOCOMPACT=120000 to force a window
#
# and if the session is already large:
#
#   claude -p "/compact" --resume <sid>                          # one call, then continue normally
#
# The trade is the same one compaction always makes: the extract keeps intent, measured facts, the list of things
# ruled out and the next step, and loses the raw tool output. A hand-written extract (this project's HANDOFF) is
# cheaper still, but whoever writes it decides what mattered -- and the model's own summary has the advantage of
# having seen everything, including what turned out not to matter.
AUTOCOMPACT="${AUTOCOMPACT:-auto}"
FORK="${FORK:-0}"

# The tool's own environment, set here rather than inherited.
export PATH="/root/.local/bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin"
export IS_SANDBOX="${IS_SANDBOX:-1}"

if [ ! -f "$BRIEF" ]; then
  echo "brief not found: $BRIEF" >&2
  echo "a dispatch must fail in one second, not after two hundred turns" >&2
  exit 1
fi
if [ ! -x "$CLAUDE_BIN" ]; then
  echo "claude not found or not executable: $CLAUDE_BIN" >&2
  exit 1
fi
mkdir -p "$STATE_DIR"

if [ "$RESUME" -eq 1 ] && [ -z "$SESSION" ] && [ -f "$STATE_DIR/session-$LABEL.txt" ]; then
  SESSION="$(cat "$STATE_DIR/session-$LABEL.txt")"
fi

# Everything the run will spend, printed before it spends it.
echo "== dispatch"
echo "   model        : $MODEL   (opus plans, sonnet executes)"
echo "   claude       : $($CLAUDE_BIN --version 2>&1 | head -1)"
echo "   brief        : $BRIEF"
echo "   payload      : $(wc -c < "$BRIEF") bytes, $(wc -l < "$BRIEF") lines"
echo "   max turns    : $MAX_TURNS"
echo "   workdir      : $PROJECT"
echo "   permission   : $PERM    (plan => Opus plans; anything else => Sonnet executes)"
if [ -n "$AGENTS_FILE" ]; then
  echo "   subagents    : $AGENTS_FILE ($(grep -c '"model"' "$AGENTS_FILE" 2>/dev/null) definition(s))"
fi
echo "   autocompact  : ${AUTOCOMPACT}   (the lever that makes a resumed session affordable)"
[ "$FORK" = "1" ] && echo "   fork         : yes (resume the history into a new session id)"
if [ "$RESUME" -eq 1 ]; then
  echo "   resume       : ${SESSION:-<none recorded>}"
else
  echo "   resume       : no (new session)"
fi
echo "   git HEAD     : $(git -C "$PROJECT" log --oneline -1)"
echo "   tree         : $(git -C "$PROJECT" status --porcelain | wc -l) changed path(s)"

cd "$PROJECT" || exit 1

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG="$STATE_DIR/run-$LABEL-$STAMP.jsonl"

ARGS=(--model "$MODEL" --permission-mode "$PERM" --max-turns "$MAX_TURNS"
      --output-format stream-json --verbose -p "$(cat "$BRIEF")")
if [ -n "$AGENTS_FILE" ] && [ -f "$AGENTS_FILE" ]; then
  ARGS=(--agents "$AGENTS_FILE" "${ARGS[@]}")
fi
# `--autocompact auto` is the tool's own default, so only pass it when it was set to something else.
if [ "$AUTOCOMPACT" != "0" ] && [ "$AUTOCOMPACT" != "auto" ]; then
  ARGS=(--autocompact "$AUTOCOMPACT" "${ARGS[@]}")
fi
[ "$FORK" = "1" ] && ARGS=(--fork-session "${ARGS[@]}")
if [ "$RESUME" -eq 1 ] && [ -n "$SESSION" ]; then
  ARGS=(--resume "$SESSION" "${ARGS[@]}")
  echo "   log          : $LOG"
  echo
  "$CLAUDE_BIN" "${ARGS[@]}" 2>&1 | tee "$LOG"
else
  echo "   log          : $LOG"
  echo
  "$CLAUDE_BIN" "${ARGS[@]}" 2>&1 | tee "$LOG"
fi

RC=$?
echo
echo "== afterwards"
echo "   exit code    : $RC"
echo "   git HEAD     : $(git -C "$PROJECT" log --oneline -1)"
echo "   tree         : $(git -C "$PROJECT" status --porcelain | wc -l) changed path(s)"
git -C "$PROJECT" status --porcelain | head -20

# Capture the session id from the stream, so a cut-off run can be resumed instead of restarted.
SESSION_SEEN="$(grep -o '"session_id":"[^"]*"' "$LOG" 2>/dev/null | head -1 | cut -d'"' -f4)"
if [ -n "$SESSION_SEEN" ]; then
  echo "$SESSION_SEEN" > "$STATE_DIR/session-$LABEL.txt"
  echo "   session      : $SESSION_SEEN"
  echo "   resume with  : bash scripts/run-claude-finish.sh $BRIEF --resume $SESSION_SEEN"
fi
exit "$RC"
