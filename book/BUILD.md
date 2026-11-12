# Building the book

The book is LaTeX with `minted`, so it needs `-shell-escape`, and it uses box-drawing
characters in the code blocks, so it needs **XeLaTeX**. `pdflatex` cannot build it.

No LaTeX lives in the development container; the compile runs in a `texlive` container on a
Linux host, reached over SSH as `$BUILD_HOST`:

```bash
# 1. copy the sources over. --strip-components=1 is NOT optional: without it the
#    archive lands as efr32book/book/latex/ while the container compiles
#    efr32book/latex/, and every build silently compiles the previous copy.
tar cf - book/latex | ssh "$BUILD_HOST" \
    'cd ~/podman/latex-remote/efr32book && tar xf - --strip-components=1'

# 2. prove the copy landed before compiling anything
ssh "$BUILD_HOST" 'cd ~/podman/latex-remote/efr32book && md5sum \
    latex/chapters/chapter-01-introduction/01.08-layer-boundaries.tex'
md5sum book/latex/chapters/chapter-01-introduction/01.08-layer-boundaries.tex   # must match

# 3. refuse the mistakes this book has already paid for (cheap, and it names the line)
python3 book/check-tex.py           # must print "0 defect(s)"

# 4. compile (twice, for the table of contents and cross-references)
ssh "$BUILD_HOST" 'podman exec --workdir /workspace/efr32book/latex latex-compiler-shared \
    xelatex -shell-escape -interaction=nonstopmode main.tex'

# 5. bring the PDF back
scp "$BUILD_HOST":~/podman/latex-remote/efr32book/latex/main.pdf .
```

If the container is stopped: `ssh "$BUILD_HOST" 'podman start latex-compiler-shared'`.

**What "it builds" means here**: `rc=0`, zero `! ` errors in `main.log`, and zero
`Missing character` warnings. The last one matters — a missing glyph does not fail the
build, it just silently deletes a character from the page (this book lost 544 of them
once: every `─ │ ├ └ ✓` in every code block). Check the page count moved when it should:

```bash
ssh "$BUILD_HOST" 'podman exec --workdir /workspace/efr32book/latex latex-compiler-shared \
    sh -c "grep -c \"^! \" main.log; grep -c \"Missing character\" main.log; \
           grep -o \"Output written on main.pdf ([0-9]* pages\" main.log | tail -1"'
```

Current: **136 pages, 0 errors, 0 missing characters, 0 check-tex defects**.

## Two traps, both of which produced a green build and a wrong PDF

1. **A stale source tree** (2026-11-12). The copy command omitted `--strip-components=1`, so the
   new sources went to `efr32book/book/latex/` and the container kept compiling
   `efr32book/latex/`. Every build reported success, and the PDF was one revision behind — a new
   section "built" without appearing. Step 2 above exists because of this: a build that succeeds
   is not evidence that it compiled what you wrote.
2. **`\codeinline{}` inside an argument** (`\caption{}`, `\textbf{}`, a section title). It is a
   verbatim-like command, so LaTeX refuses it with *"Verbatim-like command '\oldstylenums' illegal
   in argument"* — a message that names an unrelated macro and points at the wrong line. Keep
   `\codeinline` in the body, use plain text inside captions and headings.
