#!/bin/bash
# pull-pdf.sh
# Pull compiled PDF from remote build host
#
# Usage: ./pull-pdf.sh [output-name]
#
# Examples:
#   ./pull-pdf.sh                    # Copy to ../main.pdf
#   ./pull-pdf.sh book-v1.0.pdf      # Copy to ../book-v1.0.pdf
#
# Author: Marcelo Correa
# Date: 2026-01-09
# Version: 1.0

# Configuration
REMOTE="mvcorrea@lnx"
REMOTE_DIR="~/podman/latex-build/embedded-support-book"
OUTPUT_NAME="${1:-main.pdf}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Pulling PDF from Remote ===${NC}"
echo ""

# Check if PDF exists on remote
ssh "$REMOTE" "test -f $REMOTE_DIR/main.pdf"

if [ $? -ne 0 ]; then
    echo -e "${RED}✗ PDF not found on remote${NC}"
    echo -e "${YELLOW}Run build-book-remote.sh first to compile${NC}"
    exit 1
fi

# Copy PDF
rsync -avz --progress "$REMOTE:$REMOTE_DIR/main.pdf" "../$OUTPUT_NAME"

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ PDF copied successfully${NC}"
    echo ""
    echo -e "Location: ${BLUE}../$OUTPUT_NAME${NC}"
    echo -e "Size: $(ls -lh ../$OUTPUT_NAME | awk '{print $5}')"
    echo ""
    echo -e "${YELLOW}To view:${NC}"
    echo -e "  open ../$OUTPUT_NAME        # macOS"
    echo -e "  xdg-open ../$OUTPUT_NAME    # Linux"
else
    echo -e "${RED}✗ Failed to copy PDF${NC}"
    exit 1
fi
