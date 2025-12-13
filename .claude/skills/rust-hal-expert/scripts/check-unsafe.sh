#!/bin/bash
# Check for unsafe blocks in Rust code
# Usage: bash check-unsafe.sh <directory>

if [ -z "$1" ]; then
    echo "Usage: $0 <directory>"
    exit 1
fi

echo "=== Unsafe Blocks Found ==="
grep -rn "unsafe" "$1" --include="*.rs" | grep -v "// Safety:" | grep -v "/// # Safety"

echo ""
echo "=== Unsafe Blocks With Safety Comments ==="
grep -rn "unsafe" "$1" --include="*.rs" -A 2 | grep -E "(// Safety:|/// # Safety)" -B 2

echo ""
echo "=== Summary ==="
total=$(grep -r "unsafe" "$1" --include="*.rs" | wc -l)
documented=$(grep -r "unsafe" "$1" --include="*.rs" -A 2 | grep -E "(// Safety:|/// # Safety)" | wc -l)
echo "Total unsafe blocks: $total"
echo "Documented unsafe: $documented"
echo "Undocumented unsafe: $((total - documented))"
