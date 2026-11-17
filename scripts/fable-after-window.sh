#!/usr/bin/env bash
# After the account's five-hour window reopens: probe once, then run the three audits in sequence, then synthesise.
#
# This exists because two windows were spent on 2026-11-17 without producing a single finding: the review ran as one
# session with three sub-agents, they read for 78-81 tool calls each, and the limit closed before anything was written.
# The design now is one audit per session, each owning one file, each writing findings as it establishes them. If the
# window closes again, at most one audit is lost, and the ones already on disk are skipped next time.
#
# The probe runs in a FRESH tiny session: a resume probe cannot distinguish "window closed" from "session busy with
# work", because a session with work in flight answers with the work instead of PONG (measured 2026-11-17).
set -u

PROJECT="/projects/efr32mg24-rs"
WORKDIR="${PROJECT}/_tmp/review-hal-fable"
SID_FILE="${WORKDIR}/session-id.txt"
PROMPT_FILE="${WORKDIR}/resume-prompt.txt"
REPORT="${WORKDIR}/REPORT.md"
MODEL="${MODEL:-claude-fable-5-2}"

export PATH="/root/.local/bin:/usr/local/cargo/bin:${PATH}"
export IS_SANDBOX="${IS_SANDBOX:-1}"
CLAUDE="${CLAUDE:-/root/.local/bin/claude}"

echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) fable review, after the window"
[ -x "${CLAUDE}" ] || { echo "REFUSE: no claude at ${CLAUDE}"; exit 1; }

# 1. Probe. Fresh, one turn, cheap.
PROBE="$(timeout 120 "${CLAUDE}" --model "${MODEL}" -p "PONG" 2>&1 | tail -2)"
echo "probe: ${PROBE}"
case "${PROBE}" in
    *PONG*) echo "window: open" ;;
    *"5-hour limit"*|*"spend limit"*)
        echo "REFUSE: window still closed; nothing spent further"; exit 2 ;;
    *) echo "REFUSE: probe did not answer"; exit 3 ;;
esac

# 2. Salvage anything the earlier runs left in their sub-agent transcripts.
for OLD in $(cat "${SID_FILE}" 2>/dev/null); do
    [ -n "${OLD}" ] && bash "${PROJECT}/scripts/salvage-fable-subagents.sh" "${OLD}" || true
done

# 3. The three audits, in order, each skipping itself if its file already holds findings.
for A in core bus radio; do
    echo "---- audit ${A}"
    bash "${PROJECT}/scripts/fable-audit.sh" "${A}"
    RC=$?
    # A spend limit stops the sequence: there is no point starting the next audit into a closed window.
    tail -c 200 "${WORKDIR}/parts/audit-${A}.log" 2>/dev/null | grep -q "5-hour limit" && { echo "window closed during ${A}; stopping"; break; }
    [ "${RC}" -eq 2 ] && break
done

# 4. Synthesise what exists into one report, with the parts referenced rather than copied.
{
    echo "# HAL, radio and BLE review -- assembled from three audits"
    echo
    echo "Assembled $(date -u +%Y-%m-%dT%H:%M:%SZ) from:"
    for A in core bus radio; do
        F="${WORKDIR}/parts/audit-${A}.md"
        [ -f "${F}" ] && echo "- \`parts/audit-${A}.md\` -- $(grep -c '^## ' "${F}") findings, $(wc -c < "${F}") bytes"
    done
    echo
    for A in core bus radio; do
        F="${WORKDIR}/parts/audit-${A}.md"
        [ -s "${F}" ] || continue
        echo
        echo "## audit-${A}"
        echo
        cat "${F}"
    done
} > "${REPORT}"

echo "report: ${REPORT}"
echo "bytes:  $(wc -c < "${REPORT}")"
echo "findings per audit:"
for A in core bus radio; do echo "  ${A}: $(grep -c '^## ' "${WORKDIR}/parts/audit-${A}.md" 2>/dev/null || echo 0)"; done
