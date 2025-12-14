#!/bin/bash
# pre-push-check.sh - Run all CI checks locally before pushing
#
# This script runs the same checks that GitHub Actions CI will run,
# allowing you to catch issues before pushing to the repository.
#
# Usage: ./scripts/pre-push-check.sh

set -e  # Exit on first error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Running pre-push checks...${NC}"
echo ""

# Check 1: Formatting
echo -e "${YELLOW}1️⃣  Checking formatting...${NC}"
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✅ Formatting check passed${NC}"
else
    echo -e "${RED}❌ Formatting check failed${NC}"
    echo -e "${YELLOW}Run 'cargo fmt --all' to fix${NC}"
    exit 1
fi
echo ""

# Check 2: Clippy on PAC (allow warnings for generated code)
echo -e "${YELLOW}2️⃣  Running Clippy on PAC...${NC}"
if cargo clippy -p efr32mg24-pac --target thumbv8m.main-none-eabihf 2>&1 | tail -5; then
    echo -e "${GREEN}✅ PAC Clippy completed${NC}"
else
    echo -e "${RED}❌ PAC Clippy failed${NC}"
    exit 1
fi
echo ""

# Check 3: Clippy on HAL (strict - no warnings allowed)
echo -e "${YELLOW}3️⃣  Running Clippy on HAL (strict)...${NC}"
if cargo clippy -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf -- -D warnings; then
    echo -e "${GREEN}✅ HAL Clippy passed (no warnings)${NC}"
else
    echo -e "${RED}❌ HAL Clippy failed - fix all warnings${NC}"
    exit 1
fi
echo ""

# Check 4: Build PAC
echo -e "${YELLOW}4️⃣  Building PAC (this may take ~2 minutes)...${NC}"
if cargo build -p efr32mg24-pac --target thumbv8m.main-none-eabihf --release --quiet; then
    echo -e "${GREEN}✅ PAC build succeeded${NC}"
else
    echo -e "${RED}❌ PAC build failed${NC}"
    exit 1
fi
echo ""

# Check 5: Build HAL
echo -e "${YELLOW}5️⃣  Building HAL library...${NC}"
if cargo build -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf --release --quiet; then
    echo -e "${GREEN}✅ HAL build succeeded${NC}"
else
    echo -e "${RED}❌ HAL build failed${NC}"
    exit 1
fi
echo ""

# Check 6: Build examples
echo -e "${YELLOW}6️⃣  Building all examples...${NC}"
if cargo build --manifest-path efr32mg24-hal/Cargo.toml --examples --features rt --target thumbv8m.main-none-eabihf --release --quiet; then
    echo -e "${GREEN}✅ All examples built successfully${NC}"
else
    echo -e "${RED}❌ Example build failed${NC}"
    exit 1
fi
echo ""

# Check 7: Documentation
echo -e "${YELLOW}7️⃣  Building documentation...${NC}"
DOC_OUTPUT=$(cargo doc --no-deps -p efr32mg24-hal --features rt --target thumbv8m.main-none-eabihf --quiet 2>&1)
if echo "$DOC_OUTPUT" | grep -qi "warning"; then
    echo -e "${RED}❌ Documentation has warnings:${NC}"
    echo "$DOC_OUTPUT" | grep -i "warning"
    exit 1
else
    echo -e "${GREEN}✅ Documentation built without warnings${NC}"
fi
echo ""

# Optional: Report binary sizes
echo -e "${YELLOW}📊 Binary size report:${NC}"
echo ""
for example in target/thumbv8m.main-none-eabihf/release/examples/*; do
    if [ -f "$example" ] && [ -x "$example" ]; then
        SIZE=$(stat -f%z "$example" 2>/dev/null || stat -c%s "$example" 2>/dev/null || echo "N/A")
        if [ "$SIZE" != "N/A" ]; then
            SIZE_KB=$((SIZE / 1024))
            if [ "$SIZE_KB" -gt 1024 ]; then
                echo -e "  ${RED}$(basename $example): ${SIZE_KB} KB (⚠️  Large)${NC}"
            else
                echo -e "  $(basename $example): ${SIZE_KB} KB"
            fi
        fi
    fi
done
echo ""

# All checks passed
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All checks passed! Safe to push.${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}💡 Next steps:${NC}"
echo "   git add ."
echo "   git commit -m \"your message\""
echo "   git push"
echo ""
