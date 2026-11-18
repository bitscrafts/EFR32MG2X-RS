#!/usr/bin/env python3
"""Extract every register description from the EFR32xG24 Reference Manual.

Why: the vendor SVD carries `<description>No Description</description>` for 6,859 register structs, and
svd2rust copies it faithfully, so docs.rs shows "No Description" for almost every register in the crate. The
manual has the real ones, in the "Register Map" table of each peripheral chapter:

    Offset   Name              Type   Description
    0x010    ACMP_CTRL         RW     Control Register

Output: JSON on stdout, {"ACMP_CTRL": "Control Register", ...}, plus a count of what could not be resolved.

ESTADO (2026-11-18): a extracção funciona -- ACMP_CTRL -> "Control Register", CMU_CLKEN0 -> "Clock Enable
Register 0" -- mas cobre 294 registos de 31 capítulos, e o manual tem 103 capítulos. Falta: (a) casar os capítulos
cujo "Register Map" tem outro título, (b) ler todas as páginas de cada tabela e não duas (as do GPIO ocupam
quatro), (c) aplicar o mapa aos 1 681 ficheiros. NÃO usar o JSON como se estivesse completo.

Usage: python3 scripts/rm-register-map.py [--saida caminho.json]
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RM = ROOT / "docs" / "vendor" / "efr32xg24-rm.pdf"

# The PDF gives every table cell on its own line, so this works on the token stream, not on lines:
# 0x010 / ACMP_CTRL / RW / [short label] / Control Register
OFFSET = re.compile(r"^0x[0-9A-Fa-f]{2,3}$")
NOME = re.compile(r"^[A-Z][A-Z0-9_]{2,30}$")
TIPO = re.compile(r"^(R|W|RW|R/W)$")


def registos_do_texto(tokens):
    """Yield (register name, description) from a page's token stream."""
    i = 0
    while i < len(tokens):
        if OFFSET.match(tokens[i]) and i + 3 < len(tokens) and NOME.match(tokens[i + 1]):
            nome = tokens[i + 1]
            j = i + 2
            if j < len(tokens) and TIPO.match(tokens[j]):
                j += 1
                # a short field name in the Type column ("ENABLE", "CONFIG") may follow
                if j < len(tokens) and re.fullmatch(r"[A-Z][A-Z0-9_]{1,14}", tokens[j]):
                    j += 1
                if j < len(tokens):
                    desc = tokens[j].strip()
                    if desc and not OFFSET.match(desc) and not desc.lower().startswith("reserved"):
                        yield nome, desc
                        i = j
        i += 1


def capitulos(toc):
    """Level-2 entries that are a peripheral chapter: 'ACMP - Analog Comparator'."""
    for lvl, titulo, pag in toc:
        m = re.match(r"^(?:[\d.]+\s+)?([A-Z][A-Z0-9_]{1,11})\s+-\s+(.+)$", titulo.strip())
        if m and lvl <= 2:
            yield m.group(1), pag


def main():
    import pymupdf

    d = pymupdf.open(RM)
    toc = d.get_toc()
    mapa = {}
    capitulos_lidos = 0
    for sigla, pag in capitulos(toc):
        # o register map de cada capítulo é a secção que começa com "<n>.x  <SIGLA> Register Map"
        pagina = None
        for lvl, titulo, p in toc:
            if f"{sigla} Register Map" in titulo:
                pagina = p
                break
        if not pagina:
            continue
        capitulos_lidos += 1
        texto = d[pagina - 1].get_text() + "\n" + (d[pagina].get_text() if pagina < d.page_count else "")
        tokens = [l.strip() for l in texto.splitlines() if l.strip()]
        for nome, desc in registos_do_texto(tokens):
            mapa.setdefault(nome, desc)
    saida = {"registos": mapa}
    destino = None
    if "--saida" in sys.argv:
        destino = Path(sys.argv[sys.argv.index("--saida") + 1])
        destino.write_text(json.dumps(saida, ensure_ascii=False, indent=1))
        print(f"{len(mapa)} registos de {capitulos_lidos} capítulos -> {destino}", file=sys.stderr)
    else:
        print(json.dumps(saida, ensure_ascii=False, indent=1))


if __name__ == "__main__":
    main()
