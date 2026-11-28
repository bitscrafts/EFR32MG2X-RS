#!/usr/bin/env python3
"""A moved core lives in the HAL; the library compiles it by #[path]; there is one copy.

Counting "lines of its own" means counting *code*, and declarations are not code. Measured 2026-11-28: the first
version of this check called `libraries/i2c-core` a copy because of `pub mod sim;`, and `libraries/aes-core` a copy
because of a `pub use ...::{a, b, c};` whose continuation lines do not begin with `pub use`. Both are one-item
declarations, exactly what a shim is made of. The self-test now carries both as cases that must PASS, next to the
real copy that must FAIL -- because the way to loosen a check safely is to leave it failing on the thing it exists
to catch.

The item that uses this (C1) exists because the HAL cannot be published while it depends on
`libraries/` crates by path. The fix moves the source into the HAL and leaves the library as a
harness that compiles that same file, so the host tests keep running and nothing is duplicated.

`--self-test` builds three fixtures in a temp dir and requires: a good shim passes, a copy fails,
a missing target fails. A check without that control is a check that can pass while broken.
"""
import re, sys, tempfile, pathlib

RAIZ = pathlib.Path(__file__).resolve().parent.parent
LIMITE_SHIM = 0  # um shim nao tem uma linha de codigo: so o #[path], o mod e o pub use

def contar_codigo(texto: str) -> list[str]:
    """The lines that are code, not the shim's own declarations.

    A `pub use` may span lines, and only its first line carries the keyword; a `pub mod` declares a module as
    surely as `mod` does. Both are the shim's business, so both are skipped -- a multi-line re-export is tracked
    until its terminating semicolon rather than judged line by line.
    """
    codigo, dentro_do_uso = [], False
    for linha in texto.splitlines():
        tira = linha.strip()
        if dentro_do_uso:
            if tira.endswith(";"):
                dentro_do_uso = False
            continue
        if not tira or tira.startswith(("//", "#!", "#[path", "#[")):
            continue
        if tira.startswith(("mod ", "pub mod ", "pub use ", "pub(crate) use ", "use ")):
            dentro_do_uso = not tira.endswith(";")
            continue
        codigo.append(linha)
    return codigo


def analisar(lib: pathlib.Path, raiz: pathlib.Path):
    """Devolve (ok, mensagem). Um crate sem #[path] nao foi movido: nao e erro."""
    texto = lib.read_text()
    m = re.search(r'#\[path\s*=\s*"([^"]+)"\]', texto)
    if not m:
        return True, "not moved"
    alvo = (lib.parent / m.group(1)).resolve()
    if not alvo.is_file():
        return False, f"{lib} names a file that does not exist: {m.group(1)}"
    esperado = (raiz / "crates" / "efr32mg2x-hal" / "src").resolve()
    if esperado not in alvo.parents:
        return False, f"{lib} compiles {alvo}, which is outside crates/efr32mg2x-hal/src"
    uteis = contar_codigo(texto)
    if len(uteis) > LIMITE_SHIM:
        return False, f"{lib} holds code of its own ({len(uteis)} lines); it must be a shim"
    return True, f"shim -> {alvo.relative_to(raiz)}"

def correr(raiz: pathlib.Path):
    libs = sorted((raiz / "libraries").glob("*/src/lib.rs"))
    movidos, falhas = 0, []
    for lib in libs:
        ok, msg = analisar(lib, raiz)
        if not ok:
            falhas.append(msg)
        elif msg != "not moved":
            movidos += 1
    return movidos, falhas

def self_test():
    with tempfile.TemporaryDirectory() as td:
        r = pathlib.Path(td)
        core = r / "crates" / "efr32mg2x-hal" / "src" / "core"
        core.mkdir(parents=True)
        (core / "spi.rs").write_text("pub fn sdiv(x: u32) -> u32 { x }\n")
        lib = r / "libraries" / "spi-core" / "src"
        lib.mkdir(parents=True)
        bom = '#[path = "../../../crates/efr32mg2x-hal/src/core/spi.rs"]\nmod spi;\npub use spi::*;\n'
        casos = [
            ("bom", bom, True),
            ("copia", bom + "pub fn sdiv(x: u32) -> u32 { x }\n", False),
            ("ausente", '#[path = "../../../crates/efr32mg2x-hal/src/core/nada.rs"]\nmod nada;\n', False),
            ("fora", '#[path = "/etc/hostname"]\nmod nada;\n', False),
            # Declarations are not code: a submodule and a re-export that wraps are what a shim legitimately holds.
            ("declara_modulo", bom + "pub mod sim;\n", True),
            ("reuso_multilinha", bom + "pub use spi::{um, dois,\n    tres, quatro};\n", True),
            # ... and a real line of code after them is still a copy.
            ("codigo_depois_das_declaracoes",
             bom + "pub mod sim;\npub use spi::{um, dois};\nconst LIMITE: u32 = 4;\n", False),
        ]
        for nome, conteudo, esperado in casos:
            (lib / "lib.rs").write_text(conteudo)
            ok, msg = analisar(lib / "lib.rs", r)
            estado = "PASS" if ok == esperado else "FAIL"
            print(f"  {estado}  {nome}: {msg}")
            if ok != esperado:
                return 1
    print("  OK: the check fails on a copy, on a missing target and on a path outside the tree, and accepts the")
    print("      declarations a shim is made of")
    return 0

if __name__ == "__main__":
    if "--self-test" in sys.argv:
        sys.exit(self_test())
    movidos, falhas = correr(RAIZ)
    for f in falhas:
        print(f"FAIL  {f}")
    if falhas:
        print(f"VERDICT: FAIL ({len(falhas)} problem(s))")
        sys.exit(1)
    print(f"OK: {movidos} core(s) moved, one copy each")
