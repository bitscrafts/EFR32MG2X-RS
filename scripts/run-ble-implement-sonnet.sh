#!/usr/bin/env bash
# Dispatch the implementation of the BLE plan to Sonnet, after Opus has written it.
#
# Why two stages: the analysis needs judgement about what to do, and the edits need patience with a large tree. Opus
# writes `_tmp/ble-plan/PLAN.md`; this run implements it, in the order the plan gives, and stops at the first step whose
# result does not match what the plan predicted -- reporting which step, rather than improvising.
#
# What it may do: edit the tree, build, run clippy/fmt, run the gate.
# What it may NOT do: flash the board (another process owns it), commit, or touch anything outside the repository.
set -u

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/ble-plan"
PLAN="${WORKDIR}/PLAN.md"
PROMPT_FILE="${WORKDIR}/implement-prompt.txt"
SID_FILE="${WORKDIR}/session-sonnet.txt"
MODEL="${MODEL:-claude-sonnet-5-5}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"

[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
[ -s "${PLAN}" ] || { echo "REFUSE: no plan at ${PLAN} -- run the Opus stage first"; exit 1; }

PROBE="$("${CLAUDE}" --model "${MODEL}" -p "Reply with the single word: READY" 2>&1 | tail -2)"
echo "probe: ${PROBE}"
case "${PROBE}" in
    *unrecognized_model*) echo "REFUSE: this CLI does not know ${MODEL}"; exit 1 ;;
    *READY*|*"ready"*) ;;
    *"5-hour limit"*|*"spend limit"*) echo "REFUSE: the account window is closed"; exit 2 ;;
    *) echo "REFUSE: probe did not answer"; exit 3 ;;
esac

if [ ! -f "${PROMPT_FILE}" ]; then
    cat > "${PROMPT_FILE}" <<'PROMPT'
Implement the plan in _tmp/ble-plan/PLAN.md, in the order it gives, in this repository.

Rules that are not negotiable:

- Work step by step. After each step, build (`CARGO_TARGET_DIR=/projects/_cache/targets cargo build --release --target
  thumbv8m.main-none-eabihf -p efr32mg2x-xiao-bsp --features sense,stack,external-bonding --example module_ble_dosimeter`)
  and keep the tree compiling. If a step's result contradicts what the plan predicted, STOP at that step and say so in
  your final message: do not improvise a different design.
- Do not flash the board and do not run `probe-rs`: another process owns it and a flash would destroy its state.
- Do not commit. Do not run `git add`. The orchestrator commits.
- Run `cargo fmt --all` and then `bash scripts/gate.sh` at the end; it must say `VERDICT: PASS`. If it fails, fix the
  cause rather than the message.
- Write as you go: append one line per completed step to `_tmp/ble-plan/IMPLEMENTED.md` with the file touched and the
  build status, because a five-hour window can close mid-run.
- Every change carries a comment saying why it is there, with the measurement or the citation that justifies it, in the
  style the file already uses. No emoji, no meta-commentary.
PROMPT
fi

SID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
echo "${SID}" > "${SID_FILE}"
OUT="${WORKDIR}/sonnet-$(date -u +%Y%m%dT%H%M%SZ).log"
echo "session: ${SID}"
echo "log:     ${OUT}"

cd "${PROJECT}"
"${CLAUDE}" --session-id "${SID}" --model "${MODEL}" --permission-mode acceptEdits --allowedTools "Bash" "Read" "Write" "Edit" "Grep" "Glob" --max-turns 120 \
    -p "$(cat "${PROMPT_FILE}")" > "${OUT}" 2>&1
RC=$?
echo "exit: ${RC}"
grep -qiE "5-hour limit|spend limit" "${OUT}" && echo "NOTE: the window closed during the run"
echo "steps done: $(wc -l < "${WORKDIR}/IMPLEMENTED.md" 2>/dev/null || echo 0)"
tail -c 600 "${OUT}"
exit "${RC}"
