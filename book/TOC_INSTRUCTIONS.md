# TOC.yml Usage Instructions

## Overview

The `TOC.yml` file defines the complete book structure for "Implementing Rust Support for Embedded Hardware." This file serves as the master reference for the book's organization, chapter order, and file mapping.

## File Structure

```yaml
TOC.yml                           # Master table of contents (clean TOC only)
.archive/
  └── TOC_v1.0_complete.yml       # Complete original TOC with all metadata
```

**Important**: TOC.yml should contain ONLY the table of contents structure (metadata, parts, chapters, appendices). All configuration, guidelines, and processing metadata belong in this instructions file.

---

## TOC.yml Sections

### 1. Metadata

Basic information about the book:

```yaml
metadata:
  title: "Book Title"
  subtitle: "Subtitle"
  author: "Author Name"
  version: "1.0.0"
```

**Usage**: This information is used for book generation, PDF metadata, and version tracking. Update version when making structural changes.

### 2. Parts Structure

The book is divided into 4 logical parts:

```yaml
parts:
  - id: "part-1"
    title: "Part I: PAC Generation and Foundation"
    chapters: [1, 2, 3]
    description: "..."
```

**Fields**:
- `id`: Unique identifier for the part
- `title`: Display title
- `chapters`: Array of chapter numbers in this part
- `description`: Brief description of part content

**Usage**: Parts provide logical grouping for the book. Chapters must be listed in sequential order within each part.

### 3. Chapters List

Individual chapters with detailed metadata:

```yaml
chapters:
  - id: "chapter-01"
    title: "Introduction to Embedded Rust"
    description: "..."
    latex_file: "chapter-01-introduction.tex"
    estimated_pages: 30
    difficulty: "beginner"
    part: 1
    has_build_example: true
    sections:
      - id: "01-01"
        title: "Section Title"
        content_focus: ["topic1", "topic2"]
        code_examples: 5
```

**Fields**:
- `id`: Unique chapter identifier (format: `chapter-NN`)
- `title`: Chapter display title
- `description`: Brief overview of chapter content
- `latex_file`: LaTeX source file name
- `estimated_pages`: Estimated page count
- `difficulty`: "beginner", "intermediate", or "expert"
- `part`: Part number (1-4)
- `has_build_example`: true/false (chapter includes build examples)
- `has_flash_example`: true/false (chapter includes flash examples)
- `sections`: Array of sections within the chapter

**Section Fields**:
- `id`: Section identifier (format: `NN-NN`)
- `title`: Section title
- `content_focus`: Topics covered in this section
- `code_examples`: Number of code examples
- Additional flags: `build_examples`, `flash_examples`, `diagrams`, etc.

**File Naming Convention**:
- Pattern: `chapter-NN-{short-title}.tex`
- NN: Two-digit chapter number (zero-padded)
- Example: `chapter-01-introduction.tex`

**Usage**: Add new chapters by inserting into the appropriate part and incrementing chapter numbers.

### 4. Appendices

Appendices appear after the main chapters:

```yaml
appendices:
  - id: "appendix-a"
    title: "EFR32MG24 Reference Manual Quick Guide"
    latex_file: "appendix-a-reference.tex"
    content: "Brief description"
    pages: 20
```

**Fields**:
- `id`: Unique appendix identifier
- `title`: Appendix title
- `file`: LaTeX source file name
- `content`: Brief description of appendix content
- `pages`: Estimated page count

**File Naming Convention**:
- Pattern: `appendix-{letter}-{short-title}.tex`
- Letter: a, b, c, d, e
- Example: `appendix-a-reference.tex`

---

## Modifying the TOC

### Adding a New Chapter

1. **Determine the part**: Decide which part the chapter belongs in
2. **Find insertion point**: Identify where in the sequence it should go
3. **Update chapter list**: Insert new chapter entry
4. **Renumber if needed**: Update chapter numbers for subsequent chapters
5. **Update parts**: Ensure `chapters` array in correct part includes new chapter

Example - Adding a new chapter 7 about timers:

```yaml
chapters:
  # ... chapters 1-6
  - id: "chapter-07"
    title: "Timer and PWM Implementation"
    file: "chapter-07-timer.tex"
    part: 2
  # ... chapters 8+
```

### Moving a Chapter

1. Update the `part` field to new part number
2. Update `chapters` arrays in both parts (remove from old, add to new)
3. Ensure chapter sequence is correct

### Renumbering Chapters

When inserting or removing chapters:

1. Update all `id` fields: `chapter-01`, `chapter-02`, etc.
2. Update all `file` fields to match new numbers
3. Update file names in filesystem to match
4. Update cross-references throughout the book

**Important**: Always update the archived version when making structural changes.

---

## Cross-Referencing Chapters

In LaTeX, reference chapters using:

```latex
\chapter{Title}
\label{ch:chapter-01}

% Later in the book:
As discussed in Chapter \ref{ch:chapter-01}...
```

**Mapping from TOC to LaTeX**:
- `chapter-01` in YAML → `ch:chapter-01` in LaTeX
- `appendix-a` in YAML → `ap:appendix-a` in LaTeX (convention)

---

## Validation Checklist

Before committing TOC changes:

- [ ] All `id` fields are unique
- [ ] All `file` names are unique
- [ ] Chapter IDs are sequential (chapter-01, chapter-02, ...)
- [ ] Each chapter belongs to exactly one part
- [ ] All parts list correct chapter numbers
- [ ] Total chapters match across all parts
- [ ] Appendix IDs are sequential (appendix-a, appendix-b, ...)
- [ ] Version number incremented if structure changed
- [ ] Archive created if structure changed

---

## Reference Files

### Complete TOC Metadata

The `.archive/TOC_v1.0_complete.yml` file contains:
- Detailed chapter metadata (difficulty, page estimates, prerequisites)
- Section-by-section breakdowns with content focus
- Build/flash example counts for each chapter
- Code example counts and diagram specifications
- Cross-reference mappings
- Project example structure
- LLM processing metadata

**Usage**: Reference this file during book writing for detailed chapter planning.

### Chapter Count Validation

```bash
# Count total chapters
grep "^- id: \"chapter-" book/TOC.yml | wc -l

# List all chapter files
grep "file:" book/TOC.yml | awk '{print $2}'

# Validate part assignments (requires yq)
echo "Part 1:"; yq '.parts[0].chapters | length' book/TOC.yml
echo "Part 2:"; yq '.parts[1].chapters | length' book/TOC.yml
echo "Part 3:"; yq '.parts[2].chapters | length' book/TOC.yml
echo "Part 4:"; yq '.parts[3].chapters | length' book/TOC.yml
```

---

## Book Structure Quick Reference

### Current Book Structure

**Part I: PAC Generation and Foundation (Chapters 1-3)**
- Chapter 1: Introduction to Embedded Rust
- Chapter 2: SVD Files and PAC Generation
- Chapter 3: Build Systems and Cross-Compilation

**Part II: Hardware Abstraction Layer (Chapters 4-9)**
- Chapter 4: GPIO Implementation
- Chapter 5: Clock Management
- Chapter 6: USART Communication
- Chapter 7: I2C Protocol
- Chapter 8: SPI Master Mode
- Chapter 9: Advanced Peripherals (DMA, ADC, Power)

**Part III: Wireless Protocol Stacks (Chapters 10-14)**
- Chapter 10: Test-Driven Development
- Chapter 11: RAIL Foundation
- Chapter 12: Bluetooth 5.3 LE
- Chapter 13: Zigbee 3.0
- Chapter 14: Thread and Matter

**Part IV: BSP, Testing, and Production (Chapters 15-18)**
- Chapter 15: Board Support Packages
- Chapter 16: Documentation Strategies
- Chapter 17: Production Deployment
- Chapter 18: Complete Project Walkthrough

**Appendices (A-E)**
- Appendix A: EFR32MG24 Reference Manual Guide
- Appendix B: Rust Embedded Ecosystem
- Appendix C: Troubleshooting
- Appendix D: Complete Code Examples
- Appendix E: Wireless Protocol Comparison

**Total**: 18 chapters + 5 appendices, ~830 pages

---

## Troubleshooting

### Missing Chapter Files

**Error**: `! LaTeX Error: File 'chapter-NN.tex' not found`

**Solution**: Ensure:
1. File exists in `book/latex/` directory
2. File name matches exactly (including case)
3. File is included in `main.tex`

### Duplicate Chapter IDs

**Error**: Conflicting cross-references

**Solution**: Check for duplicate `id` fields in TOC:

```bash
grep "id: \"chapter-" book/TOC.yml | sort | uniq -d
```

### Chapter Not in Correct Part

**Problem**: Chapter appears in table of contents but under wrong part

**Solution**: Verify `part` field in chapter entry matches the part where it's listed in `chapters` array.

---

## Document Version

**TOC Instructions Version**: 1.0.0
**Last Updated**: 2026-01-05
**Applies to**: TOC.yml version 1.0.0 and above
---

# Book Configuration and Build System

## Book-Level Configuration

### LaTeX Packages

The book uses the following LaTeX packages (defined for the build system):

```
tikz, pgfplots, minted, hyperref, geometry, listings, xcolor, amsmath,
amssymb, graphicx, caption, subcaption, booktabs, longtable, fancyvrb
```

**Key packages**:
- **minted**: Syntax highlighting for code blocks
- **tikz**: Diagram generation
- **hyperref**: PDF hyperlinks and cross-references

### Code Highlighting Configuration

```yaml
style: "monokai"
language: "rust"
show_line_numbers: true
break_lines: true
font_size: "small"
baselinestretch: 0.8
```

**Settings**:
- Monokai color scheme for syntax highlighting
- Rust language highlighting by default
- Small font size to fit more code per page
- Tight line spacing (0.8) to maximize content density

### Diagram Configuration

**TikZ Libraries**:
```
shapes.geometric, arrows.meta, positioning, fit, calc, shadows, patterns
```

**Mermaid Diagram Types**:
```yaml
- flowchart
- sequence
- state
- gantt
- class
```

### Book Formatting

```yaml
paper_size: "a4"
font_size: "11pt"
two_sided: true
chapter_style: "default"
code_block_background: "lightgray!5"
margin_notes: true
```

**Features**:
- A4 paper size
- 11-point main font
- Two-sided printing
- Margin notes enabled for tips and warnings

---

# LLM Processing and Metadata

## Expected Questions (FAQ Entries)

The book tracks common reader questions and their answer locations:

1. **"Why implement GPIO first?"** → Chapter 4 introduction
2. **"How do I choose the correct SVD file?"** → Chapter 2 SVD selection
3. **"What's the difference between USART and EUSART?"** → Chapter 8
4. **"How do I implement embedded-hal traits?"** → Chapter 4
5. **"What testing strategies work for embedded?"** → Chapter 10
6. **"Which wireless protocol should I choose?"** → Appendix E

## Cross-Reference Mappings

Important chapter dependencies tracked for consistency:

| From Chapter | To Chapter | Reason |
|--------------|------------|--------|
| GPIO (Ch 4) | Build Systems (Ch 3) | Critical sections explained |
| USART (Ch 6) | Clocks (Ch 5) | USART depends on clock config |
| Bluetooth (Ch 12) | RAIL (Ch 11) | Bluetooth builds on RAIL |
| Zigbee (Ch 13) | RAIL (Ch 11) | Zigbee builds on RAIL |
| Thread (Ch 14) | RAIL (Ch 11) | Thread builds on RAIL |

## Code Snippet Tracking

Metadata for tracking code provenance:

```yaml
should_track_origin: true
origin_format: "file:line_number"
book_project_mapping: true
update_strategy: "periodic_sync_from_actual_implementation"
```

**Purpose**: Maintain traceability between book examples and actual EFR32MG24 implementation.

---

# Project Example Structure

## Example Projects Organization

The book includes runnable example projects for key chapters:

### Chapter 1: Bare-Metal
- **Location**: `chapter-01-bare-metal/`
- **Files**: Cargo.toml, src/main.rs, .cargo/config.toml
- **Build examples**: 3
- **Purpose**: First embedded Rust program without hardware

### Chapter 2: PAC Generation
- **Location**: `chapter-02-pac/`
- **Files**: Cargo.toml, build.rs, src/lib.rs, svd/
- **Build examples**: 3
- **Purpose**: SVD processing and PAC crate generation

### Chapter 4: GPIO
- **Location**: `chapter-04-gpio/`
- **Files**: src/gpio/mod.rs, examples/led_blink.rs, examples/button_input.rs
- **Build examples**: 5
- **Flash examples**: 5
- **Purpose**: GPIO with LED blink and button input

### Chapter 6: USART
- **Location**: `chapter-06-usart/`
- **Files**: src/usart/mod.rs, examples/serial_echo.rs, examples/printf_demo.rs
- **Build examples**: 6
- **Flash examples**: 6
- **Purpose**: Serial communication and debugging

### Chapter 8: SPI
- **Location**: `chapter-08-spi/`
- **Files**: src/spi/mod.rs, examples/spi_display.rs, examples/flash_access.rs
- **Build examples**: 8
- **Flash examples**: 8
- **Purpose**: SPI displays and memory access

### Chapter 12: Bluetooth
- **Location**: `chapter-12-bluetooth/`
- **Files**: examples/ble_beacon.rs, examples/ble_peripheral.rs, examples/ble_central.rs
- **Build examples**: 6
- **Flash examples**: 6
- **Purpose**: Bluetooth LE applications

### Chapter 13: Zigbee
- **Location**: `chapter-13-zigbee/`
- **Files**: examples/zigbee_coordinator.rs, examples/zigbee_router.rs, examples/zigbee_end_device.rs
- **Build examples**: 8
- **Flash examples**: 8
- **Purpose**: Zigbee mesh networking
- **Note**: Requires multiple devices

### Chapter 15: BSP
- **Location**: `chapter-15-bsp/`
- **Files**: src/board.rs, examples/board_blink.rs, examples/board_sensors.rs
- **Build examples**: 6
- **Flash examples**: 6
- **Purpose**: Board-specific abstractions

### Complete Workspace
- **Location**: `complete-workspace/`
- **Files**: Full workspace with all crates and examples
- **Build examples**: 15
- **Flash examples**: 15
- **Purpose**: End-to-end implementation (Chapter 18)

## Chapter-to-Example Mapping

```yaml
chapter-02 → chapter-02-pac/
chapter-04 → chapter-04-gpio/
chapter-12 → chapter-12-bluetooth/
chapter-18 → complete-workspace/
```

---

# Book Writing Guidelines

## Writing Style

- Conversational but technical
- Explain the "why" not just the "how"
- Show complete working examples
- Include common mistakes and how to avoid them
- Balance theory with practice

## Code Inclusion Standards

- Always show complete, compilable examples
- Include Cargo.toml dependencies
- Show expected output where applicable
- Use small font size to fit more code
- Reference line numbers in explanations

## Diagram Standards

- Use Tikz for architectural diagrams
- Use mermaid for flowcharts and sequences
- Include legend for all diagrams
- Show before/after states when applicable
- Color-code different components

## Pedagogical Approach

- Start with simple concepts, build complexity
- Show the problem before the solution
- Compare different approaches
- Include exercises at chapter ends
- Provide summaries of key takeaways

## Build/Flash Integration

- Every chapter has build examples
- Chapters with hardware integration include flash examples
- Show both debug and release builds
- Include size comparisons
- Show output verification methods

---

# Remote Compilation Configuration

## Container Environment

**Image**: `rust-latex:latest`

**Required Packages**:
```
texlive-full, texlive-latex-extra, texlive-fonts-extra,
texlive-science, texlive-pictures, pygments, poppler-utils, rsync
```

## Build Commands

The book uses a 4-pass LaTeX build:

1. `pdflatex -shell-escape main.tex`
2. `bibtex main`
3. `pdflatex -shell-escape main.tex`
4. `pdflatex -shell-escape main.tex`

Note: `-shell-escape` required for minted syntax highlighting and external tools.

## Sync Method

```bash
rsync -avz --progress book/latex/ remote:/build/
```

**Purpose**: Synchronize LaTeX source to remote build environment before compilation.

---

## Version History

### Version 1.0.0 (Current)

**Structural Changes**:
- Initial complete structure
- 4 parts: PAC/Foundation, HAL, Wireless, BSP/Production
- 18 chapters + 5 appendices
- All wireless protocols: Bluetooth, Zigbee, Thread, Matter
- Build/flash examples integrated throughout
- Configuration separated from TOC.yml

**TOC.yml**: Clean table of contents only (847 lines)
**TOC_INSTRUCTIONS.md**: Comprehensive guidelines and configuration (separate file)
