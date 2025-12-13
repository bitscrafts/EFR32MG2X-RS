# Project Management Research for Solo Developer Teams

**Date**: December 13, 2025
**Purpose**: Research industry-standard project management approaches for single-person software development teams

---

## Research Summary

Researched modern project management best practices for solo developers in 2025, focusing on:
- Markdown-based approaches (git-friendly)
- Kanban methodologies for personal productivity
- Industry standards for project documentation
- Tools and formats compatible with AI assistants (Claude Code)

---

## Key Findings

### Industry Best Practices (2025)

1. **Agile for Solo Developers**: Scrum and Kanban frameworks are scalable and manageable even for one-person teams
2. **Clear Scope Definition**: Defined scope is cornerstone of successful projects
3. **Documentation**: Essential for continuity and can align stakeholders throughout SDLC
4. **Markdown-Native**: Trend toward plain text, git-versioned project management
5. **AI Integration**: Tools supporting MCP (Model Context Protocol) for Claude Code compatibility

### File Structure Standards

**Common Files**:
- `README.md` - Project overview, setup instructions (root directory)
- `CHANGELOG.md` - Release notes following Conventional Commits
- `TODO.md` - Task tracking (TODO.md standard or custom)
- `PLAN.md` - Roadmap, phase breakdown
- `STATUS.md` - Current implementation status
- `BACKLOG.md` - Kanban-style task board

**Recommended Structure**:
```
project/
├── README.md
├── CHANGELOG.md
├── CLAUDE.md          # AI assistant instructions
├── docs/
│   ├── PLAN.md        # Phases, milestones
│   ├── STATUS.md      # Implementation status
│   ├── BACKLOG.md     # Kanban task board
│   └── LOG.md         # Milestone achievements
└── .archive/          # Documentation backups
```

---

## Options Evaluated

### Option 1: TODO.md Standard (Simple & Lightweight)

**Format**: Single `TODO.md` file in project root
**Standard**: GitHub Flavored Markdown (GFM) Task Lists

**Structure**:
```markdown
# TODO

## Backlog
- [ ] Feature: Add timer peripheral support
- [ ] Research: Embassy async framework

## In Progress
- [x] @critical Implement I2C master mode @started(2025-12-13)

## Done
- [x] GPIO hardware register access @done(2025-12-04)
```

**Pros**:
- Simple, one file to track everything
- GitHub/GitLab native rendering
- Easy to parse by tools
- Low overhead
- GFM task list syntax widely supported

**Cons**:
- No kanban visualization without tools
- Can get cluttered on large projects
- Limited project management features
- No time-based prioritization

**Best For**: Very small projects, quick prototypes, minimal overhead

---

### Option 2: Personal Kanban System (Markdown TUI)

**Format**: Multiple markdown files with status-based organization
**Files**: `TODO.md`, `LOG.md`, `DONE.md`, `BACKLOG.md`

**Structure**:
```
project/
├── TODO.md           # Kanban: Landing, Today, This Week, Next Week
├── LOG.md            # Milestone achievements log
├── DONE.md           # Completed tasks archive
└── BACKLOG.md        # Future ideas & deferred items
```

**Example TODO.md**:
```markdown
# TODO

## Landing (Inbox)
- Research TIMER registers in reference manual
- Review stm32-rs timer patterns

## Today
- [ ] Implement TIMER0 basic configuration
- [ ] Add clock enable for TIMER

## This Week
- [ ] Write timer module README.md
- [ ] Create PWM example

## Next Week
- [ ] Add TIMER1-4 support
```

**Features**:
- Time-based prioritization (Today/This Week/Next Week)
- Milestone logging in LOG.md
- Completed tasks archived in DONE.md
- Backlog for future ideas

**Pros**:
- Clear time-based prioritization
- Milestone logging for retrospectives
- Separates active work from archive
- Good for iterative development
- Prevents TODO.md from growing too large

**Cons**:
- Requires discipline to maintain multiple files
- Manual file management
- Need to move tasks between files
- No tooling support

**Best For**: Personal time management, iterative projects, developers who like time-boxing

---

### Option 3: Backlog.md (Git-Native with CLI)

**Format**: Git repo + Markdown files + Zero-config CLI
**Tool**: [Backlog.md](https://github.com/MrLesk/Backlog.md)
**Compatibility**: AI-friendly, MCP compatible (works with Claude Code)

**Structure**:
```
project/
├── .backlog/
│   ├── backlog.md    # All tasks
│   ├── doing.md      # In progress
│   └── done.md       # Completed
└── backlog.json      # Configuration
```

**Features**:
- Markdown-native tasks with metadata
- Instant terminal Kanban view (CLI)
- Modern web interface
- Git version controlled
- Zero external dependencies
- Works with Claude Code, Gemini CLI, Codex (MCP protocol)
- Automatic task state transitions

**CLI Commands** (typical):
```bash
backlog add "Implement TIMER peripheral"
backlog start <task-id>
backlog done <task-id>
backlog view                    # Terminal Kanban
```

**Pros**:
- CLI + Web UI
- AI assistant integration (works with Claude Code!)
- Git version control built-in
- Zero external dependencies
- Automatic state management
- Professional Kanban visualization

**Cons**:
- Requires CLI tool installation
- More setup than plain markdown
- Learning curve for CLI commands
- Hidden .backlog/ directory

**Best For**: Developers who want tooling, AI integration, professional Kanban boards

---

### Option 4: Docs-Based Project Management (Enhanced Current Approach) ⭐ **SELECTED**

**Format**: `docs/` folder with structured markdown files
**Files**: `PLAN.md`, `STATUS.md`, `BACKLOG.md`, `CHANGELOG.md`

**Structure**:
```
project/
├── README.md           # Project overview
├── CHANGELOG.md        # Release notes
├── CLAUDE.md           # AI assistant instructions
├── docs/
│   ├── PLAN.md         # Phase breakdown, milestones, roadmap
│   ├── STATUS.md       # Current implementation status, metrics
│   ├── BACKLOG.md      # Kanban-style task board (NEW)
│   ├── LOG.md          # Milestone achievement log (NEW)
│   └── README.md       # Documentation index
└── .archive/           # Documentation backups (timestamped)
```

**BACKLOG.md Format** (Kanban-style):
```markdown
# Project Backlog

**Last Updated**: December 13, 2025
**Current Sprint**: Phase B - Communication Peripherals

---

## Ready (Prioritized)
- [ ] **Phase B**: Implement TIMER0-4 basic functionality @priority(high)
- [ ] **Phase B**: Add PWM generation support @priority(high)
- [ ] **Phase B**: Create timer examples @priority(medium)

## In Progress
- [x] **Phase B**: I2C master mode @started(2025-12-12) @assignee(orchestrator)

## Review
- [ ] **Phase B**: USART implementation @review(waiting for hardware test)

## Blocked
- [ ] **Phase C**: Embassy async support @blocked(Waiting for Phase B completion)

## Done (Recent)
- [x] **Phase A**: GPIO hardware register access @done(2025-12-04)
- [x] **Phase A**: CMU clock configuration @done(2025-12-04)
- [x] **Phase B**: USART serial communication @done(2025-12-04)
- [x] **Phase B**: I2C master mode @done(2025-12-12)

## Ideas (Unprioritized)
- [ ] Add DMA support for peripherals
- [ ] Implement low-power modes (EM1-EM4)
- [ ] Security peripherals exploration
```

**LOG.md Format** (Milestone tracking):
```markdown
# Project Milestone Log

Log of significant achievements and milestones.

---

## 2025-12-13: Documentation Cleanup Complete
- Updated phase terminology from "Phase 5" to "Phase A/B"
- Archived 4 obsolete documentation files
- Enhanced orchestrator with mandatory pre-commit documentation audit
- **Impact**: Improved documentation consistency and maintainability

## 2025-12-12: Phase B - I2C Master Mode Complete
- Implemented I2C0/I2C1 register access
- Added embedded-hal v1.0 I2c traits
- Created example 05_i2c.rs
- **Metrics**: 524 lines of code, 237 lines documentation
- **Impact**: Communication peripherals 70% complete

## 2025-12-04: Phase A - Core HAL Complete
- GPIO, CMU, Delay modules implemented
- All modules use hardware register access
- 3 working examples (clock, delay, gpio)
- **Metrics**: 1034 total lines HAL code
- **Impact**: Essential peripherals ready for production
```

**Pros**:
- Aligns with existing documentation structure (PLAN.md, STATUS.md already exist)
- Clear separation of concerns:
  - PLAN.md: Long-term roadmap, phases
  - STATUS.md: Implementation status, metrics, handoff
  - BACKLOG.md: Daily/weekly tasks (Kanban)
  - LOG.md: Milestone achievements (retrospectives)
- Professional, scalable approach
- Works with existing markdown-edit skill
- Can integrate with orchestrator for automated updates
- Git version controlled
- No external dependencies

**Cons**:
- More files to maintain (4 documentation files)
- Requires discipline to keep PLAN vs STATUS vs BACKLOG synchronized
- Manual task management (no CLI automation)

**Best For**: Professional projects, teams of 1-5, projects with AI assistants, long-term maintenance

---

## Decision: Option 4 (Enhanced Docs-Based) ⭐

**Rationale**:
1. **Aligns with current structure**: EFR32MG24 project already has `docs/PLAN.md` and `docs/STATUS.md`
2. **Separation of concerns**: PLAN (strategic), STATUS (current state), BACKLOG (tactical tasks)
3. **Scalable**: Works for solo developer now, scales to small teams later
4. **AI integration**: markdown-edit skill can safely update these files
5. **Orchestrator integration**: Can enforce backlog updates before commits
6. **Professional**: Industry-standard approach for open-source projects

---

## Implementation Plan

### Phase 1: Create Files
1. Create `docs/BACKLOG.md` with Kanban structure
2. Create `docs/LOG.md` for milestone tracking
3. Populate BACKLOG.md with current Phase B tasks

### Phase 2: Create Project Management Skill
1. Create `.claude/skills/project-manager/` skill
2. Scripts for:
   - `add-task.sh` - Add task to BACKLOG.md
   - `move-task.sh` - Move task between columns
   - `update-log.sh` - Add milestone to LOG.md
   - `sync-status.sh` - Sync BACKLOG → STATUS.md
   - `view-kanban.sh` - Terminal Kanban view

### Phase 3: Orchestrator Integration
1. Update orchestrator to check BACKLOG.md before commits
2. Enforce task completion updates
3. Add quality gate: "BACKLOG.md updated with completed tasks"

### Phase 4: Documentation
1. Update `docs/README.md` with BACKLOG.md and LOG.md descriptions
2. Add BACKLOG.md usage instructions to CLAUDE.md
3. Create skill documentation (SKILL.md)

---

## References

- [TODO.md Standard](https://github.com/todo-md/todo-md)
- [Backlog.md - Git-Native Project Management](https://github.com/MrLesk/Backlog.md)
- [Markdown Personal Management System](https://renormalize.substack.com/p/my-markdown-project-management-system)
- [GitHub Projects for Solo Developers](https://www.bitovi.com/blog/github-projects-for-solo-developers)
- [Streamlined Markdown/Git Task Management](https://pankajpipada.com/posts/2024-08-13-taskmgmt-2/)

---

**Status**: Research complete, proceeding with Option 4 implementation
**Next Steps**: Create docs/BACKLOG.md and docs/LOG.md files
