#!/bin/bash
# update-timestamp.sh - Update "Last Updated" field in markdown files
# Usage: bash update-timestamp.sh <file> [date]
#
# Updates the "Last Updated: YYYY-MM-DD" field in the last 10 lines of a markdown file.
# If date is not provided, uses current date.
# Creates .archive backup before modification.

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <file> [date]"
    echo "Example: $0 docs/STATUS.md"
    echo "Example: $0 docs/STATUS.md 2025-12-13"
    exit 1
fi

FILE="$1"
DATE="${2:-$(date +%Y-%m-%d)}"

if [[ ! -f "$FILE" ]]; then
    echo "Error: File '$FILE' not found"
    exit 1
fi

# Validate date format
if ! [[ "$DATE" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    echo "Error: Date must be in YYYY-MM-DD format"
    exit 1
fi

# Check if file has "Last Updated" field in last 10 lines
if ! tail -10 "$FILE" | grep -q "^\*\*Last Updated\*\*:"; then
    echo "Error: File does not have '**Last Updated**:' field in last 10 lines"
    echo "Hint: Add this line manually first:"
    echo "**Last Updated**: YYYY-MM-DD"
    exit 1
fi

# Create .archive backup
ARCHIVE_DIR=".archive"
mkdir -p "$ARCHIVE_DIR"
BACKUP_NAME=$(echo "$FILE" | tr '/' '_')
BACKUP_FILE="$ARCHIVE_DIR/${BACKUP_NAME%.md}_$(date +%Y%m%d_%H%M%S).md"
cp "$FILE" "$BACKUP_FILE"
echo "✅ Backup created: $BACKUP_FILE"

# Update the Last Updated field
# Use awk to update only in the last 10 lines
TEMP_FILE=$(mktemp)
awk -v date="$DATE" '
{
    lines[NR] = $0
}
END {
    for (i = 1; i <= NR; i++) {
        # Only update in last 10 lines
        if (i > NR - 10 && lines[i] ~ /^\*\*Last Updated\*\*:/) {
            print "**Last Updated**: " date
        } else {
            print lines[i]
        }
    }
}
' "$FILE" > "$TEMP_FILE"

mv "$TEMP_FILE" "$FILE"

echo "✅ Updated '$FILE'"
echo "   Last Updated: $DATE"
echo ""
echo "Verify changes:"
echo "  tail -5 '$FILE'"
