#!/bin/bash
# Automated Gecko SDK download and RAIL blob extraction
# This script downloads Gecko SDK and extracts RAIL components for efr32mg24-rail-sys

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RAIL_SYS_DIR="$(dirname "$SCRIPT_DIR")"
BLOB_DIR="$RAIL_SYS_DIR/blobs/efr32mg24b220"
VENDOR_DIR="$RAIL_SYS_DIR/vendor/rail"
TEMP_DIR="$RAIL_SYS_DIR/temp_gecko_sdk"

# Gecko SDK version
GSDK_VERSION="v4.4.5"
GSDK_REPO="https://github.com/SiliconLabs/gecko_sdk.git"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Gecko SDK RAIL Blob Setup ===${NC}"
echo ""
echo "This script will:"
echo "  1. Check/install Git LFS"
echo "  2. Clone Gecko SDK $GSDK_VERSION (~300 MB download)"
echo "  3. Pull LFS files (RAIL blobs)"
echo "  4. Extract RAIL blob (~1.5 MB)"
echo "  5. Extract RAIL headers (~50 KB)"
echo "  6. Clean up temporary files"
echo ""

# Check if already installed
if [ -f "$BLOB_DIR/librail_efr32xg24_gcc_release.a" ]; then
    # Check if it's a real binary, not an LFS pointer
    if file "$BLOB_DIR/librail_efr32xg24_gcc_release.a" | grep -q "current ar archive"; then
        echo -e "${YELLOW}RAIL blob already exists at:${NC}"
        echo "  $BLOB_DIR/librail_efr32xg24_gcc_release.a"
        echo ""
        read -p "Overwrite? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborting."
            exit 0
        fi
    fi
fi

# Check for Git LFS
echo -e "${GREEN}[1/6] Checking Git LFS...${NC}"
if ! command -v git-lfs &> /dev/null; then
    echo -e "${YELLOW}Git LFS not found. Installing...${NC}"
    if command -v brew &> /dev/null; then
        brew install git-lfs
    elif command -v apt-get &> /dev/null; then
        sudo apt-get install -y git-lfs
    else
        echo -e "${RED}ERROR: Please install Git LFS manually${NC}"
        echo "  macOS (Homebrew): brew install git-lfs"
        echo "  macOS (MacPorts): sudo port install git-lfs"
        echo "  Linux: sudo apt-get install git-lfs"
        exit 1
    fi
fi

# Initialize Git LFS
git lfs install
echo -e "  ${GREEN}✓${NC} Git LFS ready"

# Create directories
echo -e "${GREEN}[2/6] Creating directories...${NC}"
mkdir -p "$BLOB_DIR"
mkdir -p "$VENDOR_DIR"

# Clone Gecko SDK (shallow clone for speed)
echo -e "${GREEN}[3/6] Downloading Gecko SDK $GSDK_VERSION...${NC}"
echo "  This may take 5-10 minutes (~300 MB download)"

if [ -d "$TEMP_DIR" ]; then
    echo "  Removing existing temp directory..."
    rm -rf "$TEMP_DIR"
fi

# Clone with LFS support
git clone --depth 1 --branch "$GSDK_VERSION" "$GSDK_REPO" "$TEMP_DIR"
cd "$TEMP_DIR"

# Pull LFS files
echo -e "${GREEN}[4/6] Pulling LFS files (RAIL blobs)...${NC}"
echo "  This may take a few minutes (~10-20 MB for RAIL blobs)"
git lfs pull --include="platform/radio/rail_lib/autogen/librail_release/*.a"

echo -e "${GREEN}[5/6] Extracting RAIL components...${NC}"

# Extract RAIL blob
BLOB_PATH="platform/radio/rail_lib/autogen/librail_release/librail_efr32xg24_gcc_release.a"
if [ ! -f "$BLOB_PATH" ]; then
    echo -e "${RED}ERROR: RAIL blob not found at $BLOB_PATH${NC}"
    echo "Available files:"
    find platform/radio/rail_lib/autogen/librail_release -name "*.a" 2>/dev/null || echo "  (none found)"
    exit 1
fi

# Verify it's a real binary, not LFS pointer
if file "$BLOB_PATH" | grep -q "ASCII text"; then
    echo -e "${RED}ERROR: RAIL blob is still an LFS pointer${NC}"
    echo "Git LFS pull may have failed. Please check your Git LFS installation."
    exit 1
fi

echo "  Copying RAIL blob..."
cp "$BLOB_PATH" "$BLOB_DIR/"
BLOB_SIZE=$(du -h "$BLOB_DIR/librail_efr32xg24_gcc_release.a" | cut -f1)
echo -e "  ${GREEN}✓${NC} librail_efr32xg24_gcc_release.a ($BLOB_SIZE)"

# Extract RAIL headers
echo "  Copying RAIL headers..."
HEADER_DIR="platform/radio/rail_lib"

# Copy common headers
if [ -d "$HEADER_DIR/common" ]; then
    cp -r "$HEADER_DIR/common/"* "$VENDOR_DIR/" 2>/dev/null || true
fi

# Copy chip-specific headers
if [ -d "$HEADER_DIR/chip/efr32/efr32xg2x" ]; then
    mkdir -p "$VENDOR_DIR/chip"
    cp -r "$HEADER_DIR/chip/efr32/efr32xg2x" "$VENDOR_DIR/chip/" 2>/dev/null || true
fi

# Copy protocol headers (for BLE, Zigbee later)
if [ -d "$HEADER_DIR/protocol" ]; then
    mkdir -p "$VENDOR_DIR/protocol"
    cp -r "$HEADER_DIR/protocol/"* "$VENDOR_DIR/protocol/" 2>/dev/null || true
fi

# Count extracted headers
HEADER_COUNT=$(find "$VENDOR_DIR" -name "*.h" | wc -l | tr -d ' ')
echo -e "  ${GREEN}✓${NC} $HEADER_COUNT header files extracted"

# Verify key files
echo ""
echo "Verifying extracted files..."
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
elif file "$BLOB_DIR/librail_efr32xg24_gcc_release.a" | grep -q "ASCII text"; then
    echo -e "  ${RED}✗${NC} RAIL blob is an LFS pointer (not pulled)"
    ERRORS=$((ERRORS + 1))
else
    echo -e "  ${GREEN}✓${NC} librail_efr32xg24_gcc_release.a (binary)"
fi

# Clean up
echo ""
echo -e "${GREEN}[6/6] Cleaning up...${NC}"
cd "$RAIL_SYS_DIR"
rm -rf "$TEMP_DIR"
echo -e "  ${GREEN}✓${NC} Temporary files removed"

# Summary
echo ""
echo -e "${GREEN}=== Setup Complete ===${NC}"
echo ""

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}All components extracted successfully!${NC}"
    echo ""
    echo "RAIL blob:"
    echo "  $BLOB_DIR/librail_efr32xg24_gcc_release.a ($BLOB_SIZE)"
    echo ""
    echo "RAIL headers:"
    echo "  $VENDOR_DIR/ ($HEADER_COUNT files)"
    echo ""
    echo "Next steps:"
    echo "  1. cd $RAIL_SYS_DIR"
    echo "  2. cargo build --target thumbv8m.main-none-eabihf"
    echo ""
    echo "The build will automatically:"
    echo "  - Link the RAIL blob"
    echo "  - Generate Rust bindings via bindgen"
    echo "  - Create target/thumbv8m.main-none-eabihf/debug/libefr32mg24_rail_sys.rlib"
    echo ""
else
    echo -e "${RED}Setup completed with $ERRORS error(s)${NC}"
    echo "Please check the output above for details."
    exit 1
fi
