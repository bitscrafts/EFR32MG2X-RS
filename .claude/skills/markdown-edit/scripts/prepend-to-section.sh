#!/bin/bash
# Add content to beginning of section (after heading)
# Usage: bash prepend-to-section.sh <file> <level> <heading> <content-to-prepend>

set -e

if [ "$#" -ne 4 ]; then
    echo "Usage: $0 <file> <level> <heading> <content-to-prepend>"
    echo "Example: $0 README.md 2 \"Installation\" \"⚠️ **Breaking Changes in v0.2.0** - See migration guide\\n\""
    exit 1
fi

FILE="$1"
LEVEL="$2"
HEADING="$3"
CONTENT="$4"

if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found"
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

# Insert after heading
INSERT_LINE=$((START_LINE + 1))
TEMP_FILE=$(mktemp)
head -n "$START_LINE" "$FILE" > "$TEMP_FILE"
echo -e "\n$CONTENT" >> "$TEMP_FILE"
tail -n +"$INSERT_LINE" "$FILE" >> "$TEMP_FILE"
mv "$TEMP_FILE" "$FILE"

echo "Prepended content to section '$HEADING' in $FILE"
echo "To restore: cp $BACKUP_PATH $FILE"
