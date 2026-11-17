#!/usr/bin/env bash
# Dispatch the BLE operational plan to Opus, which analyses and WRITES the plan to a file.
#
# Then Sonnet implements from that file (scripts/run-ble-implement-sonnet.sh). Two stages, two models, because the
# analysis and the edits need different things: judgement, and patience.
#
# Follows the house discipline: a script (never an inline command), a recognised model, a named reattachable session, a
# probe before spending, and the deliverable is a FILE.
set -u

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/ble-plan"
BRIEF="${WORKDIR}/brief.md"
SID_FILE="${WORKDIR}/session-opus.txt"
MODEL="${MODEL:-claude-opus-5-5}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"

[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
[ -f "${BRIEF}" ] || { echo "REFUSE: no brief at ${BRIEF}"; exit 1; }
head -1 "${BRIEF}" | grep -qF "# BLE: what must change so a phone pairs once" \
    || { echo "REFUSE: the brief is not the BLE plan question"; exit 1; }

# Model recognised? An unknown id answers anyway, with another model.
PROBE="$("${CLAUDE}" --model "${MODEL}" -p "PONG" 2>&1 | tail -3)"
echo "probe: ${PROBE}"
case "${PROBE}" in
    *unrecognized_model*) echo "REFUSE: this CLI does not know ${MODEL} -- run claude update"; exit 1 ;;
    *PONG*) ;;
    *"5-hour limit"*|*"spend limit"*) echo "REFUSE: the account window is closed"; exit 2 ;;
    *) echo "REFUSE: probe did not answer"; exit 3 ;;
esac

SID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
echo "${SID}" > "${SID_FILE}"
OUT="${WORKDIR}/opus-$(date -u +%Y%m%dT%H%M%SZ).log"
echo "session: ${SID}"
echo "log:     ${OUT}"

cd "${PROJECT}"
# acceptEdits, not plan mode: this run is asked to write the plan. The brief confines it to _tmp/ble-plan/.
"${CLAUDE}" --session-id "${SID}" --model "${MODEL}" --permission-mode acceptEdits --max-turns 90 \
    -p "$(cat "${BRIEF}")" > "${OUT}" 2>&1
RC=$?
echo "exit: ${RC}"
grep -qiE "5-hour limit|spend limit" "${OUT}" && echo "NOTE: the window closed during the run"
echo "plan: $(ls -la "${WORKDIR}/PLAN.md" 2>/dev/null | awk '{print $5" bytes"}')"
tail -c 500 "${OUT}"
exit "${RC}"
