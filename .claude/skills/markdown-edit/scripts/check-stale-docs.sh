#!/bin/bash
# check-stale-docs.sh - Check if markdown files need "Last Updated" field updates
# Usage: bash check-stale-docs.sh [file_pattern]
#
# Checks if the "Last Updated" date in markdown files is older than the last git commit date.
# Useful for finding documentation that hasn't been updated after code changes.

set -euo pipefail

PATTERN="${1:-./**/*.md}"

echo "=== Checking for Stale Documentation ==="
echo "Pattern: $PATTERN"
echo ""

STALE_COUNT=0
TOTAL_COUNT=0

# Find all markdown files matching pattern (excluding .archive/)
while IFS= read -r file; do
    [[ "$file" == *".archive/"* ]] && continue

    TOTAL_COUNT=$((TOTAL_COUNT + 1))

    # Extract "Last Updated" date from last 5 lines of file
    LAST_UPDATED=$(tail -5 "$file" | grep -E "^\*\*Last Updated\*\*:" | sed 's/\*\*Last Updated\*\*: //' | tr -d '\n' || echo "")

    if [[ -z "$LAST_UPDATED" ]]; then
        echo "⚠️  MISSING: $file"
        echo "   No 'Last Updated' field found in last 5 lines"
        STALE_COUNT=$((STALE_COUNT + 1))
        continue
    fi

    # Get last git commit date for this file
    if git ls-files --error-unmatch "$file" >/dev/null 2>&1; then
        GIT_DATE=$(git log -1 --format="%cs" -- "$file" 2>/dev/null || echo "")

        if [[ -z "$GIT_DATE" ]]; then
            echo "ℹ️  NEW: $file"
            echo "   Last Updated: $LAST_UPDATED (not yet committed)"
            continue
        fi

        # Compare dates (YYYY-MM-DD format)
        if [[ "$LAST_UPDATED" < "$GIT_DATE" ]]; then
            echo "🔴 STALE: $file"
            echo "   Last Updated: $LAST_UPDATED"
            echo "   Last Commit:  $GIT_DATE"
            echo "   Needs update: $(( ($(date -j -f "%Y-%m-%d" "$GIT_DATE" +%s) - $(date -j -f "%Y-%m-%d" "$LAST_UPDATED" +%s)) / 86400 )) days behind"
            STALE_COUNT=$((STALE_COUNT + 1))
        elif [[ "$LAST_UPDATED" == "$GIT_DATE" ]]; then
            echo "✅ CURRENT: $file (updated $LAST_UPDATED)"
        else
            echo "⏰ FUTURE: $file"
            echo "   Last Updated: $LAST_UPDATED"
            echo "   Last Commit:  $GIT_DATE"
            echo "   (Timestamp is in the future - might be intentional)"
        fi
    else
        echo "ℹ️  UNTRACKED: $file"
        echo "   Last Updated: $LAST_UPDATED (file not in git)"
    fi

    echo ""
done < <(find . -type f -name "*.md" 2>/dev/null)

echo "=== Summary ==="
echo "Total files checked: $TOTAL_COUNT"
echo "Stale/missing: $STALE_COUNT"
echo ""

if [[ $STALE_COUNT -gt 0 ]]; then
    echo "⚠️  Found $STALE_COUNT file(s) that need updating"
    echo "Update 'Last Updated' field to current date when modifying documentation"
    exit 1
else
    echo "✅ All documentation timestamps are current"
    exit 0
fi
