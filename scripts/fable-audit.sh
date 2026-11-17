#!/usr/bin/env bash
# Run ONE of the three audits as its own session, so a five-hour limit can cost at most one of them.
#
# Why not one big review with sub-agents: measured 2026-11-17, two consecutive windows were spent and neither produced a
# line of findings — the agents read for 78-81 tool calls each and the limit closed before anything was written. The
# audits are now independent sessions, each owning one file in parts/, each told to write as it establishes a finding.
#
# Usage: fable-audit.sh core|bus|radio [--fresh]
#   --fresh   ignore any existing part file (default: skip if the part file already has findings)
set -u

WHICH="${1:?usage: fable-audit.sh core|bus|radio [--fresh]}"
shift || true
FRESH=0
[ "${1:-}" = "--fresh" ] && FRESH=1

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/review-hal-fable"
BRIEF="${WORKDIR}/briefs/audit-${WHICH}.md"
PART="${WORKDIR}/parts/audit-${WHICH}.md"
SID_FILE="${WORKDIR}/session-${WHICH}.txt"
MODEL="${MODEL:-claude-fable-5-2}"
MAX_TURNS="${MAX_TURNS:-80}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
[ -f "${BRIEF}" ] || { echo "REFUSE: no brief at ${BRIEF}"; exit 1; }

# A stale brief answered faithfully is the classic failure: check the subject before spending.
case "$(head -1 "${BRIEF}")" in
    "# Audit "[123]"/3:"*) ;;
    *) echo "REFUSE: ${BRIEF} is not an audit brief"; exit 1 ;;
esac

# Skip when the work is already there, unless asked to redo it.
if [ "${FRESH}" -eq 0 ] && [ -f "${PART}" ]; then
    FOUND="$(grep -c '^## ' "${PART}" 2>/dev/null || echo 0)"
    if [ "${FOUND}" -ge 5 ]; then
        echo "SKIP ${WHICH}: ${PART} already has ${FOUND} findings"
        exit 0
    fi
fi

mkdir -p "${WORKDIR}/parts"
: >> "${PART}"   # the audit appends here; creating it makes the target obvious and its absence visible

SID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
echo "${SID}" > "${SID_FILE}"
echo "---- audit ${WHICH}, session ${SID}, max ${MAX_TURNS} turns, model ${MODEL}"
echo "---- part file: ${PART}"

cd "${PROJECT}"
"${CLAUDE}" --session-id "${SID}" --model "${MODEL}" --permission-mode plan --max-turns "${MAX_TURNS}" \
    -p "$(cat "${BRIEF}")" > "${WORKDIR}/parts/audit-${WHICH}.log" 2>&1
RC=$?
echo "exit: ${RC}   findings: $(grep -c '^## ' "${PART}" 2>/dev/null || echo 0)   bytes: $(wc -c < "${PART}")"
tail -c 300 "${WORKDIR}/parts/audit-${WHICH}.log"
exit "${RC}"
