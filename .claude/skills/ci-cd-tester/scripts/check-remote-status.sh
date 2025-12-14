#!/bin/bash
# check-remote-status.sh - Check GitHub Actions workflow status
# Usage: bash check-remote-status.sh [workflow-name]
#
# Requires: gh CLI (brew install gh)

set -e

WORKFLOW="${1:-}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ gh CLI not found${NC}"
    echo -e "${YELLOW}Install with: brew install gh${NC}"
    echo -e "${YELLOW}Then authenticate: gh auth login${NC}"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ Not authenticated with GitHub${NC}"
    echo -e "${YELLOW}Run: gh auth login${NC}"
    exit 1
fi

echo -e "${BLUE}📊 Checking GitHub Actions workflow status...${NC}"
echo ""

if [[ -z "$WORKFLOW" ]]; then
    # Show all workflows
    gh run list --limit 10
else
    # Show specific workflow
    gh run list --workflow "$WORKFLOW" --limit 10
fi

echo ""
echo -e "${YELLOW}💡 To view details of a specific run:${NC}"
echo "   gh run view <run-id>"
echo ""
echo -e "${YELLOW}💡 To download logs:${NC}"
echo "   bash .claude/skills/ci-cd-tester/scripts/download-logs.sh <run-id>"
