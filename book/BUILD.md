# Building the book

The book is LaTeX with `minted`, so it needs `-shell-escape`, and it uses box-drawing
characters in the code blocks, so it needs **XeLaTeX**. `pdflatex` cannot build it.

No LaTeX lives in the development container; the compile runs through the `texlive`
container on `lnx`:

```bash
# 1. copy the sources over
tar cf - book/latex | ssh mvcorrea@lnx 'mkdir -p ~/podman/latex-remote/efr32book && \
    cd ~/podman/latex-remote/efr32book && tar xf -'

# 2. compile (twice, for the table of contents and cross-references)
ssh mvcorrea@lnx 'podman exec --workdir /workspace/efr32book/latex latex-compiler-shared \
    xelatex -shell-escape -interaction=nonstopmode main.tex'

# 3. bring the PDF back
scp mvcorrea@lnx:~/podman/latex-remote/efr32book/latex/main.pdf .
```

If the container is stopped: `ssh lnx 'podman start latex-compiler-shared'`.

**What "it builds" means here**: `rc=0`, zero `! ` errors in `main.log`, and zero
`Missing character` warnings. The last one matters — a missing glyph does not fail the
build, it silently deletes a character from the page, which is how an em-dash made a
sentence read "not arbitrary it reflects" and how a directory tree lost all its lines.

Last verified: 2026-11-11, 96 pages, 0 errors, 0 missing characters.
