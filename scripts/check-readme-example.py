#!/usr/bin/env python3
"""Compile the example printed in a crate's README, the way a stranger would use it.

Why this exists: efr32mg2x-pac 0.1.0 was published with a "smallest program" snippet that did
`use panic_halt as _;` while the dependency list above it had no panic-halt. Every check we had read the crate --
the gate built the crate's own examples, check-publishable read the README's links, the dry run compiled the
package -- and none of them read the two blocks as a pair. A reader pasting the snippet got:

    error[E0432]: unresolved import `panic_halt`
    error: `#[panic_handler]` function required, but not found

Usage: check-readme-example.py <README> <dir-temporario> [--target <triple>]
Exit 0 when the snippet builds, 1 when it does not (with the compiler's own words on stdout).
"""
import re
import subprocess
import sys
from pathlib import Path


def blocos(texto):
    return re.findall(r"```([\w,]*,?[\w]*)\n(.*?)```", texto, re.S)


def main():
    if len(sys.argv) < 3:
        print("usage: check-readme-example.py <README> <dir> [--target <triple>]")
        return 2
    readme = Path(sys.argv[1])
    destino = Path(sys.argv[2])
    alvo = sys.argv[sys.argv.index("--target") + 1] if "--target" in sys.argv else None

    texto = readme.read_text()
    bs = blocos(texto)

    deps = next((b for lang, b in bs if lang.startswith("toml") and "[dependencies]" in b), None)
    codigo = next((b for lang, b in bs if lang.startswith("rust") and "no_main" in b), None)
    if deps is None or codigo is None:
        print("FAIL: the README has no [dependencies] block paired with a no_main program")
        return 1

    # Uma crate vazia com exactamente aquilo que o README manda pôr no Cargo.toml.
    (destino / "src").mkdir(parents=True, exist_ok=True)
    (destino / "Cargo.toml").write_text(
        '[package]\nname = "readme-example-check"\nversion = "0.0.0"\nedition = "2021"\n\n' + deps.strip() + "\n"
    )
    (destino / "src" / "main.rs").write_text(codigo)

    cmd = ["cargo", "build"]
    if alvo:
        cmd += ["--target", alvo]
    r = subprocess.run(cmd, cwd=destino, capture_output=True, text=True)
    if r.returncode == 0:
        print(f"OK: the README's example builds against {deps.count(chr(10)) + 1} lines of dependencies")
        return 0
    for linha in (r.stderr or "").splitlines():
        if linha.startswith("error"):
            print(linha)
    print("FAIL: the README's example does not build")
    return 1


if __name__ == "__main__":
    sys.exit(main())
