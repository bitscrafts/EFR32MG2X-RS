#!/bin/bash
# check-format.sh - Check or fix code formatting
# Usage: bash check-format.sh [--fix]

set -e

FIX=false

# Parse arguments
if [[ "$1" == "--fix" ]]; then
    FIX=true
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}🔍 Checking code formatting...${NC}"

if [[ "$FIX" == "true" ]]; then
    echo "Running cargo fmt --all..."
    cargo fmt --all
    echo -e "${GREEN}✅ Code formatted${NC}"
else
    if cargo fmt --all -- --check; then
        echo -e "${GREEN}✅ Formatting check passed${NC}"
    else
        echo -e "${RED}❌ Formatting check failed${NC}"
        echo -e "${YELLOW}Run with --fix to auto-format: $0 --fix${NC}"
        exit 1
    fi
fi
