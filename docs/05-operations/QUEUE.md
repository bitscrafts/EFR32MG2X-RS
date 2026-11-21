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
- [x] C2 | docs/05-operations/QUEUE-ITEM-C2-i2c-core.md | o mesmo movimento para i2c-core, com o check a cobrir dois cores
- [x] C3 | docs/05-operations/QUEUE-ITEM-C3-adc-core.md | o mesmo movimento para adc-core
- [x] C4 | docs/05-operations/QUEUE-ITEM-C4-dma-core.md | o mesmo movimento para dma-core
- [x] C5 | docs/05-operations/QUEUE-ITEM-C5-usart-core.md | o mesmo movimento para usart-core
- [x] C6 | docs/05-operations/QUEUE-ITEM-C6-sysrtc-core.md | o mesmo movimento para sysrtc-core, o último porque o bluetooth-sys também o chama

## Backlog — decidido, ainda sem spec

`- [~]` é **backlog**: o runner só apanha `- [ ]`, portanto nada disto arranca sozinho. Passa a
`- [ ]` no dia em que tiver spec e caminho. Esta secção existe para uma coisa só: nenhuma decisão
tomada em conversa se perder. Método completo em `docs/05-operations/QUEUE-METHOD.md`.

- [x] B1 | (sem spec) | H17 — **fechado sem código novo (2026-11-21)**: o SVD deste chip tem **zero** `DOUTSET`/`DOUTCLR`, portanto um bit tem de ser escrito por ler-modificar-escrever, e `Pin::set`/`toggle` já mantêm o par dentro de `critical_section::with`. O item pedia um registo que a peça não tem.
- [x] B2 | (sem spec) | H21/H22 — **feito (`af01bfc`)**: `Timer0::take(..., sysclk_hz)`, `Board::pwm_on`, `Board::buzzer_on`; `sysclk_hz == 0` devolve `None`. Medido antes: das 21 ocorrências de `19_000_000`, 15+ eram argumentos de teste e só **uma** era o driver a supor o relógio.
- [~] B3 | (sem spec) | H28 — **escrito (`8e74e88`), NÃO MEDIDO**: `hal::sleep` com `keep_debugger` e `SLEEPDEEP` limpo na saída, exemplo `hal_em2_sleep`. Falta a placa: `flash.sh` recusou com zero sondas. Critério = tempo de parede entre as dez linhas, não a ausência de erro.
- [x] B4 | (sem spec) | B12-resto — **feito (`cc272f5`)**: regra pura em `libraries/psa-shape-core` (5 testes no host), chamada pelo `-sys`. Recusa só contradição; zero = não especificado. **Risco declarado**: os offsets 0 e 4 do blob foram deduzidos do offset 8 medido, não medidos.
- [~] B5 | (sem spec) | B16 — mover a link layer fora do PendSV; o maior risco da revisão, sem tempo de reverter
- [x] B6 | (sem spec) | **passa (2026-11-21)**: o dry-run chega a `Uploading efr32mg2x-hal v0.1.0` e aborta por ser dry run. De passagem, a dependência do PAC já está em `version = "0.1.1"`, pelo que o item 11 do `pre-publish.sh` deixou de ter o que apanhar.
- [~] B7 | (sem spec) | publicar o HAL, depois o rádio, os dois BSPs e o `efr32mg2x-examples` volta a incluir `hal-examples`; **só com ordem expressa**

## Depois destes seis

`cargo publish --dry-run` no HAL tem de chegar ao upload sem reclamar de dependências sem versão.
Quando chegar, o HAL fica pronto para a primeira publicação — que continua a precisar de ordem
expressa e não é feita por esta fila.

- [~] B8 | (sem spec) | `queue-run-once.sh` tem dois defeitos provados em 2026-11-18: marca o item antes de commitar (dois itens ficaram "feitos" sem trabalho nem commit), e aceita um gate PASS sobre uma árvore inalterada como se fosse trabalho feito. Corrigir antes de voltar a ligar o cron.
- [~] B9 | (sem spec) | integração com o Embassy — hoje só existe como referência em docs, nenhum trait implementado. O menor passo útil é `embassy-time-driver`: implementar `now()`, `schedule_wake()` e `set_alarm()` sobre o SYSRTC, o que passa a dar `embassy-time`, `embassy-sync` e `embassy-executor` a funcionar neste HAL. Depois disso, `embedded-hal-async` para i2c/spi (depende da rota de pinos) e `embedded-io-async` para usart. Evidence: grep embassy -> só links em READMEs e docs/04-reference.
- [~] B10 | docs/05-operations/QUEUE-ITEM-B10-ieee802154-sys.md | 802.15.4 como SEGUNDO protocolo do MESMO rail-sys (BLE ja la esta; o vendor tem ble/ieee802154/sidewalk/wmbus/zwave): `RAIL_IEEE802154_Init` e `Config2p4GHzRadio`, config espelhada e verificada, UM frame transmitido. Medido: rail_ieee802154.h:508 e :524 existem no vendor. Zigbee e Thread ficam para depois.
- [~] B11 | (sem spec) | MAC 802.15.4 em Rust puro — CSMA/CA, ACK, retries, sequência: tudo decisão, tudo testável no host, no padrão de `cores/`. Só depois do B10 provar que o PHY levanta.
