#!/usr/bin/env bash
# Dispatch ONE analysis session to a Claude model. Copy into the repo's scripts/ and commit it.
#
# Everything a background shell does not inherit is set here: PATH, IS_SANDBOX, the CLI path. The session id is GIVEN to
# the CLI, so `--resume` works later. The subject of the brief is checked before spending anything.
#
# Usage: analysis-dispatch.sh <brief.md> <session-id-file> [--resume <uuid>]
set -u

BRIEF="${1:?usage: analysis-dispatch.sh <brief.md> <session-id-file> [--resume <uuid>]}"
SID_FILE="${2:?usage: analysis-dispatch.sh <brief.md> <session-id-file> [--resume <uuid>]}"
shift 2
RESUME=""
[ "${1:-}" = "--resume" ] && RESUME="${2:-}"

MODEL="${MODEL:-claude-fable-5-2}"      # or claude-opus-5-5; never the `opusplan` alias for analysis -- it resolves to sonnet
MAX_TURNS="${MAX_TURNS:-80}"            # an analysis that wanders is not improved by more turns
EXPECTED_SUBJECT="${EXPECTED_SUBJECT:-#}"   # the first line of the brief must start with this
LOG="${LOG:-$(dirname "${SID_FILE}")/dispatch-$(date -u +%Y%m%dT%H%M%SZ).log}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"    # required by the CLI's test mode, harmless otherwise
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

[ -x "${CLAUDE}" ]  || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }
[ -f "${BRIEF}" ]   || { echo "REFUSE: no brief at ${BRIEF}"; exit 1; }

# 0. The model must be RECOGNISED. An unknown id answers anyway, with another model, and the log looks fine.
PROBE="$("${CLAUDE}" --model "${MODEL}" -p "PONG" 2>&1 | tail -3)"
case "${PROBE}" in
    *unrecognized_model*) echo "REFUSE: this CLI does not know ${MODEL} -- run claude update"; echo "${PROBE}"; exit 1 ;;
    *PONG*)               echo "model: ${MODEL} recognised" ;;
    *"5-hour limit"*)     echo "REFUSE: the account window is closed"; exit 2 ;;
    *)                    echo "REFUSE: probe did not answer"; echo "${PROBE}"; exit 3 ;;
esac

# 1. The brief must be on subject: a stale brief is answered faithfully, and the window is spent on the wrong question.
head -1 "${BRIEF}" | grep -qF -- "${EXPECTED_SUBJECT}" \
    || { echo "REFUSE: ${BRIEF} first line is not '${EXPECTED_SUBJECT}...'"; head -1 "${BRIEF}"; exit 1; }

# 2. Session: given on creation, or resumed.
if [ -n "${RESUME}" ]; then
    SID="${RESUME}"
    SID_ARGS=(--resume "${SID}")
    echo "resuming ${SID}"
else
    SID="$(python3 -c 'import uuid; print(uuid.uuid4())')"
    SID_ARGS=(--session-id "${SID}")
    echo "new session ${SID}"
fi
echo "${SID}" > "${SID_FILE}"

echo "brief:  ${BRIEF} ($(wc -c < "${BRIEF}") bytes)"
echo "log:    ${LOG}"
echo "turns:  ${MAX_TURNS}"

# 3. Run. plan mode: read and report, never edit.
"${CLAUDE}" "${SID_ARGS[@]}" --model "${MODEL}" --permission-mode plan --max-turns "${MAX_TURNS}" \
    -p "$(cat "${BRIEF}")" > "${LOG}" 2>&1
RC=$?

echo "exit: ${RC}"
# A spend limit can appear with exit 0 on some paths: grep for it rather than trusting the status.
grep -qiE "5-hour limit|spend limit" "${LOG}" && echo "NOTE: the account window closed during the run"
echo "session kept in ${SID_FILE} for --resume"
tail -c 400 "${LOG}"
exit "${RC}"
