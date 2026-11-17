# BRIEF — spec-01 (para quem implementa)

**Não leias mais nada além da spec.** Este brief só diz o que fazer e como provar.

## Ficheiro do contrato

```
specs/01-treino-host-e-coeficientes-para-o-dispositivo.md
```

## O que fazer (nesta ordem)

```
1  ler a spec inteira, incluindo Guardrails
2  implementar R1 (python/treina_gmm.py)   -> gera src/modelo.rs com GMM_PESOS/MEDIAS/VARS
3  implementar R2 (python/treina_regressao.py) -> acrescenta REG_M/REG_SIGMA2/REG_A_INV
4  implementar R3 (teste de ida-e-volta gmm_ida_e_volta_host_rust + regressao_ida_e_volta)
5  implementar T5 e T6 (os CONTROLES — sem eles a ida-e-volta não prova nada)
6  garantir R5 (determinístico com --semente) e R4 (sem sklearn)
```

## Como correr

```bash
cd /projects/efr32mg24-rs/crates/dosimeter-mg24
cargo test -p dosimeter-mg24 --target x86_64-unknown-linux-gnu --lib
uv run --with numpy python/treina_gmm.py --entrada <csv> --semente 1
python3 /root/.hermes/skills/spec-author/scripts/pre-implementation-check.py --new-only specs/01-*.md
```

## O que colar na resposta final

**Saída crua, não resumo:**

```
1  o output do cargo test (linhas "test ... ok" e "test result:")
2  o diff dos controlos: T5 tem de FALHAR ao perturbar as médias, e T6 ao usar 11 colunas
   (paste do panic, com os valores, não a frase "falhou como esperado")
3  `cmp` dos dois modelo.rs gerados com a mesma semente
4  a lista de ficheiros que criaste ou alteraste
```

## Proibido

```
- commitar            (o commit e' do orchestrator)
- tocar em src/lib.rs, src/features.rs, src/gmm.rs, src/regress.rs
- adicionar dependencias ao crate
- escrever vector_controle.json a mao
- afrouxar ou apagar teste
- editar a spec
```

Se uma premissa da spec estiver errada, **para e reporta** — não codifiques em volta.
