#!/usr/bin/env bash
# Run the bond cycle and print ONE paragraph for delivery, with no model in the loop.
#
# Why: the scheduled job that ran this at 10:45 on 2026-11-17 died with "Gemini HTTP 503 (UNAVAILABLE)" before it ever
# reached the script. A measurement that depends on a language model being up is not a measurement. This wrapper does
# the whole job in shell and prints the verdict, so the cron can be script-only.
set -u

PROJECT="/projects/efr32mg24-rs"
LOG="${LOG:-${PROJECT}/_tmp/ble-bond-cycle.log}"
BEFORE_LINES=$(wc -l < "${LOG}" 2>/dev/null || echo 0)

cd "${PROJECT}" || { echo "bond cycle: the project directory is missing"; exit 0; }
bash scripts/ble-bond-cycle.sh > /dev/null 2>&1

# Everything the cycle produced after the marker, one line per measurement.
NEW=$(tail -n +"$((BEFORE_LINES + 1))" "${LOG}" 2>/dev/null)

PAIR=$(printf '%s\n' "${NEW}" | grep -o "pair: successful=[0-9]* bonded_yes=[0-9]*" | tail -1)
RECONNECT=$(printf '%s\n' "${NEW}" | grep -o "reconnect after reset: connected=[0-9]* pin_or_key_missing=[0-9]*" | tail -1)
DEVICE=$(printf '%s\n' "${NEW}" | grep "device after reset+reconnect" | tail -1 |
    grep -oE "rsn=[0-9]+|sec=[0-9]+|miss=[0-9]+/[0-9a-f]+|rst=[0-9]+/[0-9a-f]+|bh=[0-9a-f]+/[0-9a-f]+|del=[0-9]+" | tr '\n' ' ')
HOSTKEY=$(printf '%s\n' "${NEW}" | grep -o "LTK in flash:.*" | tail -1)

if printf '%s\n' "${NEW}" | grep -q "5-hour limit"; then
    echo "bond cycle at $(date -u +%H:%MZ): stopped, the CLI window is closed. Nothing spent further."
    exit 0
fi
if [ -z "${PAIR}" ] && [ -z "${RECONNECT}" ]; then
    echo "bond cycle at $(date -u +%H:%MZ): the cycle produced no measurement -- check /tmp/cycle-build.log and /tmp/cycle-flash-b.log."
    exit 0
fi

# The verdict, in one sentence, from the numbers rather than from hope.
if printf '%s\n' "${DEVICE}" | grep -q "rsn=0 " && printf '%s\n' "${DEVICE}" | grep -q "sec=1"; then
    VERDICT="RECONNECT WORKS: the link stayed encrypted after the reset"
elif printf '%s\n' "${DEVICE}" | grep -q "rsn=4102"; then
    VERDICT="still open: the device drops the reconnect with 0x1006 (PIN or Key Missing)"
else
    VERDICT="still open: the reconnect was not encrypted"
fi

echo "bond cycle at $(date -u +%H:%MZ). ${PAIR:-pair: no line}, ${RECONNECT:-reconnect: no line}. Device after the reconnect: ${DEVICE:-no line}. ${HOSTKEY}. Verdict: ${VERDICT}."
