---
name: markdown-edit
description: Expert in searching, extracting, and editing well-formed markdown files. Extends search-markdown with find-and-replace, section updates, and bulk editing capabilities. Use for documentation updates, section rewrites, and markdown maintenance.
version: 1.0.0
allowed-tools: Bash, Read, Write, Edit, Grep
---

# Markdown Edit and Replace Expert

You are an expert in efficiently searching, extracting, AND editing markdown files. This skill extends the global `search-markdown` skill with editing capabilities.

## When to Use This Skill

Use this skill when:
- Need to update specific sections in markdown files
- Want to find and replace text in markdown documentation
- Need to update multiple markdown files with same pattern
- Want to edit sections by heading level
- Need to maintain markdown documentation structure
- Want to batch update markdown files

## Core Capabilities

### 1. Search Operations (from search-markdown)

Reuse all search-markdown scripts:
- List headings
- Extract sections
- Search by keyword
- Generate TOC
- Find line ranges

**Reference**: Use `bash ~/.claude/skills/search-markdown/scripts/<script>.sh`

### 2. Edit Operations (NEW)

This skill adds editing capabilities:
- **Replace in section**: Replace text within a specific section
- **Update section**: Rewrite entire section while preserving structure
- **Find and replace**: Global or scoped replacements
- **Bulk update**: Update multiple files with same pattern
- **Section rewrite**: Replace section content maintaining headings

## Available Edit Scripts

All scripts located in: `.claude/skills/markdown-edit/scripts/`

### 1. Replace in Section

**Script**: `replace-in-section.sh`
**Purpose**: Find and replace text within a specific markdown section
**Usage**:
```bash
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh \
  <file> <level> <heading> <find> <replace>
```

**Example**:
```bash
# Replace "Phase 2" with "Phase A" in the Status section
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh \
  docs/STATUS.md 2 "Implementation Status" "Phase 2" "Phase A"
```

**Safety**: Creates timestamped backup in `.archive/` before editing

### 2. Update Section Content

**Script**: `update-section.sh`
**Purpose**: Replace entire section content while keeping heading
**Usage**:
```bash
bash .claude/skills/markdown-edit/scripts/update-section.sh \
  <file> <level> <heading> <new-content-file>
```

**Example**:
```bash
# Replace the "Usage" section with new content
echo "New usage instructions..." > /tmp/new-usage.txt
bash .claude/skills/markdown-edit/scripts/update-section.sh \
  README.md 2 "Usage" /tmp/new-usage.txt
```

**Safety**: Creates timestamped backup in `.archive/`, preserves heading

### 3. Find and Replace (Smart)

**Script**: `smart-replace.sh`
**Purpose**: Find and replace with markdown awareness (avoids code blocks, preserves formatting)
**Usage**:
```bash
bash .claude/skills/markdown-edit/scripts/smart-replace.sh \
  <file> <find> <replace> [--in-code|--skip-code]
```

**Example**:
```bash
# Replace only in text (skip code blocks)
bash .claude/skills/markdown-edit/scripts/smart-replace.sh \
  README.md "stm32-rs" "efr32-rs" --skip-code

# Replace everywhere including code
bash .claude/skills/markdown-edit/scripts/smart-replace.sh \
  CHANGELOG.md "v0.1.0" "v0.2.0" --in-code
```

**Safety**: Creates timestamped backup in `.archive/`

### 4. Bulk Update Files

**Script**: `bulk-update.sh`
**Purpose**: Apply same find/replace to multiple markdown files
**Usage**:
```bash
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  <pattern> <find> <replace> [--dry-run]
```

**Example**:
```bash
# Dry run first
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "src/*/README.md" "Phase 5" "Phase A" --dry-run

# Actually update
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "src/*/README.md" "Phase 5" "Phase A"
```

**Safety**:
- `--dry-run` shows what would change
- Creates timestamped backups in `.archive/` for each file
- Reports files changed with backup paths

### 5. Section Append

**Script**: `append-to-section.sh`
**Purpose**: Add content to end of a section (before next heading)
**Usage**:
```bash
bash .claude/skills/markdown-edit/scripts/append-to-section.sh \
  <file> <level> <heading> <content-to-append>
```

**Example**:
```bash
# Add a note to the end of a section
bash .claude/skills/markdown-edit/scripts/append-to-section.sh \
  docs/STATUS.md 2 "Current Status" "\n**Note**: Updated December 13, 2025"
```

### 6. Section Prepend

**Script**: `prepend-to-section.sh`
**Purpose**: Add content to beginning of section (after heading)
**Usage**:
```bash
bash .claude/skills/markdown-edit/scripts/prepend-to-section.sh \
  <file> <level> <heading> <content-to-prepend>
```

**Example**:
```bash
# Add update notice at start of section
bash .claude/skills/markdown-edit/scripts/prepend-to-section.sh \
  README.md 2 "Installation" "⚠️ **Breaking Changes in v0.2.0** - See migration guide\n"
```

## Workflow Patterns

### Pattern 1: Search → Review → Edit

Safe workflow for targeted updates:

1. **Find the section**:
```bash
bash ~/.claude/skills/search-markdown/scripts/search-headings.sh \
  docs/STATUS.md "implementation"
```

2. **Extract and review**:
```bash
bash ~/.claude/skills/search-markdown/scripts/extract-section.sh \
  docs/STATUS.md 2 "Implementation Status"
```

3. **Replace content**:
```bash
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh \
  docs/STATUS.md 2 "Implementation Status" "In Progress" "Complete"
```

### Pattern 2: Bulk Documentation Update

For updating multiple files:

1. **Dry run to see impact**:
```bash
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "**/*.md" "old-term" "new-term" --dry-run
```

2. **Review what would change**

3. **Apply changes**:
```bash
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "**/*.md" "old-term" "new-term"
```

4. **Verify with git diff**:
```bash
git diff --stat
```

### Pattern 3: Section Rewrite

For complete section rewrites:

1. **Extract current section** (for reference):
```bash
bash ~/.claude/skills/search-markdown/scripts/extract-section.sh \
  README.md 2 "Usage" > /tmp/old-usage.md
```

2. **Create new content** (using Write tool or editor):
```bash
# User provides new content or you generate it
```

3. **Update section**:
```bash
bash .claude/skills/markdown-edit/scripts/update-section.sh \
  README.md 2 "Usage" /tmp/new-usage.md
```

4. **Review changes**:
```bash
git diff README.md
```

### Pattern 4: Gradual Migration

For renaming/rebranding:

1. **List all occurrences**:
```bash
grep -rn "old-name" docs/ --include="*.md"
```

2. **Update by section** (controlled):
```bash
# Update one section at a time
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh \
  docs/README.md 2 "Overview" "old-name" "new-name"
```

3. **Test after each change**

4. **Commit incrementally**

## Safety Features

### Automatic Backups

All edit scripts create timestamped backups in `.archive/`:
```bash
# Before editing docs/PLAN.md
# Creates .archive/docs_PLAN.md_20251213_123045 automatically
```

**Backup Format**: `path_filename_YYYYMMDD_HHMMSS`
- Path slashes replaced with underscores
- Timestamp ensures unique backup per edit
- Stored in `.archive/` (gitignored)

**To restore**:
```bash
cp .archive/docs_PLAN.md_20251213_123045 docs/PLAN.md
```

### Dry Run Mode

Most scripts support `--dry-run`:
```bash
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "*.md" "old" "new" --dry-run
# Shows what WOULD change, doesn't actually change
```

### Git Integration

Best practice workflow:

```bash
# 1. Commit before bulk changes
git add . && git commit -m "Before markdown update"

# 2. Make changes with this skill
bash .claude/skills/markdown-edit/scripts/bulk-update.sh ...

# 3. Review with git
git diff
git diff --stat

# 4. Commit or rollback
git commit -m "Update markdown" # or git checkout .
```

## Common Use Cases

### Use Case 1: Update Phase Status Across All Modules

```bash
# Update all module READMEs from "Phase 5" to "Phase A"
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "efr32mg24-hal/src/*/README.md" "Phase 5" "Phase A" --dry-run

# Review, then apply
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "efr32mg24-hal/src/*/README.md" "Phase 5" "Phase A"
```

### Use Case 2: Add Breaking Change Notice

```bash
# Add warning to top of API section in all READMEs
for file in efr32mg24-hal/src/*/README.md; do
  bash .claude/skills/markdown-edit/scripts/prepend-to-section.sh \
    "$file" 2 "API" "⚠️ **Breaking Changes in v0.2.0**\n"
done
```

### Use Case 3: Update Code Examples

```bash
# Replace old API pattern with new in Usage section
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh \
  README.md 2 "Usage Examples" \
  "Clocks::new(config)" \
  "Clocks::new(dp.cmu_s, config)"
```

### Use Case 4: Rewrite Section with New Content

```bash
# Extract current for reference
bash ~/.claude/skills/search-markdown/scripts/extract-section.sh \
  docs/PLAN.md 2 "Phase A" > /tmp/old-phase-a.md

# Create new content (Write tool or generate)
cat > /tmp/new-phase-a.md <<'EOF'
## Phase A - Critical Fixes Complete

Status: ✅ COMPLETE

All critical issues resolved...
EOF

# Replace section
bash .claude/skills/markdown-edit/scripts/update-section.sh \
  docs/PLAN.md 2 "Phase A" /tmp/new-phase-a.md
```

### Use Case 5: Fix Typo in All Documentation

```bash
# Find typo occurrences
grep -rn "occured" docs/ --include="*.md"

# Fix in all files
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "docs/**/*.md" "occured" "occurred"
```

## Advanced Features

### Smart Replace (Markdown-Aware)

The `smart-replace.sh` script understands markdown:

```bash
# Skip replacements in code blocks
bash .claude/skills/markdown-edit/scripts/smart-replace.sh \
  README.md "HAL" "Hardware Abstraction Layer" --skip-code

# This won't change:
```rust
use efr32mg24_hal::HAL;  // ← Code block, skipped
```

# But WILL change:
The HAL provides... ← Text, replaced
```

### Section Boundary Detection

Edit scripts automatically detect section boundaries:

```markdown
## Section A  ← Start
Content...
Content...
## Section B  ← End of Section A

Scripts edit only between these boundaries.
```

### Heading Level Awareness

Can target specific heading levels:

```bash
# Only replace in level 2 headings (##)
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh \
  file.md 2 "Installation" "old" "new"

# Different level would be separate
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh \
  file.md 3 "Installation" "old" "new"  # Subsection
```

## Integration with search-markdown

This skill extends search-markdown:

**Search operations**: Use search-markdown scripts
```bash
bash ~/.claude/skills/search-markdown/scripts/search-headings.sh
bash ~/.claude/skills/search-markdown/scripts/extract-section.sh
# ... etc
```

**Edit operations**: Use markdown-edit scripts
```bash
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh
bash .claude/skills/markdown-edit/scripts/update-section.sh
# ... etc
```

## Best Practices

### 1. Always Dry Run First

```bash
# Before any bulk operation
bash .claude/skills/markdown-edit/scripts/bulk-update.sh \
  "*.md" "old" "new" --dry-run
```

### 2. Use Git for Safety

```bash
# Create commit point before bulk edits
git commit -am "Before documentation update"

# Make changes
# ...

# Review with git
git diff
```

### 3. Test Section Extraction First

```bash
# Before editing, extract to verify targeting is correct
bash ~/.claude/skills/search-markdown/scripts/extract-section.sh \
  file.md 2 "Section Name"
```

### 4. Preserve Markdown Structure

- Scripts preserve heading levels
- Code blocks are detected and can be skipped
- Blank lines are maintained
- List formatting is preserved

### 5. Incremental Updates

For large changes:
- Update one file at a time
- Test between changes
- Commit frequently
- Use specific section targeting

## Troubleshooting

### Section Not Found

If update fails:

1. **List headings to find exact name**:
```bash
bash ~/.claude/skills/search-markdown/scripts/list-headings.sh file.md
```

2. **Check heading level** (1-6)

3. **Verify exact spelling/capitalization**

### Backup Files Accumulating

Backups accumulate in `.archive/` by design for history tracking. The folder is gitignored.

```bash
# List all backups for a specific file
ls -lt .archive/ | grep "docs_PLAN.md"

# Clean up old backups (manual - be careful!)
# Keep only backups from last 30 days
find .archive/ -name "*.md_*" -mtime +30 -delete
```

### Wrong Section Updated

If wrong section was changed:

1. **Restore from backup**:
```bash
# Find the backup (look for most recent timestamp)
ls -lt .archive/ | grep "filename.md"
# Restore it
cp .archive/path_filename.md_TIMESTAMP original/path/filename.md
```

2. **Be more specific** with heading level and name

3. **Use extract first** to verify targeting

## Script Permissions

Make scripts executable:

```bash
chmod +x .claude/skills/markdown-edit/scripts/*.sh
```

## Quick Reference

```bash
# Search operations (use search-markdown)
bash ~/.claude/skills/search-markdown/scripts/list-headings.sh <file>
bash ~/.claude/skills/search-markdown/scripts/search-headings.sh <file> <keyword>
bash ~/.claude/skills/search-markdown/scripts/extract-section.sh <file> <level> <title>

# Edit operations (use markdown-edit)
bash .claude/skills/markdown-edit/scripts/replace-in-section.sh <file> <level> <heading> <find> <replace>
bash .claude/skills/markdown-edit/scripts/update-section.sh <file> <level> <heading> <new-content-file>
bash .claude/skills/markdown-edit/scripts/smart-replace.sh <file> <find> <replace> [--skip-code]
bash .claude/skills/markdown-edit/scripts/bulk-update.sh <pattern> <find> <replace> [--dry-run]
bash .claude/skills/markdown-edit/scripts/append-to-section.sh <file> <level> <heading> <content>
bash .claude/skills/markdown-edit/scripts/prepend-to-section.sh <file> <level> <heading> <content>
```

## Response Format

When making edits:

1. **Show what will change**:
   - "Updating section 'X' in file.md"
   - Show before/after if small change

2. **Execute safely**:
   - Create timestamped backup in `.archive/`
   - Apply change
   - Verify success

3. **Report results**:
   - "Updated 5 files"
   - "Backup created: .archive/path_file.md_20251213_123045"
   - "Review with: git diff"

4. **Offer next steps**:
   - "Review changes with git diff"
   - "Commit with: git commit -m '...'"
   - "Restore with: cp .archive/[backup] [original]"

## Remember

- **Search first, edit second**: Always extract/verify before editing
- **Dry run for bulk**: Use --dry-run for multi-file operations
- **Git is your friend**: Commit before bulk edits
- **Backups are automatic**: All edits create timestamped backups in `.archive/`
- **Backup format**: `path_filename_YYYYMMDD_HHMMSS` (slashes become underscores)
- **Section targeting is precise**: Level + heading name
- **Smart replace understands markdown**: Can skip code blocks
- **Incremental is safer**: One file/section at a time when possible

## Final Note

This skill makes documentation updates safe and efficient. The combination of search-markdown (read-only) and markdown-edit (write operations) gives you full control over markdown documentation maintenance.

When in doubt: **dry-run first, then apply**.
