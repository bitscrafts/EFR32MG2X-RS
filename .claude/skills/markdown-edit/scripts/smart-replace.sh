#!/bin/bash
# Smart find and replace with markdown awareness
# Usage: bash smart-replace.sh <file> <find> <replace> [--in-code|--skip-code]
#
# IMPORTANT: For multi-line replacements, use the Edit tool directly instead.
# This script handles single-line and simple multi-line patterns.

set -e

if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <file> <find> <replace> [--in-code|--skip-code]"
    echo "Example: $0 README.md \"stm32-rs\" \"efr32-rs\" --skip-code"
    echo ""
    echo "Note: For complex multi-line replacements, use the Edit tool directly."
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

# Ensure .archive directory exists
mkdir -p .archive

# Create backup in .archive with timestamp
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME=$(echo "$FILE" | sed 's|/|_|g')
BACKUP_PATH=".archive/${BACKUP_NAME}_${TIMESTAMP}"
cp "$FILE" "$BACKUP_PATH"
echo "Backup created: $BACKUP_PATH"

# Use Python for robust string replacement (handles all special characters and multi-line)
python3 - <<EOF
import sys
import re

# Read the file
with open('$FILE', 'r', encoding='utf-8') as f:
    content = f.read()

# Get search and replace strings (passed as environment variables for safety)
find_str = '''$FIND'''
replace_str = '''$REPLACE'''

# Mode: skip code or replace everywhere
mode = '$MODE'

if mode == '--skip-code':
    # Split by code blocks and only replace outside them
    parts = content.split('```')
    for i in range(0, len(parts), 2):  # Even indices are outside code blocks
        parts[i] = parts[i].replace(find_str, replace_str)
    result = '```'.join(parts)
    print("Replaced text (skipped code blocks)", file=sys.stderr)
else:
    # Replace everywhere (literal string replacement, not regex)
    result = content.replace(find_str, replace_str)
    print("Replaced text everywhere (including code blocks)", file=sys.stderr)

# Write result back
with open('$FILE', 'w', encoding='utf-8') as f:
    f.write(result)
EOF

echo "Updated: $FILE"
echo "To restore: cp $BACKUP_PATH $FILE"
