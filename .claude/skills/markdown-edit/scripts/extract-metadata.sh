#!/bin/bash
# extract-metadata.sh - Extract metadata fields from markdown file footer
# Usage: bash extract-metadata.sh <file> [field]
#
# Extracts metadata from the last 10 lines of markdown files.
# If field is specified, returns only that field's value.
# If no field specified, returns all metadata in JSON format.

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <file> [field]"
    echo ""
    echo "Fields:"
    echo "  last-updated    - Last Updated date"
    echo "  version         - Template/File version"
    echo "  template-version - Template Version"
    echo "  status          - File status"
    echo "  maintained-by   - Maintainer"
    echo "  all             - All metadata (JSON)"
    echo ""
    echo "Example: $0 docs/STATUS.md last-updated"
    echo "Example: $0 docs/STATUS.md all"
    exit 1
fi

FILE="$1"
FIELD="${2:-all}"

if [[ ! -f "$FILE" ]]; then
    echo "Error: File '$FILE' not found" >&2
    exit 1
fi

# Extract last 10 lines
FOOTER=$(tail -10 "$FILE")

# Extract individual fields
LAST_UPDATED=$(echo "$FOOTER" | grep -E "^\*\*Last Updated\*\*:" | sed 's/\*\*Last Updated\*\*: //' || echo "")
VERSION=$(echo "$FOOTER" | grep -E "^\*\*Version\*\*:" | sed 's/\*\*Version\*\*: //' || echo "")
TEMPLATE_VERSION=$(echo "$FOOTER" | grep -E "^\*\*Template Version\*\*:" | sed 's/\*\*Template Version\*\*: //' || echo "")
STATUS=$(echo "$FOOTER" | grep -E "^\*\*Status\*\*:" | sed 's/\*\*Status\*\*: //' || echo "")
MAINTAINED_BY=$(echo "$FOOTER" | grep -E "^\*\*Maintained by\*\*:" | sed 's/\*\*Maintained by\*\*: //' || echo "")
MAINTAINED_BY="${MAINTAINED_BY:-$(echo "$FOOTER" | grep -E "^\*\*Log Maintained By\*\*:" | sed 's/\*\*Log Maintained By\*\*: //' || echo "")}"

# Return based on requested field
case "$FIELD" in
    last-updated)
        echo "$LAST_UPDATED"
        ;;
    version)
        echo "$VERSION"
        ;;
    template-version)
        echo "$TEMPLATE_VERSION"
        ;;
    status)
        echo "$STATUS"
        ;;
    maintained-by)
        echo "$MAINTAINED_BY"
        ;;
    all)
        # Output as JSON
        echo "{"
        echo "  \"file\": \"$FILE\","
        [[ -n "$LAST_UPDATED" ]] && echo "  \"last_updated\": \"$LAST_UPDATED\","
        [[ -n "$VERSION" ]] && echo "  \"version\": \"$VERSION\","
        [[ -n "$TEMPLATE_VERSION" ]] && echo "  \"template_version\": \"$TEMPLATE_VERSION\","
        [[ -n "$STATUS" ]] && echo "  \"status\": \"$STATUS\","
        [[ -n "$MAINTAINED_BY" ]] && echo "  \"maintained_by\": \"$MAINTAINED_BY\","
        echo "  \"extracted_at\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\""
        echo "}"
        ;;
    *)
        echo "Error: Unknown field '$FIELD'" >&2
        echo "Valid fields: last-updated, version, template-version, status, maintained-by, all" >&2
        exit 1
        ;;
esac
