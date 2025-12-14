# [Project Name] - Claude Code Instructions

## Project Context

This project provides [brief description of what the project does].

**Status**: [Current phase and completion percentage]

**Target Hardware**: [Specific hardware/platform]

**Key Documentation**:
- [docs/PLAN.md](docs/PLAN.md) - Development roadmap
- [docs/STATUS.md](docs/STATUS.md) - Current implementation status
- [docs/BACKLOG.md](docs/BACKLOG.md) - Task tracking board
- [docs/LOG.md](docs/LOG.md) - Milestone achievement log

---

## Project Structure

```
ProjectRoot/
├── Cargo.toml                      # Workspace root
├── crate-pac/                      # Peripheral Access Crate (if applicable)
│   ├── Cargo.toml
│   └── src/lib.rs
├── crate-hal/                      # Hardware Abstraction Layer (if applicable)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── module1/                # Module with README.md
│   │   └── module2/
│   ├── examples/                   # Example programs
│   ├── docs/                       # Crate-specific documentation
│   ├── memory.x                    # Linker memory layout
│   ├── device.x                    # Interrupt vector table
│   └── build.rs                    # Build script
├── docs/                           # Workspace documentation
│   ├── README.md
│   ├── PLAN.md                     # Strategic roadmap
│   ├── STATUS.md                   # Current status
│   ├── BACKLOG.md                  # Task board
│   └── LOG.md                      # Milestone log
├── .claude/                        # Claude Code configuration
│   ├── agents/
│   ├── commands/
│   └── skills/
└── CLAUDE.md                       # This file
```

---

## Current Status

### Completed (Phase A)

**Component List**:
1. **Module 1** - Complete
   - Feature list
   - embedded-hal traits implemented

2. **Module 2** - Complete
   - Feature list

**Build Results**:
- Component 1: X KB .rlib
- Component 2: Y KB .rlib
- Build time: ~X minutes (release mode)
- All examples compile without errors

### In Progress (Phase B)

Next implementation targets:
- Module 3 - Description
- Module 4 - Description
- Module 5 - Description

---

## Development Guidelines

### Code Style (User Preferences)

- No emojis in code or commit messages
- All modules in src/ must have their own README.md file
- Use mermaid for diagrams
- Keep files around 400 lines (split into submodules if larger)
- Avoid repeating information (single source of truth)

### Commit Messages

- Avoid mentioning Claude in commit messages
- Use conventional commit format:
  - `feat:` New feature
  - `fix:` Bug fix
  - `docs:` Documentation changes
  - `refactor:` Code refactoring
  - `test:` Test additions/changes

### Documentation Standards

**IMPORTANT**: All documentation MUST use templates from `.claude/skills/markdown-edit/templates/`

- Module README in `src/module_name/README.md` for each peripheral
- Status and planning docs in `docs/` folders
- Only README.md in root folders (all other docs in `docs/`)
- Update status docs when completing phases
- Use mermaid diagrams for architecture
- **Always backup documentation before modification** (automatic via markdown-edit)

### Documentation Backup Workflow

**Critical Requirement**: All documentation modifications MUST use markdown-edit skill scripts for automatic `.archive/` backups.

```bash
# NEVER edit markdown files directly
# ALWAYS use markdown-edit scripts:
bash .claude/skills/markdown-edit/scripts/smart-replace.sh \
  docs/STATUS.md "old text" "new text"
```

**What gets archived**:
- Planning documents (PLAN.md, STATUS.md)
- Module documentation (README.md files when doing major rewrites)
- Project documentation (CLAUDE.md, README.md)
- Changelogs (CHANGELOG.md)

**Archive location**: `.archive/` folder at project root
**Retention policy**: Archives kept indefinitely for history
**Purpose**: Quick rollback capability without git operations

### File Organization

- Files >400 lines should be split into submodules
- Use module folders (e.g., `src/gpio/` with `mod.rs`, `types.rs`, `pin.rs`, `traits.rs`)
- Coordinator module (`mod.rs`) should be small, mostly re-exports
- Implementation details in separate files

---

## Key Technical Patterns

### Pattern 1: [Name]

```rust
// Example code demonstrating the pattern
```

**Why**: Explanation of why this pattern is used.

### Pattern 2: [Name]

```rust
// Example code
```

**Why**: Explanation.

---

## Component Generation (if applicable)

### How [Component] Was Generated

If your project includes generated code (e.g., PAC from SVD):

**Critical Steps**:

1. **Obtain Source Files**:
```bash
# Example commands
```

2. **Install Tools** (Important!):
```bash
# MUST use --locked to pin dependencies
cargo install tool-name --locked
```

**Why --locked is required**: Explanation of dependency issues.

3. **Generate Component**:
```bash
# Generation commands
```

**Result**:
- Lines of code generated
- Output size
- Key characteristics

---

## Common Commands

### Build Commands

```bash
# Check library
cargo check --features rt

# Build examples
cargo build --examples --features rt --release

# Build specific example
cargo build --example example_name --features rt --release

# Flash to device (requires probe-rs)
cargo run --example example_name --features rt --release
```

### Development

```bash
# Add new peripheral module
mkdir src/module_name
touch src/module_name/{mod.rs,types.rs,README.md}

# Use template for README
cp .claude/skills/markdown-edit/templates/MODULE_README_TEMPLATE.md \
   src/module_name/README.md

# Check file sizes
find src -name "*.rs" -exec wc -l {} + | sort -n

# Generate docs
cargo doc --no-deps --features rt --open
```

---

## Hardware Specifications

**[Hardware Name]**:
- Processor: Details
- Memory: Flash/RAM sizes
- Peripherals: Key peripherals available
- Package: Pin count and type
- Special features: Unique capabilities

**Rust Target**: `target-triple`

---

## Phase Completion Summary

### What Was Accomplished

Brief summary of major milestones completed.

**Key Files Modified**:
- List of important files and what changed

---

## Known Issues & Limitations

### Current Limitations

1. Limitation 1 - Reason and workaround
2. Limitation 2 - Reason and workaround

### Future Work

1. Future enhancement 1
2. Future enhancement 2
3. Future enhancement 3

---

## Success Metrics

**Phase A** (Complete/In Progress):
- [ ] Success criterion 1
- [ ] Success criterion 2
- [ ] Success criterion 3

**Phase B** (In Progress):
- [ ] Success criterion 1
- [ ] Success criterion 2

**Long Term Goals**:
- [ ] Published to crates.io
- [ ] Hardware tested
- [ ] 10+ working examples
- [ ] Community contributors

---

## Claude Code Skills Integration

This project uses specialized Claude Code skills for development:

**Active Skills**:
- **rust-hal-expert**: Code review and quality enforcement
- **markdown-edit**: Documentation management with templates
- **project-manager**: Task tracking via BACKLOG.md and LOG.md
- **rust-embedded-orchestrator**: Workflow orchestration and quality gates

**Skill Enforcement**:
- All code reviews use rust-hal-expert
- All documentation edits use markdown-edit (automatic .archive backups)
- All task operations use project-manager
- All new documentation uses templates

See `.claude/agents/rust-embedded-orchestrator.md` for complete workflow.

---

## Resources

**Official Documentation**:
- [Reference Manual](link) - Hardware reference
- [Datasheet](link) - Electrical specifications

**Rust Resources**:
- [Embedded Rust Book](https://docs.rust-embedded.org/book/)
- [embedded-hal Documentation](https://docs.rs/embedded-hal/)
- [cortex-m Documentation](https://docs.rs/cortex-m/)

**Repository References**:
- [Similar Project 1](link) - Inspiration
- [Similar Project 2](link) - Patterns reference

---

## Contact & Repository

**Author**: Your Name <email@example.com>
**Repository**: https://github.com/username/repo
**License**: MIT OR Apache-2.0

---

<!-- META: last_updated=YYYY-MM-DD phase=Phase_X progress=XX% next_milestone="Brief description" -->
