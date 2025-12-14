---
name: project-manager
description: Manage project tasks using markdown-based Kanban board (BACKLOG.md) and milestone log (LOG.md). Use when adding tasks, moving tasks between columns, completing tasks, logging milestones, or viewing project status. Works with docs/BACKLOG.md and docs/LOG.md files. Uses markdown-edit skill scripts for safe file manipulation with automatic .archive backups.
allowed-tools: Read, Edit, Bash
version: 1.1.0
---

# Project Manager Skill

Manages EFR32MG24 HAL development tasks using markdown-based Kanban board (docs/BACKLOG.md) and milestone achievement log (docs/LOG.md).

## When to Use This Skill

Use this skill when you need to:
- **Add new tasks** to the backlog
- **Move tasks** between Kanban columns (Ready → In Progress → Done)
- **Complete tasks** and archive to Done column
- **Log milestones** after significant achievements
- **View project status** (current sprint, progress)
- **Update task metadata** (@priority, @phase, @started, @done)
- **Archive completed tasks** from Done → LOG.md (monthly)

## Do NOT Use This Skill For

- Editing PLAN.md (use markdown-edit skill instead)
- Editing STATUS.md (use markdown-edit skill instead)
- Creating new documentation files
- Code reviews or implementation work

## File Structure

This skill manages two key files:

```
docs/
├── BACKLOG.md    # Kanban task board (daily updates)
└── LOG.md        # Milestone achievement log (append-only)
```

## Kanban Columns in BACKLOG.md

1. **Ready (Prioritized)**: Tasks ready to start, sorted by priority
2. **In Progress**: Currently being worked on (limit to 1-2 tasks)
3. **Review**: Waiting for validation or testing
4. **Blocked**: Cannot proceed until blocker resolved
5. **Done (Recent)**: Completed in last 30 days
6. **Ideas (Unprioritized)**: Future work, not prioritized yet

## Task Metadata Tags

Tasks use GFM (GitHub Flavored Markdown) checkboxes with metadata:

```markdown
- [ ] **TIMER**: Implement TIMER0 basic functionality @priority(high) @phase(B)
- [x] **GPIO**: Hardware register access @done(2025-12-04) @phase(A)
```

**Metadata tags**:
- `@priority(high|medium|low)` - Task priority
- `@phase(A|B|C|...)` - Development phase
- `@started(YYYY-MM-DD)` - Start date
- `@done(YYYY-MM-DD)` - Completion date
- `@blocked(reason)` - Blocking reason
- `@review(status)` - Review status
- `@assignee(name)` - Task owner (usually "orchestrator")

## Core Operations

**IMPORTANT**: All modifications to docs/BACKLOG.md and docs/LOG.md MUST use markdown-edit skill scripts for automatic .archive backups and safe manipulation.

### 1. Add Task to BACKLOG.md

**When**: User requests "add task: implement TIMER peripheral"

**Steps**:
1. Read docs/BACKLOG.md
2. Determine appropriate column (usually "Ready")
3. Determine priority based on current sprint goals
4. Prepare task content with metadata tags
5. Use markdown-edit scripts to insert task:
   ```bash
   # Use append-section.sh or replace-section.sh from markdown-edit
   bash .claude/skills/markdown-edit/scripts/append-section.sh \
     docs/BACKLOG.md "Ready (Prioritized)" "- [ ] **TIMER**: Description @priority(high) @phase(B)"
   ```
6. Verify with git diff

**Example**:
```markdown
## Ready (Prioritized for Phase B)

### High Priority
- [ ] **TIMER**: Implement TIMER0 basic functionality @priority(high) @phase(B)
  - Register access (CTRL, CMD, CNT, TOP)
  - Counter modes (up, down, up-down)
  - Clock prescaler configuration
```

### 2. Move Task Between Columns

**When**: User says "start working on TIMER task" or "TIMER implementation complete"

**Steps**:
1. Read docs/BACKLOG.md
2. Find the task by keyword search
3. Prepare updated task content with new metadata:
   - Add `@started(YYYY-MM-DD)` when moving to "In Progress"
   - Add `@done(YYYY-MM-DD)` when moving to "Done"
   - Change checkbox `- [ ]` to `- [x]` when done
4. Use markdown-edit smart-replace.sh to move task atomically:
   ```bash
   # Use smart-replace.sh for atomic find-and-replace
   bash .claude/skills/markdown-edit/scripts/smart-replace.sh \
     docs/BACKLOG.md \
     "OLD_TASK_LINE_WITH_CONTEXT" \
     "NEW_TASK_LINE_IN_NEW_SECTION"
   ```
5. Verify with git diff

**Common transitions**:
- Ready → In Progress: Add @started
- In Progress → Done: Mark [x], add @done
- In Progress → Blocked: Add @blocked(reason)
- Blocked → Ready: Remove @blocked

### 3. Complete Task

**When**: Task finished, need to mark as done

**Steps**:
1. Move task from "In Progress" to "Done (Recent)"
2. Mark checkbox as completed: `- [x]`
3. Add @done(YYYY-MM-DD) metadata
4. Keep task in Done column for 30 days
5. After 30 days, archive to LOG.md

**Example completion**:
```markdown
## Done (Recent - Last 30 Days)

### Week of December 13-19, 2025

- [x] **TIMER**: Implement TIMER0 basic functionality @done(2025-12-15) @phase(B)
  - Register access complete
  - Counter modes working
  - 245 lines of code
```

### 4. Log Milestone to LOG.md

**When**: Significant achievement completed (phase done, major feature shipped)

**Steps**:
1. Read docs/LOG.md
2. Create new milestone entry content (most recent first)
3. Include:
   - Date (YYYY-MM-DD)
   - Milestone title
   - Achievements list
   - Impact description
   - Metrics (LOC, time, examples, etc.)
   - Technical highlights (optional)
4. Use markdown-edit prepend-to-file.sh to add at top:
   ```bash
   # Prepend new milestone entry to LOG.md
   bash .claude/skills/markdown-edit/scripts/prepend-to-file.sh \
     docs/LOG.md \
     "NEW_MILESTONE_ENTRY_CONTENT"
   ```
5. Verify with git diff

**Milestone entry template**:
```markdown
## YYYY-MM-DD: Milestone Title

**Milestone**: Brief description

**Achievements**:
- Implemented feature X
- Added Y capability
- Created Z examples

**Impact**:
- How this advances the project
- What's now possible

**Metrics**:
- Lines of code: X
- Build time: Y minutes
- Examples: Z total

**Technical Highlights** (optional):
- Key technical decisions
- Interesting implementation details
```

### 5. View Project Status

**When**: User asks "what's the current status?" or "show backlog"

**Steps**:
1. Read docs/BACKLOG.md
2. Summarize:
   - Current sprint (from header)
   - Tasks in "In Progress"
   - Count of Ready tasks
   - Count of Blocked tasks
   - Recent completions
   - Project metrics section
3. Present formatted summary

**Example output**:
```
Current Sprint: Phase B - Communication Peripherals (70% complete)

In Progress:
- None (ready to start new task)

Ready (High Priority):
- TIMER: Implement TIMER0 basic functionality
- PWM: Add PWM generation support
- Examples: Create 06_timer.rs example

Blocked:
- Phase C: Embassy async support (waiting for Phase B completion)

Recent Completions (last 7 days):
- Documentation: Phase terminology update
- I2C: Master mode implementation

Metrics:
- Phase B Progress: 70% (3 of 5 peripherals done)
- Total LOC: ~1,931 lines
- Examples: 5 working examples
```

## Best Practices

### Task Management

1. **Limit WIP**: Keep 1-2 tasks in "In Progress" maximum
2. **Prioritize Ready**: Sort Ready column by priority (high → medium → low)
3. **Update metadata**: Always add @started, @done dates
4. **Archive monthly**: Move Done tasks older than 30 days to LOG.md
5. **Keep Ideas separate**: Don't prioritize Ideas until ready

### Milestone Logging

1. **Log after completion**: Don't log until work is done
2. **Include metrics**: LOC, time, examples count
3. **Document decisions**: Why certain choices were made
4. **Add retrospectives**: What went well, challenges, lessons
5. **Never edit history**: LOG.md is append-only, prepend new entries

### Integration with Other Files

- **PLAN.md**: Strategic roadmap (phases, milestones) - don't duplicate here
- **STATUS.md**: Current state snapshot - sync after milestones
- **BACKLOG.md**: Tactical tasks (daily work) - this file
- **LOG.md**: Historical record - archive completed work

## Scripts

The skill uses markdown-edit scripts for all file modifications:

**From markdown-edit skill** (.claude/skills/markdown-edit/scripts/):
1. **smart-replace.sh**: Find and replace with .archive backup
2. **append-section.sh**: Append content to markdown section
3. **replace-section.sh**: Replace entire markdown section
4. **prepend-to-file.sh**: Add content at beginning of file

**From this skill** (.claude/skills/project-manager/scripts/):
1. **view-status.sh**: Display current project status (read-only)

**Important**: Never use Edit tool directly on BACKLOG.md or LOG.md. Always use markdown-edit scripts for automatic .archive backups.

## Common Workflows

### Workflow 1: Starting New Task

```
1. User: "Start working on TIMER implementation"
2. Read docs/BACKLOG.md
3. Find TIMER task in "Ready" column
4. Move to "In Progress"
5. Add @started(2025-12-13)
6. Report: "Started TIMER task, moved to In Progress"
```

### Workflow 2: Completing Task

```
1. User: "TIMER implementation done"
2. Read docs/BACKLOG.md
3. Find TIMER task in "In Progress"
4. Move to "Done (Recent)"
5. Mark [x], add @done(2025-12-15)
6. Report completion with summary
```

### Workflow 3: Logging Milestone

```
1. User: "Log Phase B completion milestone"
2. Read docs/LOG.md
3. Create milestone entry with:
   - Date, title, achievements
   - Metrics (LOC, time, progress)
   - Impact and retrospectives
4. Prepend to LOG.md
5. Report: "Milestone logged"
```

### Workflow 4: Weekly Review

```
1. User: "Show weekly status"
2. Read docs/BACKLOG.md
3. Summarize:
   - Tasks completed this week
   - Current in-progress
   - Upcoming priorities
   - Blockers
4. Check if Done tasks need archiving (>30 days)
5. Report comprehensive status
```

## Error Handling

**Task not found**:
- Search across all columns
- Suggest similar tasks
- Ask user to clarify

**Conflicting metadata**:
- Task marked [x] but no @done date → add @done with today
- Task in "Done" but not marked [x] → mark [x]
- Task @started but still in "Ready" → move to "In Progress"

**File format issues**:
- Preserve existing structure
- Don't break markdown formatting
- Keep GFM checkbox syntax valid

## Maintenance

**Monthly archiving** (automated or manual):
1. Find Done tasks older than 30 days
2. Create summary for LOG.md
3. Move to LOG.md under appropriate date
4. Remove from BACKLOG.md Done column

**Quarterly review**:
1. Review Ideas column - prioritize or remove
2. Check Blocked tasks - resolve or document status
3. Update project metrics in BACKLOG.md
4. Add retrospective entry to LOG.md

## Integration with Orchestrator

The orchestrator agent should:
1. **Before commits**: Check if BACKLOG.md needs updates
2. **After completions**: Move tasks to Done
3. **After milestones**: Add entry to LOG.md
4. **Quality gate**: "BACKLOG.md updated with completed tasks"

## Examples

See EXAMPLES.md for detailed usage scenarios with full before/after markdown examples.

---

**Created**: December 13, 2025
**Version**: 1.0.0
**Maintained by**: Rust Embedded Orchestrator
