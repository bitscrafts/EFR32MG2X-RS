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
- [x] B2 | (sem spec) | H21/H22 — **feito e provado (`af01bfc`, `d2999bb`, `cb11d3b`)**: o relógio entrou no `Board` (`clock_hz()` segue o boost e o cristal), e o H21 fechou na placa: cristal de 39 MHz estabiliza sem `CTUNE`, o `CLKSEL` muda e o núcleo sobrevive, e cinco `delay_ms(1000)` duraram 5,01 s medidos. Antes da última correcção duraram 2,46 s — metade, porque `Board::delay` e `Module::delay` ainda passavam `DEFAULT_SYSCLK_HZ`. Dois defeitos achados por sintoma e um por `grep`; o padrão era sempre o mesmo: o valor vivo existia e quatro sítios preferiram a constante.
- [x] B3 | (sem spec) | H28 — **provado na placa (2026-11-21, `1e74cb6`)**: dez sonecas de ~1 s em EM2, acordadas pelo comparador do SYSRTC. Intervalos medidos 0,92–1,03 s, média 0,99, total 9,92 s, onze interrupções contadas. A RTT sobreviveu a todos os sonos (`keep_debugger`). Pelo caminho apareceu um defeito de produto: `Sysrtc::take` não armava o canal de comparação do grupo 0 no caminho quente — quando o stack BLE arranca primeiro, o caminho NORMAL — e nenhum agendador da aplicação teria interrupções, em silêncio. Por medir: consumo em EM2 (sem medidor) e acordar por GPIO.
- [x] B4 | (sem spec) | B12-resto — **feito (`cc272f5`)**: regra pura em `libraries/psa-shape-core` (5 testes no host), chamada pelo `-sys`. Recusa só contradição; zero = não especificado. **Risco declarado**: os offsets 0 e 4 do blob foram deduzidos do offset 8 medido, não medidos.
- [~] B5 | (sem spec) | B16 — mover a link layer fora do PendSV; o maior risco da revisão, sem tempo de reverter
- [x] B6 | (sem spec) | **passa (2026-11-21)**: o dry-run chega a `Uploading efr32mg2x-hal v0.1.0` e aborta por ser dry run. De passagem, a dependência do PAC já está em `version = "0.1.1"`, pelo que o item 11 do `pre-publish.sh` deixou de ter o que apanhar.
- [~] B7 | (sem spec) | publicar o HAL, depois o rádio, os dois BSPs e o `efr32mg2x-examples` volta a incluir `hal-examples`; **só com ordem expressa**

## Depois destes seis

`cargo publish --dry-run` no HAL tem de chegar ao upload sem reclamar de dependências sem versão.
Quando chegar, o HAL fica pronto para a primeira publicação — que continua a precisar de ordem
expressa e não é feita por esta fila.

- [x] B8 | (sem spec) | `queue-run-once.sh` tinha dois defeitos provados em 2026-11-18 — **corrigido e provado (2026-11-21)**: (a) o item era marcado ANTES do commit, portanto um commit que falhasse deixava a fila a dizer "feito"; agora marca-se, commita-se, e se o commit falhar a marcação é reposta de uma cópia. (b) um gate PASS sobre uma árvore inalterada contava como trabalho feito; agora, antes de marcar, pergunta-se se houve trabalho — e como a pergunta anterior pode ser enganada por um ficheiro perdido na árvore, verifica-se depois o conteúdo do commit: se só trouxe a fila e o log do lane, o commit é desfeito e o item volta à fila (saída 4, distinta da 3 = "fila vazia", para um chamador não parar a linha por causa de um worker que não fez nada). O `QUEUE_PI` passou a ser configurável pela mesma razão que o `QUEUE_MODEL`: um script que só corre com o worker real não se pode testar. `scripts/test-queue-run-once.sh` exercita o script real num repositório descartável, com quatro casos (sem trabalho, trabalho, commit que falha, worker que só mexe na fila), e **está no gate** — a garantia é verificada em cada commit, não só no dia em que foi escrita. Antes de voltar a ligar o cron: o cron continua desligado, e a decisão de o ligar não é deste item.
- [x] B9 | (sem spec) | integração com o Embassy — **driver provado na placa (2026-11-21, este commit)**: `emb: wakes=10 driver_ms=1000 wall_ms=997` → `status=PASS`. A cadeia inteira do driver funciona: `embassy_time_driver::schedule_wake` escolhe o prazo (`alarm::earliest`), converte 64→32 bits (`ticks::comparator_value`), arma o comparador pelo `Platform`, a interrupção dispara, `on_alarm` entrega o evento, e o **waker é chamado dez vezes**. Dois relógios independentes concordam: 1000 ms pelo próprio driver, 997 ms pelo DWT. Para isso o HAL ganhou `Sysrtc::now_wide_global()` e `schedule_at_global()` — um driver de tempo é chamado de contextos que não podem possuir o periférico. **Não há `embassy-executor` aqui de propósito**: o que se prova é a cadeia do driver, não o executor (código de terceiros), e provar os dois juntos esconderia qual metade falhou. **Falta**: (a) um executor com uma tarefa que imprima ao acordar — é o consumidor, não a cadeia; (b) opcional, contar os overflows por `GRP0_IF.OVF` para tirar a exigência de chamar `now_wide` a cada 36,4 h; (c) depois disso, `embedded-hal-async` para i2c/spi e `embedded-io-async` para usart.
- [x] B10 | docs/05-operations/QUEUE-ITEM-B10-ieee802154-sys.md | 802.15.4 como SEGUNDO protocolo do MESMO rail-sys — **feito e provado na placa (2026-11-21, `ba1d4ea`)**: `154: init=0 phy=0 pa=0 dbm=0 fifo=128 espaco=113 w=15 tx=0 hw=1 ev=16777216` → `status=PASS`, com `hw=1` a ser o rádio a levantar `RAIL_EVENT_TX_PACKET_SENT`. A causa do bloqueio era **um byte**: o primeiro byte da FIFO não é o frame control, é o **comprimento** (PHY header do 802.15.4, escrito pelo transmissor a partir do primeiro byte que encontra). Sem ele o rádio leu `0x41` como 65 e levantou `TX_UNDERFLOW` (bit 30). Só apareceu quando a pergunta passou de "porque não há este evento" para "que evento há" — com um único bit ligado, um aborto e um silêncio dão o mesmo zero. **Não prova recepção**: não há segundo nó nesta bancada. Zigbee e Thread ficam para depois.
- [x] B11 | (sem spec) | MAC 802.15.4 em Rust puro — **feito na camada de decisão (2026-11-21, `10c444c` + este commit)**: `libraries/ieee802154-mac-core`, 34 testes no host, `no_std`, sem dependências, sem `unsafe`. Decide quatro coisas e só quatro: os bytes a transmitir com o **byte de comprimento à frente** (o defeito que custou a noite é o primeiro teste), o **CSMA/CA** (canal ocupado → expoente cresce até `macMaxBE`, espera aleatória dentro da janela, `macMaxCSMABackoffs` tentativas e depois **falha reportada**, não laço infinito), o **ACK** (se pertence à sequência enviada, contando o FCS de 2 bytes que o rádio acrescenta) e a **recepção** (`classify`: é para mim? broadcast ou o meu endereço; senão `NotMine`, porque confirmar frame alheio silencia o vizinho). O exemplo da placa já **chama** esta crate em vez de montar os bytes à mão — medido: `w=15 hw=1 ev=16777216`. Controlos por mutação: repor o `0x41` em byte 0 falha os dois testes do PHY header; remover a guarda do destinatário falha o `NotMine`. **Falta**: ligar a `rx.rs` do rail-sys a esta camada, e a recepção só se prova com um segundo nó.
