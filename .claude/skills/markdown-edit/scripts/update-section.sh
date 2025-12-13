#!/bin/bash
# Replace entire section content while keeping heading
# Usage: bash update-section.sh <file> <level> <heading> <new-content-file>

set -e

if [ "$#" -ne 4 ]; then
    echo "Usage: $0 <file> <level> <heading> <new-content-file>"
    echo "Example: $0 README.md 2 \"Usage\" /tmp/new-usage.txt"
    exit 1
fi

FILE="$1"
LEVEL="$2"
HEADING="$3"
NEW_CONTENT="$4"

if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found"
    exit 1
fi

if [ ! -f "$NEW_CONTENT" ]; then
    echo "Error: New content file '$NEW_CONTENT' not found"
    exit 1
fi

# Ensure .archive directory exists
mkdir -p .archive

# Create backup in .archive with timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME=$(echo "$FILE" | sed 's|/|_|g')
BACKUP_PATH=".archive/${BACKUP_NAME}_${TIMESTAMP}"
cp "$FILE" "$BACKUP_PATH"
echo "Backup created: $BACKUP_PATH"

# Find section heading
HEADING_PATTERN="^$(printf '#%.0s' $(seq 1 "$LEVEL")) $HEADING"
START_LINE=$(grep -n "$HEADING_PATTERN" "$FILE" | head -1 | cut -d: -f1)

if [ -z "$START_LINE" ]; then
    echo "Error: Heading not found in file"
    exit 1
fi

# Find next heading of same or higher level
NEXT_HEADING_PATTERN="^$(printf '#%.0s' $(seq 1 "$LEVEL")) "
END_LINE=$(tail -n +$((START_LINE + 1)) "$FILE" | grep -n "$NEXT_HEADING_PATTERN" | head -1 | cut -d: -f1)

if [ -z "$END_LINE" ]; then
    # No next heading, section goes to end of file
    END_LINE=$(wc -l < "$FILE")
    END_LINE=$((END_LINE - START_LINE))
else
    END_LINE=$((END_LINE - 1))
fi

# Reconstruct file
TEMP_FILE=$(mktemp)
head -n "$START_LINE" "$FILE" > "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
cat "$NEW_CONTENT" >> "$TEMP_FILE"
echo "" >> "$TEMP_FILE"
tail -n +"$((START_LINE + END_LINE + 1))" "$FILE" >> "$TEMP_FILE"

mv "$TEMP_FILE" "$FILE"

echo "Updated section '$HEADING' in $FILE"
echo "To restore: cp $BACKUP_PATH $FILE"
