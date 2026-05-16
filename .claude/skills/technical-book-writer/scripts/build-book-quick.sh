#!/bin/bash
# build-book-quick.sh
# Quick single-pass build for error checking
#
# Usage: ./build-book-quick.sh
#
# Author: Marcelo Correa
# Date: 2026-01-09
# Version: 1.0

set -e

# Configuration
REMOTE="mvcorrea@lnx"
REMOTE_DIR="~/podman/latex-build/embedded-support-book"
LOCAL_DIR="../latex"
CONTAINER="embedded-rust-book"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Quick Build (Single Pass) ===${NC}"
echo ""

# Sync files
echo -e "${YELLOW}Syncing files...${NC}"
rsync -avz --progress "$LOCAL_DIR/" "$REMOTE:$REMOTE_DIR/"
echo ""

# Single compilation pass
echo -e "${YELLOW}Compiling (1 pass only)...${NC}"
ssh "$REMOTE" "podman exec $CONTAINER bash -c '\
  cd /workspace && \
  xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee compile.log'
  "

COMPILE_RESULT=$?

if [ $COMPILE_RESULT -ne 0 ]; then
    echo -e "${RED}✗ Compilation failed${NC}"
    rsync -avz "$REMOTE:$REMOTE_DIR/compile.log" ./
    echo -e "${YELLOW}Check compile.log for errors${NC}"
    exit 1
fi

# Check for errors in log
echo ""
echo -e "${YELLOW}Error summary:${NC}"
ssh "$REMOTE" "podman exec $CONTAINER bash -c '\
  cd /workspace && \
  grep -c \"^! LaTeX Error\" compile.log 2>/dev/null || echo 0' | xargs echo 'LaTeX errors:'"

ssh "$REMOTE" "podman exec $CONTAINER bash -c '\
  cd /workspace && \
  grep -c \"^! Package\" compile.log 2>/dev/null || echo 0' | xargs echo 'Package errors:'"

ssh "$REMOTE" "podman exec $CONTAINER bash -c '\
  cd /workspace && \
  grep -c \"^! Undefined\" compile.log 2>/dev/null || echo 0' | xargs echo 'Undefined errors:'"

echo ""
echo -e "${GREEN}✓ Quick build complete${NC}"
echo -e "${YELLOW}Note: Single pass may show 'undefined reference' warnings (normal)${NC}"
echo -e "${YELLOW}Run build-book-remote.sh for complete 3-pass build${NC}"
echo ""
