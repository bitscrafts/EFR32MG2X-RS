# LDMA linked descriptors — measured 2026-11-27

Item **C2-DMA.3** of the night queue (`_tmp/night/STATE-NIGHT2.md`). The question was narrow: can channel 0 run two
memory-to-memory legs in order, with the second descriptor fetched by the controller itself, and nothing writing to the
DMA between them?

## Record

Example `hal_dma_link`, flashed twice, two boots, byte-identical, `status=PASS` on both:

```
dmalink: ok=1 d1=8 d2=8 s2=1 lk=0x0 cto=1 c1=8 c2=0 c1r=1 clk=0x0 keep=0xdeadbeef ctl=0x81f0008 ssrc=0x2003ee7c sdst=0x2003eebc
```

| field | meaning | value |
|---|---|---|
| `ok` | the chained call returned `Ok` | 1 |
| `d1` | words of leg 1 that equal their source | 8 |
| `d2` | words of leg 2 that equal their source | 8 |
| `s2` | channel 0's `SRC` register ended at or past the start of leg 2 (the second descriptor was fetched and consumed) | 1 |
| `lk` | `CH0_LINK` read back after the chain: the last descriptor links nowhere | 0 |
| `cto` | the control (`LINK` clear) came back `Timeout` instead of claiming a chain | 1 |
| `c1` | words of the control's first leg that moved | 8 |
| `c2` | words of the control's second leg that moved | 0 |
| `c1r` | in the control, `SRC` still inside leg 1 — no load happened | 1 |
| `clk` | `CH0_LINK` after the control | 0 |
| `keep` | first word of the control's untouched second destination (sentinel) | `0xdeadbeef` |
| `ctl` | `CH0_CTRL` after the chain: `STRUCTREQ` set (bit 3), `XFERCNT` = 0, `BLOCKSIZE` = all, `DONEIFS` set | `0x081f0008` |
| `ssrc` / `sdst` | channel 0's address registers after the chain, both inside leg 2 | `0x2003ee7c` / `0x2003eebc` |

## What the part required, and how it was found

The first implementation put the second descriptor in **channel 1's register set** — `CH1_CTRL`, `CH1_SRC`, `CH1_DST`,
`CH1_LINK` are four contiguous words, which is exactly a descriptor's layout — and pointed the head's link at
`CH1_CTRL`. Two boots gave `d1=8 d2=0 s2=1`: the controller fetched the second descriptor (its address landed in the
channel's `SRC`) and stopped.

A temporary probe (not committed; kept in `_trash/`) ran eight configurations in one boot. Each case reports
`leg1 / leg2 / source-register-past-leg-2-start`:

```
probe: c1=8/0/1 c2=8/0/1 c3=8/0/1 c4=8/0/1 c5=8/0/1 c6=8/8/1 c7=8/8/1 c8=0/0/0
```

| case | second descriptor in | extra | result |
|---|---|---|---|
| c1 | CH1 registers | — | fetched, not run |
| c2 | CH1 registers | `STRUCTREQ` on both | fetched, not run |
| c3 | CH1 registers | `STRUCTREQ` + `REQMODE=ALL` | fetched, not run |
| c4 | CH1 registers | `REQMODE=ALL` | fetched, not run |
| c5 | CH1 registers | `STRUCTREQ` on the loaded descriptor only | fetched, not run |
| c6 | CH1 registers | one **software** `SWREQ` after the fetch | both legs ran |
| c7 | RAM (four words on the stack) | `STRUCTREQ` | both legs ran |
| c8 | RAM | — | nothing ran |

Three facts come out of it:

1. **The fetched descriptor is valid.** c6 runs leg 2 after a software `SWREQ` and nothing else changed, so the words
   the controller read were a working descriptor. What was missing was a trigger, not an encoding.
2. **`STRUCTREQ` is that trigger, and only a descriptor in RAM can carry it.** Writing 1 to bit 3 of `CH1_CTRL` reads
   back 0 — the channel register does not store the bit (the PAC exposes `STRUCTREQ` as a reader only, which matches).
   The same word placed in RAM with the bit set runs the leg (c7); the same word without it does not (c8). c8 is the
   control that says c7's success came from the bit.
3. **`REQMODE` is not the lever.** c3 and c4 differ only in `STRUCTREQ` and both fail; `BLOCKSIZE=ALL` was set in every
   case.

So the second descriptor of a chain lives in RAM. The driver builds its `CTRL` from the word the PAC's typed writer
produced for the head — only `XFERCNT` is replaced and `STRUCTREQ` is added — so no field of the register is encoded by
hand. The probe's nine `CTRL` bits that were not the count are, by construction, the same ones that move a single
blocking transfer today.

## The control, and why it is the item

`ChainMode::FirstOnly` programs the same data with the head's `LINK` bit clear. Both legs' descriptors are present,
complete and valid; the second is never fetched. The call reports `Timeout` — not `Ok` — because the driver's
completion test is "every destination equals its source", so a leg that did not run is a failure it cannot paper over.
`c2=0` and `keep=0xdeadbeef` are the second destination read back untouched: an absence of change with a value, not an
absence of a value.

## Completion detection

`DONE` is one bit per channel (`IF`, RM 25.7.11): the first leg sets it and the second sets the same bit, so it is not
a chain-completion signal, and the driver does not treat it as one. It waits for `DONE`, settles ~256 us (two
memory-to-memory legs of at most 2047 units are a few microseconds at 78 MHz), and then compares the buffers. The only
way to get `Ok` is for every segment to have arrived; the only way to get `Timeout` with the data correct would be a
segment longer than the settle, and the settle is bounded so no such case hangs the caller.

## Delivery

- `crates/efr32mg2x-hal/src/dma/linked.rs` — `Segment`, `ChainMode`, `ChannelState`, `channel_state`, `transfer_chain`
- `crates/efr32mg2x-hal/examples/hal_dma_link.rs` — the instrument above
- `CLAUDE.md`, `STATUS.md`, `BACKLOG.md`, `HAL-STATUS.md` — the "deferred" line removed