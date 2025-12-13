#!/bin/bash
# Extract common HAL patterns for comparison
# Usage: bash compare-patterns.sh <directory>

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <directory>"
    echo "Example: $0 src/clock/"
    exit 1
fi

DIR="$1"

if [ ! -d "$DIR" ]; then
    echo "Error: Directory '$DIR' not found"
    exit 1
fi

echo "=== Common HAL Patterns in $DIR ==="
echo ""

echo "--- Struct Definitions ---"
grep -rn "^pub struct" "$DIR" --include="*.rs" | head -20
echo ""

echo "--- Peripheral Consumption (new/init methods) ---"
grep -rn "pub fn new\|pub fn init" "$DIR" --include="*.rs" -A 2 | head -30
echo ""

echo "--- Critical Sections ---"
grep -rn "critical_section::with" "$DIR" --include="*.rs" -B 1 -A 3 | head -30
echo ""

echo "--- Inline Accessors ---"
grep -rn "#\[inline\]" "$DIR" --include="*.rs" -A 2 | head -20
echo ""

echo "--- Error Types ---"
grep -rn "pub enum.*Error" "$DIR" --include="*.rs" -A 5 | head -20
echo ""

echo "--- Type States ---"
grep -rn "PhantomData\|marker::PhantomData" "$DIR" --include="*.rs" | head -10
echo ""

echo "=== Pattern Summary ==="
echo "Structs: $(grep -r "^pub struct" "$DIR" --include="*.rs" | wc -l)"
echo "Critical sections: $(grep -r "critical_section::with" "$DIR" --include="*.rs" | wc -l)"
echo "Inline functions: $(grep -r "#\[inline\]" "$DIR" --include="*.rs" | wc -l)"
echo "Error types: $(grep -r "pub enum.*Error" "$DIR" --include="*.rs" | wc -l)"
