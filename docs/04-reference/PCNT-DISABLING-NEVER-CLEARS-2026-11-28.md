# PCNT: `EN.DISABLING` never clears, and that is why the give-up was harmless

**Written 2026-11-28. A measurement, not a fix.** The code is as it was; this is the premise that has to be settled
before the wait can be made fallible.

## What was tried

An outside review observed that several waits in this crate bound themselves and then discard the timeout. The worst
of them were in `src/pcnt.rs`, because the block's own module documentation says what happens when a synchronised
register is touched twice inside one synchroniser period: a **precise bus error** (`BFAR = PCNT0 + 0x14`, the `CMD`
register) that halts the core. A wait that gives up and writes anyway is therefore a fault with a delay on it.

So `wait_disabled` and `sync` were made to return `bool`, `write_cmd` to return `Result<(), Error>` and to **not write**
when the synchroniser never freed, and the six public command methods to return `Result<(), Error>`. The example was
taught to act on that answer (a `must()` that records and halts).

## What the board said

    seq=1 status=FAIL len=10
    pcnt: init

`Pcnt::init` is `write_cmd(CORERST)` followed by `wait_disabled()`, and it is the **first** thing the example does after
taking the block. `wait_disabled` polls `EN.DISABLING` in a loop of 200 000 iterations of `delay(64)` -- about 0.7 s at
19 MHz, against a handshake whose own documentation puts it at 30 us (one 32.768 kHz branch period). It never cleared.

So the premise "`DISABLING` is set briefly and clears" is **false at least in this state**: right after `CORERST`, with
`EN` low, that bit does not clear. The old `break` was therefore hiding a condition that is benign here -- the next
`CFG` write did not fault, which is why the example used to run to its own end (and then wait for an edge that never
came, leaving `magic 0`).

## Why the code went back

Turning that benign path into an error is worse than the silence it replaced: an error the hardware does not have is a
false negative on a healthy board, and it would have made `hal_pcnt_gpio` record `FAIL` from the first line. `src/pcnt.rs`
and the example are exactly as they were at `f847ae5`.

## What would settle it

One instrument, one boot, recording the raw words instead of a boolean:

* `EN` before `CORERST`, immediately after it, and at the end of the wait;
* `SYNCBUSY` at the same three moments;
* `CMU`'s clock select for the block, to show it is running at all -- a block with no clock reads like a block whose
  handshake never finishes;
* the count of iterations the wait actually spent, so "never" and "slow" can be told apart.

Two outcomes, each with its own fix: if `DISABLING` only means something while `EN` has been high, the wait belongs
after `disable()` and not after `CORERST`, and `init` needs no wait at all. If the bit is genuinely stuck on this part,
the wait has to poll `SYNCBUSY` instead, which the module already trusts for the command register.
