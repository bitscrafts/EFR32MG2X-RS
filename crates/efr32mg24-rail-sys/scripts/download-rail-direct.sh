#!/bin/bash
# Direct download of RAIL blob without Git LFS
# Alternative method when Git LFS is not available or git is still building

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIL_SYS_DIR="$(dirname "$SCRIPT_DIR")"
BLOB_DIR="$RAIL_SYS_DIR/blobs/efr32mg24b220"
VENDOR_DIR="$RAIL_SYS_DIR/vendor/rail"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Direct RAIL Blob Download ===${NC}"
echo ""
echo "This script downloads RAIL components directly from GitHub"
echo "without requiring Git LFS (useful while git is building)."
echo ""

# Create directories
mkdir -p "$BLOB_DIR"
mkdir -p "$VENDOR_DIR"

# GitHub raw content URLs for v4.4.5
GSDK_VERSION="v4.4.5"
GSDK_RAW="https://github.com/SiliconLabs/gecko_sdk/raw/$GSDK_VERSION"

# LFS blob URL (GitHub's LFS CDN)
LFS_OID="072f0ff0eb708091e49a3fd536c45f4f0568ed6f163c2889f5a8536b234c66a2"
BLOB_URL="https://github.com/SiliconLabs/gecko_sdk/raw/$GSDK_VERSION/platform/radio/rail_lib/autogen/librail_release/librail_efr32xg24_gcc_release.a"

echo -e "${GREEN}[1/3] Downloading RAIL blob (~1.5 MB)...${NC}"
echo "  This may take a minute depending on your connection"

# Try direct download with curl
if command -v curl &> /dev/null; then
    curl -L "$BLOB_URL" -o "$BLOB_DIR/librail_efr32xg24_gcc_release.a"
elif command -v wget &> /dev/null; then
    wget "$BLOB_URL" -O "$BLOB_DIR/librail_efr32xg24_gcc_release.a"
else
    echo -e "${RED}ERROR: Neither curl nor wget found${NC}"
    exit 1
fi

# Verify it's a binary, not HTML error page
if file "$BLOB_DIR/librail_efr32xg24_gcc_release.a" | grep -q "HTML"; then
    echo -e "${RED}ERROR: Download failed (got HTML instead of binary)${NC}"
    echo "GitHub may have rate-limited or the file is LFS-only."
    echo ""
    echo "Alternative: Wait for git to finish building, then run setup-rail.sh"
    rm "$BLOB_DIR/librail_efr32xg24_gcc_release.a"
    exit 1
fi

BLOB_SIZE=$(du -h "$BLOB_DIR/librail_efr32xg24_gcc_release.a" | cut -f1)
echo -e "  ${GREEN}✓${NC} librail_efr32xg24_gcc_release.a ($BLOB_SIZE)"

echo -e "${GREEN}[2/3] Downloading RAIL headers...${NC}"

# Key headers to download
HEADERS=(
    "platform/radio/rail_lib/common/rail.h"
    "platform/radio/rail_lib/common/rail_types.h"
    "platform/radio/rail_lib/common/rail_features.h"
    "platform/radio/rail_lib/common/rail_assert_error_codes.h"
    "platform/radio/rail_lib/chip/efr32/efr32xg2x/rail_chip_specific.h"
)

for header in "${HEADERS[@]}"; do
    filename=$(basename "$header")
    target_dir="$VENDOR_DIR"

    # Handle chip-specific directory
    if [[ "$header" == *"chip/efr32"* ]]; then
        mkdir -p "$VENDOR_DIR/chip/efr32xg2x"
        target_dir="$VENDOR_DIR/chip/efr32xg2x"
    fi

    if command -v curl &> /dev/null; then
        curl -s -L "$GSDK_RAW/$header" -o "$target_dir/$filename"
    else
        wget -q "$GSDK_RAW/$header" -O "$target_dir/$filename"
    fi

    echo "  Downloaded: $filename"
done

HEADER_COUNT=$(find "$VENDOR_DIR" -name "*.h" | wc -l | tr -d ' ')
echo -e "  ${GREEN}✓${NC} $HEADER_COUNT header files downloaded"

echo -e "${GREEN}[3/3] Verifying...${NC}"
ERRORS=0

if [ ! -f "$VENDOR_DIR/rail.h" ]; then
    echo -e "  ${RED}✗${NC} rail.h not found"
    ERRORS=$((ERRORS + 1))
else
    echo -e "  ${GREEN}✓${NC} rail.h"
fi

if [ ! -f "$VENDOR_DIR/rail_types.h" ]; then
    echo -e "  ${RED}✗${NC} rail_types.h not found"
    ERRORS=$((ERRORS + 1))
else
    echo -e "  ${GREEN}✓${NC} rail_types.h"
fi

if [ ! -f "$BLOB_DIR/librail_efr32xg24_gcc_release.a" ]; then
    echo -e "  ${RED}✗${NC} RAIL blob not found"
    ERRORS=$((ERRORS + 1))
else
    echo -e "  ${GREEN}✓${NC} librail_efr32xg24_gcc_release.a (binary)"
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}=== Download Complete ===${NC}"
    echo ""
    echo "RAIL blob: $BLOB_DIR/librail_efr32xg24_gcc_release.a ($BLOB_SIZE)"
    echo "RAIL headers: $VENDOR_DIR/ ($HEADER_COUNT files)"
    echo ""
    echo "Next step:"
    echo "  cargo build --target thumbv8m.main-none-eabihf"
    echo ""
else
    echo -e "${RED}Download completed with $ERRORS error(s)${NC}"
    exit 1
fi
