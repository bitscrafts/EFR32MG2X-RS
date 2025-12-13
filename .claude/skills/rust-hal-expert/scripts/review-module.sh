#!/bin/bash
# Automated checks for a Rust HAL module
# Usage: bash review-module.sh <file.rs>

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <file.rs>"
    echo "Example: $0 src/gpio/mod.rs"
    exit 1
fi

FILE="$1"

if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found"
    exit 1
fi

echo "=== Module Review: $FILE ==="
echo ""

# Basic metrics
echo "--- Metrics ---"
echo "Lines of code: $(wc -l < "$FILE")"
echo "Public items: $(grep -c "^pub " "$FILE" || echo 0)"
echo "Unsafe blocks: $(grep -c "unsafe" "$FILE" || echo 0)"
echo "Critical sections: $(grep -c "critical_section::with" "$FILE" || echo 0)"
echo ""

# Check for unsafe without documentation
echo "--- Unsafe Analysis ---"
UNSAFE_COUNT=$(grep -n "unsafe" "$FILE" | grep -v "// Safety:" | grep -v "/// # Safety" | wc -l || echo 0)
if [ "$UNSAFE_COUNT" -gt 0 ]; then
    echo "⚠️  Found $UNSAFE_COUNT unsafe blocks without Safety comments:"
    grep -n "unsafe" "$FILE" | grep -v "// Safety:" | grep -v "/// # Safety"
else
    echo "✅ All unsafe blocks are documented"
fi
echo ""

# Check for panics
echo "--- Panic Check ---"
PANIC_COUNT=$(grep -n "panic!\|unwrap()\|expect(" "$FILE" | wc -l || echo 0)
if [ "$PANIC_COUNT" -gt 0 ]; then
    echo "⚠️  Found $PANIC_COUNT potential panics:"
    grep -n "panic!\|unwrap()\|expect(" "$FILE"
else
    echo "✅ No panics found"
fi
echo ""

# Check for TODO/FIXME
echo "--- TODO/FIXME ---"
TODO_COUNT=$(grep -n "TODO\|FIXME" "$FILE" | wc -l || echo 0)
if [ "$TODO_COUNT" -gt 0 ]; then
    echo "Found $TODO_COUNT items:"
    grep -n "TODO\|FIXME" "$FILE"
else
    echo "No TODOs or FIXMEs"
fi
echo ""

# Check for inline annotations
echo "--- Optimization ---"
INLINE_COUNT=$(grep -c "#\[inline\]" "$FILE" || echo 0)
echo "Inline functions: $INLINE_COUNT"
echo ""

# Check error handling
echo "--- Error Handling ---"
RESULT_COUNT=$(grep -c "Result<" "$FILE" || echo 0)
ERROR_ENUM=$(grep -c "pub enum.*Error" "$FILE" || echo 0)
echo "Result returns: $RESULT_COUNT"
echo "Error enums: $ERROR_ENUM"
echo ""

# Check embedded-hal traits
echo "--- embedded-hal Integration ---"
grep -n "impl.*embedded_hal" "$FILE" || echo "No embedded-hal traits implemented"
echo ""

echo "=== Review Complete ==="
