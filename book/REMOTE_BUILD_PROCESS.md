# EFR32MG24 Book: Remote Build Process Documentation

## Containerized LaTeX Compilation Workflow

**Author**: Marcelo Correa
**Date**: 2026-01-05
**Version**: 1.0.0
**Build Environment**: Podman container on lnx host
**Container**: embedded-rust-book (texlive/texlive:latest)

---

## Overview

This document describes the complete workflow for building the Embedded Rust book using a containerized remote compilation environment. This approach ensures consistent builds across all machines and isolates the complex LaTeX toolchain.

### Why Remote Build?

1. **Consistent Environment**: Same TeX Live version, packages, fonts everywhere
2. **No Local Setup**: No need to install 4GB TeX Live on development machines
3. **Isolated Dependencies**: LaTeX packages don't conflict with system packages
4. **Parallel Work**: Edit locally, compile remotely without interruption
5. **Version Control**: All compilation steps are documented and repeatable

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Local Development Machine                 │
│                                                               │
│  ┌────────────────────────────────────────────┐             │
│  │   Book Source Files (~/book/latex/)       │             │
│  │   ├── main.tex                             │             │
│  │   ├── preamble.tex                         │             │
│  │   ├── chapters/                            │             │
│  │   │   └── chapter-01-introduction/         │             │
│  │   │       ├── 01.01-why-rust.tex          │             │
│  │   │       ├── 01.02-bare-metal-program.tex│             │
│  │   │       └── ...                          │             │
│  │   └── tikz/                                │             │
│  │       └── 01.03-pac-hal-bsp-architecture.tex│            │
│  └────────────────────────────────────────────┘             │
│                                                               │
│  Workflow:                                                    │
│  1. edit files locally ✏️                                    │
│  2. rsync to remote 🔄                                        │
│  3. compile remotely 🖥️                                       │
│  4. analyze errors 📋                                         │
│  5. fix locally ✏️                                            │
│  ← repeat until success                                      │
│  6. copy PDF locally 📄                                       │
│                                                               │
└─────────────────────────────────────────────────────────────┘
                                  |
                                  | rsync (files up)
                                  | scp (PDF down)
                                  |
┌─────────────────────────────────────────────────────────────┐
│                    Remote Build Host: lnx                   │
│                                                               │
│  ┌────────────────────────────────────────────┐             │
│  │   Container: embedded-rust-book            │             │
│  │   Image: texlive/texlive:latest            │             │
│  │   Workspace: /workspace                    │             │
│  │   Volume: ./embedded-support-book:/workspace│            │
│  │                                           │             │
│  │   ┌────────────────────────────────┐     │             │
│  │   │  Compilation Process          │     │             │
│  │   │  1. xelatex -shell-escape      │     │             │
│  │   │     main.tex                   │     │             │
│  │   │  2. Generate logs              │     │             │
│  │   │     main.log, compile.log      │     │             │
│  │   │  3. If errors, → logs          │     │             │
│  │   │  4. If success, → PDF          │     │             │
│  │   │     main.pdf                   │     │             │
│  │   └────────────────────────────────┘     │             │
│  └────────────────────────────────────────────┘             │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## Build Workflow (Step-by-Step)

### Phase 1: Initial Compilation

**Step 1**: Start container (if not running)
```bash
ssh mvcorrea@lnx
cd ~/podman/latex-build
podman-compose -f podman-compose-embedded.yml up -d
podman ps | grep embedded-rust-book  # Verify running
```

**Step 2**: Initial compilation (3 passes)
```bash
podman exec embedded-rust-book bash -c 'cd /workspace && \
  echo "=== Pass 1 ===" && \
  xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee compile.log && \
  echo "=== Pass 2 ===" && \
  xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee -a compile.log && \
  echo "=== Pass 3 ===" && \
  xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee -a compile.log'
```

**Understanding Passes**:
- **Pass 1**: Generates `.aux`, `.toc` files (may show undefined references)
- **Pass 2**: Resolves cross-references using `.aux` files
- **Pass 3**: Final pass with complete TOC and all references

**Step 3**: Check compilation result
```bash
podman exec embedded-rust-book bash -c 'cd /workspace && ls -lh main.pdf main.log compile.log 2>/dev/null || echo "No PDF generated"'
```

**Step 4**: Pull logs for local analysis (if errors)
```bash
# On local machine:
rsync -avz mvcorrea@lnx:podman/latex-build/embedded-support-book/compile.log ./
rsync -avz mvcorrea@lnx:podman/latex-build/embedded-support-book/main.log ./
```

---

### Phase 2: Error Analysis and Fix Cycle

**Step 5**: Analyze errors locally
```bash
# On local machine:
grep -E "^! (LaTeX|Package|Undefined|Missing|You can)" compile.log | head -20
```

**Common Errors and Solutions**:

| Error | Cause | Fix |
|-------|-------|-----|
| `! Package minted Error: outputdir` | Minted v3+ doesn't need outputdir | Edit preamble.tex, remove `[outputdir=build]` |
| `! LaTeX Error: Missing \begin{document}` | Malformed LaTeX before \begin{document} | Check chapter files for corruption |
| `! You can't use 'macro parameter character #'` | `#` not escaped in minted | Use `\#![...]` inside minted |
| `! Package titlesec Error: Nested titles` | Section in wrong place | Check for sections inside tables/lists |
| `! LaTeX Error: Environment example undefined` | Missing exercise package | Add `\usepackage{exercise}` to preamble |
| `! Undefined control sequence` | Missing package import | Add required \usepackage |

**Step 6**: Fix errors in local files
```bash
# Edit file based on error:
# Example: fix preamble.tex
vim ./book/latex/preamble.tex

# Example: fix chapter
vim ./book/latex/chapters/chapter-01-introduction/01.04-project-organization.tex
```

**Step 7**: Rsync corrected files
```bash
rsync -avz ./book/latex/ mvcorrea@lnx:podman/latex-build/embedded-support-book/
```

**Step 8**: Repeat compilation
```bash
# Back to Step 2 (repeat until success)
```

**Iteration Pattern**: Most fixes require 2-3 edit/compile cycles
**Record Keeping**: Keep all compile.log files for comparison

---

### Phase 3: Success (PDF Generated)

**Step 9**: Verify successful compilation
```bash
podman exec embedded-rust-book bash -c 'cd /workspace && \
  ls -lh main.pdf && \
  echo "PDF size: $(stat -c%s main.pdf) bytes" && \
  echo "Pages: $(pdftk main.pdf dump_data | grep NumberOfPages)"'
```

**Step 10**: copy PDF to local machine
```bash
# Method 1: rsync
rsync -avz mvcorrea@lnx:podman/latex-build/embedded-support-book/main.pdf ./

# Method 2: scp
scp mvcorrea@lnx:~/podman/latex-build/embedded-support-book/main.pdf ./

# Method 3: From within SSH session on remote
podman cp embedded-rust-book:/workspace/main.pdf ./
scp ./main.pdf your-local-machine:~/Downloads/
```

**Step 11**: Verify PDF locally
```bash
# Check PDF integrity
pdfinfo main.pdf

# View PDF
open main.pdf  # macOS
xdg-open main.pdf  # Linux
start main.pdf  # Windows
```

---

## Detailed Error Resolution Examples

### Example 1: Minted outputdir Error

**Error from compile.log**:
```
! Package minted Error: Package option "outputdir" is no longer needed with min
 ted v3+
```

**Analysis**:
- minted v3+ automatically detects output directory
- The `outputdir` option is deprecated and causes failure

**Local Fix**:
```bash
# Edit locally:
vim ./book/latex/preamble.tex

# Change line 47 from:
\usepackage[outputdir=build]{minted}

# To:
\usepackage{minted}
```

**Transfer**:
```bash
rsync -avz ./book/latex/preamble.tex mvcorrea@lnx:podman/latex-build/embedded-support-book/
```

**Recompile**:
```bash
ssh mvcorrea@lnx 'podman exec embedded-rust-book bash -c "cd /workspace && xelatex -shell-escape main.tex"'
```

### Example 2: Markdown Fence Error

**Error from compile.log**:
```
! LaTeX Error: Missing \begin{document}.
l.121 ```
```

**Analysis**:
- Markdown fence ``` found instead of proper minted environment
- LaTeX expected egin{minted} but found ```

**Local Fix**:
```bash
# Edit chapter file:
vim ./book/latex/chapters/chapter-01-introduction/01.04-project-organization.tex

# Change:
```
code here
```

# To:
\begin{minted}{text}
code here
\end{minted}
```

**Transfer**:
```bash
rsync -avz ./book/latex/chapters/chapter-01-introduction/01.04-project-organization.tex mvcorrea@lnx:podman/latex-build/embedded-support-book/
```

### Example 3: Markdown Header Error

**Error from compile.log**:
```
! Undefined control sequence.
l.8 ### Rust Toolchain
```

**Analysis**:
- Markdown header `###` found in LaTeX file
- Should be LaTeX \subsection or \textbf

**Local Fix**:
```bash
# Edit chapter file:
vim ./book/latex/chapters/chapter-01-introduction/01.05-development-prerequisites.tex

# Change:
### Rust Toolchain

# To:
\textbf{Rust Toolchain}:
# or
\subsection{Rust Toolchain}
```

**Transfer**:
```bash
rsync -avz ./book/latex/chapters/chapter-01-introduction/01.05-development-prerequisites.tex mvcorrea@lnx:podman/latex-build/embedded-support-book/
```

---

## Log File Analysis

### Quick Error Count

```bash
# Count errors by type
ssh mvcorrea@lnx 'podman exec embedded-rust-book bash -c "cd /workspace && grep -c \'^! LaTeX Error\' compile.log"'  # LaTeX errors
ssh mvcorrea@lnx 'podman exec embedded-rust-book bash -c "cd /workspace && grep -c \'^! Package Error\' compile.log"'   # Package errors
ssh mvcorrea@lnx 'podman exec embedded-rust-book bash -c "cd /workspace && grep -c \'^! Undefined control\' compile.log"'  # Undefined commands
```

### View Errors with Context

```bash
# Pull log and analyze locally
rsync -avz mvcorrea@lnx:podman/latex-build/embedded-support-book/compile.log ./

# Show errors with 3 lines before/after
grep -B3 -A3 "^! " compile.log | head -50

# Show only specific error types
grep "^! LaTeX Error" compile.log
grep "^! Package Error" compile.log
grep "^! Undefined" compile.log
```

### Common Error Patterns

**Pattern 1**: Missing package
```
! Undefined control sequence.
l.123 \begin{listing}

```
**Fix**: Add `\usepackage{float}` to preamble.tex

**Pattern 2**: Missing environment
```
! LaTeX Error: Environment notebox undefined.
```
**Fix**: Define notebox environment in preamble.tex

**Pattern 3**: Special characters in minted
```
! You can't use `macro parameter character #' in horizontal mode.
l.162 #![no_std]
```
**Fix**: Escape special chars: `\#![no_std]` with `\#` inside minted

---

## Container Management

### Check Container Status

```bash
# View running containers
ssh mvcorrea@lnx 'podman ps'

# View all containers
ssh mvcorrea@lnx 'podman ps -a'

# Check container logs
ssh mvcorrea@lnx 'podman logs embedded-rust-book'

# Get container shell
ssh mvcorrea@lnx 'podman exec -it embedded-rust-book bash'
```

### Restart After Changes

```bash
# If you need to restart container:
ssh mvcorrea@lnx 'cd ~/podman/latex-build && \
  podman-compose -f podman-compose-embedded.yml restart'

# Or full restart:
ssh mvcorrea@lnx 'cd ~/podman/latex-build && \
  podman-compose -f podman-compose-embedded.yml down && \
  podman-compose -f podman-compose-embedded.yml up -d'
```

---

## Performance Optimization

### Speed Up Compilations

1. **Incremental builds**: Only changed files are recompiled
2. **Keep container running**: Avoid container restart overhead
3. **Use rsync delta**: Only transfer changed portions of files
4. **Parallel operations**: Edit locally while previous compile runs

### Expected Timings

| Operation | Time |
|-----------|------|
| Container startup | 10 seconds |
| Single xelatex pass | 30-45 seconds |
| Full 3-pass build | 2-3 minutes |
| rsync changed file | <1 second |
| PDF copy back | <1 second |

---

## Backup and Recovery

### Backup Compiled PDFs

```bash
# Keep versioned copies
rsync -avz mvcorrea@lnx:podman/latex-build/main.pdf ./main-$(date +%Y%m%d_%H%M%S).pdf

# Backup entire workspace
rsync -avz mvcorrea@lnx:~/podman/latex-build/embedded-support-book/ ./backup-$(date +%Y%m%d)/
```

### Restore from Backup

```bash
rsync -avz ./backup-20260105_143022/ mvcorrea@lnx:~/podman/latex-build/embedded-support-book/
```

---

## Debugging Commands

### Inside Container

```bash
# Check file permissions
podman exec embedded-rust-book bash -c 'ls -la /workspace/*.tex'

# Check disk space
podman exec embedded-rust-book bash -c 'df -h /workspace'

# View specific file
podman exec embedded-rust-book bash -c 'cat /workspace/main.log | head -50'

# Test single command
podman exec embedded-rust-book which xelatex
podman exec embedded-rust-book xelatex --version
```

### Package Verification

```bash
# Check if minted package is installed
podman exec embedded-rust-book kpsewhich minted.sty

# List all available packages
podman exec embedded-rust-book tlmgr list --only-installed | grep -E "minted|tikz|pgfplots"
```

---

## Complete Build Script

```bash
#!/bin/bash
# build-book-complete.sh

REMOTE="mvcorrea@lnx"
REMOTE_DIR="~/podman/latex-build/embedded-support-book"
LOCAL_DIR="./book/latex"

set -e  # Exit on error

echo "=== STEP 1: Sync files to remote ==="
rsync -avz --progress "$LOCAL_DIR/" "$REMOTE:$REMOTE_DIR/"

echo ""
echo "=== STEP 2: Compile on remote (3 passes) ==="
ssh "$REMOTE" "cd ~/podman/latex-build && \
  podman exec embedded-rust-book bash -c '\
    cd /workspace && \
    for i in 1 2 3; do \
      echo \"=== Pass \\$i ===\" && \
      xelatex -shell-escape -interaction=nonstopmode main.tex 2>&1 | tee -a compile.log; \
    done'
  "

echo ""
echo "=== STEP 3: Check if PDF exists ==="
ssh "$REMOTE" "cd ~/podman/latex-build && \
  podman exec embedded-rust-book bash -c '\
    cd /workspace && \
    if [ -f main.pdf ]; then \
      echo \"✓ SUCCESS: main.pdf created\"; \
      ls -lh main.pdf; \
      exit 0; \
    else \
      echo \"✗ FAILED: PDF not generated\"; \
      grep -c \"^! LaTeX Error\" compile.log | xargs echo \"Total errors:\"; \
      exit 1; \
    fi'
  "

if [ $? -eq 0 ]; then
  echo ""
  echo "=== STEP 4: Copy PDF locally ==="
  rsync -avz "$REMOTE:$REMOTE_DIR/main.pdf" ./
  echo "✓ Build complete! PDF: ./main.pdf"
else
  echo ""
  echo "=== STEP 4: Pull logs for debugging ==="
  rsync -avz "$REMOTE:$REMOTE_DIR/compile.log" ./
  rsync -avz "$REMOTE:$REMOTE_DIR/main.log" ./
  echo "✗ Build failed. Check logs: compile.log, main.log"
  exit 1
fi
