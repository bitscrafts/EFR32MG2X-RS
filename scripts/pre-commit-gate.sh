#!/usr/bin/env bash
# Refuse to commit while the gate is red.
#
# A commit made with the gate red is a claim that the tree is good, written at the exact moment the tree says it is
# not; the fix that follows it turns a lie into a tidy history and hides the fact that it was ever broken. This repo
# has already paid for that once (2026-11-17: a commit landed with `cargo fmt --check` failing).
#
# Installed as `.git/hooks/pre-commit` by `scripts/install-git-hooks.sh`, and kept here so a fresh clone gets the same
# rule. Set SKIP_GATE_HOOK=1 only for a commit that cannot possibly need it -- a documentation-only change on a machine
# without the toolchain -- and say so in the message.
set -u

if [ "${SKIP_GATE_HOOK:-0}" = "1" ]; then
    echo "pre-commit: gate SKIPPED on request (SKIP_GATE_HOOK=1)" >&2
    exit 0
fi

cd "$(dirname "$0")/../.." || exit 1

echo "pre-commit: running the gate; a commit is only allowed on a green tree" >&2
LOG="$(mktemp)"
if bash scripts/gate.sh >"${LOG}" 2>&1; then
    tail -1 "${LOG}" >&2
    rm -f "${LOG}"
    exit 0
fi

echo "pre-commit: GATE FAILED -- commit refused" >&2
grep -E "FAIL" "${LOG}" | head -10 >&2
echo "pre-commit: full log at ${LOG}" >&2
exit 1
