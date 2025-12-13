#!/bin/bash
# Apply same find/replace to multiple markdown files
# Usage: bash bulk-update.sh <pattern> <find> <replace> [--dry-run]

set -e

if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <pattern> <find> <replace> [--dry-run]"
    echo "Example: $0 \"src/*/README.md\" \"Phase 5\" \"Phase A\" --dry-run"
    exit 1
fi

PATTERN="$1"
FIND="$2"
REPLACE="$3"
DRY_RUN="${4:-}"

# Find matching files
FILES=$(find . -path "./$PATTERN" 2>/dev/null || true)

if [ -z "$FILES" ]; then
    echo "No files matched pattern: $PATTERN"
    exit 1
fi

echo "Files matching pattern '$PATTERN':"
echo "$FILES" | while read -r file; do
    echo "  - $file"
done
echo ""

if [ "$DRY_RUN" = "--dry-run" ]; then
    echo "DRY RUN MODE - Showing what would change:"
    echo ""

    echo "$FILES" | while read -r file; do
        if [ -f "$file" ]; then
            MATCHES=$(grep -n "$FIND" "$file" 2>/dev/null || true)
            if [ -n "$MATCHES" ]; then
                echo "File: $file"
                echo "$MATCHES" | head -5
                echo ""
            fi
        fi
    done

    echo "Run without --dry-run to apply changes"
    exit 0
fi

# Actually perform replacements
FILES_CHANGED=0

echo "$FILES" | while read -r file; do
    if [ -f "$file" ] && grep -q "$FIND" "$file" 2>/dev/null; then
        # Create backup
        cp "$file" "$file.bak"

        # Perform replacement
        sed -i.tmp "s|${FIND}|${REPLACE}|g" "$file"
        rm -f "$file.tmp"

        echo "Updated: $file"
        FILES_CHANGED=$((FILES_CHANGED + 1))
    fi
done

echo ""
echo "Bulk update complete"
echo "Files changed: Check individual .bak files for each modified file"
echo "To restore all: find . -name '*.md.bak' -exec sh -c 'mv \"\$1\" \"\${1%.bak}\"' _ {} \\;"
