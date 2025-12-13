# EFR32MG24 Rust Project Documentation

This directory contains all documentation for the EFR32MG24 Rust PAC and HAL development project.

**Organization**: Documents are organized by topic for easy navigation and to establish single sources of truth.

---

## Quick Start

- **New to the project?** Start with [../README.md](../README.md) (main project README)
- **Current status?** See [STATUS.md](STATUS.md)
- **Want to contribute?** Check [PLAN.md](PLAN.md) for roadmap

---

## Core Documentation

### Project Management

- **[STATUS.md](STATUS.md)** - **Current project status** (Single Source of Truth)
  - What's completed, in progress, and planned
  - Current blockers and next steps
  - Handoff checklist for future sessions
  - Project metrics and statistics

- **[PLAN.md](PLAN.md)** - **Development roadmap**
  - 9-phase development plan with timelines
  - Milestones and deliverables
  - Progress tracking

- **[BACKLOG.md](BACKLOG.md)** - **Kanban task board** (Daily tactical planning)
  - Ready: Prioritized tasks for current sprint
  - In Progress: Active tasks
  - Review: Waiting for validation
  - Blocked: Cannot proceed
  - Done: Recent completions (last 30 days)
  - Ideas: Unprioritized future work
  - Task metadata: @priority, @phase, @started, @done, @blocked

- **[LOG.md](LOG.md)** - **Milestone achievement log**
  - Chronological record of significant milestones
  - Key decisions and their rationale
  - Technical achievements and metrics
  - Retrospective notes and lessons learned
  - Statistics summary (code, time, progress)

---

## Technical Documentation by Topic

### Hardware (`hardware/`)

- **[hardware/XIAO_MG24_HARDWARE.md](hardware/XIAO_MG24_HARDWARE.md)** - **Seeed Studio XIAO MG24 Sense** (SSOT)
  - Exact chip variant: EFR32MG24B220F1536IM48-B
  - Board specifications and features
  - Pinout and peripherals
  - Product lifecycle information

### PAC (`pac/`)

- **[pac/SVD_PACK_EXTRACTION.md](pac/SVD_PACK_EXTRACTION.md)** - **SVD file acquisition** (SSOT)
  - How to download Silicon Labs DFP pack
  - Complete extraction procedure for all 41 SVD files
  - File inventory and organization
  - Pack management and future updates

- **[pac/B220_VS_A020_COMPARISON.md](pac/B220_VS_A020_COMPARISON.md)** - **Current PAC analysis**
  - B220 vs A020 detailed comparison
  - Why B220 is 36% smaller
  - Compilation metrics
  - Migration recommendations

- **[pac/SVD_PROCESSING_HISTORY.md](pac/SVD_PROCESSING_HISTORY.md)** - Historical reference
  - Original A020 PAC generation story
  - svd2rust tooling issue and solution
  - Kept for historical context

### HAL (`hal/`)

- **[hal/HAL_STRUCTURE_PLAN.md](hal/HAL_STRUCTURE_PLAN.md)** - **HAL architecture** (SSOT)
  - Module structure and organization
  - embedded-hal v1.0 trait coverage
  - Implementation phases and timeline
  - Multi-device support strategy

---

## Archived Documentation (`archive/`)

Historical documents preserved for reference:

- **[archive/FINDINGS.md](archive/FINDINGS.md)** - Historical technical findings (3500+ lines)
  - Initial research and ecosystem analysis
  - Early challenges and solutions
  - Tooling compatibility investigations

- **[archive/PROJECT_SUMMARY.md](archive/PROJECT_SUMMARY.md)** - Historical project summary
  - Complete project overview from earlier phase
  - Now superseded by STATUS.md

- **[archive/EFR32MG24_RESEARCH_SUMMARY.md](archive/EFR32MG24_RESEARCH_SUMMARY.md)** - Initial research
  - Market analysis
  - Existing Rust support evaluation
  - Technical specifications research

---

## Single Source of Truth (SSOT) Index

To avoid duplicate information, each topic has one authoritative document:

| Topic | Master Document | Purpose |
|-------|----------------|---------|
| **XIAO MG24 Hardware** | [hardware/XIAO_MG24_HARDWARE.md](hardware/XIAO_MG24_HARDWARE.md) | All hardware specs, pinouts, features |
| **SVD File Acquisition** | [pac/SVD_PACK_EXTRACTION.md](pac/SVD_PACK_EXTRACTION.md) | How to obtain and extract SVD files |
| **Current PAC** | [pac/B220_VS_A020_COMPARISON.md](pac/B220_VS_A020_COMPARISON.md) | B220 PAC details and comparison |
| **Project Status** | [STATUS.md](STATUS.md) | What's done, what's next, blockers |
| **Development Roadmap** | [PLAN.md](PLAN.md) | Phases, timeline, milestones |
| **Task Backlog** | [BACKLOG.md](BACKLOG.md) | Kanban board, daily tasks, priorities |
| **Milestone Log** | [LOG.md](LOG.md) | Achievement history, decisions, retrospectives |
| **HAL Architecture** | [hal/HAL_STRUCTURE_PLAN.md](hal/HAL_STRUCTURE_PLAN.md) | HAL structure, modules, implementation |

**Principle**: If information appears in the SSOT document, other documents should **link to it** rather than duplicate it.

---

## Document Relationships

```
README.md (you are here)
├── STATUS.md → Current state (what's done/in progress)
├── PLAN.md → Strategic roadmap (phases, milestones)
├── BACKLOG.md → Tactical tasks (daily Kanban board)
├── LOG.md → Milestone history (achievements, decisions)
│
├── hardware/
│   └── XIAO_MG24_HARDWARE.md → Hardware specs (SSOT)
│
├── pac/
│   ├── SVD_PACK_EXTRACTION.md → SVD acquisition (SSOT)
│   ├── B220_VS_A020_COMPARISON.md → Current PAC (SSOT)
│   └── SVD_PROCESSING_HISTORY.md → Historical A020 story
│
├── hal/
│   └── HAL_STRUCTURE_PLAN.md → HAL architecture (SSOT)
│
└── archive/ → Historical reference only
    ├── FINDINGS.md
    ├── PROJECT_SUMMARY.md
    └── EFR32MG24_RESEARCH_SUMMARY.md
```

---

## Quick Reference

### PAC Generation (Current - B220)

```bash
# For Seeed Studio XIAO MG24 Sense
cd efr32mg24-pac
svd2rust -i svd/EFR32MG24B220F1536IM48.svd \
         --target cortex-m \
         -o src

# Result: 138,448 lines, ~2.6 minute compile time
```

See [pac/B220_VS_A020_COMPARISON.md](pac/B220_VS_A020_COMPARISON.md) for details.

### Build Status

| Component | Status | Details |
|-----------|--------|---------|
| **PAC (B220)** | ✅ Generated | 138,448 lines, compiles in 2.6 min |
| **HAL Tier 1** | ✅ Complete | GPIO, CMU, Delay with hardware register access |
| **HAL Tier 2** | 🚧 In Progress | USART ✅, I2C/SPI/Timers next |
| **Examples** | ✅ Building | 5 examples, all compile to ARM binaries |

### Current Phase

**Phase B - Communication Peripherals (70% complete)**: USART, I2C, SPI done; Timers next (December 13, 2025)

See [PLAN.md](PLAN.md) for strategic roadmap, [STATUS.md](STATUS.md) for implementation status, and [BACKLOG.md](BACKLOG.md) for daily task board.

---

## Where to Document What

Use this guide to know where specific information should be written:

### Hardware Information → `hardware/`

**Write in [hardware/XIAO_MG24_HARDWARE.md](hardware/XIAO_MG24_HARDWARE.md)**:
- Chip specifications (part numbers, memory, peripherals)
- Board features (sensors, connectors, power)
- Pinout and GPIO mappings
- Product lifecycle and availability
- Electrical characteristics

**Future**: Create separate files for other boards (e.g., `hardware/EFR32MG24_DK.md`)

### SVD & PAC Information → `pac/`

**Write in [pac/SVD_PACK_EXTRACTION.md](pac/SVD_PACK_EXTRACTION.md)**:
- How to download SVD files
- Pack extraction procedures
- SVD file inventory and naming
- Pack version updates

**Write in [pac/B220_VS_A020_COMPARISON.md](pac/B220_VS_A020_COMPARISON.md)**:
- Current PAC generation status
- PAC metrics (size, compile time, peripherals)
- Variant comparisons
- Migration guidance

**Write in [pac/SVD_PROCESSING_HISTORY.md](pac/SVD_PROCESSING_HISTORY.md)** (historical only):
- Historical A020 PAC generation story
- Tooling issues that were resolved
- Do not update - kept for reference only

### HAL Information → `hal/`

**Write in [hal/HAL_STRUCTURE_PLAN.md](hal/HAL_STRUCTURE_PLAN.md)**:
- HAL module architecture
- embedded-hal trait implementations
- Implementation phases and timeline
- API design decisions
- Module dependencies

**Future**: Add `hal/MODULE_NAME.md` for complex modules (e.g., `hal/DMA.md`, `hal/RADIO.md`)

### Project Management → Root `docs/`

**Write in [STATUS.md](STATUS.md)** (update after milestones):
- Current phase and overall progress
- Completed milestones
- Implementation metrics
- Handoff checklist
- Overall project status

**Write in [PLAN.md](PLAN.md)** (update when strategy changes):
- Development phases (strategic)
- Timeline estimates
- Milestone definitions
- Resource requirements

**Write in [BACKLOG.md](BACKLOG.md)** (update daily/weekly):
- Tasks ready to start (prioritized)
- Tasks in progress (actively working)
- Tasks in review (waiting for validation)
- Blocked tasks (cannot proceed)
- Recent completions (last 30 days)
- Future ideas (unprioritized)

**Write in [LOG.md](LOG.md)** (append after significant events):
- Milestone achievements with dates
- Key technical decisions and rationale
- Statistics and metrics snapshots
- Retrospective notes (what went well, challenges, lessons)
- Never edit historical entries - append only

### Historical Information → `archive/`

**Do NOT add new content to archive** - these are historical references only

---

## Contributing to Documentation

### Before Writing

1. **Backup First** - ALWAYS create timestamped backup in `.archive/` before modifying
2. **Check SSOT Index** - Is there already a master document for this topic?
3. **Find Correct Location** - Use "Where to Document What" guide above
4. **Review Existing Content** - Read the target document first
5. **Avoid Duplication** - Link to SSOT rather than copying content

### Documentation Backup Workflow

**CRITICAL REQUIREMENT**: Before modifying any documentation file, create a timestamped backup:

```bash
# Backup any documentation file before modification
cp docs/STATUS.md .archive/STATUS_$(date +%Y%m%d_%H%M%S).md
# Then proceed with modifications
```

**What gets archived**:
- Planning documents (PLAN.md, STATUS.md)
- Module documentation (README.md files when doing major rewrites)
- Project documentation (CLAUDE.md, README.md, docs/README.md)
- Changelogs (CHANGELOG.md)

**Archive location**: `.archive/` folder at project root (not in docs/)
**Retention policy**: Archives kept indefinitely for history
**Purpose**: Quick rollback capability without git operations
**See also**: `.archive/README.md` for complete archive documentation

### When Adding New Documents

1. **Choose appropriate folder**: hardware/, pac/, hal/, or root
2. **Follow template** (see below)
3. **Add to this README** index
4. **Cross-reference** related documents
5. **Update SSOT Index** if creating a new master document

### Document Templates

All documents should include:
```markdown
# Document Title

**Date**: YYYY-MM-DD
**Status**: Active | Historical | Deprecated
**SSOT**: Yes/No (is this the single source of truth?)

---

## Overview
... your content ...

---

**Last Updated**: YYYY-MM-DD
**Version**: X.X
```

---

## Document History

- **December 13, 2025**: Added BACKLOG.md and LOG.md for project management
- **December 4, 2025**: Updated for Phase 5 Tier 1 completion - Added backup workflow documentation
- **December 3, 2025**: Major reorganization - created topic-based folders
- **December 3, 2025**: Added B220 PAC documentation
- **December 3, 2025**: Archived historical documents
- **December 2-3, 2025**: Initial documentation creation

---

## Need Help?

- **Project questions**: See [STATUS.md](STATUS.md) or main [../README.md](../README.md)
- **Current tasks**: [BACKLOG.md](BACKLOG.md) (Kanban board)
- **Milestone history**: [LOG.md](LOG.md) (achievement log)
- **Hardware details**: [hardware/XIAO_MG24_HARDWARE.md](hardware/XIAO_MG24_HARDWARE.md)
- **PAC issues**: [pac/](pac/) folder
- **HAL status**: [../efr32mg24-hal/docs/STATUS.md](../efr32mg24-hal/docs/STATUS.md)
- **HAL planning**: [hal/HAL_STRUCTURE_PLAN.md](hal/HAL_STRUCTURE_PLAN.md)
- **Backup workflow**: [../.archive/README.md](../.archive/README.md)

---

**Last Updated**: December 13, 2025
**Total Documents**: 16+ (docs folder + HAL docs + module READMEs + project management)
