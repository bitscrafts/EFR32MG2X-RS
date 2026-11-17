#!/usr/bin/env python3
"""Treina a regressão linear bayesiana e acrescenta os coeficientes a `src/modelo.rs`.

## O modelo (o mesmo de `src/regress.rs`)

    y = w0 + w1·x + ruído,   ruído ~ N(0, σ²)   com prior  w ~ N(0, α⁻¹I)

Posterior:   A = αI + ΦᵀΦ/σ²        m = A⁻¹Φᵀy/σ²
Preditivo:   média = mᵀφ(x*)        variância = σ²·(1 + φᵀA⁻¹φ)

Guarda-se `m` (2 valores), `σ²` e `A⁻¹` (2×2) — 7 números no dispositivo, sem pontos de
treino em memória. É a razão de ser desta escolha em vez de um GP com kernel.

## Nota sobre o que este script NÃO faz

Não escolhe α nem σ² por optimização — são declarados na linha de comando e registados no
`vector_controle_reg.json`, para que o teste do Rust use os mesmos valores. Escolher por
evidência é a tarefa E1-P3 (optimização bayesiana) da especificação do produto.

Uso:
    uv run --with numpy python/treina_regressao.py --semente 1 --alpha 1.0 --sigma2 1.0
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np

AQUI = Path(__file__).resolve().parent
RAIZ = AQUI.parent


def gera_niveis_sinteticos(n: int, semente: int) -> tuple[np.ndarray, np.ndarray]:
    """Série de níveis (dB) com inércia — a mesma física que o modelo assume.

    Um campo sonoro não salta de 45 para 90 dB entre janelas consecutivas: a reverberação e
    a dinâmica da fonte impõem continuidade. É essa continuidade que o coeficiente w1 mede.
    """
    rng = np.random.default_rng(semente)
    x = np.empty(n + 1, dtype=np.float64)
    x[0] = 55.0
    for i in range(n):
        # passeio aleatório com reversão à média (AR(1) com φ = 0,95)
        x[i + 1] = 0.95 * x[i] + 0.05 * 55.0 + rng.normal(0.0, 1.5)
    return x[:-1], x[1:]


def ajusta(xs: np.ndarray, ys: np.ndarray, alpha: float, sigma2: float) -> dict:
    phi = np.column_stack([np.ones_like(xs), xs])          # φ = [1, x]
    a = alpha * np.eye(2) + (phi.T @ phi) / sigma2
    a_inv = np.linalg.inv(a)
    m = a_inv @ (phi.T @ ys) / sigma2
    return {"m": m, "sigma2": sigma2, "a_inv": a_inv, "alpha": alpha}


def prediz(modelo: dict, x: float) -> tuple[float, float]:
    phi = np.array([1.0, x])
    media = float(modelo["m"] @ phi)
    var = float(modelo["sigma2"] * (1.0 + phi @ modelo["a_inv"] @ phi))
    return media, var


def fmt(v: float) -> str:
    return f"{np.float32(v):.7e}f32"


def acrescenta_ao_modelo(caminho: Path, modelo: dict) -> None:
    """Acrescenta o bloco da regressão **sem destruir** o bloco do GMM (requisito R2)."""
    texto = caminho.read_text() if caminho.exists() else ""
    marca = "// --- regressão"
    if marca in texto:
        texto = texto[: texto.index(marca)].rstrip() + "\n"
    bloco = [
        "",
        marca + " (acrescentado por python/treina_regressao.py) ---",
        f"pub const REG_ALPHA: f32 = {fmt(modelo['alpha'])};",
        f"pub const REG_SIGMA2: f32 = {fmt(modelo['sigma2'])};",
        f"pub const REG_M: [f32; 2] = [{fmt(modelo['m'][0])}, {fmt(modelo['m'][1])}];",
        "pub const REG_A_INV: [[f32; 2]; 2] = [",
        f"    [{fmt(modelo['a_inv'][0][0])}, {fmt(modelo['a_inv'][0][1])}],",
        f"    [{fmt(modelo['a_inv'][1][0])}, {fmt(modelo['a_inv'][1][1])}],",
        "];",
        "",
    ]
    caminho.write_text(texto + "\n".join(bloco))
    print(f"acrescentado bloco da regressão a {caminho.relative_to(RAIZ)}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--entrada", type=Path, default=None, help="CSV com pares (nivel_t, nivel_t+1)")
    ap.add_argument("--semente", type=int, required=True)
    ap.add_argument("--alpha", type=float, default=1.0)
    ap.add_argument("--sigma2", type=float, default=1.0)
    ap.add_argument("--pontos", type=int, default=600)
    ap.add_argument("--src", type=Path, default=RAIZ / "src" / "modelo.rs")
    args = ap.parse_args()

    if args.entrada and args.entrada.exists():
        pares = np.loadtxt(args.entrada, delimiter=",")
        xs, ys = pares[:, 0], pares[:, 1]
        print(f"lido {args.entrada} — {len(xs)} pares")
    else:
        xs, ys = gera_niveis_sinteticos(args.pontos, args.semente)
        print(f"SEM SENSOR: {len(xs)} pares sintéticos (semente {args.semente})")

    if args.alpha <= 0.0 or args.sigma2 <= 0.0:
        raise SystemExit("alpha e sigma2 têm de ser positivos")

    modelo = ajusta(xs, ys, args.alpha, args.sigma2)
    acrescenta_ao_modelo(args.src, modelo)

    x_controle = float(xs[0])
    media, var = prediz(modelo, x_controle)
    controle = AQUI / "vector_controle_reg.json"
    controle.write_text(
        json.dumps(
            {
                "x": x_controle,
                "media": media,
                "variancia": var,
                "alpha": args.alpha,
                "sigma2": args.sigma2,
                "semente": args.semente,
            },
            indent=2,
        )
        + "\n"
    )
    print(f"escrito {controle.relative_to(RAIZ)} — média {media:.4f}, variância {var:.4f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
