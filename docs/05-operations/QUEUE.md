# Fila de trabalho — efr32mg24-rs

**Como funciona.** `scripts/queue-run-once.sh` apanha a **primeira linha não marcada**, faz esse item,
corre o gate, e só commita se o gate estiver verde. Um item por invocação: se o processo morrer, a
fila fica onde estava e o disparo seguinte retoma.

**Formato, para se poder acrescentar itens à mão:**
`- [ ] <ID> | <spec> | <objectivo em uma linha>`

Marcar é trabalho do runner, não teu. Quando todas as linhas estiverem marcadas, o runner escreve
`QUEUE EMPTY` e o cron avisa — é o único aviso que sai daqui.

**Regras que o runner impõe e o worker não pode contornar:** nunca publica no crates.io; nunca
flasha; três vermelhos seguidos param a fila com `BLOCKED`.

- [x] C1 | specs/09-one-core-moves-and-the-pattern-proves-itself.md | mover spi-core para dentro do HAL e provar que só existe uma cópia
- [ ] C2 | docs/05-operations/QUEUE-ITEM-C2-i2c-core.md | o mesmo movimento para i2c-core, com o check a cobrir dois cores
- [ ] C3 | docs/05-operations/QUEUE-ITEM-C3-adc-core.md | o mesmo movimento para adc-core
- [ ] C4 | docs/05-operations/QUEUE-ITEM-C4-dma-core.md | o mesmo movimento para dma-core
- [ ] C5 | docs/05-operations/QUEUE-ITEM-C5-usart-core.md | o mesmo movimento para usart-core
- [ ] C6 | docs/05-operations/QUEUE-ITEM-C6-sysrtc-core.md | o mesmo movimento para sysrtc-core, o último porque o bluetooth-sys também o chama

## Backlog — decidido, ainda sem spec

`- [~]` é **backlog**: o runner só apanha `- [ ]`, portanto nada disto arranca sozinho. Passa a
`- [ ]` no dia em que tiver spec e caminho. Esta secção existe para uma coisa só: nenhuma decisão
tomada em conversa se perder. Método completo em `docs/05-operations/QUEUE-METHOD.md`.

- [~] B1 | (sem spec) | H17 — `DOUT` escrito por ler-modificar-escrever em vez de `DOUT_SET`/`DOUT_CLR`; prova na placa com o bit-bang do IMU a receber ACK
- [~] B2 | (sem spec) | H21/H22 — um relógio de runtime em vez de `19_000_000` espalhado; fecha antes da primeira publicação porque muda a API
- [~] B3 | (sem spec) | H28 — entrada e saída de EM2, só provável na placa
- [~] B4 | (sem spec) | B12-resto — validar tipo e bits em `psa_import_key` (o audit já deu `0x2400` e `128` nos três chamadores)
- [~] B5 | (sem spec) | B16 — mover a link layer fora do PendSV; o maior risco da revisão, sem tempo de reverter
- [~] B6 | (sem spec) | `cargo publish --dry-run` do HAL tem de chegar ao upload sem reclamar de dependências sem versão — é o sinal de que os seis C fecharam
- [~] B7 | (sem spec) | publicar o HAL, depois o rádio, os dois BSPs e o `efr32mg2x-examples` volta a incluir `hal-examples`; **só com ordem expressa**

## Depois destes seis

`cargo publish --dry-run` no HAL tem de chegar ao upload sem reclamar de dependências sem versão.
Quando chegar, o HAL fica pronto para a primeira publicação — que continua a precisar de ordem
expressa e não é feita por esta fila.

- [~] B8 | (sem spec) | `queue-run-once.sh` tem dois defeitos provados em 2026-11-18: marca o item antes de commitar (dois itens ficaram "feitos" sem trabalho nem commit), e aceita um gate PASS sobre uma árvore inalterada como se fosse trabalho feito. Corrigir antes de voltar a ligar o cron.
