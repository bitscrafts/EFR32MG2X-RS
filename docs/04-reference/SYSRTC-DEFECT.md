# SYSRTC: the counter does not start, and what the measurements rule out

Open defect, 2026-11-13. The Bluetooth stack needs one last symbol to link — `sl_sleeptimer_restart_timer`
— and the plan was to provide it as a HAL timer over SYSRTC (see `PURE-RUST-PATH.md`). The timer module
exists (`crates/efr32mg24-hal/src/sysrtc.rs`) and does not work. This file records exactly how far it got,
because the next attempt should not repeat any of it.

## What the hardware does

```
FAULT step=3 pc=0x333d7065 cfsr=0x00008200 bfar=0x500a8010
                          ^^^^^^^^            ^^^^^^^^^^
                          PRECISERR + BFARVALID  SYSRTC0->CMD, non-secure alias
```

A **precise bus error on the write to `CMD`** (`base + 0x10`), on both the secure (`0x400A8010`) and
non-secure (`0x500A8010`) alias. Earlier runs showed the same signature at the secure address. In one run
the write appeared to hang instead of faulting, which is the same event seen from outside: a synchronised
register that is never acknowledged.

## What is ruled out, by measurement

| Suspect | How it was tested | Result |
|---|---|---|
| The PAC's register layout | repeated the whole sequence with raw pointers at the header's addresses, and read `IPVERSION` | same fault — the PAC is not the cause |
| The secure/non-secure alias | ran the sequence on `0x400A8000` and on `0x500A8000` | both fault at `+0x10` |
| The bus clock gate | enabled `CMU_CLKEN0.SYSRTC0` (bit 30) and read `CLKEN0` back | bit set in the read-back |
| The LFRCO gate | also enabled `CMU_CLKEN0.LFRCO` (bit 22), as emlib does in `CMU_OscillatorEnable` | no change |
| The register offsets | compared against `SYSRTC_TypeDef` in `efr32mg24_sysrtc.h`: IPVERSION 0x00, EN 0x04, SWRST 0x08, CFG 0x0C, CMD 0x10, STATUS 0x14, CNT 0x18, SYNCBUSY 0x1C, GRP0_IF 0x40 | offsets correct |
| The init order | followed `sl_hal_sysrtc_init` + `sleeptimer_hal_init_timer` from Gecko SDK v4.4.5: wait `SYNCBUSY`, `CFG`, group config, interrupts off and cleared, `EN_SET`, `CMD = START` | same fault at `CMD` |
| Writing `CMD` before `EN` | the first version of the HAL module did exactly that | faults too |

The one thing **not** ruled out, and the strongest remaining hypothesis: **the SYSRTC's clock is not actually
running**, so its synchronised registers can never complete a write and the bus returns an error. The gates
are set, `CMU_SYSRTC0CLKCTRL.CLKSEL` reads back as LFRCO, and emlib suggests nothing further is needed —
which leaves the possibility that the oscillator needs its own start (`LFRCO->CTRL`/`SYNCBUSY`) or that a
device-init step that Simplicity's startup code performs (`sl_device_init_clocks`) is being skipped here.

## What this cost, and the rule it reinforces

Roughly forty flash-and-read cycles, most of them spent on hypotheses that a five-second check would have
settled: the PAC versus raw pointers, one alias versus the other. The two techniques that actually produced
information are now in the tree and should be used first next time:

- **`crates/efr32mg24-pac/examples/pac_sysrtc_raw.rs`** — a probe that publishes a step marker after every
  register access, plus a `HardFault` handler that writes the stacked PC, `CFSR` and `BFAR` to a **second**
  mailbox slot. The second slot matters: the first version's handler overwrote the markers, so a fault
  destroyed the evidence of how far the run had got.
- **`BFAR` names the register.** Reading it turned "it faults somewhere in the init" into "it faults on this
  word at this address" in one step, after several cycles of guessing from a stale mailbox.

## The decision this forces

The stack needs a timer; it does not need *this* timer. Two ways forward:

1. **Keep digging on SYSRTC.** The oscillator-start path is the remaining lead, and the measurement to make
   next is reading `LFRCO->STATUS`/`SYNCBUSY` after selecting it — if the READY bit never sets, the
   hypothesis is confirmed and the fix is in the CMU, not in this module.
2. **Take TIMER0 instead.** TIMER0 is already proven on this board (the buzzer and PWM examples run on it).
   A HAL timer over TIMER0 unblocks the stack today at a stated cost: it cannot keep time in EM2, so the
   device cannot sleep deeply while a Bluetooth connection is idle.

Either way the timer stays a HAL service, so Zigbee and Thread inherit it unchanged — which was the whole
point of writing it there rather than as a `sl_sleeptimer` shim.
