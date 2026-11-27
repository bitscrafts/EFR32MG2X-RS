# mvp

The MVP — the on-chip matrix/vector processor — driven through a safe API.

## What it is

The MVP — the on-chip matrix/vector processor — driven through a safe API.

The MVP holds eight instructions of three 32-bit words, eight loop controllers and five array
controllers, and it reads and writes *system memory*: an operand is a slice you hand it, not a copy you
push. That is the whole point of it here — a matrix multiply costs the CPU the loop, not the arithmetic.

## What this driver supports, and what it does not

One operation, done properly: `add_fp16` — element-wise addition of two `binary16` vectors into a third.
That is the operation whose encoding is proven on this board (the PAC example `pac_mvp_add` measured
`exp=396 got=396`), and it is the one the dosimeter's GMM could use as a building block.

Everything else — MAC with accumulation, transposed operands, other base types — needs the instruction
encodings from sl_mvp.h, which are **not** in this repository for those cases. Guessing them would
produce a driver that compiles, runs, and returns wrong numbers, which is worse than a driver that says
what it does not do. When a case is added, it comes with a self-checking example in the same change.

## The traps this driver encodes, all of them measured

- `EN` must be set and `DISABLING` must clear before any configuration write; writes are dropped silently
while the block is coming up.
- Configuration writes read back as zero if the CMU gate is off, so `take()` turns the gate on first.
- `CMD.INIT` goes in its own access: writing INIT and START together left the sequencer IDLE with the
output array untouched, which is what a run of zero instructions looks like.
- The loop passes `NOINCR`: the instruction's streams advance the arrays. Incrementing a dimension in the
loop as well skipped every element twice.
- The opcode is `ADDC` (334), not the `ADDR` written in some vendor examples; and the register set is
`X = R1, A = R2, Z = R3`.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Error` | enum | — |
| `MVP_MAX_LEN` | const | Largest element count the MVP can encode: `SIZE` and `NUMITERS` are 10-bit fields (`0x3FF`). |
| `Mvp` | struct | The MVP, with the borrow of its register block. |
| `add_fp16` | fn | Element-wise addition of two `binary16` vectors into `out`. |
| `is_idle` | fn | — |
| `take` | fn | Take the block, turn its clock gate on, enable it and wait for the enable to settle. |

## Registers it touches

`CmuNs`, `MvpNs`

## What was measured

No mailbox record in `docs/` names this module, so treat its API as unmeasured on hardware. The first measurement belongs here, with the line the board produced and the controls around it.

## What is not proven

- not measured    MAC with accumulation (needs encodings from sl_mvp.h that the repository does not have). (MEASURED-2026-11-17.md (`../../../../docs/03-development/MEASURED-2026-11-17`))
- C -- advanced | complete to stage 2 | `adc`, `dma`, `emu`, `rtc`/`sysrtc`, `wdog`, `gpcrc`, `mvp`, `letimer`, `prs`; DMA stages 3 and 4 deferred |. (PLAN.md (`../../../../docs/03-development/PLAN`))

## Using it

No example imports this module yet, so its API has never been exercised from `examples/`.
