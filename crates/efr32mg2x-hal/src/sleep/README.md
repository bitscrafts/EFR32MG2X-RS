# sleep

Entrar em EM2 — e sair.

## What it is

Entrar em EM2 — e sair.

## Porque é que isto é uma API e não um `wfi` no exemplo

Entrar em EM2 não é dormir: é desligar o relógio de alta frequência e ficar dependente de uma
interrupção que já esteja armada. Um `wfi` solto com o `SLEEPDEEP` posto e sem fonte de acordar
deixa o aparelho a dormir até alguém lhe tocar no reset — e o sintoma, visto de fora, é uma placa
que morreu.

Nesta série, o `EMU` tem `CTRL.EM2DBGEN` e **não** tem bloqueio de EM2: quem impede a entrada são
as condições do resto do sistema, não um bit. As duas que este módulo trata:

- **o debugger**: com `EM2DBGEN = 0`, a ligação de debug morre em EM2 e não volta com a placa a
correr — [`Sleep::keep_debugger`] liga-o antes de dormir;
- **o `SLEEPDEEP`**: fica posto durante o `wfi` e é limpo a seguir, para que o próximo `wfi` que
alguém escreva à mão não entre em EM2 sem saber.

## O que este módulo não faz

Não arma fontes de acordar. Quem chama tem de ter a interrupção ligada no periférico **e** no
`NVIC` antes de entrar, senão o `wfi` não regressa. É de propósito: escolher a fonte de acordar é
decisão do produto, e um `sleep_ms` que decidisse por nós seria a mesma classe de suposição que o
`19_000_000` escondia no driver do timer.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Sleep` | struct | A entrada e a saída de EM2. |
| `enter` | fn | Dorme em EM2 até uma interrupção armada acordar o núcleo. |
| `keep_debugger` | fn | Mantém a ligação de debug viva enquanto o núcleo está em EM2. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

```
    pending: the interrupt that ended the sleep was the controller's. The handler cleared the flag (`af=0`), and the
```

Controls and the full story: [DMA-IRQ-2026-11-27.md](https://github.com/bitscrafts/EFR32MG2X-RS/blob/master/docs/04-reference/DMA-IRQ-2026-11-27.md)

## What is not proven

- ~~H28~~ | EM2 entry and exit sequence | **CLOSED 2026-11-21** (`B3`): ten ~1 s naps in EM2 woken by the SYSRTC comparator, intervals 0.92-1.03 s, mean 0.99, total 9.92 s, eleven interrupts. (HAL-GAP-ANALYSIS-2026-11-18.md (`../../../../docs/04-reference/HAL-GAP-ANALYSIS-2026-11-18`))

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
