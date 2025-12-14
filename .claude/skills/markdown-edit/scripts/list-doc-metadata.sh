#!/bin/bash
# list-doc-metadata.sh - List metadata for all markdown files
# Usage: bash list-doc-metadata.sh [directory]
#
# Scans all markdown files and displays their metadata in a table format.
# Useful for getting an overview of documentation status.

set -euo pipefail

DIR="${1:-.}"

echo "=== Documentation Metadata Report ==="
echo "Directory: $DIR"
echo ""

# Header
printf "%-50s | %-12s | %-10s | %-8s\n" "File" "Last Updated" "Version" "Status"
printf "%-50s-+-%-12s-+-%-10s-+-%-8s\n" "$(printf '%.0s-' {1..50})" "$(printf '%.0s-' {1..12})" "$(printf '%.0s-' {1..10})" "$(printf '%.0s-' {1..8})"

TOTAL=0
MISSING=0

while IFS= read -r file; do
    # Skip .archive directory
    [[ "$file" == *".archive/"* ]] && continue

    TOTAL=$((TOTAL + 1))

    # Extract metadata from last 10 lines
    FOOTER=$(tail -10 "$file")
    LAST_UPDATED=$(echo "$FOOTER" | grep -E "^\*\*Last Updated\*\*:" | sed 's/\*\*Last Updated\*\*: //' | tr -d '\n' || echo "N/A")
    VERSION=$(echo "$FOOTER" | grep -E "^\*\*Version\*\*:" | sed 's/\*\*Version\*\*: //' | tr -d '\n' || echo "N/A")
    TEMPLATE_VERSION=$(echo "$FOOTER" | grep -E "^\*\*Template Version\*\*:" | sed 's/\*\*Template Version\*\*: //' | tr -d '\n' || echo "")
    VERSION="${VERSION:-$TEMPLATE_VERSION}"
    STATUS=$(echo "$FOOTER" | grep -E "^\*\*Status\*\*:" | sed 's/\*\*Status\*\*: //' | tr -d '\n' || echo "N/A")

    # Count files with missing metadata
    [[ "$LAST_UPDATED" == "N/A" || "$LAST_UPDATED" == "YYYY-MM-DD" ]] && MISSING=$((MISSING + 1))

    # Shorten file path if too long
    DISPLAY_FILE="$file"
    if [[ ${#DISPLAY_FILE} -gt 50 ]]; then
        DISPLAY_FILE="...${DISPLAY_FILE: -47}"
    fi

    printf "%-50s | %-12s | %-10s | %-8s\n" "$DISPLAY_FILE" "$LAST_UPDATED" "$VERSION" "$STATUS"

done < <(find "$DIR" -type f -name "*.md" 2>/dev/null | sort)

echo ""
echo "=== Summary ==="
echo "Total files: $TOTAL"
echo "Missing/placeholder metadata: $MISSING"

if [[ $MISSING -gt 0 ]]; then
    echo ""
    echo "⚠️  Files with missing 'Last Updated' metadata:"
    while IFS= read -r file; do
        [[ "$file" == *".archive/"* ]] && continue
        FOOTER=$(tail -10 "$file")
        LAST_UPDATED=$(echo "$FOOTER" | grep -E "^\*\*Last Updated\*\*:" | sed 's/\*\*Last Updated\*\*: //' | tr -d '\n' || echo "N/A")
        if [[ "$LAST_UPDATED" == "N/A" || "$LAST_UPDATED" == "YYYY-MM-DD" ]]; then
            echo "  - $file"
        fi
    done < <(find "$DIR" -type f -name "*.md" 2>/dev/null | sort)
fi
