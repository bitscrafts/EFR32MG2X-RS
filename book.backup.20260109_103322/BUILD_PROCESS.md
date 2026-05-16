# Building the Embedded Rust Book
## Containerized Build with Podman

**Last Updated**: 2026-01-05
**Build Environment**: Podman container on lnx host
**Container**: `embedded-rust-book` (texlive/texlive:latest)

---

## Quick Reference

| Command | Purpose |
|---------|---------|
| `podman-compose -f podman-compose-embedded.yml up -d` | Start container |
| `podman exec embedded-rust-book bash -c 'cd /workspace && xelatex -shell-escape main.tex'` | Compile book |
| `podman exec embedded-rust-book bash -c 'cd /workspace && for i in 1 2 3; do xelatex -shell-escape main.tex; done'` | Full build (3 passes) |
| `podman cp embedded-rust-book:/workspace/main.pdf ./` | Copy PDF locally |

---

## Build Environment Setup

### 1. Remote Host Access
```bash
ssh mvcorrea@lnx
```
**Note**: This build process requires SSH access to `lnx` host.

### 2. Container Configuration
**File**: `podman/latex-build/podman-compose-embedded.yml`

```yaml
version: '3.8'
services:
  latex:
    image: docker.io/texlive/texlive:latest
    container_name: embedded-rust-book
    working_dir: /workspace
    volumes:
      - ./embedded-support-book:/workspace:z
    environment:
      - LANG=en_US.UTF-8
    command: tail -f /dev/null  # Keep container running
```

**Key Features**:
- Uses official TeX Live Docker image
- Workspace mounted at `/workspace`
- Container runs continuously for easy exec
- Full LaTeX installation with all packages

### 3. Directory Structure on Remote Host

```
lnx:~/podman/latex-build/
├── embedded-support-book/          # Book source files
│   ├── main.tex                    # Master document
│   ├── preamble.tex                # LaTeX configuration
│   ├── chapters/
│   │   ├── chapter-01-introduction.tex
│   │   └── chapter-01-introduction/
│   │       ├── 01.01-why-rust.tex
│   │       ├── 01.02-bare-metal-program.tex
│   │       ├── 01.03-pac-hal-bsp-architecture.tex
│   │       ├── 01.04-project-organization.tex
│   │       ├── 01.05-development-prerequisites.tex
│   │       └── 01.06-building-first-binary.tex
│   └── tikz/
│       └── 01.03-pac-hal-bsp-architecture.tex
└── podman-compose-embedded.yml   # Container configuration
```

---

## Build Commands

### 1. Start/Restart Container
```bash
cd ~/podman/latex-build
podman-compose -f podman-compose-embedded.yml up -d
```

Output:
```
Pulling docker.io/texlive/texlive:latest...
Creating embedded-rust-book ... done
```

Verify container is running:
```bash
podman ps | grep embedded-rust-book
```

### 2. Single Compilation Pass
```bash
podman exec embedded-rust-book bash -c 'cd /workspace && xelatex -shell-escape main.tex'
```

**Purpose**: Quick compile to check for errors

### 3. Full Build (3 Passes for TOC/References)
```bash
podman exec embedded-rust-book bash -c 'cd /workspace && \
  for i in 1 2 3; do \
    echo "=== Pass $i ===" && \
    xelatex -shell-escape main.tex; \
  done'
```

**Purpose**: Complete build with proper TOC, cross-references, and bibliography

### 4. Compile with Error Log
```bash
podman exec embedded-rust-book bash -c 'cd /workspace && xelatex -shell-escape -file-line-error main.tex 2>&1 | tee compile.log'
```

**Purpose**: Debug compilation errors with line numbers

### 5. Stop Container
```bash
cd ~/podman/latex-build
podman-compose -f podman-compose-embedded.yml down -v
```

---

## Build Process Details

### Compilation Steps

1. **First Pass**:
   - Processes all `.tex` files
   - Generates `.aux` files for cross-references
   - Creates `.toc` for table of contents
   - May show "undefined reference" warnings (normal)

2. **Second Pass**:
   - Resolves cross-references using `.aux` files
   - Generates proper TOC
   - Still may have unresolved references

3. **Third Pass**:
   - Final resolution of all references
   - Complete TOC with proper page numbers
   - All cross-references resolved

### Required Compiler Options

- **`-shell-escape`**: Required for `minted` package (syntax highlighting)
- **`-interaction=nonstopmode`**: Continue on errors (useful for debugging)
- **`-file-line-error`**: Show errors with file names and line numbers
- **`xelatex`**: Required for Unicode and advanced font features

### Expected PDF Output

When build succeeds:
- **File**: `/workspace/main.pdf`
- **Location**: Remote host `lnx:~/podman/latex-build/embedded-support-book/`
- **Size**: ~1-2 MB (grows as more chapters added)
- **Pages**: Currently 44+ pages (Chapter 1 complete)

---

## Troubleshooting

### Container Won't Start
```bash
# Check logs
podman logs embedded-rust-book

# Restart
podman-compose -f podman-compose-embedded.yml restart

# Remove and recreate
podman-compose -f podman-compose-embedded.yml down -v
podman-compose -f podman-compose-embedded.yml up -d
```

### Compilation Fails

1. **Check error log**:
   ```bash
   podman exec embedded-rust-book bash -c 'cat /workspace/compile.log | grep -A5 -B5 "Error\|error:\" | tail -50'
   ```

2. **Common errors**:
   - `! Undefined control sequence` → Missing package in preamble
   - `! You can't use 'macro parameter character #'` → `#` not escaped in code
   - `! LaTeX Error: File ended while scanning` → Mismatched environment
   - `! Package minted Error` → Missing `-shell-escape` flag

3. **Rsync updated files**:
   ```bash
   rsync -avz ./ latex/ mvcorrea@lnx:podman/latex-build/embedded-support-book/
   ```

### Font/Missing Character Warnings

Box-drawing characters (├, ─, └) may show warnings:
```
Missing character: There is no ├ (U+251C) in font
```

**These are warnings, not errors**. The PDF will still build successfully.

To fix, you can:
1. Use ASCII art instead: `+--`
2. Or ignore (document still builds fine)

---

## Pulling Build Artifacts

### Copy PDF from Container to Local Machine

```bash
# From local machine (not in SSH session):
scp mvcorrea@lnx:~/podman/latex-build/embedded-support-book/main.pdf ./
```

Or from within SSH session:
```bash
# On remote host:
podman cp embedded-rust-book:/workspace/main.pdf ./
scp ./main.pdf your-local-machine:~/Downloads/
```

### View Log Files

```bash
# View compile log
podman exec embedded-rust-book cat /workspace/compile.log

# View main log for detailed errors
podman exec embedded-rust-book cat /workspace/main.log | grep -A10 -B10 "Error\|error:" | head -100
```

---

## Build Process Documentation

### What Happens During Compilation

1. **Container starts**: Downloads texlive image (if not cached)
2. **Volume mount**: Host directory mounted to `/workspace`
3. **XeLaTeX runs**:
   - Reads `main.tex`
   - Includes `preamble.tex`
   - Processes all `ef`, `ootnote`, `	ableofcontents` commands
   - Runs minted for syntax highlighting (requires `-shell-escape`)
   - Generates TikZ diagrams
   - Uses forest for directory trees
4. **Output**: `main.pdf` written to mounted volume

### Docker vs Podman Notes

- **Podman** is a daemonless container engine (drop-in replacement for Docker)
- Commands are identical to Docker: `podman-compose` ≈ `docker-compose`
- Rootless containers (run as unprivileged user)
- Same OCI containers work with both

### Image Details

- **Image**: `docker.io/texlive/texlive:latest`
- **Size**: ~4 GB (includes full TeX Live distribution)
- **Contents**: All TeX Live packages, fonts, tools
- **Base OS**: Debian/Ubuntu
- **Updates**: Image is rebuilt monthly with latest packages

---

## Continuous Integration Hook

To run build automatically on changes:

```bash
#!/bin/bash
# watch-and-build.sh

WATCH_DIR="./latex"
REMOTE="mvcorrea@lnx:podman/latex-build/embedded-support-book"

rsync -avz --delete "$WATCH_DIR/" "$REMOTE/" && \
podman exec embedded-rust-book bash -c 'cd /workspace && xelatex -shell-escape main.tex'
```

Run:
```bash
./watch-and-build.sh
```

---

## Performance Notes

- **Container startup**: ~10 seconds (image download first time only)
- **Compilation time**: ~30-60 seconds per pass (depending on content)
- **Full 3-pass build**: ~2-3 minutes
- **Incremental builds**: ~30 seconds (if only content changed, not structure)

---

## Environment Variables

| Variable | Value | Purpose |
|----------|-------|---------|
| `LANG` | `en_US.UTF-8` | Unicode support |
| `SHELL` | `/bin/bash` | Bash shell inside container |
| `PWD` | `/workspace` | Working directory |

---

## Quick Build Script

Create this file for convenience:

```bash
#!/bin/bash
# build-book-remote.sh

# Remote host
HOST="mvcorrea@lnx"
REMOTE_DIR="~/podman/latex-build/embedded-support-book"

# Sync files
echo "=== Syncing files to remote host ==="
rsync -avz --progress ./latex/ "$HOST:$REMOTE_DIR/"

# Compile
echo ""
echo "=== Compiling on remote host ==="
ssh "$HOST" "cd ~/podman/latex-build && \
  podman exec embedded-rust-book bash -c 'cd /workspace && \
    for i in 1 2 3; do \
      echo \"Pass \$i\" && \
      xelatex -shell-escape main.tex; \
    done'"

# Copy PDF back
echo ""
echo "=== Copying PDF to local machine ==="
scp "$HOST:$REMOTE_DIR/main.pdf" ./

echo ""
echo "✓ Build complete! PDF: ./main.pdf"
```

Make executable:
```bash
chmod +x build-book-remote.sh
./build-book-remote.sh
```

---

## Build Verification Checklist

After successful build, verify:

- [ ] `main.pdf` file exists
- [ ] File size > 100 KB
- [ ] Can open with PDF viewer
- [ ] Chapter 1 content visible
- [ ] Table of Contents populated
- [ ] Page numbers correct
- [ ] Code syntax highlighting works
- [ ] TikZ diagrams render
- [ ] Margin notes visible
- [ ] No major errors in compile.log

---

## Next Steps

When ready to add more chapters:

1. Create new chapter files locally
2. Rsync to remote: `rsync -avz ./latex/ mvcorrea@lnx:podman/latex-build/embedded-support-book/`
3. Recompile: `podman exec embedded-rust-book bash -c 'cd /workspace && xelatex -shell-escape main.tex'`
4. Repeat for 3 passes

---

## Support

For build issues:
1. Check compile.log for specific errors
2. Verify file permissions (should be readable)
3. Check container is running: `podman ps`
4. Review this document's Troubleshooting section

---

## Document Version

**Build Process Version**: 1.0.0
**Last Updated**: 2026-01-05
**Container**: texlive/texlive:latest
**Compiler**: XeLaTeX 3.14159265
**Build Host**: lnx
