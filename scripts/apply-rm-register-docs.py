#!/usr/bin/env python3
"""Replace the SVD's "No Description" on every register with the manual's own words.

The chain, and why each link is needed:

  SVD    tells which register each file is (`src/acmp0_ns/ctrl.rs` -> GPIO/acmp0_ns, register CTRL), which is the
         only reliable way to read a filename like `if_.rs` or `pa_dout.rs`.
  RM     the EFR32xG24 Reference Manual's register map tables give the description, keyed by the vendor's own name
         (`ACMP_CTRL`). Every page is scanned, because each chapter's table runs over two to four pages.
  source the generated `#[doc = "No Description\\n\\nYou can [read]..."]` has its first sentence replaced and the
         rest of the sentence left alone, so the svd2rust guidance survives.

Usage: apply-rm-register-docs.py [--check] [--verbose]
Exit 1 with --check when any register still says "No Description".
"""
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CRATE = ROOT / "crates" / "efr32mg2x-pac"
SRC = CRATE / "src"
SVD = CRATE / "svd" / "EFR32MG24B220F1536IM48.svd"
RM = ROOT / "docs" / "vendor" / "efr32xg24-rm.pdf"

OFFSET = re.compile(r"^0x[0-9A-Fa-f]{2,3}$")
NOME = re.compile(r"^[A-Z][A-Z0-9_]{2,30}$")
TIPO = re.compile(r"^[RWH]{1,3}(\s+[A-Z][A-Z0-9_]{1,14})?$")  # R, W, RW, RH, WH -- o manual usa RH para leitura com handshake


def familia(nome):
    return re.sub(r"\d+$", "", re.sub(r"_(S|NS)$", "", nome))


def do_svd():
    """(peripheral_lower, register_lower) -> register name, for every peripheral and register."""
    raiz = ET.parse(SVD).getroot()
    mapa = {}
    for p in raiz.findall(".//peripherals/peripheral"):
        pnome = p.findtext("name")
        for r in p.findall("./registers/register"):
            rnome = r.findtext("name")
            if pnome and rnome:
                mapa[(pnome.lower(), rnome.lower())] = rnome
    return mapa


def do_manual():
    """Register name as the vendor writes it (ACMP_CTRL) -> description, from every page."""
    import pymupdf

    doc = pymupdf.open(RM)
    mapa = {}
    # O nome pode vir sem prefixo ("CTRL") e nesse caso o prefixo é o do capítulo em que a página está. Sem isto,
    # metade das tabelas era descartada por nao conter "_" -- foi o que deixou a cobertura em 461 registos.
    capitulos = []
    for lvl, titulo, pag in doc.get_toc():
        m = re.match(r"^(?:[\d.]+\s+)?([A-Z][A-Z0-9_]{1,11})\s+-\s+", titulo.strip())
        if m and lvl <= 2:
            capitulos.append((pag, m.group(1)))
    capitulos.sort()
    for indice, pagina in enumerate(doc, start=1):
        sigla = ""
        for pag, s in capitulos:
            if pag <= indice:
                sigla = s
            else:
                break
        tokens = [l.strip() for l in pagina.get_text().splitlines() if l.strip()]
        i = 0
        while i < len(tokens):
            if OFFSET.match(tokens[i]) and i + 3 < len(tokens) and NOME.match(tokens[i + 1]):
                nome = tokens[i + 1]
                if "_" not in nome:
                    if not sigla:
                        i += 1
                        continue
                    nome = f"{sigla}_{nome}"
                j = i + 2
                # A celula do tipo pode trazer o rotulo junto ("RWH INTFLAG"), e o tipo pode ter tres letras.
                if TIPO.match(tokens[j]):
                    j += 1
                    m2 = re.match(r"^[RWH]{1,3}\s+[A-Z][A-Z0-9_]{1,14}$", tokens[j]) if j < len(tokens) else None
                    if j < len(tokens) and re.fullmatch(r"[A-Z][A-Z0-9_]{1,14}", tokens[j]):
                        j += 1
                    if j < len(tokens):
                        desc = tokens[j].strip()
                        if desc and not OFFSET.match(desc) and not desc.lower().startswith("reserved"):
                            mapa.setdefault(nome, desc)
                            i = j
            i += 1
    return mapa


def main():
    check = "--check" in sys.argv
    verbose = "--verbose" in sys.argv

    svd = do_svd()
    manual = do_manual()
    if verbose:
        print(f"svd: {len(svd)} registos | manual: {len(manual)} registos descritos")

    substituidos = sem_fonte = 0
    por_descricao = {}
    for pasta in sorted(p for p in SRC.iterdir() if p.is_dir()):
        perif = pasta.name
        for f in sorted(pasta.glob("*.rs")):
            corpo = f.read_text()
            if "No Description" not in corpo:
                continue
            reg = svd.get((perif, f.stem))
            if not reg:
                sem_fonte += 1
                continue
            desc = manual.get(f"{familia(perif.upper())}_{reg}")
            if not desc:
                sem_fonte += 1
                continue
            novo = corpo.replace('#[doc = "No Description\\n', f'#[doc = "{desc}\\n', 1)
            if novo == corpo:
                sem_fonte += 1
                continue
            if not check:
                f.write_text(novo)
            substituidos += 1
            por_descricao[desc] = por_descricao.get(desc, 0) + 1

    if check and substituidos:
        print(f"FAIL {substituidos} register(s) still say 'No Description' (run scripts/apply-rm-register-docs.py)")
        sys.exit(1)
    acao = "would replace" if check else "replaced"
    print(f"{acao}: {substituidos} register descriptions | without a source: {sem_fonte}")
    if verbose:
        for d, n in sorted(por_descricao.items(), key=lambda kv: -kv[1])[:12]:
            print(f"   {n:4} x {d}")


if __name__ == "__main__":
    main()
