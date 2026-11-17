#!/usr/bin/env bash
# Install the repository's git hooks. Run once per clone.
#
# The pre-commit hook refuses a commit while `scripts/gate.sh` is red, which is a rule this repo has broken once and
# should not be able to break again. Hooks live in `.git/`, which is not part of the tree, so this script is what makes
# them reproducible.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

if [ ! -d .git ]; then
    echo "not a git repository: ${ROOT}" >&2
    exit 1
fi

mkdir -p .git/hooks
ln -sf ../../scripts/pre-commit-gate.sh .git/hooks/pre-commit
chmod +x scripts/pre-commit-gate.sh
echo "installed .git/hooks/pre-commit -> scripts/pre-commit-gate.sh"
