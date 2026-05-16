#!/bin/bash
# sync-to-remote.sh
# Sync local LaTeX files to remote build host
#
# Usage: ./sync-to-remote.sh
#
# Author: Marcelo Correa
# Date: 2026-01-09
# Version: 1.0

# Configuration
REMOTE="mvcorrea@lnx"
REMOTE_DIR="~/podman/latex-build/embedded-support-book"
LOCAL_DIR="../latex"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Syncing LaTeX Files to Remote ===${NC}"
echo ""

rsync -avz --progress \
  --exclude='*.pdf' \
  --exclude='*.aux' \
  --exclude='*.log' \
  --exclude='*.toc' \
  --exclude='*.out' \
  "$LOCAL_DIR/" "$REMOTE:$REMOTE_DIR/"

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ Sync complete${NC}"
    echo ""
    echo -e "${YELLOW}To compile on remote:${NC}"
    echo -e "  ./build-book-quick.sh    # Quick single-pass"
    echo -e "  ./build-book-remote.sh   # Full 3-pass build"
else
    echo -e "${RED}✗ Sync failed${NC}"
    exit 1
fi
