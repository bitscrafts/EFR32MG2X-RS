# Hierarchical Identifier Scheme

**Created**: December 27, 2025
**Purpose**: Define a structured, hierarchical naming convention for tracking project work

---

## Overview

This project uses a **hierarchical identifier system** inspired by industry best practices:
- [Work Breakdown Structure (WBS)](https://www.numberanalytics.com/blog/mastering-project-planning-in-embedded-systems) from embedded systems project management
- [Agile Epic-Story-Task hierarchy](https://www.atlassian.com/agile/project-management/epics-stories-themes) from software development
- [Aerospace part numbering schemes](https://www.eng-tips.com/threads/part-numbering-conventions-whats-the-best-practice-in-aerospace.565772/) for embedded systems

**Format**: `[PHASE][STAGE]-[MODULE]`

**Benefits**:
- ✅ Unique identifier for every work item
- ✅ Self-documenting and sortable
- ✅ Easy to reference in commits, issues, docs
- ✅ Shows hierarchy and dependencies at a glance
- ✅ Scalable as project grows

---

## Identifier Structure

### Level 1: Phase-Stage Identifier

**Format**: `[LETTER][NUMBER]`

- **LETTER** (A-Z): Project Phase (major milestone)
- **NUMBER** (1-N): Sequential stage within that phase

**Examples**:
```
A1 = Phase A, Stage 1
A2 = Phase A, Stage 2
B1 = Phase B, Stage 1
C3 = Phase C, Stage 3
```

### Level 2: Module Identifier

**Format**: `[PHASE][STAGE]-[MODULE]`

- **[PHASE][STAGE]**: From Level 1
- **[MODULE]**: Peripheral or component name (uppercase)

**Examples**:
```
A1-GPIO   = Phase A, Stage 1: GPIO peripheral
A2-CMU    = Phase A, Stage 2: Clock Management Unit
B1-USART  = Phase B, Stage 1: USART serial communication
C2-DMA    = Phase C, Stage 2: Direct Memory Access
C3-EMU    = Phase C, Stage 3: Energy Management Unit
```

### Level 3: Implementation Sub-Stage (Optional)

**Format**: `[PHASE][STAGE]-[MODULE].[SUBSTAGE]`

- **[PHASE][STAGE]-[MODULE]**: From Level 2
- **[SUBSTAGE]**: Decimal sub-stage number (1, 2, 3...)

**Examples**:
```
C2-DMA.1  = Phase C, Stage 2, DMA, Sub-stage 1 (memory-to-memory)
C2-DMA.2  = Phase C, Stage 2, DMA, Sub-stage 2 (peripheral transfers)
C2-DMA.3  = Phase C, Stage 2, DMA, Sub-stage 3 (interrupt-driven)
```

---

## Current Project Mapping

### Phase A: Essential Peripherals

| Identifier | Module | Description | Status |
|------------|--------|-------------|--------|
| **A1-GPIO** | GPIO | Digital I/O pins | ✅ Complete |
| **A2-CMU** | Clock | Clock management | ✅ Complete |
| **A3-Delay** | Delay | SysTick delays | ✅ Complete |

**Phase A Status**: 3/3 complete (100%)

### Phase B: Communication Peripherals

| Identifier | Module | Description | Status |
|------------|--------|-------------|--------|
| **B1-USART** | USART | Serial communication | ✅ Complete |
| **B2-I2C** | I2C | I2C master mode | ✅ Complete |
| **B3-SPI** | SPI | SPI master mode | ✅ Complete |
| **B4-Timer** | Timer | Timers and PWM | ✅ Complete |

**Phase B Status**: 4/4 complete (100%)

### Phase C: Advanced Peripherals

| Identifier | Module | Description | Status |
|------------|--------|-------------|--------|
| **C1-ADC** | IADC | 12-bit ADC | ✅ Complete |
| **C2-DMA** | LDMA | DMA controller | ✅ C2-DMA.1 Complete |
| **C2-DMA.1** | LDMA | Memory-to-memory | ✅ Complete |
| **C2-DMA.2** | LDMA | Peripheral transfers | ⏳ Planned |
| **C2-DMA.3** | LDMA | Linked descriptors | ⏳ Planned |
| **C2-DMA.4** | LDMA | Interrupt-driven | ⏳ Planned |
| **C3-EMU** | EMU | Energy management | ⏳ Pending |
| **C4-RTC** | RTCC | Real-time clock | ⏳ Pending |
| **C5-WDOG** | WDOG | Watchdog timer | ⏳ Pending |

**Phase C Status**: 2/5 peripherals complete, C2-DMA.1/4 sub-stages complete (40%)

### Phase D: Radio & Security (Future)

| Identifier | Module | Description | Status |
|------------|--------|-------------|--------|
| **D1-Radio** | Radio | 2.4 GHz radio | ⏳ Planned |
| **D2-AES** | Crypto | AES acceleration | ⏳ Planned |
| **D3-Vault** | Security | Secure Vault | ⏳ Planned |

**Phase D Status**: Not started (0%)

---

## Usage Guidelines

### In Documentation

**File Headers**:
```markdown
# GPIO Module (A1-GPIO)

**Identifier**: A1-GPIO
**Phase**: A (Essential Peripherals)
**Stage**: 1
**Status**: Complete
```

**Status Updates**:
```markdown
## Phase C Progress

- ✅ C1-ADC: Complete (12-bit ADC)
- ✅ C2-DMA.1: Complete (memory-to-memory transfers)
- ⏳ C2-DMA.2: Planned (peripheral transfers)
- ⏳ C3-EMU: Pending (energy management)
```

### In Task Tracking (BACKLOG.md)

```markdown
## Ready

- [ ] **C3-EMU**: Energy Management Unit @priority(high) @phase(C) @stage(3)
  - Energy mode transitions (EM0-EM4)
  - Voltage scaling

## In Progress

- [ ] **C2-DMA.2**: DMA peripheral transfers @started(2025-12-27) @phase(C) @stage(2.2)

## Done

- [x] **C2-DMA.1**: DMA memory-to-memory @done(2025-12-27) @phase(C) @stage(2.1)
- [x] **C1-ADC**: 12-bit ADC implementation @done(2025-12-19) @phase(C) @stage(1)
```

### In Commit Messages

**Format**: `[IDENTIFIER] type(scope): description`

**Examples**:
```
[C2-DMA.1] feat(hal): Add memory-to-memory DMA transfers

Implements Phase C, Stage 2, DMA sub-stage 1 with blocking mode.
- Type-safe channels with const generics
- TransferElement trait for compile-time size selection
- Production-ready for Stage 1 scope
```

```
[C3-EMU] feat(hal): Implement energy mode transitions

Phase C, Stage 3: Energy Management Unit implementation.
- EM0-EM4 mode transitions
- Voltage scaling support
- Low-power operation
```

```
[A1-GPIO,A2-CMU] docs: Update Phase A documentation

Update documentation for GPIO (A1) and CMU (A2) modules.
```

### In Code Comments

```rust
/// GPIO Peripheral (A1-GPIO)
///
/// Phase A, Stage 1: Essential digital I/O functionality
/// Status: Production-ready
pub mod gpio;

/// DMA Controller (C2-DMA)
///
/// Phase C, Stage 2: Direct Memory Access
/// - C2-DMA.1: Memory-to-memory (Complete)
/// - C2-DMA.2: Peripheral transfers (Planned)
/// - C2-DMA.3: Linked descriptors (Planned)
pub mod dma;
```

### In Examples

```rust
//! GPIO Example (A1-GPIO)
//!
//! **Identifier**: A1-GPIO
//! **Phase**: A (Essential Peripherals)
//! **Stage**: 1
//!
//! Demonstrates digital I/O operations...
```

---

## Comparison to Industry Standards

### 1. WBS (Work Breakdown Structure) - Project Management Standard

**WBS uses decimal notation** for hierarchical task decomposition:

| WBS Level | Notation | Our Equivalent | Example |
|-----------|----------|----------------|---------|
| **Level 1** | 1 | Phase | A (Essential Peripherals) |
| **Level 2** | 1.1 | Stage | A1 (GPIO) |
| **Level 3** | 1.1.1 | Sub-stage | A1.1 (if needed) |

**Standard**: [WBS Numbering](https://kirkwood.pressbooks.pub/projectmanagementbasics/chapter/wbs-numbering/) - Hierarchical decimal system (1, 1.1, 1.1.1...)

**Best Practice**: Keep hierarchy to 2-4 levels for most projects ([Microsoft Dynamics 365](https://learn.microsoft.com/en-us/dynamics365/project-operations/prod-pma/work-breakdown-structures))

**Our Implementation**:
```
EFR32MG24-HAL (Project)
├─ A (Phase A: Essential)
│   ├─ A1-GPIO
│   ├─ A2-CMU
│   └─ A3-Delay
├─ C (Phase C: Advanced)
│   ├─ C1-ADC
│   ├─ C2-DMA
│   │   ├─ C2-DMA.1 (Memory-to-memory)
│   │   ├─ C2-DMA.2 (Peripheral transfers)
│   │   └─ C2-DMA.3 (Linked descriptors)
│   └─ C3-EMU
```

### 2. JIRA Hierarchy - Agile Project Management

**JIRA uses Initiative → Epic → Story → Task** for large projects:

| JIRA Level | Our Equivalent | Example |
|------------|----------------|---------|
| **Initiative** | Project | EFR32MG24 HAL Development |
| **Epic** | Phase | Phase C: Advanced Peripherals |
| **Story** | Stage | C2-DMA: DMA Controller |
| **Task** | Sub-stage | C2-DMA.1: Memory-to-memory transfers |

**Reference**: [JIRA Epic vs Story vs Task](https://www.upscale.tech/blog/epic-story-task-hierarchy-in-jira)

**Sizing Guidelines** ([TitanApps](https://titanapps.io/blog/epic-vs-story-vs-task/)):
- Task: Minutes
- Story: Hours, delivers user value
- Epic: Days/Weeks, multiple stories
- Initiative: Months, strategic objectives

**Best Practice**: Limit to 3-4 levels for manageability ([Seibert Group](https://seibert.group/products/blog/jira-story-vs-task-vs-epic/))

### 3. Military Operations - Hierarchical Naming

**Military uses prefix-based hierarchical codes**:

| Level | System | Example |
|-------|--------|---------|
| **Command** | 2-letter prefix | OA-OF (Africa Command) |
| **Operation** | Prefix + Name | OD-Odyssey (Libya) |
| **Organization** | Hierarchical code | DN11 (sub-org), DN (Navy), D (DoD) |

**Reference**: [Pentagon Operation Naming](https://www.govtech.com/em/disaster/how-pentagon-names-military-operations.html)

**NICKA System**: Validates and prevents duplication of operational names ([Mental Floss](https://www.mentalfloss.com/article/28711/how-military-operations-get-their-code-names))

**Our Implementation**: Phase letters (A-Z) act as command prefixes, stages provide uniqueness

### 4. Agile Theme → Epic → Story → Task

Our scheme maps to standard Agile hierarchy:

| Agile Level | Our Equivalent | Example |
|-------------|----------------|---------|
| **Theme** | Project | EFR32MG24 HAL Development |
| **Epic** | Phase | Phase C: Advanced Peripherals |
| **Story** | Stage | C2-DMA: DMA Controller |
| **Task** | Sub-stage | C2-DMA.1: Memory-to-memory transfers |

**Reference**: [Atlassian Agile Epics](https://www.atlassian.com/agile/project-management/epics-stories-themes)

### 5. Summary Comparison Table

| Standard | Hierarchy Levels | Notation | Strengths | Our Adoption |
|----------|------------------|----------|-----------|--------------|
| **WBS** | 1, 1.1, 1.1.1 | Decimal | Industry standard, unlimited depth | 2-3 levels (A1, A1.1) |
| **JIRA** | Initiative/Epic/Story/Task | Alphanumeric | Agile-friendly, time-based | Maps directly to our phases/stages |
| **Military** | Command/Operation | Prefix-code | Unique, validated system | Phase letters as command prefixes |
| **Semantic Ver** | Major.Minor.Patch | Decimal | Clear versioning | Aligns with release strategy |

**Key Sources**:
- [WBS Numbering](https://kirkwood.pressbooks.pub/projectmanagementbasics/chapter/wbs-numbering/)
- [JIRA Hierarchy](https://www.upscale.tech/blog/epic-story-task-hierarchy-in-jira)
- [Pentagon Operations](https://www.govtech.com/em/disaster/how-pentagon-names-military-operations.html)
- [Embassy HAL](https://embassy.dev/blog/embassy-hals-released/)

### Semantic Versioning (Rust Ecosystem)

Rust embedded HALs use semantic versioning for releases:

| Version Type | Our Equivalent | Example |
|--------------|----------------|---------|
| **Major** | Phase | Phase C = 0.3.x |
| **Minor** | Stage | C2-DMA = 0.3.2 |
| **Patch** | Sub-stage | C2-DMA.1 = 0.3.2.1 |

**Reference**: [Embassy HAL Releases](https://embassy.dev/blog/embassy-hals-released/)

---

## Sorting and Organization

### Alphabetical Sorting

Identifiers sort naturally:
```
A1-GPIO
A2-CMU
A3-Delay
B1-USART
B2-I2C
B3-SPI
B4-Timer
C1-ADC
C2-DMA
C2-DMA.1
C2-DMA.2
C3-EMU
C4-RTC
C5-WDOG
```

### By Status (Kanban)

```markdown
## Completed
- ✅ A1-GPIO, A2-CMU, A3-Delay (Phase A: 100%)
- ✅ B1-USART, B2-I2C, B3-SPI, B4-Timer (Phase B: 100%)
- ✅ C1-ADC, C2-DMA.1 (Phase C: 40%)

## In Progress
- ⏳ C2-DMA.2 (Peripheral DMA transfers)

## Planned
- 📋 C2-DMA.3, C2-DMA.4 (DMA enhancements)
- 📋 C3-EMU, C4-RTC, C5-WDOG (Phase C remaining)
```

### By Phase

```markdown
**Phase A**: 3/3 complete (100%) - A1, A2, A3
**Phase B**: 4/4 complete (100%) - B1, B2, B3, B4
**Phase C**: 2/5 in progress (40%) - C1✅, C2.1✅, C2.2-4⏳, C3-5⏳
**Phase D**: Not started (0%)
```

---

## Migration from Current System

### Old → New Mapping

| Old Notation | New Identifier | Notes |
|--------------|----------------|-------|
| Phase A: GPIO | A1-GPIO | Stage 1 of Phase A |
| Phase B: USART | B1-USART | Stage 1 of Phase B |
| Phase C: DMA Stage 1 | C2-DMA.1 | DMA is Stage 2, Sub-stage 1 |
| Phase C: DMA Stage 2 | C2-DMA.2 | DMA is Stage 2, Sub-stage 2 |
| Phase C: EMU | C3-EMU | Stage 3 of Phase C |

### Transition Plan

1. **Update TERMINOLOGY.md** with new identifier scheme
2. **Update BACKLOG.md** to use identifiers in all tasks
3. **Update STATUS.md** with identifier-based progress tracking
4. **Update PLAN.md** roadmap with identifier milestones
5. **Update module READMEs** to include identifier in header
6. **Update skills** to enforce identifier usage
7. **Future commits** use identifier prefix

---

## Benefits Summary

### ✅ Unique Identification
Every work item has a globally unique identifier:
- No ambiguity about which "DMA Stage 1" you're referring to
- Clear distinction between C2-DMA.1 (implementation stage) and C2 (peripheral stage)

### ✅ Self-Documenting
Identifier tells you everything at a glance:
- **C2-DMA.1** = Phase C, Stage 2 (DMA), Sub-stage 1 (first implementation)
- **A3-Delay** = Phase A, Stage 3 (Delay module)

### ✅ Sortable and Searchable
- Alphabetical sorting works naturally (A1, A2, B1, C1...)
- Easy to grep: `git log --grep="C2-DMA"`
- Filter by phase: `grep "^C[0-9]-" docs/BACKLOG.md`

### ✅ Hierarchical Dependencies
Sub-stages show progressive implementation:
```
C2-DMA (parent)
├─ C2-DMA.1 (memory-to-memory)
├─ C2-DMA.2 (peripheral transfers) ← depends on .1
└─ C2-DMA.3 (linked descriptors)  ← depends on .2
```

### ✅ Scalable
- Phase letters: A-Z (26 phases)
- Stages per phase: 1-99 (99 stages)
- Sub-stages: Unlimited decimal levels (1.1.1.1...)

### ✅ Industry Standard
Aligns with:
- Agile Epic-Story-Task hierarchy
- Work Breakdown Structure (WBS)
- Semantic versioning concepts
- Aerospace part numbering schemes

---

## Enforcement

### In Skills

**project-manager skill** will enforce:
- All tasks must have `@identifier(XX-YYY)` tag
- Identifier format validation
- No duplicate identifiers

**orchestrator skill** will enforce:
- Commit messages start with `[IDENTIFIER]`
- Documentation includes identifier in headers
- Proper identifier progression (don't skip stages)

### In Documentation

**Standard header format**:
```markdown
# Module Name (IDENTIFIER)

**Identifier**: XX-YYY
**Phase**: X (Phase Name)
**Stage**: Y
**Status**: Complete/In Progress/Planned
```

### In Commits

**Required format**:
```
[IDENTIFIER] type(scope): description

Detailed explanation...
```

**Examples**:
```
[C2-DMA.1] feat(hal): Implement memory-to-memory DMA
[C3-EMU] feat(hal): Add energy mode transitions
[B4-Timer] fix(hal): Correct PWM frequency calculation
[A1-GPIO,A2-CMU] docs: Update Phase A documentation
```

---

## Quick Reference

### Identifier Format
```
[PHASE][STAGE]-[MODULE]           # Basic
[PHASE][STAGE]-[MODULE].[SUBSTAGE] # With sub-stage
```

### Examples
```
A1-GPIO        # Phase A, Stage 1: GPIO
B4-Timer       # Phase B, Stage 4: Timer
C2-DMA         # Phase C, Stage 2: DMA (all sub-stages)
C2-DMA.1       # Phase C, Stage 2, DMA, Sub-stage 1
C3-EMU         # Phase C, Stage 3: EMU
```

### Current Status
```
Phase A: 3/3 ✅ (A1, A2, A3)
Phase B: 4/4 ✅ (B1, B2, B3, B4)
Phase C: 2/5 ⏳ (C1✅, C2.1✅, C2.2-4⏳, C3-5⏳)
Phase D: 0/X 📋
```

---

## Sources

- [Mastering Project Planning in Embedded Systems](https://www.numberanalytics.com/blog/mastering-project-planning-in-embedded-systems) - WBS methodology
- [Atlassian: Epics, Stories, and Initiatives](https://www.atlassian.com/agile/project-management/epics-stories-themes) - Agile hierarchy
- [Aerospace Part Numbering Best Practices](https://www.eng-tips.com/threads/part-numbering-conventions-whats-the-best-practice-in-aerospace.565772/) - Hierarchical naming
- [Embassy HAL Releases](https://embassy.dev/blog/embassy-hals-released/) - Rust embedded versioning

---

<!-- META: last_updated=2025-12-27 version=1.0.0 template-version=1.0.0 status=active maintained-by=orchestrator -->
