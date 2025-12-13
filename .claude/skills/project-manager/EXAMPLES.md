# Project Manager Skill - Usage Examples

This document shows concrete examples of using the project-manager skill to manage tasks in docs/BACKLOG.md and docs/LOG.md.

---

## Example 1: Adding a New Task

**User request**: "Add task to implement LETIMER peripheral for low-power timing"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Determine: This is a Phase C task (advanced features), medium priority
3. Add to "Ideas (Unprioritized)" section since Phase B is current focus

**Before** (docs/BACKLOG.md):
```markdown
## Ideas (Unprioritized Future Work)

### Phase C Candidates
- [ ] **EMU**: Energy Management Unit
  - Energy mode transitions (EM0-EM4)
```

**After** (docs/BACKLOG.md):
```markdown
## Ideas (Unprioritized Future Work)

### Phase C Candidates
- [ ] **EMU**: Energy Management Unit
  - Energy mode transitions (EM0-EM4)

- [ ] **LETIMER**: Low-Energy Timer @priority(medium) @phase(C)
  - Real-time clock functionality
  - Wakeup from low-power modes
  - Integration with EMU for EM1-EM4
```

**Response**: "Added LETIMER task to Ideas section (Phase C candidate). Currently in Ideas since Phase B is active. Will be prioritized after Phase B completion."

---

## Example 2: Starting Work on a Task

**User request**: "Start working on TIMER peripheral"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Find TIMER task in "Ready (Prioritized)" section
3. Move to "In Progress"
4. Add @started metadata with current date

**Before** (docs/BACKLOG.md):
```markdown
## Ready (Prioritized for Phase B)

### High Priority
- [ ] **TIMER**: Implement TIMER0 basic functionality @priority(high) @phase(B)
  - Register access (CTRL, CMD, CNT, TOP, IEN, IF)
  - Counter modes (up, down, up-down)

## In Progress

Currently no active tasks (documentation cleanup just completed).
```

**After** (docs/BACKLOG.md):
```markdown
## Ready (Prioritized for Phase B)

### High Priority
- [ ] **TIMER**: Implement TIMER1-4 basic functionality @priority(high) @phase(B)
  - Replicate TIMER0 implementation for TIMER1-4

## In Progress

- [ ] **TIMER**: Implement TIMER0 basic functionality @started(2025-12-13) @priority(high) @phase(B) @assignee(orchestrator)
  - Register access (CTRL, CMD, CNT, TOP, IEN, IF)
  - Counter modes (up, down, up-down)
  - Clock prescaler configuration
  - Interrupt support
```

**Response**: "Started TIMER task. Moved to In Progress column with @started(2025-12-13). Task is now active."

---

## Example 3: Completing a Task

**User request**: "TIMER implementation complete - 312 lines of code, all tests passing"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Find TIMER task in "In Progress"
3. Move to "Done (Recent)"
4. Mark checkbox [x], add @done metadata
5. Include completion notes

**Before** (docs/BACKLOG.md):
```markdown
## In Progress

- [ ] **TIMER**: Implement TIMER0 basic functionality @started(2025-12-13) @priority(high) @phase(B) @assignee(orchestrator)
  - Register access (CTRL, CMD, CNT, TOP, IEN, IF)
  - Counter modes (up, down, up-down)

## Done (Recent - Last 30 Days)

### Week of December 9-13, 2025

- [x] **Phase B**: I2C master mode implementation @done(2025-12-12)
```

**After** (docs/BACKLOG.md):
```markdown
## In Progress

Currently no active tasks.

## Done (Recent - Last 30 Days)

### Week of December 13-19, 2025

- [x] **TIMER**: Implement TIMER0 basic functionality @done(2025-12-15) @started(2025-12-13) @phase(B)
  - Register access (CTRL, CMD, CNT, TOP, IEN, IF)
  - Counter modes (up, down, up-down)
  - Clock prescaler configuration
  - Interrupt support
  - **Metrics**: 312 lines of code, all tests passing

### Week of December 9-13, 2025

- [x] **Phase B**: I2C master mode implementation @done(2025-12-12)
```

**Response**: "TIMER task completed! Marked as done with @done(2025-12-15). Added metrics (312 LOC, tests passing). Task moved to Done column."

---

## Example 4: Blocking a Task

**User request**: "Can't proceed with PWM implementation - need TIMER done first"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Find PWM task in "Ready" or "In Progress"
3. Move to "Blocked" column
4. Add @blocked metadata with reason

**Before** (docs/BACKLOG.md):
```markdown
## Ready (Prioritized for Phase B)

### High Priority
- [ ] **PWM**: Add PWM generation support @priority(high) @phase(B)
  - CC (Capture/Compare) channel configuration

## Blocked

- [ ] **Phase C**: Embassy async support @blocked(Waiting for Phase B completion)
```

**After** (docs/BACKLOG.md):
```markdown
## Ready (Prioritized for Phase B)

### High Priority
(moved to Blocked)

## Blocked

- [ ] **PWM**: Add PWM generation support @blocked(Requires TIMER0 implementation) @priority(high) @phase(B)
  - CC (Capture/Compare) channel configuration
  - Duty cycle control
  - Frequency configuration
  - **Blocker**: TIMER peripheral must be complete first

- [ ] **Phase C**: Embassy async support @blocked(Waiting for Phase B completion)
```

**Response**: "PWM task blocked. Moved to Blocked column with reason: Requires TIMER0 implementation. Will be unblocked when TIMER is complete."

---

## Example 5: Unblocking a Task

**User request**: "TIMER is done, unblock PWM task"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Find PWM task in "Blocked"
3. Remove @blocked metadata
4. Move back to "Ready (Prioritized)"
5. Adjust priority if needed

**Before** (docs/BACKLOG.md):
```markdown
## Ready (Prioritized for Phase B)

### High Priority
(empty)

## Blocked

- [ ] **PWM**: Add PWM generation support @blocked(Requires TIMER0 implementation) @priority(high) @phase(B)
```

**After** (docs/BACKLOG.md):
```markdown
## Ready (Prioritized for Phase B)

### High Priority
- [ ] **PWM**: Add PWM generation support @priority(high) @phase(B)
  - CC (Capture/Compare) channel configuration
  - Duty cycle control
  - Frequency configuration
  - embedded-hal PWM traits

## Blocked

(blocker removed)
```

**Response**: "PWM task unblocked! Blocker removed and task moved back to Ready (High Priority). Ready to start implementation."

---

## Example 6: Logging a Milestone

**User request**: "Log Phase B completion milestone - all communication peripherals done"

**Skill actions**:
1. Read docs/LOG.md
2. Create milestone entry with current date
3. Include achievements, impact, metrics, retrospectives
4. Prepend to LOG.md (most recent first)

**Before** (docs/LOG.md):
```markdown
# EFR32MG24 HAL Development - Milestone Log

---

## 2025-12-13: Documentation Cleanup & Project Management System

**Milestone**: Documentation standardization and project tracking infrastructure
```

**After** (docs/LOG.md - prepended):
```markdown
# EFR32MG24 HAL Development - Milestone Log

---

## 2025-12-15: Phase B Complete - Communication Peripherals

**Milestone**: All Phase B communication peripherals implemented and tested

**Achievements**:
- Implemented TIMER0-4 basic functionality (counter modes, prescaler, interrupts)
- Added PWM generation support (CC channels, duty cycle, frequency control)
- Created examples 06_timer.rs and 07_pwm.rs
- All 7 examples compile and build successfully
- Module READMEs updated with Phase B status

**Impact**:
- Phase B now 100% complete (USART, I2C, SPI, TIMER, PWM all done)
- Ready for Phase C (advanced peripherals: DMA, ADC, low-power modes)
- Solid foundation for real-time control and signal generation
- All basic embedded use cases now supported

**Metrics**:
- TIMER module: 312 lines of code
- PWM support: 187 lines (integrated into TIMER)
- Total HAL code: ~2,430 lines
- Examples: 7 working examples
- Build time: ~3.5 minutes (release mode)
- Time investment Phase B: ~12 hours total

**Technical Highlights**:
- Hardware timer register manipulation for precise control
- PWM frequency calculation: `freq = HCLK / ((prescaler + 1) * (TOP + 1))`
- Zero-cost abstraction for timer modes using type-state pattern
- embedded-hal PWM trait implementation

**Retrospective**:
- **What went well**: Consistent register access patterns from Phase A made implementation faster
- **Challenges**: Timer interrupt handling required careful critical section management
- **Lessons learned**: Type-state pattern prevents runtime errors, compile-time guarantees valuable
- **Process improvement**: BACKLOG.md Kanban board kept work organized and visible

**Next Steps**: Begin Phase C planning (LDMA, IADC, EMU, LETIMER)

---

## 2025-12-13: Documentation Cleanup & Project Management System

**Milestone**: Documentation standardization and project tracking infrastructure
```

**Response**: "Phase B completion milestone logged! Added comprehensive entry to LOG.md with achievements, metrics, and retrospectives. Ready to begin Phase C planning."

---

## Example 7: Viewing Current Status

**User request**: "Show current project status"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Extract and summarize:
   - Current sprint info
   - In Progress tasks
   - Ready tasks count
   - Blocked tasks
   - Recent completions
   - Project metrics

**Response**:
```
=== Project Status ===

Current Sprint: Phase B - Communication Peripherals (100% complete)
Last Updated: December 15, 2025

📋 In Progress (0):
  None - ready to start Phase C tasks

✅ Ready (High Priority) (2):
  1. EMU: Energy Management Unit - EM0-EM4 transitions
  2. LDMA: Linked DMA Controller - peripheral transfers

🚫 Blocked (1):
  1. Embassy async support (waiting for stable HAL)

✨ Recent Completions (last 7 days):
  - TIMER: Implement TIMER0 basic functionality (Dec 15)
  - PWM: Add PWM generation support (Dec 15)
  - Phase B milestone logged (Dec 15)

📊 Project Metrics:
  - Phase Progress: Phase B 100% complete, Phase C next
  - Total HAL Code: ~2,430 lines
  - Examples: 7 working examples
  - Build Time: ~3.5 minutes (release)
  - Time Investment: ~20 hours total (Phases A+B)

🎯 Next Milestone: Phase C kickoff
```

---

## Example 8: Monthly Archive

**User request**: "Archive completed tasks older than 30 days to LOG.md"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Find Done tasks with @done dates >30 days ago
3. Create summary entry for LOG.md
4. Remove from BACKLOG.md
5. Prepend to LOG.md under "Archived Tasks" section

**Before** (docs/BACKLOG.md - Done section):
```markdown
## Done (Recent - Last 30 Days)

### Week of November 15-21, 2025  (older than 30 days)
- [x] **Phase A**: GPIO implementation @done(2025-11-20)
- [x] **Phase A**: CMU clock configuration @done(2025-11-21)

### Week of December 13-19, 2025  (within 30 days)
- [x] **TIMER**: Implementation @done(2025-12-15)
```

**After - BACKLOG.md**:
```markdown
## Done (Recent - Last 30 Days)

### Week of December 13-19, 2025
- [x] **TIMER**: Implementation @done(2025-12-15)

(Older tasks archived to LOG.md on 2025-12-16)
```

**After - LOG.md** (prepended):
```markdown
## 2025-12-16: November 2025 Tasks Archived

**Archived Tasks** (completed >30 days ago):

### Week of November 15-21, 2025
- GPIO implementation (Phase A) - 617 lines
- CMU clock configuration (Phase A) - 317 lines

These tasks have been archived from BACKLOG.md Done column per 30-day retention policy.

---
```

**Response**: "Archived 2 completed tasks from November to LOG.md. BACKLOG.md Done column now contains only tasks from last 30 days."

---

## Example 9: Adding Task with Dependencies

**User request**: "Add task: create BSP for XIAO MG24 board - depends on all Phase A+B peripherals"

**Skill actions**:
1. Read docs/BACKLOG.md
2. Identify this as Phase C task with dependencies
3. Add to Ready or Ideas based on current phase
4. Document dependencies in task description

**Result** (docs/BACKLOG.md):
```markdown
## Ready (Prioritized for Phase C)

### Medium Priority
- [ ] **BSP**: Create Board Support Package for XIAO MG24 Sense @priority(medium) @phase(C)
  - Pin definitions for board-specific peripherals
  - Sensor integration (IMU LSM6DS3TR-C, PDM microphone)
  - Battery management
  - LED and button abstractions
  - **Dependencies**: Requires Phase A (GPIO, CMU) and Phase B (I2C for sensors)
```

**Response**: "Added BSP task to Ready (Medium Priority) for Phase C. Documented dependencies on Phase A (GPIO, CMU) and Phase B (I2C). Will be ready to start once Phase B completes."

---

## Example 10: Skill Integration with Orchestrator

**User workflow**: Completing a module implementation

1. **User**: "Implement TIMER module"
2. **Orchestrator**:
   - Uses project-manager skill: "Start TIMER task"
   - Implements TIMER module
   - Runs rust-hal-expert review
   - Uses markdown-edit for README.md
   - Builds examples
   - **Uses project-manager skill**: "Complete TIMER task with metrics"
3. **Project-manager skill**: Updates BACKLOG.md with completion
4. **User**: "Commit TIMER implementation"
5. **Orchestrator**: Pre-commit documentation audit checks BACKLOG.md updated
6. **Success**: Commit proceeds

This shows seamless integration between:
- project-manager skill (task tracking)
- rust-hal-expert skill (code review)
- markdown-edit skill (documentation)
- orchestrator agent (workflow enforcement)

---

**Created**: December 13, 2025
**Purpose**: Demonstrate project-manager skill usage patterns
**Maintained by**: Project management documentation
