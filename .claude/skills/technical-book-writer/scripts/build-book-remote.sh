#!/bin/bash
# build-book-remote.sh
# Complete 3-pass remote LaTeX build with error handling
#
# Usage: ./build-book-remote.sh
#
# Author: Marcelo Correa
# Date: 2026-01-09
# Version: 2.0

set -e  # Exit on error

# Configuration
REMOTE="mvcorrea@lnx"
REMOTE_DIR="~/podman/latex-build/embedded-support-book"
LOCAL_DIR="../../../../book/latex"
CONTAINER="embedded-rust-book"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   EFR32MG24 Book Remote Build${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Sync files to remote
echo -e "${YELLOW}=== STEP 1: Syncing files to remote host ===${NC}"
rsync -avz --progress "$LOCAL_DIR/" "$REMOTE:$REMOTE_DIR/"

if [ $? -ne 0 ]; then
    echo -e "${RED}✗ FAILED: Could not sync files to remote${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Files synced successfully${NC}"
echo ""

# Step 2: Check container status
echo -e "${YELLOW}=== STEP 2: Checking container status ===${NC}"
ssh "$REMOTE" "podman ps | grep $CONTAINER" > /dev/null

if [ $? -ne 0 ]; then
    echo -e "${YELLOW}Container not running. Starting container...${NC}"
    ssh "$REMOTE" "cd ~/podman/latex-build && podman-compose -f podman-compose-embedded.yml up -d"
    sleep 5
fi

echo -e "${GREEN}✓ Container is running${NC}"
echo ""

# Step 3: Compile on remote (3 passes)
echo -e "${YELLOW}=== STEP 3: Compiling on remote (3 passes) ===${NC}"
ssh "$REMOTE" "cd ~/podman/latex-build && \
  podman exec $CONTAINER bash -c '\
    cd /workspace && \
    for i in 1 2 3; do \
      echo \"=== Pass \$i ===\" && \
      xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee -a compile.log; \
    done'
  "

COMPILE_RESULT=$?

if [ $COMPILE_RESULT -ne 0 ]; then
    echo -e "${RED}✗ FAILED: Compilation returned error${NC}"
    echo ""
    echo -e "${YELLOW}Pulling logs for analysis...${NC}"
    rsync -avz "$REMOTE:$REMOTE_DIR/compile.log" ../../../../
    rsync -avz "$REMOTE:$REMOTE_DIR/main.log" ../../../../
    echo -e "${RED}Check logs: compile.log and main.log in project root${NC}"
    exit 1
fi

echo ""

# Step 4: Check if PDF exists
echo -e "${YELLOW}=== STEP 4: Verifying PDF generation ===${NC}"
ssh "$REMOTE" "cd ~/podman/latex-build && \
  podman exec $CONTAINER bash -c '\
    cd /workspace && \
    if [ -f main.pdf ]; then \
      echo \"✓ SUCCESS: main.pdf created\"; \
      ls -lh main.pdf; \
      exit 0; \
    else \
      echo \"✗ FAILED: PDF not generated\"; \
      exit 1; \
    fi'
  "

PDF_CHECK=$?

if [ $PDF_CHECK -ne 0 ]; then
    echo -e "${RED}✗ FAILED: PDF was not generated${NC}"
    echo ""
    echo -e "${YELLOW}Pulling logs for analysis...${NC}"
    rsync -avz "$REMOTE:$REMOTE_DIR/compile.log" ../../../../
    rsync -avz "$REMOTE:$REMOTE_DIR/main.log" ../../../../

    echo ""
    echo -e "${YELLOW}Error summary:${NC}"
    grep -c "^! LaTeX Error" ../../../../compile.log 2>/dev/null | xargs echo "LaTeX errors:" || echo "LaTeX errors: 0"
    grep -c "^! Package" ../../../../compile.log 2>/dev/null | xargs echo "Package errors:" || echo "Package errors: 0"
    grep -c "^! Undefined" ../../../../compile.log 2>/dev/null | xargs echo "Undefined errors:" || echo "Undefined errors: 0"

    exit 1
fi

echo -e "${GREEN}✓ PDF generated successfully${NC}"
echo ""

# Step 5: Copy PDF locally
echo -e "${YELLOW}=== STEP 5: Copying PDF to local machine ===${NC}"
rsync -avz "$REMOTE:$REMOTE_DIR/main.pdf" ../../../../book/

if [ $? -ne 0 ]; then
    echo -e "${RED}✗ FAILED: Could not copy PDF locally${NC}"
    exit 1
fi

echo -e "${GREEN}✓ PDF copied successfully${NC}"
echo ""

# Success summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}   BUILD COMPLETE!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "PDF location: ${BLUE}book/main.pdf${NC} (in project root)"
echo -e "PDF size: $(ls -lh ../../../../book/main.pdf | awk '{print $5}')"
echo ""
echo -e "${YELLOW}To view:${NC}"
echo -e "  open book/main.pdf        # macOS (from project root)"
echo -e "  xdg-open book/main.pdf    # Linux (from project root)"
echo ""
