# Spec B10: 802.15.4 in the radio crate — one RAIL, a second protocol

**Status: spec.** Written 2026-11-18, after measuring the vendor API. No code yet; the full
Zigbee/Thread implementation is explicitly deferred, as the operator asked ("não descartar a
implementação completa para o futuro").

## Overview

The radio on this part speaks BLE through RAIL, and the same RAIL speaks IEEE 802.15.4 — the
protocol headers are already in the tree:

```
crates/efr32mg24-rail-sys/vendor/rail/protocol/ieee802154/rail_ieee802154.h
    RAIL_Status_t RAIL_IEEE802154_Init(RAIL_Handle_t, const RAIL_IEEE802154_Config_t *);   :508
    RAIL_Status_t RAIL_IEEE802154_Config2p4GHzRadio(RAIL_Handle_t);                        :524
    (plus AntDiv/Coex/Fem/2Mbps variants, :540-:660)
```

**Corrected 2026-11-18 by the operator: the radio crate is common to BLE, Zigbee and Thread, and it
already exists.** The vendor treats the protocols as modules of one RAIL:

```
crates/efr32mg24-rail-sys/vendor/rail/protocol/
    ble/  ieee802154/  sidewalk/  wmbus/  zwave/
crates/efr32mg24-rail-sys/src/
    lib.rs  ble_config.rs          <- one of the five is bound; the other four are not
```

So this item does not create a crate. It adds a second protocol to the crate that already speaks the
first: `efr32mg24-rail-sys/src/ieee802154.rs`, binding the two entry points, with an example that
brings the radio up in 802.15.4 mode and transmits one frame. Zigbee and Thread then stand on
`efr32mg24-radio`, where the BLE protocol logic lives today — one floor, three protocols above it.

## Measured before writing a line

```
vendor header     rail_ieee802154.h:508 (Init), :524 (Config2p4GHzRadio) -- 7 fields in the config
the blob already exports both symbols:
    nm -g librail_efr32xg24_gcc_release.a
      00000001 T RAIL_IEEE802154_Config2p4GHzRadio
      00000001 T RAIL_IEEE802154_Init
rail-sys today    src/lib.rs is 80 lines and both `extern "C" {}` blocks are EMPTY;
                  src/ble_config.rs is a data table, not an FFI call
build.rs          links the pinned Gecko SDK 4.4.5 archive (sha256 above it)
```

This is the whole reason the item is small: the vendor blob already contains the 802.15.4 stack, the
header is in the tree, and the two symbols are exported. There is nothing to vendor, nothing to
download, and no licence question beyond the one the BLE side already answered.

**The feature already exists.** `crates/efr32mg24-radio/Cargo.toml` declares
`rail = []  # RAIL (proprietary and 802.15.4) -- enabled once emlib and a radio config are vendored`.
So the exit criterion is not "a new crate compiles" but "`--features rail` compiles and transmits one
frame".

## What makes it more than an afternoon, honestly

Three things, in order of risk:

1. **The struct layout.** Seven fields, `#[repr(C)]`, and the header may use a union or a bitfield for
   the PTI radio config. This is where the check earns its keep: field by field, compared by script.
2. **The handle.** `RAIL_Init` must run before `RAIL_IEEE802154_Init`, with a RAIL config built for
   802.15.4 rather than for BLE. The BLE path in this tree builds its config as a table; the 802.15.4
   path needs the analogous table and nobody has written it.
3. **The interrupt.** Receiving a frame is an interrupt, and the callback plumbing is the part this
   project has got wrong before (`abort_session`, the timer callback contract). Transmitting one frame
   avoids it; receiving is the next item.

## Requirements

**R1 -- the existing radio crate gains a second protocol, and nothing speculative.**
`efr32mg24-rail-sys` exposes `ieee802154::init(handle, &Config) -> Status` and
`ieee802154::config_2p4ghz(handle) -> Status`, beside the BLE bindings it already has, over the same
vendor blob.
Example: `cargo check -p efr32mg24-rail-sys --target thumbv8m.main-none-eabihf` is silent.

**R2 -- the config struct matches the vendor header field for field.**
`RAIL_IEEE802154_Config_t` is mirrored as `#[repr(C)]` with the same field order and the same types,
verified against the header by a script and not by eye.
Example: `python3 scripts/check-ieee802154-config.py` compares field names in order and prints
`OK: N fields match rail_ieee802154.h`, or names the first difference.

**R3 -- a frame can be transmitted, and that is the proof.**
An example brings the radio up, sets a 802.15.4 channel, and transmits one frame with a known
payload.
Example on the board (through `scripts/flash.sh` with `--confirm` and the `0x08006000` guard): a
second receiver, or the same board in RX, reports the payload.

## Files

- `crates/efr32mg24-rail-sys/src/lib.rs`
- `crates/efr32mg24-rail-sys/src/ieee802154.rs`
- `crates/efr32mg24-rail-sys/examples/tx_one_frame.rs`
- `crates/efr32mg24-rail-sys/build.rs`
- `scripts/check-ieee802154-config.py`

## TDD Contract

| Test Name | Input | Expected Output |
|---|---|---|
| the_config_layout_matches_the_header | `RAIL_IEEE802154_Config_t` in the header | every field name, in order, with the same size and alignment |
| a_missing_field_is_caught | a mirror with one field removed | `FAIL`, naming the field and its offset |
| a_reordered_field_is_caught | two fields swapped | `FAIL`, naming both |
| the_status_maps_to_the_vendor_codes | 0, 1, 2 from `RAIL_Status_t` | `Ok`, `InvalidParam`, `Unsupported`, and no unknown code silently accepted |

## Exit Criteria

- [ ] `python3 scripts/check-ieee802154-config.py`
- [ ] `cargo check -q -p efr32mg24-rail-sys --target thumbv8m.main-none-eabihf`
- [ ] `cargo check -q -p efr32mg24-radio --target thumbv8m.main-none-eabihf` (nada partido acima)
- [ ] `bash scripts/gate.sh`

## Guardrails

- Do NOT write a MAC, a Zigbee stack or a Thread stack here. One frame transmitted is the whole item.
- Do NOT guess a field of `RAIL_IEEE802154_Config_t`: read the header, and let the check compare.
- Do NOT flash without `--confirm` and without the `load_min >= 0x08006000` guard.
- Do NOT publish: the blob licensing for 802.15.4 is a separate question from the BLE one.
- Do NOT delete the deferred design: Zigbee and Thread stay on the roadmap, and this item exists so
  they have a floor to stand on.

## What is deliberately out of scope, and where it goes

```
B11  the MAC in pure Rust: CSMA/CA, ACK, retries, sequence numbers -- host-testable, `cores/` style
     Zigbee: NWK + APS + ZCL, or EmberZNet as a blob
     Thread: 6LoWPAN + MLE + CoAP, or OpenThread as a blob
```

The evidence for the order: `embassy-nrf` ships 802.15.4 PHY/MAC and nothing above it, and
`nrf-802154` wraps Nordic's blob for the rest. Nobody in Rust has a full Zigbee or Thread stack, so
the floor is the honest first step and the ceiling is a decision to be taken with numbers.

## O limite de prova neste workspace (medido 2026-11-21)

**Não há segundo rádio 802.15.4 aqui.** A bancada tem uma placa XIAO MG24, o `lnx` e um telemóvel. Isso
muda o que se pode chamar "provado" neste item, e é melhor escrevê-lo antes de começar do que descobri-lo
a meio:

| passo | como se mede aqui | o que isso prova |
|---|---|---|
| configurar o rádio e pôr um canal | `RAIL_ConfigChannels` + `RAIL_IEEE802154_Config2p4GHzRadio` a devolver `RAIL_STATUS_NO_ERROR` | que a API aceitou a configuração, **não** que o rádio está num canal |
| transmitir um frame | `RAIL_StartTx` a terminar com `RAIL_EVENT_TX_PACKET_SENT` | que a sequência de TX correu e o rádio reportou envio |
| receber um frame | **impossível sem par** | nada |

Um `TX_PACKET_SENT` não é um frame no ar: o rádio pode reportar envio com a antena desafinada, e sem um
receptor não há como saber. Portanto o critério deste item tem de ser mais fraco do que os do H21/H28, e
dizer isso:

```text
provavel neste workspace   a API aceita a configuração e o evento de TX dispara
falta para "transmite"     um segundo rádio 802.15.4 -- outra placa, ou um dongle com sniffer
falta para "recebe"        o mesmo par, com o `rx.rs` a entregar o payload que o outro enviou
```

**Alternativa para não ficar parado:** o `RX` pode ser provado contra o **próprio** rádio? Não — um
transceptor não se ouve a si mesmo numa banda com duplex por comutação; o `RAIL_StartTx` desliga o
receptor.

**O que fica válido mesmo assim:** o `portions()` do `rx.rs` já está provado no host (seis testes, incluindo
pacote maior que o buffer a devolver `None`), e as duas chamadas do vendor estão ligadas e compilam. O que
não existe é a prova de ponta a ponta — e ela precisa de hardware que o workspace não tem.

## O que ficou medido, chamada a chamada (2026-11-21, `rail_ieee802154_tx`)

O exemplo existe e corre na placa. O registo final, lido com `probe-rs read b32 0x2003F000 28`:

```text
154: init=0 phy=0 pa=0 dbm=0 w=14 tx=0 hw=0
          ^       ^     ^      ^      ^       ^
          |       |     |      |      |       +-- o radio NAO reportou TX_PACKET_SENT
          |       |     |      |      +---------- RAIL_StartTx aceitou (NO_ERROR)
          |       |     |      +----------------- 14 bytes entraram na FIFO
          |       |     +------------------------ potencia a 200 (20 dBm) aceite
          |       +------------------------------ Config2p4GHzRadio aceite
          +-------------------------------------- IEEE802154_Init aceite
```

**Quatro coisas ficaram aprendidas por medicao, e cada uma custou um ciclo:**

1. **`RAIL_StartTx` não leva o pacote.** Os bytes vão para a FIFO antes (`RAIL_WriteTxFifo`); a assinatura tem
   `channel`, `options` e `schedulerInfo`, e nada mais.
2. **A tabela de canais não se escreve.** `RAIL_IEEE802154_Config2p4GHzRadio` traz os canais 11–26 (documentado
   no header). O item pedia uma tabela à mão por leitura incompleta do header.
3. **`RAIL_IEEE802154_Init` exige transições de estado não nulas.** Com a estrutura toda a zero devolvia
   `1` (`INVALID_PARAMETER`); com as transições a `RAIL_RF_STATE_RX` (2) e as timings do exemplo do header
   devolve `0`. Zero é `RAIL_RF_STATE_INACTIVE`, que não é um estado para onde voltar.
4. **A ordem é Init do protocolo depois do PHY.** `Config2p4GHzRadio` toma o lugar de `RAIL_ConfigChannels`, e
   o `RAIL_IEEE802154_Init` valida parâmetros que precisam de um canal já definido.

5. **O primeiro byte da FIFO é o comprimento.** O 802.15.4 tem um PHY header de um byte com o tamanho do
   payload, e em transmissão é o transmissor que o escreve a partir do primeiro byte da FIFO. Este era o
   bloqueio da noite inteira.

**Como o bloqueio caiu, porque o método importa mais do que a resposta.** Eu estava a pedir um único evento,
`RAIL_EVENT_TX_PACKET_SENT` (bit 24), e a ler um zero. Um zero assim responde "não houve envio" mas não diz se
o rádio ficou calado ou se tentou e foi travado -- são duas falhas diferentes com o mesmo sintoma. Pedindo
**todos** os eventos de TX (bits 23–31) a resposta apareceu à primeira:

```
154: init=0 phy=0 pa=0 dbm=0 w=14 tx=0 hw=0 ev=1073741824
                                              ^^^^^^^^^^ 2^30 = RAIL_EVENT_TX_UNDERFLOW
```

`TX_UNDERFLOW` é "a FIFO esvaziou antes do fim do pacote". A seguir vieram dois números que separam as duas
causas possíveis -- FIFO mal montada, ou rádio a pedir bytes a mais:

```
fifo=128    RAIL_SetTxFifo devolveu 128 (sucesso; o retorno era descartado, e 0 significa erro)
espaco=114  depois de escrever 14, sobram 114 -> os 14 bytes ESTAVAM lá
```

FIFO sã, cheia, e mesmo assim underflow. O número que resolveu o resto: `frame[0] = 0x41` = **65**. O rádio leu
o frame control como comprimento e foi pedir 65 bytes. Com o comprimento à frente (`w=15` em vez de `w=14`):

```
154: init=0 phy=0 pa=0 dbm=0 fifo=128 espaco=113 w=15 tx=0 hw=1 ev=16777216
                                                              ^^^^  ^^^^^^^^ 2^24 = TX_PACKET_SENT
status=PASS
```

**Descartado por medição, sem mexer o resultado:** tamanho da FIFO, alinhamento do buffer a 4,
`repr(align(4))` vs pilha, `static` vs local, `RAIL_ConfigData` em PACKET_MODE, posição da FIFO antes e depois
do init do protocolo, e a máscara de eventos pedida antes ou depois do PHY. A causa era outra, e só apareceu
quando a pergunta passou de "porque não há este evento" para "que evento há".

**O que este exemplo prova e o que não prova.** Prova que o rádio transmite um frame 802.15.4 completo e o
reporta. **Não** prova que alguém o recebe: não há segundo nó, e um transceptor não se ouve a si mesmo. A
recepção fica para quando houver com quem falar -- o `src/rx.rs` já existe e tem as porções da FIFO testadas no
host.
