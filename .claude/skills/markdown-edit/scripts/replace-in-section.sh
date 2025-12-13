#!/bin/bash
# Replace text within a specific markdown section
# Usage: bash replace-in-section.sh <file> <level> <heading> <find> <replace>

set -e

if [ "$#" -ne 5 ]; then
    echo "Usage: $0 <file> <level> <heading> <find> <replace>"
    echo "Example: $0 README.md 2 \"Installation\" \"old-text\" \"new-text\""
    exit 1
fi

FILE="$1"
LEVEL="$2"
HEADING="$3"
FIND="$4"
REPLACE="$5"

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

# Use the global search-markdown script to extract section
SECTION_SCRIPT="$HOME/.claude/skills/search-markdown/scripts/extract-section.sh"

if [ ! -f "$SECTION_SCRIPT" ]; then
    echo "Error: search-markdown extract-section.sh not found at $SECTION_SCRIPT"
    exit 1
fi

# Extract section to temp file
TEMP_SECTION=$(mktemp)
bash "$SECTION_SCRIPT" "$FILE" "$LEVEL" "$HEADING" > "$TEMP_SECTION"

if [ ! -s "$TEMP_SECTION" ]; then
    echo "Error: Section not found or empty"
    rm -f "$TEMP_SECTION"
    exit 1
fi

# Perform replacement in the section
sed -i.tmp "s|${FIND}|${REPLACE}|g" "$TEMP_SECTION"
rm -f "$TEMP_SECTION.tmp"

# Find section boundaries
HEADING_PATTERN="^$(printf '#%.0s' $(seq 1 "$LEVEL")) $HEADING"
START_LINE=$(grep -n "$HEADING_PATTERN" "$FILE" | head -1 | cut -d: -f1)

if [ -z "$START_LINE" ]; then
    echo "Error: Heading not found in file"
    rm -f "$TEMP_SECTION"
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
cat "$TEMP_SECTION" >> "$TEMP_FILE"
tail -n +"$((START_LINE + END_LINE + 1))" "$FILE" >> "$TEMP_FILE"

mv "$TEMP_FILE" "$FILE"
rm -f "$TEMP_SECTION"

echo "Replaced '$FIND' with '$REPLACE' in section '$HEADING' of $FILE"
echo "To restore: cp $BACKUP_PATH $FILE"
