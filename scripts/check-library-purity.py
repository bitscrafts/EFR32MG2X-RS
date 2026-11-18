#!/usr/bin/env python3
"""libraries/ must stay testable on the host: no hardware in the closure of any of its crates."""
import json, subprocess, sys, pathlib

HARD = ("efr32mg24", "efr32mg2x-pac", "efr32mg2x-hal", "efr32mg2x-radio",
        "efr32mg2x-rail-sys", "motion-detection", "cortex-m", "sl-")

raiz = pathlib.Path(__file__).resolve().parent.parent

def meta():
    o = subprocess.run(["cargo", "metadata", "--format-version", "1"],
                       cwd=raiz, capture_output=True, text=True)
    if o.returncode:
        sys.exit("FAIL: cargo metadata: " + o.stderr.strip()[:200])
    return json.loads(o.stdout)

def verificar(m, base):
    nomes = {p["id"]: p["name"] for p in m["packages"]}
    nos = {n["id"]: n for n in m["resolve"]["nodes"]}
    falhas, total = [], 0
    for p in m["packages"]:
        if not p["manifest_path"].startswith(str(base)):
            continue
        total += 1
        vistos, pilha = set(), [p["id"]]
        while pilha:
            i = pilha.pop()
            if i in vistos:
                continue
            vistos.add(i)
            pilha.extend(d["pkg"] for d in nos[i]["deps"])
        maus = sorted(nomes[i] for i in vistos if nomes[i].startswith(HARD))
        if maus:
            falhas.append((p["name"], maus))
    return total, falhas

def main():
    base = raiz / "libraries"
    total, falhas = verificar(meta(), base)
    for nome, maus in falhas:
        print(f"FAIL  {nome} reaches the hardware stack: {', '.join(maus)}")
    if falhas:
        print(f"VERDICT: FAIL ({len(falhas)} of {total} libraries are not host-testable)")
        return 1
    print(f"OK  {total} libraries: nothing from the hardware stack in any closure")
    return 0

if __name__ == "__main__":
    sys.exit(main())
