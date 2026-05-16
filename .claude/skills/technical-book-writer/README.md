# Technical Book Writer Skill

**Expert technical book writer with LaTeX/TikZ proficiency and deep research capabilities**

## Overview

This skill provides comprehensive technical writing capabilities for creating professional documentation, textbooks, and reference materials. It combines deep subject-matter expertise, LaTeX/TikZ mastery, pedagogical skills, and thorough research capabilities.

## 10 Core Competencies

1. **Deep Subject-Matter Expertise** - Thorough research using WebSearch/WebFetch
2. **Excellent Technical Writing Skills** - Clear, precise, pedagogical content
3. **Ability to Explain Complexity Simply** - Scaffolded learning, multiple perspectives
4. **Information Architecture and Organization** - Professional book structure
5. **Research and Fact-Verification Skills** - Multi-source verification, citations
6. **Tool Proficiency** - Expert LaTeX and TikZ code generation
7. **Editing and Revision Discipline** - Multi-pass editing protocol
8. **Communication and Collaboration Skills** - Reader engagement, clarity
9. **Empathy for Your Reader** - Audience-aware, accessible content
10. **Lifelong Learning and Curiosity** - Continuous research, current info (2026)

## How to Use

### Invocation

```bash
# Use the Skill tool in Claude Code
/technical-book-writer "Write a chapter about GPIO configuration"
```

Or simply ask:
```
"Use the technical book writer skill to create a chapter about USART serial communication"
```

### Example Requests

**Simple request**:
```
Write a chapter about EFR32MG24 memory layout
```

**Detailed request**:
```
Write a chapter about the EFR32MG24 bootloader and memory layout.
Target audience: intermediate embedded developers.
Include TikZ diagrams showing:
- Memory map with bootloader and application regions
- Flash organization
- Address ranges with annotations
```

**Research-heavy request**:
```
Write a chapter about Bluetooth LE 5.3 features on EFR32MG24.
Research the latest specifications and Silicon Labs implementation.
Include practical examples and code snippets.
```

## What You Get

### LaTeX Output
Complete, compilable LaTeX chapters with:
- Proper document structure
- Custom theorem environments
- Professional tables (booktabs)
- Code listings (with syntax highlighting)
- Cross-references (cleveref)
- Bibliography (BibTeX)

### TikZ Diagrams
Professional visualizations:
- System architecture diagrams
- Timing diagrams
- Memory layout maps
- Block diagrams
- State machines
- Data flow diagrams

### Content Structure
- **Introduction**: Motivation, objectives, prerequisites
- **Body**: Scaffolded content with examples
- **Summary**: Key takeaways
- **Exercises**: Graded difficulty
- **Further reading**: Curated references

## Features

### Automatic Research
- Searches authoritative sources (IEEE, official docs, standards)
- Cross-references multiple sources
- Verifies technical accuracy
- Includes proper citations

### Multi-Level Audience Support
- **Beginner**: Assumes no prior knowledge
- **Intermediate**: Builds on fundamentals
- **Advanced**: Focuses on subtleties and optimization

### Pedagogical Excellence
- Scaffolded learning progression
- Multiple explanation approaches
- Concrete examples
- Common pitfalls warnings
- Self-assessment questions

## Examples

### Example 1: Memory Layout Chapter

**Request**:
```
Write a chapter about EFR32MG24 memory layout with TikZ diagram
```

**Output** (excerpt):
```latex
\chapter{Memory Layout and Flash Organization}
\label{ch:memory-layout}

\section{Introduction}

The EFR32MG24 uses a Harvard architecture with separate
code and data memory spaces...

\begin{figure}[htbp]
\centering
\begin{tikzpicture}[scale=0.8]
  \draw[thick] (0,0) rectangle (4,8);
  \fill[blue!20] (0,6) rectangle (4,8);
  \node at (2,7) {Bootloader};
  \node[right] at (4.2,7) {0x08000000 - 0x08005FFF};
  ...
\end{tikzpicture}
\caption{EFR32MG24 memory layout}
\label{fig:memory-layout}
\end{figure}
```

### Example 2: USART Communication

**Request**:
```
Write a section about USART baud rate calculation with examples
```

**Output** (excerpt):
```latex
\section{Baud Rate Configuration}

The USART baud rate is configured using the CLKDIV register...

\begin{equation}
\text{CLKDIV} = \frac{f_{PCLK}}{16 \times \text{baud rate}} - 1
\label{eq:clkdiv}
\end{equation}

\begin{lstlisting}[language=Rust, caption={Baud rate calculation}]
fn calculate_clkdiv(pclk_freq: u32, baud: u32) -> u32 {
    (pclk_freq / (16 * baud)) - 1
}
\end{lstlisting}
```

## Best Practices

### When Requesting Content

1. **Be specific**: State topic, audience level, desired format
2. **Request diagrams**: Explicitly ask for TikZ visualizations
3. **Specify audience**: Beginner, intermediate, or advanced
4. **Provide context**: Include relevant constraints or requirements
5. **Emphasize research**: Request focus on specific authoritative sources

### Quality Expectations

The skill ensures:
- All technical claims verified
- Code examples tested
- Diagrams accurate and labeled
- Cross-references correct
- Consistent terminology
- Complete citations
- Comprehensive content

## Integration with Project

Perfect for:
- **HAL documentation**: Document peripheral APIs
- **Tutorial chapters**: Getting-started guides
- **Reference material**: Register layouts, configurations
- **Examples**: Explain complex embedded concepts
- **Architecture docs**: Visualize system design

## Technical Capabilities

### Remote PDF Compilation (NEW)

**Compile LaTeX remotely** using containerized build environment:

```bash
cd .claude/skills/technical-book-writer/scripts
./build-book-remote.sh    # Full 3-pass build
./build-book-quick.sh     # Quick error check
./pull-pdf.sh            # Pull compiled PDF
```

**Features**:
- Containerized TeX Live environment (no local install needed)
- Automatic error detection and log retrieval
- 3-pass compilation for complete TOC/references
- SSH-based remote execution on lnx host

**Integration**: Use Bash tool to run build scripts after writing LaTeX content.

### LaTeX Packages Used
- `tikz` - Diagrams and visualizations
- `listings` - Code syntax highlighting
- `amsmath, amssymb, amsthm` - Mathematical notation
- `booktabs` - Professional tables
- `cleveref` - Smart cross-referencing
- `hyperref` - PDF links and bookmarks
- `microtype` - Typography improvements

### TikZ Libraries
- `positioning` - Relative positioning
- `shapes` - Custom node shapes
- `arrows` - Arrow styles
- `patterns` - Fill patterns
- `calc` - Coordinate calculations
- `circuits` - Circuit diagrams

### Code Languages Supported
- Rust (primary)
- C/C++
- Assembly
- Python
- Shell scripts
- Pseudo-code

## Maintenance

The skill automatically:
- Researches latest information
- Verifies technical accuracy
- Updates for current year (2026)
- Follows best practices
- Maintains consistent terminology

## File Structure

```
.claude/skills/technical-book-writer/
├── SKILL.md       # Main skill definition
└── README.md      # This file
```

## Notes

- Skill assumes current year is 2026-01-09
- All code examples are tested for correctness
- Citations follow standard academic format
- Diagrams use consistent visual language
- Content organized for learning, not just reference

## Support

For issues or enhancements:
1. Check the SKILL.md for detailed instructions
2. Verify your request is specific and clear
3. Ensure you've specified the target audience
4. Request diagrams explicitly if needed

---

**Created**: 2026-01-09
**Project**: EFR32MG24 Rust HAL Development
**Version**: 1.0
