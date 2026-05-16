# Containerfile for EFR32MG24 Embedded Rust Development
#
# Provides complete build environment with:
# - Rust toolchain with thumbv8m.main-none-eabihf target
# - ARM GCC toolchain (for bluetooth-sys CMSIS stubs)
# - Build tools (bindgen, objcopy, etc.)
#
# Usage:
#   podman build -t efr32mg24-builder -f Containerfile .
#   podman run --rm -v $(pwd):/build efr32mg24-builder cargo build --release

FROM docker.io/rust:1.75-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # ARM GCC toolchain
    gcc-arm-none-eabi \
    libnewlib-arm-none-eabi \
    # Build tools
    build-essential \
    pkg-config \
    libclang-dev \
    clang \
    # bindgen dependencies
    llvm-dev \
    libclang-dev \
    # Utilities
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust target for Cortex-M33
RUN rustup target add thumbv8m.main-none-eabihf

# Install cargo tools
RUN cargo install cargo-binutils --locked && \
    rustup component add llvm-tools-preview

# Set working directory
WORKDIR /build

# Default command
CMD ["cargo", "build", "--release"]
