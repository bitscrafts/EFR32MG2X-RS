# Checklist pré-publicação

**Uma entrada por defeito que já passou por aqui.** Cada linha diz o que é verificado e qual foi o caso que a
obrigou a existir — uma verificação sem história é a primeira que alguém apaga por parecer excessiva.

Corre-se com:

```bash
bash scripts/pre-publish.sh                 # por omissão: crates/efr32mg2x-pac
bash scripts/pre-publish.sh crates/<outra>  # qualquer crate do workspace
```

E os checks 1, 6, 7 e 8 correm também em `scripts/gate.sh`, portanto **no CI**, em cada push.

| # | Verificação | Porque existe |
|---|---|---|
| 1 | `gate.sh`: build, clippy, fmt, testes no host, todos os exemplos | o gate tinha de passar antes de um commit; passou a valer também antes de uma publicação |
| 2 | `cargo publish --dry-run` chega a "aborting upload due to dry run" | o dry-run compila o pacote e **nunca o lê**; falha aqui é fatal, mas passar aqui não prova nada |
| 3 | `check-publishable.py`: identidade, links, cada alvo de `include_str!` | um README que aponta para `docs/PHASE2_PLAN.md` inexistente já foi publicado; e um `include_str!` fora do pacote só rebenta no build de quem descarrega |
| 4 | tamanho do `.crate` < 10 MB | é o limite do crates.io; um `svd/` ou um PDF dentro do pacote passa disso sem ninguém notar |
| 5 | nenhum caminho desta máquina no pacote (`/projects`, `/root`, `/tmp`, `_tmp`) | um caminho absoluto compila aqui e em lado nenhum |
| 6 | `check-readmes.py`: todos os READMEs — links resolvem, caminhos nomeados existem | o índice da documentação apontava para `STATUS.md`, `PLAN.md`, `BACKLOG.md` e `LOG.md`, todos apagados |
| 7 | ficheiros gerados iguais aos geradores: README, `examples.md`, `ble_config.rs`, `//!` dos módulos | um ficheiro gerado que deriva é uma mentira com build; o `gpio_s`/`gpio_ns` empatado em 173 módulos deu ordem diferente no CI e no `--check` |
| 8 | **o exemplo do README compila numa crate vazia** | `0.1.0` foi publicado com `use panic_halt as _;` e sem `panic-halt` nas dependências: `unresolved import panic_halt`. Todos os outros checks lêem a crate; nenhum lia o par (Cargo.toml, programa) que o leitor copia |
| 9 | a lista de dependências cobre o que o exemplo importa (derivada do corpo, não escrita à mão) | é o mesmo defeito do 8, visto do lado que o impede de voltar |
| 10 | `memory.x` com `ORIGIN = 0x08006000` | os primeiros 24 KiB são do Gecko bootloader; uma imagem em `0x08000000` inutiliza a placa |
| 11 | a linha de instalação do README diz a versão que está a ser publicada | o README anunciava `0.1.0` com features de outra versão |
| 12 | a versão ainda não está no crates.io | uma versão publicada é imutável; publicar é irreversível |
| 13 | identidade e mensagem do commit nomeiam uma pessoa, não uma ferramenta | 35 commits num repositório saíram com `Hermes Agent` no autor |
| 14 | **ordem expressa** | "NÃO HÁ PUBLICAÇÃO SEM ORDEM EXPRESSA" — o script imprime `Publishing still needs an express order` e nunca sobe nada |

## O que o script deliberadamente não faz

- **Não publica.** Nem com tudo verde: o verde é uma condição necessária, não a autorização.
- **Não corrige.** Um check que conserta sozinho esconde a avaria que o fez disparar.
- **Não afrouxa.** Se um check falha num trabalho legítimo, corrige-se o check com a razão escrita ao lado — como
  o `OUT_DIR/bindings.rs` e o `?v=2` dos badges — e nunca se baixa o critério para o trabalho passar.
