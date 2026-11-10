#!/bin/bash
# Automated Gecko SDK download and Bluetooth blob extraction
# This script downloads Gecko SDK and extracts Bluetooth components for efr32mg24-bluetooth-sys

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BT_SYS_DIR="$(dirname "$SCRIPT_DIR")"
BLOB_DIR="$BT_SYS_DIR/blobs/efr32mg24b220"
VENDOR_DIR="$BT_SYS_DIR/vendor/bluetooth"
TEMP_DIR="$BT_SYS_DIR/temp_gecko_sdk"

# Gecko SDK version
GSDK_VERSION="v4.4.5"
GSDK_REPO="https://github.com/SiliconLabs/gecko_sdk.git"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Gecko SDK Bluetooth Blob Setup ===${NC}"
echo ""
echo "This script will:"
echo "  1. Check/install Git LFS"
echo "  2. Clone Gecko SDK $GSDK_VERSION (~300 MB download)"
echo "  3. Pull LFS files (Bluetooth blobs)"
echo "  4. Extract Bluetooth components (controller + host + common)"
echo "  5. Extract BGAPI headers (~20 KB)"
echo "  6. Clean up temporary files"
echo ""
echo -e "${YELLOW}Note: Gecko SDK v4.4.5 uses split Bluetooth stack:${NC}"
echo "  - libbluetooth_controller (link layer)"
echo "  - libbluetooth_host (host stack)"
echo "  - libbgcommon (common components)"
echo ""

# Check if already installed
if [ -f "$BLOB_DIR/libbluetooth_host_efr32xg24_gcc_release.a" ]; then
    # Check if it's a real binary, not an LFS pointer
    if file "$BLOB_DIR/libbluetooth_host_efr32xg24_gcc_release.a" | grep -q "current ar archive"; then
        echo -e "${YELLOW}Bluetooth blobs already exist in:${NC}"
        echo "  $BLOB_DIR/"
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
echo -e "${GREEN}[4/6] Pulling LFS files (Bluetooth blobs)...${NC}"
echo "  This may take a few minutes (~10-20 MB for Bluetooth blobs)"
git lfs pull --include="protocol/bluetooth/lib/*.a,protocol/bluetooth/bgstack/ll/lib/*.a,protocol/bluetooth/bgcommon/lib/*.a"

echo -e "${GREEN}[5/6] Extracting Bluetooth components...${NC}"

# Extract Bluetooth components (split architecture in Gecko SDK v4.4.5+)
# Required components:
#   1. libbluetooth_controller_efr32xg24_gcc_release.a (link layer)
#   2. libbluetooth_host_efr32xg24_gcc_release.a (host stack)
#   3. libbgcommon_efr32xg24_gcc_release.a (common components)

ERRORS=0

# Extract controller component
echo "  [1/3] Extracting Bluetooth controller (link layer)..."
CONTROLLER_PATH="protocol/bluetooth/bgstack/ll/lib/libbluetooth_controller_efr32xg24_gcc_release.a"
if [ -f "$CONTROLLER_PATH" ]; then
    if file "$CONTROLLER_PATH" | grep -q "ASCII text"; then
        echo -e "    ${RED}✗${NC} Controller blob is still an LFS pointer"
        ERRORS=$((ERRORS + 1))
    else
        cp "$CONTROLLER_PATH" "$BLOB_DIR/"
        BLOB_SIZE=$(du -h "$BLOB_DIR/libbluetooth_controller_efr32xg24_gcc_release.a" | cut -f1)
        echo -e "    ${GREEN}✓${NC} libbluetooth_controller_efr32xg24_gcc_release.a ($BLOB_SIZE)"
    fi
else
    echo -e "    ${RED}✗${NC} Controller blob not found at $CONTROLLER_PATH"
    ERRORS=$((ERRORS + 1))
fi

# Extract host component
echo "  [2/3] Extracting Bluetooth host (host stack)..."
HOST_PATH="protocol/bluetooth/lib/libbluetooth_host_efr32xg24_gcc_release.a"
if [ -f "$HOST_PATH" ]; then
    if file "$HOST_PATH" | grep -q "ASCII text"; then
        echo -e "    ${RED}✗${NC} Host blob is still an LFS pointer"
        ERRORS=$((ERRORS + 1))
    else
        cp "$HOST_PATH" "$BLOB_DIR/"
        BLOB_SIZE=$(du -h "$BLOB_DIR/libbluetooth_host_efr32xg24_gcc_release.a" | cut -f1)
        echo -e "    ${GREEN}✓${NC} libbluetooth_host_efr32xg24_gcc_release.a ($BLOB_SIZE)"
    fi
else
    echo -e "    ${RED}✗${NC} Host blob not found at $HOST_PATH"
    ERRORS=$((ERRORS + 1))
fi

# Extract common components
echo "  [3/3] Extracting Bluetooth common components..."
COMMON_PATH="protocol/bluetooth/bgcommon/lib/libbgcommon_efr32xg24_gcc_release.a"
if [ -f "$COMMON_PATH" ]; then
    if file "$COMMON_PATH" | grep -q "ASCII text"; then
        echo -e "    ${RED}✗${NC} Common blob is still an LFS pointer"
        ERRORS=$((ERRORS + 1))
    else
        cp "$COMMON_PATH" "$BLOB_DIR/"
        BLOB_SIZE=$(du -h "$BLOB_DIR/libbgcommon_efr32xg24_gcc_release.a" | cut -f1)
        echo -e "    ${GREEN}✓${NC} libbgcommon_efr32xg24_gcc_release.a ($BLOB_SIZE)"
    fi
else
    echo -e "    ${RED}✗${NC} Common blob not found at $COMMON_PATH"
    ERRORS=$((ERRORS + 1))
fi

if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}ERROR: Failed to extract Bluetooth blobs ($ERRORS errors)${NC}"
    echo "Git LFS pull may have failed. Please check your Git LFS installation."
    exit 1
fi

# Extract Bluetooth BGAPI headers
echo "  Copying BGAPI headers..."

# Try to find the BGAPI include directory
BGAPI_INCLUDE=""
for candidate in \
    "protocol/bluetooth/inc" \
    "protocol/bluetooth/bgapi/inc" \
    "protocol/bluetooth/include"; do
    if [ -d "$candidate" ]; then
        BGAPI_INCLUDE="$candidate"
        break
    fi
done

if [ -n "$BGAPI_INCLUDE" ]; then
    cp -r "$BGAPI_INCLUDE/"*.h "$VENDOR_DIR/" 2>/dev/null || true
    HEADER_COUNT=$(find "$VENDOR_DIR" -name "*.h" | wc -l | tr -d ' ')
    echo -e "  ${GREEN}✓${NC} $HEADER_COUNT header files extracted"
else
    echo -e "  ${YELLOW}⚠${NC} BGAPI headers directory not found, trying alternate locations..."
    # Fallback: search for sl_bt_api.h anywhere in protocol/bluetooth
    find protocol/bluetooth -name "*.h" -exec cp {} "$VENDOR_DIR/" \; 2>/dev/null || true
    HEADER_COUNT=$(find "$VENDOR_DIR" -name "*.h" | wc -l | tr -d ' ')
    if [ $HEADER_COUNT -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} $HEADER_COUNT header files extracted (fallback search)"
    else
        echo -e "  ${RED}✗${NC} No headers found"
    fi
fi

# Verify key files
echo ""
echo "Verifying extracted files..."
ERRORS=0

# Check for BGAPI main header
if [ ! -f "$VENDOR_DIR/sl_bt_api.h" ]; then
    echo -e "  ${RED}✗${NC} sl_bt_api.h not found"
    ERRORS=$((ERRORS + 1))
else
    echo -e "  ${GREEN}✓${NC} sl_bt_api.h"
fi

# Check for blob
if [ ! -f "$BLOB_DIR/libbluetooth_efr32xg24_gcc_release.a" ]; then
    echo -e "  ${RED}✗${NC} Bluetooth blob not found"
    ERRORS=$((ERRORS + 1))
elif file "$BLOB_DIR/libbluetooth_efr32xg24_gcc_release.a" | grep -q "ASCII text"; then
    echo -e "  ${RED}✗${NC} Bluetooth blob is an LFS pointer (not pulled)"
    ERRORS=$((ERRORS + 1))
else
    echo -e "  ${GREEN}✓${NC} libbluetooth_efr32xg24_gcc_release.a (binary)"
fi

# Clean up
echo ""
echo -e "${GREEN}[6/6] Cleaning up...${NC}"
cd "$BT_SYS_DIR"
rm -rf "$TEMP_DIR"
echo -e "  ${GREEN}✓${NC} Temporary files removed"

# Summary
echo ""
echo -e "${GREEN}=== Setup Complete ===${NC}"
echo ""

if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}All components extracted successfully!${NC}"
    echo ""
    echo "Bluetooth blob:"
    echo "  $BLOB_DIR/libbluetooth_efr32xg24_gcc_release.a ($BLOB_SIZE)"
    echo ""
    echo "BGAPI headers:"
    echo "  $VENDOR_DIR/ ($HEADER_COUNT files)"
    echo ""
    echo "Next steps:"
    echo "  1. cd $BT_SYS_DIR"
    echo "  2. cargo build --target thumbv8m.main-none-eabihf"
    echo ""
    echo "The build will automatically:"
    echo "  - Link the Bluetooth blob"
    echo "  - Generate Rust bindings via bindgen"
    echo "  - Create target/thumbv8m.main-none-eabihf/debug/libefr32mg24_bluetooth_sys.rlib"
    echo ""
else
    echo -e "${RED}Setup completed with $ERRORS error(s)${NC}"
    echo "Please check the output above for details."
    echo ""
    if [ ! -f "$VENDOR_DIR/sl_bt_api.h" ]; then
        echo "Possible solutions for missing headers:"
        echo "  1. Check the Gecko SDK structure for BGAPI headers"
        echo "  2. Manually copy headers from protocol/bluetooth/ to vendor/bluetooth/"
        echo "  3. Update this script with correct paths for SDK $GSDK_VERSION"
    fi
    exit 1
fi
