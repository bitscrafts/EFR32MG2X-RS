# PCNT: the command never completes, and the first explanation was wrong

**Written and corrected 2026-11-28. Two measurements, the second of which refuted the first.**

## How this started

An outside review observed that several waits in this crate bound themselves and then discard the timeout. The worst
were in `src/pcnt.rs`, where the module's own documentation records that two accesses to a synchronised register inside
one synchroniser period raise a **precise bus error** (`BFAR = PCNT0 + 0x14`, the `CMD` register) that halts the core.

`wait_disabled`, `sync` and `write_cmd` were made fallible, along with the six public command methods, and the example
was taught to act on the answer. The board answered immediately:

    seq=1 status=FAIL len=10
    pcnt: init

## The first explanation, which was wrong

I read that as `wait_disabled` -- the `EN.DISABLING` poll, 200 000 iterations of `delay(64)`, about 0.7 s at 19 MHz --
never finishing, and wrote a document saying so. It fitted: `init` is `CORERST` followed by that wait.

## The measurement that refuted it

`examples/hal_pcnt_request.rs` records the raw words at the three moments that tell the candidates apart, plus the
iterations actually spent:

    seq=3 status=FAIL len=114
    pcnt-req: en0=00000000 sync0=00000000 en1=00000000 sync1=00000002 en2=00000000 sync2=00000002 spins=0 clk=00000001

* `spins=0` -- the `DISABLING` loop ran **zero** times. `EN.DISABLING` was clear throughout. The wait I blamed is fine.
* `sync1 = sync2 = 0x2` -- `SYNCBUSY.CMD` (bit 1) is set by the `CORERST` write and is **still set** 0.7 s later. The
  command is never acknowledged: the block's clock domain does not advance, so the crossing never completes.
* `clk = 0x1` -- `CMU.PCNT0CLKCTRL.CLKSEL` selects EM23GRPACLK, which is the branch the driver intends.

So the `Err` came from `write_cmd`'s **post-write** `sync(SYNCBUSY.CMD)`, not from `wait_disabled`. That also explains
why `hal_pcnt_gpio` sits waiting for an edge that never arrives: a counter whose command never lands was never started.

## What is established now

* `EN.DISABLING` is clear after `CORERST` -- so the wait after `init` is not where the failure is, and making it
  fallible changes nothing on this board.
* `SYNCBUSY.CMD` stays set after a `CMD` write in the state the example runs in. A `sync` that polls it will time out,
  and the old `break` was hiding exactly that.
* The clock select points at EM23GRPACLK sourced from LFRCO, and opening `CLKEN0.LFRCO` does not clear
  `SYNCBUSY.CMD` -- so the missing piece is below the gate, not above it.

## The clock branch, interrogated: it is not the LFRCO gate

`Board::pcnt` already opens `CLKEN1.PCNT0` and points the chain at EM23GRPACLK, whose source it sets to LFRCO. What it
does **not** do is open `CLKEN0.LFRCO` (bit 22): the first record shows `clken0 = 0x04000000`, which is GPIO alone.

Opening it was tried, and the board answered:

    before:  clken0=04000000   sync2=00000002
    after:   clken0=04400000   sync2=00000002

The gate opened (bit 22 set) and `SYNCBUSY.CMD` stayed set. So the LFRCO gate is not the cause, and the change was
reverted -- it altered the observable and not the behaviour, which is a measurement, not a fix. The chain still shows
`clk = 0x1` (EM23GRPACLK selected) and `em23 = 0x1` (**LFRCO**), so the source and the selector agree; what remains
unmeasured is whether LFRCO is actually oscillating, and this device's CMU has no `OSCENCMD` in the generated PAC to
ask it.

## The clock branch, excluded

The next question was whether the branch has a clock at all, since a block whose clock domain does not advance answers
every write the way a broken one does. `ULFRCO` is the one oscillator on this part with no enable bit, so the branch was
sourced from it and the same command written again -- in the same boot, with the previous reading still in the record:

    pcnt-req: en0=00000000 sync0=00000000 en1=00000000 sync1=00000002 en2=00000000 sync2=00000002
              spins=0 clk=00000001 clken0=05000000 em23=00000003 sync_ulfrco=00000002

`em23 = 3` is ULFRCO selected, `clken0 = 0x05000000` has its gate (bit 24) with GPIO's, and `sync_ulfrco = 0x2` says
`SYNCBUSY.CMD` **still** does not clear. So the clock is not the subject: with an always-running source the command is
just as unacknowledged. Three experiments have now narrowed this to the block's command path itself.

## The third experiment, and the fault that answers it

`EN` read `0` at every moment of the first experiment, so the next hypothesis was that `CMD` only completes while the
counter is enabled. The test: write `CMD`, set `EN`, poll; clear `EN`, write `CMD`, poll. The instrument now links
`common/fault_record.rs`, because writing to registers the driver owns with the block enabled is exactly where a silent
stop would be ambiguous. It did not hang -- it faulted, and said so:

    seq=2 status=FAIL len=75
    fault: cfsr=00008200 hfsr=40000000 bfar=59030014 mmfar=59030014 pc=08006564

`BFAR = 0x5903_0014` is `PCNT0 + 0x14`: **the `CMD` register**, the very address this module's documentation has
recorded since 2026-11-26. `CFSR = 0x8200` is `BFARVALID | PRECISERR`, a precise bus error on a write -- the same
signature the project has met on `BURTC_NS + 0x08` and `ACMP0_S + 0x04`, both of them a write to a peripheral whose clock
was not enabled.

So the hypothesis is answered *and* displaced: the write does not sit unacknowledged, it faults, and it faults the way an
unclocked block faults. `SYNCBUSY.CMD` reading back set was the symptom of a write that never landed, not of a command
in flight.

## The next question, narrowed to one bit

The branch oscillator is excluded (ULFRCO, same result). What has never been read is `CLKEN1.PCNT0` -- the block's own
bus gate, the one `Board::pcnt` opens -- and `CMU`'s view of whether the block is in reset. The next boot records those
two words before the first command:

    clken1            the bus gate bit for PCNT0, read after `board.pcnt()` returns
    cmu.status        whatever this device exposes about a peripheral held in reset

If the gate reads closed after the driver opened it, the defect is in `Board::pcnt` and the fault is explained. If it
reads open and the write still faults, the block is being reset by something else, and the search moves to the reset
causes -- `RSTCAUSE` and the peripheral's own enable sequencing.

## The pattern worth testing next (superseded by the fault above)

`EN` reads `0` at all three moments of the first experiment. If `SYNCBUSY.CMD` only completes while the counter is
enabled -- that is, if a command issued with `EN` low waits for a domain that is switched off on purpose -- then every
command this driver writes before enabling the block sits in that state, and the first `sync` to time out is
`write_cmd`'s. The test is one boot: set `EN`, write `CMD`, read `SYNCBUSY` again; then clear `EN` and repeat. If the
first clears and the second does not, `init` must enable the block before commanding it, and the module's own rule
("`CFG` is refused with `EN` set") does not apply to `CMD` -- which is a distinction this driver has never made.

## What would settle the rest

One more boot, same instrument, with the clock side added: `CMU.CLKEN0` (the block's own gate), `CMU.EM23GRPACLKCTRL`,
and the `STATUS` of the branch's oscillator. If the branch is not enabled, the fix is in `Board::pcnt` and the counter
was never able to count -- which would explain both this and the missing edges. If it is enabled and the command still
never lands, `CORERST` is the wrong first command on this part and `init` should start with something else.

Until one of those is measured, `src/pcnt.rs` stays as it was, and the give-up stays -- with this file as the reason it
is a give-up and not an oversight.
