# MVP (matrix/vector accelerator) — research notes, 2026-11-11

**Status: documented, example written, execution not yet proven.** The research
answers what the accelerator is and how it is programmed; the PAC example builds,
flashes and runs, but the result vector stays zero, so the program is not being
executed. Details and next steps at the end.

## What the hardware is

EFR32xG24 Reference Manual, chapter 31 (Rev 1.1, Jan 2024):

- a co-processor with its own **sequencer**, offloading floating point matrix and
  vector work from the Cortex-M33;
- **one complex multiply-accumulate per cycle** — four single-precision
  multipliers and four adders, quoted at **6× the M33 FMAC rate**;
- **eight instructions**, each **three 32-bit words**;
- **eight loop controllers**, nestable;
- **five array controllers**, each iterating up to three dimensions, 1024
  elements per dimension, stride up to 2047;
- **three AHB ports** (two read, one write) so it streams from system memory
  without touching the CPU's bus path;
- ALU operands are fp16 pairs (real/imaginary) packed in a 32-bit register, with
  negation and zero-masking per part, and eight ALU registers for accumulation.

## The part that decides how to write code for it

The same chapter's revision history says, verbatim:

> **Revision 1.1:** "MVP: Removed Registers and Definitions from MVP section - use
> of the MVP shouldn't require access to these registers"

Silicon Labs deliberately stopped documenting the register interface. The
supported path is their SDK driver, which does not program registers one at a
time at all: it builds a `sli_mvp_program_t` in RAM and memcpys it over
`MVP->ALU[0]`, then writes `CMD = INIT | START` as the last word of that copy.
The program *is* the register block.

The register layout had to be reconstructed from two sources that disagree on
order, which matters:

| source | order |
|---|---|
| SDK `sl_mvp.h` struct `sli_mvp_program_t` | ALU[8], ARRAY[5], LOOP[8], INSTR[8], CMD |
| this PAC, from the SVD (`mvp_ns.rs`) | STATUS 0x10, IF 0x1C, IEN 0x20, **ALU[8] 0x64**, **ARRAY[5] 0x84**, **LOOP[8] 0xD4**, **INSTR[8] 0x114**, CMD 0x174 |

The PAC offsets are the hardware truth (they come from the vendor SVD), so the
SDK's memcpy only works because the SDK's struct happens to be laid out against
those offsets in its own build. Any hand-written program must use the offsets
above — an earlier attempt in this session read `0x20`/`0x24` believing they were
array configuration, when they are `IEN` and the first loop register.

## Instruction encoding (from `sli_mvp.h`, Zlib)

- **CFG0** — `aluin0regid` bits 0..2, then per-input flags (real/imag zero and
  negate) at bits 4..7, 12..15, 20..23; `aluoutregid` bits 28..30.
- **CFG1** — three streams: input0 bits 0..9, input1 bits 10..19, output bits
  20..29. Each carries `regid`, `load`/`store`, `arrayid`, and three
  increment-dimension bits.
- **CFG2** — `loopNbegin`/`loopNend` for loops 0..7 at bits 0..15, `aluop` bits
  20..28, `endprog` bit 31.
- **Array DIM0** — `size` 0..9, `basetype` 12..13, `complex` 14, `stride` 16..27.
  Basetypes: `UINT8 0`, `INT8 1`, `BINARY16 2`, complex variants 4/5/6.
- **Loop CFG** — `numiters` 0..9 (iterations minus one), then a four-bit nibble
  per array giving which dimensions it advances (`4` = width/inner).
- Opcodes exist as an enum in this PAC: `Addr = 74`, `Macr2a = 471`, `Mulr2a =
  407`, `Max = 75`, `Relu2 = 70`, and so on.

## What the SDK accelerates, and its limits

From Silicon Labs' ML documentation: `ADD`, `FULLY_CONNECTED`, `CONV_2D`,
`DEPTHWISE_CONV_2D`, `TRANSPOSE_CONV_2D`, `AVERAGE_POOL_2D`, `MAX_POOL_2D` are
handed to the MVP when tensor shapes allow, otherwise the runtime silently falls
back to CMSIS-NN or reference kernels. Inputs and outputs are int8; **internally
the accelerator computes in fp16**, so accuracy drops slightly on long
accumulations. Even channel counts let it pack two int8 values into one complex
word, doubling effective width to 2048.

That is the practical answer to "how do I use the ML accelerator": through
TensorFlow Lite Micro with Silicon Labs' kernels — not by hand-programming the
sequencer for a network.

## The example in this repository

`crates/efr32mg24-pac/examples/pac_mvp_add.rs` — the smallest honest exercise:
`out[k] = a[k] + b[k]` over eight fp16 elements, one instruction, one loop, three
arrays, with the expected sum computed by the CPU so a wrong program cannot pass.

It compiles and flashes cleanly. It reports:

```
mvp sum exp=396 got=0
```

So the engine did not execute the program. What is established:

- `CMU_CLKEN1.MVP` (bit 30) is the bus clock gate, and it reads back set;
- the MVP has two aliases, `0x4D00_0000` (secure) and `0x5D00_0000` (non-secure);
- with the clock gated off, reads of the block fail with a bus error; with the
  firmware running there are reads that succeed and reads that fail, which means
  the access conditions are not yet understood;
- the PAC register offsets are correct (they come from the SVD), so the writes in
  the example do reach the documented addresses.

Next steps, in order:

1. **Use the SDK as the oracle.** Build one of their samples (or `sl_math_mvp_*`
   test) for this board, run it, and dump the register block with SWD at the
   moment before `CMD` is written. That yields the exact register values for a
   known-good program, and this example can then be diffed against it field by
   field. Guessing the last steps from a removed register chapter is not a plan.
2. Confirm whether `CMD = INIT | START` in one write clears the program (if INIT
   resets the configuration, the program must be reloaded *after* INIT, which is
   what the SDK's struct copy with CMD last would achieve in one burst).
3. Check the AHB port mapping (`MVPAHBDATA0/1/2PORTSEL`) if the engine starts but
   the arrays read zero: the ports remap to specific RAM blocks.
