#!/bin/bash
# Smart find and replace with markdown awareness
# Usage: bash smart-replace.sh <file> <find> <replace> [--in-code|--skip-code]

set -e

if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <file> <find> <replace> [--in-code|--skip-code]"
    echo "Example: $0 README.md \"stm32-rs\" \"efr32-rs\" --skip-code"
    exit 1
fi

FILE="$1"
FIND="$2"
REPLACE="$3"
MODE="${4:---in-code}"  # Default: replace everywhere

if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found"
    exit 1
fi

# Create backup
cp "$FILE" "$FILE.bak"
echo "Backup created: $FILE.bak"

if [ "$MODE" = "--skip-code" ]; then
    # Skip code blocks
    TEMP_FILE=$(mktemp)
    IN_CODE_BLOCK=0

    while IFS= read -r line; do
        # Detect code block boundaries
        if echo "$line" | grep -q '^```'; then
            if [ $IN_CODE_BLOCK -eq 0 ]; then
                IN_CODE_BLOCK=1
            else
                IN_CODE_BLOCK=0
            fi
            echo "$line" >> "$TEMP_FILE"
        elif [ $IN_CODE_BLOCK -eq 1 ]; then
            # Inside code block, don't replace
            echo "$line" >> "$TEMP_FILE"
        else
            # Outside code block, do replace
            echo "$line" | sed "s|${FIND}|${REPLACE}|g" >> "$TEMP_FILE"
        fi
    done < "$FILE"

    mv "$TEMP_FILE" "$FILE"
    echo "Replaced '$FIND' with '$REPLACE' in text (skipped code blocks)"
else
    # Replace everywhere including code blocks
    sed -i.tmp "s|${FIND}|${REPLACE}|g" "$FILE"
    rm -f "$FILE.tmp"
    echo "Replaced '$FIND' with '$REPLACE' everywhere (including code blocks)"
fi

echo "Updated: $FILE"
echo "To restore: mv $FILE.bak $FILE"
