#!/usr/bin/env bash
# Resume the Sonnet implementation run with shell access, so it can verify what it edits.
#
# The first run stopped after step 1: `--permission-mode acceptEdits` allows edits but not `cargo build`, so the model
# did the edit it could do and correctly refused to pretend it had verified anything. This resumes the SAME session
# (its context already holds the plan and step 1) with the tools it was missing.
set -u

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/ble-plan"
SID_FILE="${WORKDIR}/session-sonnet.txt"
MODEL="${MODEL:-claude-sonnet-5-5}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"

[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
SID="$(tr -d '[:space:]' < "${SID_FILE}" 2>/dev/null || true)"
[ -n "${SID}" ] || { echo "REFUSE: no session to resume"; exit 1; }

PROMPT='The build is now allowed: this session has Bash, so `cargo build`, `cargo check`, `cargo fmt` and
`bash scripts/gate.sh` will run. Verify step 1 compiles (fix what it broke), then continue with the plan in
_tmp/ble-plan/PLAN.md from step 2 onwards, following the same rules: keep the tree compiling after every step, stop at
the first step whose result contradicts the plan, never flash the board, never commit, and append one line per completed
step to _tmp/ble-plan/IMPLEMENTED.md.'

OUT="${WORKDIR}/sonnet-resume-$(date -u +%Y%m%dT%H%M%SZ).log"
echo "session: ${SID}"
echo "log:     ${OUT}"

cd "${PROJECT}"
"${CLAUDE}" --resume "${SID}" --model "${MODEL}" --permission-mode acceptEdits \
    --allowedTools "Bash" "Read" "Write" "Edit" "Grep" "Glob" --max-turns 140 \
    -p "${PROMPT}" > "${OUT}" 2>&1
RC=$?
echo "exit: ${RC}"
grep -qiE "5-hour limit|spend limit" "${OUT}" && echo "NOTE: the window closed during the run"
echo "steps recorded: $(wc -l < "${WORKDIR}/IMPLEMENTED.md" 2>/dev/null || echo 0)"
tail -c 700 "${OUT}"
exit "${RC}"
