# [Project Name] - Task Backlog

Kanban-style task tracking for daily tactical planning.

**Current Sprint**: Phase X - [Name]
**Sprint Duration**: YYYY-MM-DD to YYYY-MM-DD
**Last Updated**: YYYY-MM-DD

---

## Ready (Prioritized for Phase X)

### High Priority

- [ ] **Task Name**: Brief description @priority(high) @phase(X)
  - Sub-task 1
  - Sub-task 2
  - Dependencies: Task Y

### Medium Priority

- [ ] **Task Name**: Brief description @priority(medium) @phase(X)
  - Sub-task 1
  - Sub-task 2

### Low Priority

- [ ] **Task Name**: Brief description @priority(low) @phase(X)
  - Sub-task 1

---

## In Progress

- [ ] **Task Name**: Brief description @started(YYYY-MM-DD) @priority(high) @phase(X) @assignee(name)
  - Sub-task 1 (complete)
  - Sub-task 2 (in progress)
  - **Notes**: Current status update

---

## Review

- [ ] **Task Name**: Brief description @review(pending) @started(YYYY-MM-DD) @phase(X)
  - Waiting for: Code review / Testing / Documentation
  - **Reviewer**: Name

---

## Blocked

- [ ] **Task Name**: Brief description @blocked(Reason) @priority(high) @phase(X)
  - **Blocker**: Detailed description of what's blocking
  - **Mitigation**: Steps being taken
  - **ETA**: When blocker might be resolved

---

## Done (Recent - Last 30 Days)

### Week of Month DD-DD, YYYY

- [x] **Task Name**: Brief description @done(YYYY-MM-DD) @started(YYYY-MM-DD) @phase(X)
  - Sub-task 1 (completed)
  - Sub-task 2 (completed)
  - **Metrics**: XXX lines of code, X tests passing

### Week of Month DD-DD, YYYY

- [x] **Task Name**: Brief description @done(YYYY-MM-DD) @phase(X)

(Tasks older than 30 days archived to LOG.md on YYYY-MM-DD)

---

## Ideas (Unprioritized Future Work)

### Phase Y Candidates

- [ ] **Idea Name**: Brief description
  - Why this is valuable
  - Rough effort estimate

### Phase Z Candidates

- [ ] **Idea Name**: Brief description

---

## Metadata Tags Reference

- `@priority(high|medium|low)` - Task priority
- `@phase(A|B|C|...)` - Development phase
- `@started(YYYY-MM-DD)` - Start date
- `@done(YYYY-MM-DD)` - Completion date
- `@blocked(reason)` - Blocking reason
- `@review(status)` - Review status
- `@assignee(name)` - Task owner

---

## Metrics

- **Phase Progress**: Phase X at XX% complete
- **Tasks Ready**: X high priority, Y medium priority
- **Tasks In Progress**: X (target: 1-2)
- **Tasks Blocked**: X (target: 0)
- **Completion Rate**: X tasks per week (avg)
- **Cycle Time**: X days per task (avg)

---

**Maintained by**: [Team/Person]
**Update Frequency**: Daily
**Retention**: Done tasks kept 30 days, then archived to LOG.md

---

<!-- META: last_updated=YYYY-MM-DD template_version=1.0 update_frequency=Daily -->
