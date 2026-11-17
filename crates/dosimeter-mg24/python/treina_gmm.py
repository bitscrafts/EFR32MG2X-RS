#!/usr/bin/env python3
"""Treina o GMM diagonal e escreve `src/modelo.rs` + o vector de controlo.

## O que faz, e porque assim

O dispositivo não treina: recebe coeficientes como arrays `const`. Este script é o lado do
host desse contrato — ajusta um GMM diagonal por EM e escreve os arrays prontos a compilar.

A verosimilhança calculada aqui é **a mesma fórmula** de `src/gmm.rs::log_verosimilhanca`:

    log p(x) = logsumexp_k [ ln(w_k) + Σ_d −0,5·(ln(2π) + ln(v_kd) + (x_d−μ_kd)²/v_kd) ]

Se as duas fórmulas divergirem, o teste de ida-e-volta em `tests/ida_e_volta.rs` falha — é
para isso que ele existe.

## Determinismo

`--semente` é obrigatório e controla a inicialização k-means++. Duas corridas com a mesma
semente produzem o mesmo `modelo.rs` byte a byte (critério R5 da spec-01).

Uso:
    uv run --with numpy python/treina_gmm.py --semente 1
    uv run --with numpy python/treina_gmm.py --entrada dados/features.csv --componentes 3
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np

AQUI = Path(__file__).resolve().parent
RAIZ = AQUI.parent

# Fronteiras de banda usadas por src/features.rs (12 bandas). Só servem para gerar dados
# sintéticos plausíveis: o treino real recebe as features já calculadas pelo firmware.
N_BANDAS = 12


def gera_features_sinteticas(n_janelas: int, semente: int) -> np.ndarray:
    """Duas paisagens acústicas: ruído de ventilador e voz, em dB por banda.

    Sintético de propósito — não há sensor ligado. A forma (média por banda + variação)
    segue o que se espera de um quarto com ventilador e de uma pessoa a falar: o ventilador
    é plano e estável, a voz tem mais energia nas bandas baixas e varia mais.
    """
    rng = np.random.default_rng(semente)
    n_por_estado = n_janelas // 2

    ventilador = np.full(N_BANDAS, -45.0) + np.linspace(0.0, 6.0, N_BANDAS)
    voz = np.array([-38.0, -36.0, -34.0, -40.0, -46.0, -50.0,
                    -54.0, -57.0, -60.0, -62.0, -64.0, -65.0])

    a = ventilador + rng.normal(0.0, 1.2, (n_por_estado, N_BANDAS))
    b = voz + rng.normal(0.0, 3.5, (n_por_estado, N_BANDAS))
    dados = np.vstack([a, b])
    rng.shuffle(dados)
    return dados


def inicializa_kmeans_plus_plus(x: np.ndarray, k: int, rng: np.random.Generator) -> np.ndarray:
    """k-means++ determinístico dado o gerador — evita componentes colapsadas."""
    n = x.shape[0]
    centros = [x[rng.integers(n)]]
    for _ in range(1, k):
        d2 = np.min(
            ((x[:, None, :] - np.array(centros)[None, :, :]) ** 2).sum(axis=2), axis=1
        )
        prob = d2 / d2.sum() if d2.sum() > 0 else np.full(n, 1.0 / n)
        centros.append(x[rng.choice(n, p=prob)])
    return np.array(centros)


def treina_em(x: np.ndarray, k: int, semente: int, iteracoes: int = 60) -> dict:
    """EM para GMM diagonal. Devolve pesos, médias e variâncias."""
    rng = np.random.default_rng(semente)
    n, d = x.shape
    medias = inicializa_kmeans_plus_plus(x, k, rng)
    vars_ = np.tile(x.var(axis=0) + 1e-3, (k, 1))
    pesos = np.full(k, 1.0 / k)

    for _ in range(iteracoes):
        # E-step: log responsabilidades
        log_r = np.zeros((n, k))
        for j in range(k):
            r = x - medias[j]
            log_r[:, j] = (
                np.log(pesos[j])
                - 0.5 * (np.log(2 * np.pi) + np.log(vars_[j]) + r**2 / vars_[j]).sum(axis=1)
            )
        m = log_r.max(axis=1, keepdims=True)
        log_norm = m + np.log(np.exp(log_r - m).sum(axis=1, keepdims=True))
        resp = np.exp(log_r - log_norm)

        # M-step
        nk = resp.sum(axis=0) + 1e-9
        pesos = nk / n
        medias = (resp.T @ x) / nk[:, None]
        for j in range(k):
            r = x - medias[j]
            vars_[j] = (resp[:, j][:, None] * r**2).sum(axis=0) / nk[j] + 1e-6

    return {"pesos": pesos, "medias": medias, "vars": vars_}


def log_verosimilhanca(modelo: dict, x: np.ndarray) -> float:
    """A MESMA fórmula de src/gmm.rs — se divergir, o teste de ida-e-volta falha."""
    k = len(modelo["pesos"])
    termos = []
    for j in range(k):
        r = x - modelo["medias"][j]
        log_n = (-0.5 * (np.log(2 * np.pi) + np.log(modelo["vars"][j]) + r**2 / modelo["vars"][j])).sum()
        termos.append(np.log(modelo["pesos"][j]) + log_n)
    termos = np.array(termos)
    m = termos.max()
    return float(m + np.log(np.exp(termos - m).sum()))


def fmt(v: float) -> str:
    """f32 em notação com sufixo — evita literais que o parser trate como f64."""
    return f"{np.float32(v):.7e}f32"


def escreve_modelo(caminho: Path, modelo: dict) -> None:
    k = len(modelo["pesos"])
    linhas = [
        "// GERADO por python/treina_gmm.py e python/treina_regressao.py — NÃO EDITAR À MÃO.",
        "//",
        "// Coeficientes do dosímetro: densidade (GMM) e regressão do nível.",
        "// Consumidos por tests/ida_e_volta.rs e, no dispositivo, por src/gmm.rs / src/regress.rs.",
        "",
        f"pub const N_COMP: usize = {k};",
        f"pub const N_BANDAS_M: usize = {N_BANDAS};",
        "",
        f"pub const GMM_PESOS: [f32; N_COMP] = [{', '.join(fmt(w) for w in modelo['pesos'])}];",
        "",
        "pub const GMM_MEDIAS: [[f32; N_BANDAS_M]; N_COMP] = [",
    ]
    for linha in modelo["medias"]:
        linhas.append("    [" + ", ".join(fmt(v) for v in linha) + "],")
    linhas += ["];", "", "pub const GMM_VARS: [[f32; N_BANDAS_M]; N_COMP] = ["]
    for linha in modelo["vars"]:
        linhas.append("    [" + ", ".join(fmt(v) for v in linha) + "],")
    linhas += ["];", ""]
    caminho.write_text("\n".join(linhas))
    print(f"escrito {caminho.relative_to(RAIZ)} ({k} componentes, {N_BANDAS} bandas)")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--entrada", type=Path, default=None, help="CSV de features (12 colunas)")
    ap.add_argument("--componentes", type=int, default=2)
    ap.add_argument("--semente", type=int, required=True)
    ap.add_argument("--janelas-sinteticas", type=int, default=600)
    ap.add_argument("--src", type=Path, default=RAIZ / "src" / "modelo.rs")
    args = ap.parse_args()

    if args.entrada and args.entrada.exists():
        x = np.loadtxt(args.entrada, delimiter=",")
        if x.ndim == 1:
            x = x[None, :]
        print(f"lido {args.entrada} — {x.shape[0]} janelas x {x.shape[1]} bandas")
    else:
        x = gera_features_sinteticas(args.janelas_sinteticas, args.semente)
        print(f"SEM SENSOR: {x.shape[0]} janelas sintéticas (semente {args.semente})")

    if x.shape[1] != N_BANDAS:
        raise SystemExit(f"esperava {N_BANDAS} colunas, recebi {x.shape[1]}")

    modelo = treina_em(x, args.componentes, args.semente)
    escreve_modelo(args.src, modelo)

    # Vector de controlo para o teste de ida-e-volta: uma janela real do conjunto e o valor
    # calculado AQUI. O Rust tem de chegar ao mesmo número.
    vetor = x[0]
    ll = log_verosimilhanca(modelo, vetor)
    controle = AQUI / "vector_controle.json"
    controle.write_text(
        json.dumps(
            {
                "vector": [float(v) for v in vetor],
                "log_verosimilhanca": ll,
                "componentes": args.componentes,
                "semente": args.semente,
                "origem": "sintetico" if not (args.entrada and args.entrada.exists()) else str(args.entrada),
            },
            indent=2,
        )
        + "\n"
    )
    print(f"escrito {controle.relative_to(RAIZ)} — ll = {ll:.6f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
