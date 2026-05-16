#!/bin/bash
# Build script for EFR32MG24 book
# Run from book/latex directory

set -e

echo "=== Building EFR32MG24 Embedded Rust Book ==="
echo "Date: $(date)"
echo "Working directory: $(pwd)"
echo ""

# Check for required files
echo "Checking files..."
if [ ! -f "main.tex" ]; then echo "Error: main.tex not found!"; exit 1; fi
if [ ! -f "preamble.tex" ]; then echo "Error: preamble.tex not found!"; exit 1; fi
echo "✓ All required files present"
echo ""

# Run pdflatex (first pass)
echo "=== First XeLaTeX pass ==="
if xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee build.log; then
    echo "✓ First pass completed"
else
    echo "✗ First pass failed - check build.log"
    echo ""
    echo "Last 50 lines of errors:"
    grep -B2 -A2 "Error\|error:\|Undefined" build.log | tail -50
    exit 1
fi
echo ""

# Run pdflatex (second pass for TOC)
echo "=== Second XeLaTeX pass ==="
if xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee -a build.log; then
    echo "✓ Second pass completed"
else
    echo "✗ Second pass failed - check build.log"
    exit 1
fi
echo ""

# Check if PDF was created
if [ -f "main.pdf" ]; then
    echo "=== Build successful! ==="
    ls -lh main.pdf
    echo ""
    echo "PDF info:"
    pdfinfo main.pdf 2>/dev/null | head -10 || echo "pdfinfo not available"
else
    echo "✗ Build failed - no PDF generated"
    exit 1
fi

echo ""
echo "=== Build complete ==="
