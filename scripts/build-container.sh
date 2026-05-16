#!/bin/bash
# Container-based build script for EFR32MG24 firmware
#
# Usage:
#   ./build-container.sh                           # Build all
#   ./build-container.sh --example 00_minimal_blink # Build specific example
#   ./build-container.sh --clean                   # Clean build

set -e

# Configuration
CONTAINER_NAME="efr32mg24-builder"
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if podman is installed
if ! command -v podman &> /dev/null; then
    log_error "Podman not found. Please install podman:"
    echo "  macOS: brew install podman"
    echo "  Linux: sudo apt install podman"
    exit 1
fi

# Build container image if it doesn't exist
if ! podman image exists "$CONTAINER_NAME" 2>/dev/null; then
    log_info "Building container image: $CONTAINER_NAME"
    podman build -t "$CONTAINER_NAME" -f "$PROJECT_ROOT/Containerfile" "$PROJECT_ROOT"
else
    log_info "Using existing container image: $CONTAINER_NAME"
fi

# Parse arguments
CARGO_ARGS=()
CLEAN=false
EXAMPLE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --clean)
            CLEAN=true
            shift
            ;;
        --example)
            EXAMPLE="$2"
            CARGO_ARGS+=("--example" "$2")
            shift 2
            ;;
        --release)
            CARGO_ARGS+=("--release")
            shift
            ;;
        *)
            CARGO_ARGS+=("$1")
            shift
            ;;
    esac
done

# Clean if requested
if [ "$CLEAN" = true ]; then
    log_info "Cleaning build artifacts"
    podman run --rm \
        -v "$PROJECT_ROOT:/build:Z" \
        -w /build \
        "$CONTAINER_NAME" \
        cargo clean
    log_info "Clean complete"
    exit 0
fi

# Add --release if not specified
if [[ ! " ${CARGO_ARGS[@]} " =~ " --release " ]]; then
    CARGO_ARGS+=("--release")
fi

# Determine which package to build
PACKAGE=""
if [ -n "$EXAMPLE" ]; then
    # Example is in xiao-mg24-bsp
    PACKAGE="xiao-mg24-bsp"
    WORKDIR="/build/xiao-mg24-bsp"
else
    # Build entire workspace
    WORKDIR="/build"
fi

# Build command
BUILD_CMD="cargo build ${CARGO_ARGS[*]}"

log_info "Building firmware in container"
log_info "Command: $BUILD_CMD"
log_info "Working directory: $WORKDIR"

# Run build in container
podman run --rm \
    -v "$PROJECT_ROOT:/build:Z" \
    -w "$WORKDIR" \
    "$CONTAINER_NAME" \
    bash -c "$BUILD_CMD"

# Check if build succeeded
if [ $? -eq 0 ]; then
    log_info "Build successful!"

    # Show output binary location
    if [ -n "$EXAMPLE" ]; then
        BINARY_PATH="target/thumbv8m.main-none-eabihf/release/examples/$EXAMPLE"
        if [ -f "$PROJECT_ROOT/$BINARY_PATH" ]; then
            SIZE=$(du -h "$PROJECT_ROOT/$BINARY_PATH" | cut -f1)
            log_info "Binary: $BINARY_PATH ($SIZE)"
            log_info ""
            log_info "Flash with:"
            echo "  probe-rs run --chip EFR32MG24B220F1536 $BINARY_PATH"
        fi
    fi
else
    log_error "Build failed!"
    exit 1
fi
