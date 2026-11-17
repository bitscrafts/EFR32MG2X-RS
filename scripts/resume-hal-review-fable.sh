#!/usr/bin/env bash
# Resume the HAL/radio/BLE review in the SAME Fable session, once the account's five-hour window reopens.
#
# Why a script: a cron job runs in a fresh environment and inherits nothing. `PATH`, `IS_SANDBOX`, the CLI location and
# the session id all have to be set here, and the resume handle has to live outside the job (measured 2026-11-17: the
# run died on "You've hit your 5-hour limit", and everything it had learned lives only in that session).
#
# Usage: resume-hal-review-fable.sh [--now]
#   without --now it only probes and reports whether the window is open
set -u

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/review-hal-fable"
SID_FILE="${WORKDIR}/session-id.txt"
PROMPT_FILE="${WORKDIR}/resume-prompt.txt"
MODEL="${MODEL:-claude-fable-5-2}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

# A background shell inherits nothing: set what the dispatch needs, inside the dispatch.
export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"

[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
[ -f "${SID_FILE}" ] || { echo "REFUSE: no session id at ${SID_FILE}"; exit 1; }
[ -f "${PROMPT_FILE}" ] || { echo "REFUSE: no resume prompt at ${PROMPT_FILE}"; exit 1; }
SID="$(tr -d '[:space:]' < "${SID_FILE}")"

# The subject check, so a stale file cannot be answered faithfully in place of the review.
head -1 "${PROJECT}/_tmp/review-hal-fable/brief.md" | grep -q "^# HAL, radio and BLE review" \
    || { echo "REFUSE: the brief is not the HAL review"; exit 1; }

echo "session: ${SID}"
echo "model:   ${MODEL}"

# Step 1 -- probe, one turn, before spending anything on a long run.
echo "---- probe"
PROBE_OUT="$("${CLAUDE}" --resume "${SID}" --model "${MODEL}" -p "PONG" 2>&1 | tail -3)"
echo "${PROBE_OUT}"
case "${PROBE_OUT}" in
    *PONG*) ;;
    *"5-hour limit"*|*"spend limit"*)
        echo "REFUSE: the window is still closed; not resuming"
        exit 2 ;;
    *) echo "REFUSE: probe did not answer PONG"; exit 3 ;;
esac

if [ "${1:-}" != "--now" ]; then
    echo "probe OK -- window open (run with --now to resume the review)"
    exit 0
fi

# Step 2 -- resume, and keep the answer on disk: plan mode writes to ~/.claude/plans and the final message goes to
# stdout, so this file is the report.
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="${WORKDIR}/report-${STAMP}.md"
echo "---- resuming, report to ${OUT}"
"${CLAUDE}" --resume "${SID}" --model "${MODEL}" --permission-mode plan --max-turns 120 \
    -p "$(cat "${PROMPT_FILE}")" > "${OUT}" 2>&1
RC=$?
echo "exit: ${RC}"
echo "bytes: $(wc -c < "${OUT}")"
echo "report: ${OUT}"
exit "${RC}"
