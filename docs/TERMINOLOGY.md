# Project Terminology Reference

**Created**: December 27, 2025
**Purpose**: Clarify terminology used throughout the EFR32MG24 HAL project to avoid confusion

---

## Two-Level Development Structure

This project uses a **two-level hierarchy** for organizing development work:

### Level 1: Project Phases (Uppercase: A, B, C, D...)

**Project Phases** represent major milestones in the overall HAL development roadmap.

**Defined in**: `docs/PLAN.md`

**Naming**: Phase A, Phase B, Phase C, Phase D...

**Examples**:
- **Phase A**: Essential peripherals (GPIO, Clock, Delay)
- **Phase B**: Communication peripherals (USART, I2C, SPI, Timer)
- **Phase C**: Advanced peripherals (ADC, DMA, EMU, RTC, WDOG)
- **Phase D**: (Future) Radio, Security, Advanced features

**Scope**: Entire project roadmap spanning months

**Usage**:
```markdown
✅ Phase B complete - All communication peripherals implemented
⏳ Phase C in progress - 2 of 5 peripherals done
```

### Level 2: Module Implementation Stages (Stage 1, Stage 2...)

**Module Stages** represent incremental rollout of features within a single peripheral module.

**Defined in**: Individual module README.md files (e.g., `src/dma/README.md`)

**Naming**: Stage 1, Stage 2, Stage 3...

**Examples** (DMA module):
- **Stage 1**: Memory-to-memory transfers, single channel, blocking mode
- **Stage 2**: All 8 channels, peripheral transfers, linked descriptors, interrupts

**Scope**: Single module implementation spanning days/weeks

**Usage**:
```markdown
✅ DMA Stage 1 complete - Memory-to-memory transfers production-ready
⏳ DMA Stage 2 planned - Peripheral transfers and async support
```

---

## Why Two Levels?

### Project Phases Answer:
- "What major milestone is the project at?"
- "Which group of peripherals are we implementing?"
- "What's the overall project timeline?"

### Module Stages Answer:
- "Which features of this peripheral are complete?"
- "What functionality is available in this module today?"
- "What's planned for the next release of this module?"

---

## Clear Examples

### ✅ CORRECT Usage

```markdown
Phase C: DMA (LDMA) Stage 1 Complete
- Project Phase: C (Advanced Peripherals)
- Module Stage: 1 (Basic features)
```

```markdown
Phase B: Timer Stage 1 Production-Ready
- Project Phase: B (Communication)
- Module Stage: 1 (Core timer/PWM functionality)
```

### ❌ INCORRECT Usage (Confusing)

```markdown
Phase C: DMA Phase 1 Complete  ← WRONG: "Phase" used at both levels
Phase B Phase 1                ← WRONG: Ambiguous which phase is which
DMA Phase 1 of Phase C         ← WRONG: Redundant and confusing
```

---

## Reference Table

| Term | Level | Scope | Timeline | Example |
|------|-------|-------|----------|---------|
| **Phase A/B/C/D** | Project | Multiple peripherals | Months | Phase B: Communication peripherals |
| **Stage 1/2/3** | Module | Single peripheral | Days/Weeks | DMA Stage 1: Memory-to-memory |

---

## Documentation Standards

### In Module READMEs

Use "Stage" for implementation levels:

```markdown
# DMA (LDMA) Module

**Status**: Phase C - Production-Ready (Stage 1: Memory-to-Memory Transfers)

## Features

### Implemented (Stage 1)
- ✅ Memory-to-memory transfers
- ✅ Single channel (CH0)
- ✅ Blocking operation

### Planned (Stage 2)
- ⏳ All 8 channels
- ⏳ Peripheral-to-memory
- ⏳ Interrupt-driven
```

### In Project Docs (STATUS.md, PLAN.md)

Use "Phase" for project milestones:

```markdown
## Phase C Status

**PARTIAL** ⏳ - DMA (LDMA) Stage 1 complete, additional peripherals in progress

**Completed**:
- ✅ **ADC (IADC)**: Single-shot 12-bit conversion (December 19, 2025)
- ✅ **DMA (LDMA)**: Stage 1 - Memory-to-memory transfers (December 27, 2025)
```

### In Task Tracking (BACKLOG.md)

Use both, clearly distinguished:

```markdown
## Ready (Prioritized for Phase C)

- [ ] **EMU**: Energy Management Unit @priority(high) @phase(C)
  - Stage 1: Basic energy mode transitions
  - Stage 2: Advanced voltage scaling
```

```markdown
## Done (Recent)

- [x] **LDMA**: Implement DMA controller @done(2025-12-27) @phase(C)
  - Stage 1 complete: Memory-to-memory transfers
  - Production-ready for Stage 1 scope
```

---

## Commit Message Examples

### ✅ GOOD Commit Messages

```
feat(hal): Add DMA Stage 1 - Phase C

Implements memory-to-memory transfers for DMA controller.
This completes DMA Stage 1 as part of Phase C (Advanced Peripherals).
```

```
docs: Update Phase C status - DMA Stage 1 complete

Mark DMA Stage 1 as production-ready in Phase C documentation.
```

### ❌ BAD Commit Messages (Confusing)

```
feat(hal): Add DMA Phase 1 - Phase C    ← CONFUSING: Two "phases"
docs: Phase 1 complete                   ← AMBIGUOUS: Which phase?
Phase C Phase 1                          ← REDUNDANT
```

---

## Quick Decision Tree

**Question**: "Am I talking about..."

```
Multiple peripherals across months?
  → Use "Phase A/B/C/D"
  → Example: "Phase C includes ADC, DMA, EMU"

A single peripheral's rollout?
  → Use "Stage 1/2/3"
  → Example: "DMA Stage 1 is memory-to-memory only"

Both levels together?
  → Use "Phase X: Module Stage Y"
  → Example: "Phase C: DMA Stage 1 Complete"
```

---

## Historical Note

**Why the change?**

Originally, DMA documentation used "Phase 1" and "Phase 2" for module stages, which caused confusion with Project Phases (A, B, C...). On December 27, 2025, all documentation was updated to use "Stage 1/2" for module implementation levels to eliminate ambiguity.

**Files updated**:
- `efr32mg24-hal/src/dma/README.md`
- `docs/STATUS.md`
- `docs/BACKLOG.md`
- `.claude/skills/orchestrator/SKILL.md`

---

## Summary

- **Project Phases** (A, B, C, D) = Overall HAL development roadmap
- **Module Stages** (1, 2, 3) = Incremental feature rollout within a peripheral

**Always use**: "Phase C: DMA Stage 1 Complete" (not "Phase C: DMA Phase 1")

---

## Hierarchical Identifiers (December 27, 2025 Update)

To eliminate all ambiguity, we now use **unique hierarchical identifiers** for every work item:

### Identifier Format

**Format**: `[PHASE][STAGE]-[MODULE].[SUBSTAGE]`

**Examples**:
```
A1-GPIO        = Phase A, Stage 1: GPIO module
C2-DMA.1       = Phase C, Stage 2 (DMA), Substage 1 (memory-to-memory)
C3-EMU         = Phase C, Stage 3: Energy Management Unit
```

### Benefits

- ✅ **Globally unique** - No ambiguity
- ✅ **Self-documenting** - Shows hierarchy at a glance
- ✅ **Sortable** - Natural alphabetical ordering
- ✅ **Searchable** - Easy to grep in git logs

### Complete Reference

See [IDENTIFIER_SCHEME.md](IDENTIFIER_SCHEME.md) for complete specification, industry standards comparison, and usage guidelines.

### Quick Examples

**In commits**:
```
[C2-DMA.1] feat(hal): Add memory-to-memory DMA transfers
[C3-EMU] feat(hal): Implement energy mode transitions
```

**In documentation**:
```markdown
# GPIO Module (A1-GPIO)

**Identifier**: A1-GPIO
**Phase**: A (Essential Peripherals)
**Stage**: 1
**Status**: Complete
```

---

<!-- META: last_updated=2025-12-27 version=2.0.0 template-version=1.0.0 status=active maintained-by=orchestrator -->
