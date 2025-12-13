#!/bin/bash
# Add content to end of a section (before next heading)
# Usage: bash append-to-section.sh <file> <level> <heading> <content-to-append>

set -e

if [ "$#" -ne 4 ]; then
    echo "Usage: $0 <file> <level> <heading> <content-to-append>"
    echo "Example: $0 docs/STATUS.md 2 \"Current Status\" \"\\n**Note**: Updated December 13, 2025\""
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

# Create backup
cp "$FILE" "$FILE.bak"
echo "Backup created: $FILE.bak"

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
    # No next heading, append to end of file
    TEMP_FILE=$(mktemp)
    cat "$FILE" > "$TEMP_FILE"
    echo -e "\n$CONTENT" >> "$TEMP_FILE"
    mv "$TEMP_FILE" "$FILE"
else
    # Insert before next heading
    INSERT_LINE=$((START_LINE + END_LINE))
    TEMP_FILE=$(mktemp)
    head -n "$((INSERT_LINE - 1))" "$FILE" > "$TEMP_FILE"
    echo -e "$CONTENT" >> "$TEMP_FILE"
    tail -n +"$INSERT_LINE" "$FILE" >> "$TEMP_FILE"
    mv "$TEMP_FILE" "$FILE"
fi

echo "Appended content to section '$HEADING' in $FILE"
echo "To restore: mv $FILE.bak $FILE"
