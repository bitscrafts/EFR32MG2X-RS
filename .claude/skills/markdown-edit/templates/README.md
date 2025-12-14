# Markdown Templates

Standard markdown file templates for consistent documentation structure across the project.

## Purpose

These templates ensure:
- Consistent heading structure across all documentation
- Complete information in each file type
- Searchability through standard section names
- Easy navigation and maintenance

## Available Templates

### 1. README_TEMPLATE.md

**Use for**: Workspace/project root README.md files

**Required Sections**:
- Overview
- Features
- Quick Start (Prerequisites, Installation, Basic Example)
- Project Structure
- Documentation (links to key docs)
- Examples
- Development (Building, Testing, Code Quality)
- Hardware Setup
- Architecture (with mermaid diagram)
- Contributing
- Roadmap
- Performance
- Compatibility
- Troubleshooting
- License
- Contact

**Search Keywords**: "Overview", "Quick Start", "Features", "Examples", "Development"

---

### 2. CLAUDE_TEMPLATE.md

**Use for**: CLAUDE.md project context files for Claude Code

**Required Sections**:
- Project Context
- Project Structure
- Current Status (Completed, In Progress)
- Development Guidelines (Code Style, Commit Messages, Documentation Standards)
- Documentation Backup Workflow
- File Organization
- Key Technical Patterns
- Component Generation (if applicable)
- Common Commands
- Hardware Specifications
- Phase Completion Summary
- Known Issues & Limitations
- Success Metrics
- Claude Code Skills Integration
- Resources
- Contact & Repository

**Search Keywords**: "Project Context", "Current Status", "Development Guidelines", "Common Commands"

---

### 3. MODULE_README_TEMPLATE.md

**Use for**: Module documentation in `src/module_name/README.md`

**Required Sections**:
- Overview
- Features
- Hardware Registers
- API Structure (with mermaid diagram)
- Usage Example
- Implementation Details
- embedded-hal Traits
- Testing
- Known Limitations
- References

**Search Keywords**: "Overview", "Features", "Hardware Registers", "Usage Example"

---

### 4. STATUS_TEMPLATE.md

**Use for**: Project status files (`docs/STATUS.md`, `crate/docs/STATUS.md`)

**Required Sections**:
- Quick Summary
- Current Phase
- Completed Work
- In Progress
- Blocked
- Next Steps
- Metrics
- Known Issues
- Handoff Checklist
- Recent Changes

**Search Keywords**: "Current Phase", "In Progress", "Metrics", "Handoff"

---

### 5. PLAN_TEMPLATE.md

**Use for**: Strategic planning documents (`docs/PLAN.md`)

**Required Sections**:
- Vision
- Success Criteria
- Development Phases
- Timeline (with mermaid gantt)
- Milestones
- Resource Requirements
- Risks and Mitigations
- Review Schedule

**Search Keywords**: "Vision", "Development Phases", "Milestones", "Timeline"

---

### 6. LOG_TEMPLATE.md

**Use for**: Milestone achievement logs (`docs/LOG.md`)

**Required Sections**:
- Milestone entries (most recent first)
- Key Decisions Log
- Statistics Summary
- Retrospective Notes

**Search Keywords**: "Milestone", "Achievements", "Metrics", "Retrospective"

---

### 7. CHANGELOG_TEMPLATE.md

**Use for**: Version change tracking (`CHANGELOG.md`)

**Required Sections**:
- Unreleased changes
- Version entries (semantic versioning)
- Change categories (Added, Changed, Deprecated, Removed, Fixed, Security)

**Search Keywords**: "Unreleased", "Added", "Changed", "Fixed"

---

### 8. BACKLOG_TEMPLATE.md

**Use for**: Task tracking (`docs/BACKLOG.md`)

**Required Sections**:
- Ready (Prioritized)
- In Progress
- Review
- Blocked
- Done (Recent - Last 30 Days)
- Ideas (Unprioritized)
- Metadata Tags Reference
- Metrics

**Search Keywords**: "Ready", "In Progress", "Blocked", "Done", "Metrics"

---

## Template Usage

### When Creating New Documentation

1. **Copy template**:
   ```bash
   cp .claude/templates/MODULE_README_TEMPLATE.md src/new_module/README.md
   ```

2. **Fill in placeholders**:
   - Replace `[Module Name]` with actual module name
   - Replace `YYYY-MM-DD` with current date
   - Fill in all required sections

3. **Verify structure**:
   - Ensure all required headings present
   - Check that search keywords are findable
   - Validate mermaid diagrams render correctly

### When Updating Existing Documentation

1. **Check template compliance**:
   ```bash
   # Compare headings
   grep "^## " existing_file.md
   grep "^## " .claude/templates/TEMPLATE.md
   ```

2. **Add missing sections**:
   - Use markdown-edit skill to add missing required sections
   - Preserve existing content

3. **Update metadata**:
   - Update "Last Updated" date
   - Increment version if significant changes

## Enforcement

The rust-embedded-orchestrator agent enforces template usage:

- **MUST**: Use templates for new documentation
- **MUST**: Include all required sections
- **MUST**: Update "Last Updated" metadata
- **SHOULD**: Follow template structure exactly

## Validation Script

Check if a file follows template structure:

```bash
# Usage: bash .claude/templates/validate-structure.sh path/to/file.md TEMPLATE_NAME
bash .claude/templates/validate-structure.sh src/gpio/README.md MODULE_README
```

(Script to be created in future enhancement)

## Template Maintenance

### Updating Templates

1. Discuss change need (missing section, better structure)
2. Update template file
3. Increment version in template metadata
4. Update existing files to match (use markdown-edit bulk-update)
5. Document change in this README

### Template Versions

- **v1.0**: Initial template set (2025-12-13)

---

**Created**: 2025-12-13
**Version**: 1.0
**Maintained by**: Project orchestrator
