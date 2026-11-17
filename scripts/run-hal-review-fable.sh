#!/usr/bin/env bash
# Dispatch the HAL review to Fable 5.2 in a NAMED, REATTACHABLE session.
#
# The user's standing rule (2026-09-21): never a fire-and-forget `claude -p` for analysis, because the session's
# accumulated context is worth continuing — `claude --resume <session_id>` re-enters it for a fraction of the cost.
# So: a fixed session id is generated once, written where the next run can find it, and passed explicitly.
#
# Plan mode is deliberate: this run reads and reports, it does not edit the tree.
#
# Usage: run-hal-review-fable.sh [--resume]
set -eu

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/review-hal-fable"
BRIEF="${WORKDIR}/brief.md"
SID_FILE="${WORKDIR}/session-id.txt"
MODEL="${MODEL:-claude-fable-5-2}"
MAX_TURNS="${MAX_TURNS:-200}"
mkdir -p "${WORKDIR}"

# P14 from the opus-review skill: a stale brief is answered faithfully. Print the first line and stop if it is not the
# subject.
echo "---- brief first line:"
head -1 "${BRIEF}"
case "$(head -1 "${BRIEF}")" in
    "# HAL, radio and BLE review:"*) ;;
    *) echo "REFUSE: ${BRIEF} is not the HAL review brief" >&2; exit 1 ;;
esac

# P13: test mode refuses to run as root without this; plan mode does not need it, but a later resume may.
export IS_SANDBOX="${IS_SANDBOX:-1}"
# A background shell does not inherit the interactive environment: this is why the dispatch sets its own PATH.
export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"
[ -x "${CLAUDE}" ] || { echo "REFUSE: claude not found at ${CLAUDE}" >&2; exit 1; }

RESUME_ARGS=()
if [ "${1:-}" = "--resume" ] && [ -f "${SID_FILE}" ]; then
    SID="$(cat "${SID_FILE}")"
    echo "---- resuming session ${SID}"
    RESUME_ARGS=(--resume "${SID}")
else
    # The id has to be GIVEN to the CLI, not imagined here: without `--session-id` the id written below is a fiction
    # and a later `--resume` fails on a session that does not exist (measured 2026-11-17).
    SID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
    echo "${SID}" > "${SID_FILE}"
    RESUME_ARGS=(--session-id "${SID}")
    echo "---- new session ${SID} (kept in ${SID_FILE})"
fi

echo "---- model ${MODEL}, plan mode, max ${MAX_TURNS} turns"

cd "${PROJECT}"
exec "${CLAUDE}" "${RESUME_ARGS[@]}" \
    --model "${MODEL}" \
    --permission-mode plan \
    --max-turns "${MAX_TURNS}" \
    -p "$(cat "${BRIEF}")"
