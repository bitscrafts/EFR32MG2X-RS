#!/usr/bin/env python3
"""Figuras do relatório e do paper — geradas de coeficientes reais, não desenhadas à mão.

## O que produz

    fig1_nivel_com_incerteza.png   nível previsto + banda de 90 % (P2, o argumento central)
    fig2_densidade_por_estado.png  log-verosimilhança do GMM separando os dois estados (P1)
    fig3_dose_nr15.png             tempo admissível vs nível, com o limite de 8 h (aplicabilidade)

## Honestidade das figuras

O modelo vem de `src/modelo.rs` — os mesmos coeficientes que o dispositivo usa. Os dados de
entrada são **sintéticos** nesta fase (não há sensor ligado). Cada figura leva a marca da
origem no título, para que ninguém cite uma figura sintética como medida de campo.

Uso:
    uv run --with numpy --with matplotlib python/graficos.py
"""
from __future__ import annotations

import re
from pathlib import Path

import numpy as np

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

AQUI = Path(__file__).resolve().parent
RAIZ = AQUI.parent
FIGURAS = RAIZ / "docs" / "figuras"
ORIGEM = "dados sintéticos — sensor ainda não ligado"


def le_const(nome: str) -> np.ndarray:
    """Lê um array const de `src/modelo.rs` sem depender de crate nenhum."""
    texto = (RAIZ / "src" / "modelo.rs").read_text()
    m = re.search(rf"pub const {nome}[^=]*=\s*(.*?);", texto, re.S)
    if not m:
        raise SystemExit(f"const {nome} não encontrada em src/modelo.rs")
    return np.array([float(v) for v in re.findall(r"(-?\d+\.?\d*e?[-+]?\d*)f32", m.group(1))])


def fig1_nivel_com_incerteza() -> Path:
    """O argumento central: a previsão vale pelo intervalo que a acompanha."""
    m = le_const("REG_M")
    sigma2 = float(le_const("REG_SIGMA2")[0])
    a_inv = le_const("REG_A_INV").reshape(2, 2)

    x = np.linspace(45.0, 85.0, 200)
    media = m[0] + m[1] * x
    # var = sigma² (1 + φᵀ A⁻¹ φ), φ = [1, x]
    var = sigma2 * (
        1.0 + a_inv[0, 0] + 2.0 * a_inv[0, 1] * x + a_inv[1, 1] * x * x
    )
    dp = np.sqrt(np.maximum(var, 0.0))

    fig, ax = plt.subplots(figsize=(7.0, 4.2), dpi=160)
    ax.fill_between(x, media - 1.645 * dp, media + 1.645 * dp, alpha=0.25,
                    label="banda de 90 %")
    ax.plot(x, media, lw=2, label="nível previsto (média posterior)")
    ax.plot([45, 85], [45, 85], "--", lw=1, color="grey", label="sem mudança (referência)")
    ax.set_xlabel("nível actual, dB(A)")
    ax.set_ylabel("nível na janela seguinte, dB(A)")
    ax.set_title("P2 — previsão do nível com incerteza\n" + ORIGEM, fontsize=10)
    ax.legend(loc="upper left", fontsize=8)
    ax.grid(alpha=0.3)
    fig.tight_layout()
    destino = FIGURAS / "fig1_nivel_com_incerteza.png"
    fig.savefig(destino)
    plt.close(fig)
    return destino


def fig2_densidade_por_estado() -> Path:
    """P1: a densidade separa estados acústicos — se não separasse, o GMM não servia."""
    pesos = le_const("GMM_PESOS")
    medias = le_const("GMM_MEDIAS").reshape(len(pesos), -1)
    vars_ = le_const("GMM_VARS").reshape(len(pesos), -1)
    n_bandas = medias.shape[1]

    # Usa a janela de controlo como amostra e perturba-a para mostrar a sensibilidade
    rng = np.random.default_rng(7)
    def log_ver(x: np.ndarray) -> np.ndarray:
        termos = []
        for j in range(len(pesos)):
            r = x - medias[j]
            ln = (-0.5 * (np.log(2 * np.pi) + np.log(vars_[j]) + r**2 / vars_[j])).sum(axis=-1)
            termos.append(np.log(pesos[j]) + ln)
        termos = np.stack(termos, axis=-1)
        mx = termos.max(axis=-1, keepdims=True)
        return (mx + np.log(np.exp(termos - mx).sum(axis=-1, keepdims=True))).squeeze(-1)

    # Duas famílias de janelas: em torno do componente 0 e do componente 1
    amostras_a = medias[0] + rng.normal(0, 0.3, (400, n_bandas))
    amostras_b = medias[1] + rng.normal(0, 0.3, (400, n_bandas))
    ll_a, ll_b = log_ver(amostras_a), log_ver(amostras_b)

    fig, ax = plt.subplots(figsize=(7.0, 4.0), dpi=160)
    ax.hist(ll_a, bins=30, alpha=0.65, label="estado A (ventilador)")
    ax.hist(ll_b, bins=30, alpha=0.65, label="estado B (voz)")
    ax.set_xlabel("log-verosimilhança da janela")
    ax.set_ylabel("janelas")
    ax.set_title("P1 — a densidade separa estados acústicos\n" + ORIGEM, fontsize=10)
    ax.legend(fontsize=8)
    ax.grid(alpha=0.3)
    fig.tight_layout()
    destino = FIGURAS / "fig2_densidade_por_estado.png"
    fig.savefig(destino)
    plt.close(fig)
    return destino


def fig3_dose_nr15() -> Path:
    """Aplicabilidade: o tempo admissível cai a metade a cada +5 dB (taxa de troca)."""
    niveis = np.linspace(80, 105, 200)
    tempo_h = 8.0 * 2.0 ** ((85.0 - niveis) / 5.0)

    fig, ax = plt.subplots(figsize=(7.0, 4.0), dpi=160)
    ax.semilogy(niveis, tempo_h, lw=2)
    ax.axhline(8.0, ls="--", lw=1, color="grey")
    ax.axvline(85.0, ls="--", lw=1, color="grey")
    ax.plot([85.0], [8.0], "o", ms=7, color="black")
    for n in (85.0, 90.0, 95.0):
        t = 8.0 * 2.0 ** ((85.0 - n) / 5.0)
        ax.annotate(f"{n:.0f} dB → {t:.1f} h", (n, t), textcoords="offset points",
                    xytext=(8, 6), fontsize=8)
    ax.set_xlabel("nível, dB(A)")
    ax.set_ylabel("exposição admissível, horas")
    ax.set_title("Aplicabilidade — NR-15, taxa de troca 5 dB\nlimite de 85 dB(A) para 8 h", fontsize=10)
    ax.grid(alpha=0.3, which="both")
    fig.tight_layout()
    destino = FIGURAS / "fig3_dose_nr15.png"
    fig.savefig(destino)
    plt.close(fig)
    return destino


def main() -> int:
    FIGURAS.mkdir(parents=True, exist_ok=True)
    for f in (fig1_nivel_com_incerteza, fig2_densidade_por_estado, fig3_dose_nr15):
        p = f()
        print(f"escrito {p.relative_to(RAIZ)} ({p.stat().st_size // 1024} KB)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
