#!/bin/bash
# view-status.sh - Display current project status from BACKLOG.md
# Usage: bash view-status.sh

set -euo pipefail

BACKLOG_FILE="docs/BACKLOG.md"

if [[ ! -f "$BACKLOG_FILE" ]]; then
    echo "Error: $BACKLOG_FILE not found"
    exit 1
fi

echo "=== Project Status ==="
echo ""

# Extract current sprint from header
SPRINT=$(grep "Current Sprint" "$BACKLOG_FILE" | head -n 1 | sed 's/\*\*Current Sprint\*\*: //')
LAST_UPDATED=$(grep "Last Updated" "$BACKLOG_FILE" | head -n 1 | sed 's/\*\*Last Updated\*\*: //')

echo "Current Sprint: $SPRINT"
echo "Last Updated: $LAST_UPDATED"
echo ""

# Count tasks in each column
IN_PROGRESS=$(grep -A 50 "## In Progress" "$BACKLOG_FILE" | grep "^- \[" | wc -l | tr -d ' ')
READY_HIGH=$(grep -A 20 "### High Priority" "$BACKLOG_FILE" | grep "^- \[" | wc -l | tr -d ' ')
READY_MEDIUM=$(grep -A 20 "### Medium Priority" "$BACKLOG_FILE" | grep "^- \[" | wc -l | tr -d ' ')
BLOCKED=$(grep -A 50 "## Blocked" "$BACKLOG_FILE" | grep "^- \[" | wc -l | tr -d ' ')

echo "📋 In Progress ($IN_PROGRESS):"
if [[ "$IN_PROGRESS" -eq 0 ]]; then
    echo "  None - ready to start new tasks"
else
    grep -A 50 "## In Progress" "$BACKLOG_FILE" | grep "^- \[" | sed 's/^/  /'
fi
echo ""

echo "✅ Ready (Prioritized):"
echo "  High Priority: $READY_HIGH tasks"
echo "  Medium Priority: $READY_MEDIUM tasks"
if [[ "$READY_HIGH" -gt 0 ]]; then
    echo "  Top 3 high priority:"
    grep -A 20 "### High Priority" "$BACKLOG_FILE" | grep "^- \[" | head -n 3 | sed 's/^/    /'
fi
echo ""

echo "🚫 Blocked ($BLOCKED):"
if [[ "$BLOCKED" -eq 0 ]]; then
    echo "  None"
else
    grep -A 50 "## Blocked" "$BACKLOG_FILE" | grep "^- \[" | sed 's/^/  /'
fi
echo ""

# Show recent completions (last week)
echo "✨ Recent Completions (last 7 days):"
grep -A 50 "## Done" "$BACKLOG_FILE" | grep -A 10 "### Week" | head -n 15 | grep "^- \[x\]" | head -n 5 | sed 's/^/  /'
echo ""

# Extract metrics
echo "📊 Project Metrics:"
grep -A 20 "## Metrics" "$BACKLOG_FILE" | grep "^-" | sed 's/^/  /'
echo ""

echo "=== End Status ==="
