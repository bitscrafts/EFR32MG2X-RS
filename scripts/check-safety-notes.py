#!/usr/bin/env python3
"""Every `unsafe` block needs a SAFETY note; every `unsafe fn` needs a `# Safety` section.

The note is the only thing a reviewer can audit: `unsafe { w.bits(x) }` says nothing about whether `x` fits the field,
and a crate that publishes 196 of them without one is asking to be trusted rather than read. This is the check that
keeps the sweep from being a one-off -- without it the notes decay the moment someone adds the next register write.

Rules, and they are the ones the code follows:

  * `unsafe { ... }`  -> a comment containing `SAFETY` in the eight lines above the block. Eight because a note that
    explains a register write sometimes needs three lines of field layout first.
  * `unsafe fn`      -> a `# Safety` section in the doc comment above, or `SAFETY` in the six lines above it when the
    contract is inherited from a trait (a test double implementing `cores::dma::Channel`, for instance).

Usage: `python3 scripts/check-safety-notes.py [crate-dir ...]`, default `crates/efr32mg2x-hal/src`.
Exit code 0 when every site is annotated; 1 with a list of the sites that are not.
"""

import pathlib
import re
import sys

DIRS = sys.argv[1:] or ["crates/efr32mg2x-hal/src"]
BLOCO_JANELA = 8
TRAIT_JANELA = 6

bloco = re.compile(r"unsafe\s*\{")
funcao = re.compile(r"unsafe\s+fn\s")

falhas = []
total = 0

for raiz in DIRS:
    for ficheiro in sorted(pathlib.Path(raiz).rglob("*.rs")):
        linhas = ficheiro.read_text(errors="ignore").splitlines()
        for i, linha in enumerate(linhas):
            if bloco.search(linha):
                total += 1
                antes = "\n".join(linhas[max(0, i - BLOCO_JANELA) : i])
                if "SAFETY" not in antes:
                    falhas.append(f"{ficheiro}:{i + 1}: unsafe block without a SAFETY note")
            elif funcao.search(linha):
                total += 1
                antes = "\n".join(linhas[max(0, i - TRAIT_JANELA) : i])
                # A `# Safety` section sits above the doc, which can be long; look at everything above the signature.
                documento = "\n".join(linhas[max(0, i - 30) : i])
                if "SAFETY" not in antes and "# Safety" not in documento:
                    falhas.append(f"{ficheiro}:{i + 1}: unsafe fn without a # Safety section")

if falhas:
    print("unsafe sites without their reasoning:")
    for f in falhas:
        print("  " + f)
    print(f"VERDICT: FAIL ({len(falhas)} of {total} unsafe sites are undocumented)")
    sys.exit(1)

print(f"VERDICT: PASS ({total} unsafe sites, every one with the invariant it relies on)")