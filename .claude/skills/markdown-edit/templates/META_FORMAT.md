# Markdown File Metadata Format

## Compact META Tag Format

All markdown documentation files MUST include a metadata comment at the end of the file using this format:

```markdown
<!-- META: key1=value1 key2=value2 key3="value with spaces" -->
```

## Standard Fields

### Required Fields (All Templates)
- `last_updated=YYYY-MM-DD` - Date of last modification
- `version=X.Y.Z` - Document/template version

### Common Optional Fields
- `status=Active|Complete|Deprecated` - Document status
- `maintainer=Name` - Person/team responsible
- `phase=Phase_X` - Project phase (for CLAUDE.md, STATUS.md)
- `progress=XX%` - Completion percentage
- `template_version=X.Y` - Which template version was used

## Template-Specific Metadata

### README.md (Workspace)
```markdown
<!-- META: last_updated=2025-12-13 version=0.1.0 status=Active_Development maintainer=YourName -->
```

### CLAUDE.md (Project Context)
```markdown
<!-- META: last_updated=2025-12-13 phase=Phase_B progress=70% next_milestone="Complete timers" -->
```

### MODULE_README.md (Module Documentation)
```markdown
<!-- META: last_updated=2025-12-13 version=0.1.0 maintainer=TeamName status=Complete -->
```

### STATUS.md (Project Status)
```markdown
<!-- META: last_updated=2025-12-13 version=1.2.0 phase=Phase_B progress=70% -->
```

### PLAN.md (Strategic Plan)
```markdown
<!-- META: last_updated=2025-12-13 version=2.0.0 next_review=2025-12-20 -->
```

### LOG.md (Milestone Log)
```markdown
<!-- META: last_updated=2025-12-13 template_version=1.0 retention=Permanent -->
```

### CHANGELOG.md (Version History)
```markdown
<!-- META: last_updated=2025-12-13 template_version=1.0 latest_version=0.2.0 -->
```

### BACKLOG.md (Task Tracking)
```markdown
<!-- META: last_updated=2025-12-13 template_version=1.0 update_frequency=Daily -->
```

## Parsing the META Tag

### Extract All Metadata (bash)
```bash
# Extract META line
META=$(tail -5 "$file" | grep "<!-- META:" | sed 's/<!-- META: //; s/ -->//')

# Parse individual fields
LAST_UPDATED=$(echo "$META" | grep -oE 'last_updated=[^ ]+' | cut -d= -f2)
VERSION=$(echo "$META" | grep -oE 'version=[^ ]+' | cut -d= -f2)
STATUS=$(echo "$META" | grep -oE 'status=[^ ]+' | cut -d= -f2)
```

### Extract Specific Field (bash)
```bash
tail -5 file.md | grep "<!-- META:" | grep -oE 'last_updated=[^ ]+' | cut -d= -f2
```

### Update META Field (bash)
```bash
# Update last_updated to current date
sed -i.bak 's/last_updated=[0-9-]*/last_updated='$(date +%Y-%m-%d)'/' file.md
```

## Validation

### Check if File Has META Tag
```bash
if tail -5 "$file" | grep -q "<!-- META:"; then
    echo "META tag found"
else
    echo "Missing META tag"
fi
```

### Check if last_updated is Current
```bash
LAST_UPDATED=$(tail -5 "$file" | grep "<!-- META:" | grep -oE 'last_updated=[0-9-]+' | cut -d= -f2)
GIT_DATE=$(git log -1 --format="%cs" -- "$file")

if [[ "$LAST_UPDATED" < "$GIT_DATE" ]]; then
    echo "Stale: needs update"
fi
```

## Benefits of Compact Format

1. **Machine-Readable**: Easy to parse with grep/sed
2. **Single Line**: Doesn't clutter the document footer
3. **Comment Format**: Invisible when rendered as HTML/markdown
4. **Extensible**: Can add new fields without breaking parsers
5. **Git-Friendly**: Single line = cleaner diffs

## Migration from Old Format

Old multi-line format:
```markdown
**Last Updated**: 2025-12-13
**Version**: 1.0.0
**Status**: Active
```

New compact format:
```markdown
<!-- META: last_updated=2025-12-13 version=1.0.0 status=Active -->
```

Use this sed command to convert:
```bash
# This is a simplified example - actual conversion requires more complex logic
tail -10 file.md | grep "^\*\*Last Updated\*\*:" # Extract old format
# Then replace with new format
```

## Scripts

See `.claude/skills/markdown-edit/scripts/` for utility scripts:
- `extract-metadata.sh` - Extract metadata fields
- `update-timestamp.sh` - Update last_updated field
- `check-stale-docs.sh` - Find files needing updates
- `list-doc-metadata.sh` - Report on all file metadata

---

<!-- META: last_updated=2025-12-13 version=1.0.0 type=Documentation -->
