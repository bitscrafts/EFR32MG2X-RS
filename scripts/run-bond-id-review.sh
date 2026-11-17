#!/usr/bin/env bash
# Dispatch the bond-identifier question to the analysis model, once the account window has reopened.
#
# The question is narrow and the brief carries every number, so this run is short: one session, plan mode, bounded
# turns. It writes its answer to a file -- the same rule as the HAL review: the file is the deliverable.
set -u

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/review-bond-id"
BRIEF="${WORKDIR}/brief.md"
SID_FILE="${WORKDIR}/session-id.txt"
MODEL="${MODEL:-claude-fable-5-2}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"

[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
[ -f "${BRIEF}" ] || { echo "REFUSE: no brief at ${BRIEF}"; exit 1; }
head -1 "${BRIEF}" | grep -qF "# Why does the security manager ask PSA for key 0x48003" \
    || { echo "REFUSE: the brief is not the bond-identifier question"; exit 1; }

# The window probe: a fresh, one-turn session. A resume probe cannot tell "window closed" from "session busy".
PROBE="$("${CLAUDE}" --model "${MODEL}" -p "PONG" 2>&1 | tail -2)"
echo "probe: ${PROBE}"
case "${PROBE}" in
    *PONG*) ;;
    *"5-hour limit"*|*"spend limit"*) echo "REFUSE: window still closed"; exit 2 ;;
    *) echo "REFUSE: probe did not answer"; exit 3 ;;
esac

SID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
echo "${SID}" > "${SID_FILE}"
OUT="${WORKDIR}/answer-$(date -u +%Y%m%dT%H%M%SZ).md"
echo "session: ${SID}"
echo "answer:  ${OUT}"

cd "${PROJECT}"
"${CLAUDE}" --session-id "${SID}" --model "${MODEL}" --permission-mode plan --max-turns 70 \
    -p "$(cat "${BRIEF}")" > "${OUT}" 2>&1
RC=$?
echo "exit: ${RC}"
echo "bytes: $(wc -c < "${OUT}")"
grep -qiE "5-hour limit|spend limit" "${OUT}" && echo "NOTE: the window closed during the run"
tail -c 400 "${OUT}"
exit "${RC}"
