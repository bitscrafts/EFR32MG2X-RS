#!/usr/bin/env bash
# Dispatch the f20 (transmit timeout) instrument to Sonnet.
#
# Why a script and not an inline command: the dispatch is the expensive action, it must be reviewable before it runs
# and re-runnable identically after it fails, and the session id is the only handle for a resume once the process
# exits. See HARD RULE 0d.
#
# What it may do: edit the tree, build, run the gate, flash the board on `lnx`, read the mailbox.
# What it may NOT do: commit, stash, checkout, reset, delete anything.
#
#   scripts/run-hal-tx-timeout-sonnet.sh            # fresh session
#   scripts/run-hal-tx-timeout-sonnet.sh --resume   # continue the recorded session
set -u

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/f20"
BRIEF="${WORKDIR}/brief.md"
SID_FILE="${WORKDIR}/session-sonnet.txt"
LOG="${WORKDIR}/run-$(date +%Y%m%d-%H%M%S).log"
MODEL="${MODEL:-claude-sonnet-5-5}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"
export TMPDIR="/projects/_tmp/tmp"
export CARGO_TARGET_DIR="/projects/_tmp/tmp/target-cold"

[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
[ -s "${BRIEF}" ] || { echo "REFUSE: no brief at ${BRIEF}"; exit 1; }

RESUME=0
for arg in "$@"; do
    case "${arg}" in
        --resume) RESUME=1 ;;
        *) echo "REFUSE: unknown argument '${arg}'"; exit 1 ;;
    esac
done

echo "model:     ${MODEL} (Anthropic, account tier)"
echo "project:   ${PROJECT}"
echo "brief:     ${BRIEF} ($(wc -c < "${BRIEF}") bytes)"
echo "log:       ${LOG}"
echo "target:    ${CARGO_TARGET_DIR}"
echo "tree:      $(cd "${PROJECT}" && git status --porcelain | wc -l) file(s) modified, HEAD $(cd "${PROJECT}" && git log --oneline -1)"
echo

PROBE="$("${CLAUDE}" --model "${MODEL}" -p "Reply with the single word: READY" 2>&1 | tail -2)"
echo "probe: ${PROBE}"
case "${PROBE}" in
    *unrecognized_model*) echo "REFUSE: this CLI does not know ${MODEL}"; exit 1 ;;
    *READY*|*Ready*|*ready*) ;;
    *"5-hour limit"*|*"spend limit"*) echo "REFUSE: the account window is closed"; exit 2 ;;
    *) echo "REFUSE: probe did not answer"; exit 3 ;;
esac

mkdir -p "${WORKDIR}"

if [ "${RESUME}" = "1" ] && [ -s "${SID_FILE}" ]; then
    SID="$(cat "${SID_FILE}")"
    echo "resume:    ${SID}"
    "${CLAUDE}" --model "${MODEL}" --resume "${SID}" -p "$(cat "${BRIEF}")" 2>&1 | tee "${LOG}"
else
    "${CLAUDE}" --model "${MODEL}" -p "$(cat "${BRIEF}")" 2>&1 | tee "${LOG}"
    SID="$(grep -oE '"session_id":"[^"]+"' "${LOG}" | tail -1 | cut -d'"' -f4)"
    [ -n "${SID}" ] && echo "${SID}" > "${SID_FILE}" && echo "session:   ${SID} (recorded in ${SID_FILE})"
fi

echo
echo "tree after the run:"
(cd "${PROJECT}" && git status --porcelain)
echo "HEAD: $(cd "${PROJECT}" && git log --oneline -1)"
