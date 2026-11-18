# What is still missing in the HAL: the gaps, measured and inherited

**Written 2026-11-18, after six cores moved into the HAL and the dry-run reached the upload.** Nothing
is published. This page exists so the question "what is missing?" has one answer that separates what
was measured tonight from what earlier reviews asserted and nobody has re-checked.

## Where the HAL stands tonight

```
crates/efr32mg2x-hal/src/          22 top-level modules: adc alias board bus buzzer clock cores delay
                                   display dma dwt emu i2c lib microphone mvp pin pwm rtc sensors spi
                                   sysrtc timer usart wdog
cores/                             the six former libraries, one copy each, 29 host tests still green
cargo publish --dry-run            reaches "Uploading efr32mg2x-hal v0.1.1" -- no unpublished dependency
```

That is the state after `C1`..`C6`. It says the HAL **can** be packaged; it says nothing about whether
it **should** be published, which is what the rest of this page is about.

## 1. Two peripherals do not reach a pin at all (the largest functional hole)

Measured in an earlier session and not re-checked tonight: the hardware `i2c`, `usart` and `spi`
drivers have no pin routing. On this part a peripheral is connected to a pin by setting `ROUTEPEN`
and `ROUTELOC` in the peripheral's `ROUTE` register; without it the peripheral works internally and
never touches the pad. No example can exercise them, which is consistent with there being no example.

Consequence: what the product uses on the bus today is the **bit-bang** implementation
(`bus/bitbang.rs`), which is also reported to return a false ACK. If the board is ever to use the
hardware peripherals, this is the work.

**How to close it:** one spec per peripheral, each with an example that toggles a real pin and an
exit criterion that measures the pin, not the register.

## 2. The review's hardware items are open, and the board is not the obstacle

**Corrected 2026-11-18 by the operator:** during normal implementation the module is connected to
the lnx host and can be used. This section previously said these items needed "a bench session",
which framed them as blocked. They are not blocked; they need a flash and an observation, under the
rules that already exist (`scripts/flash.sh` with `--confirm`, the `load_min >= 0x08006000` guard and
the read-back, never a chip erase).

These come from `04-reference/HAL-RADIO-BLE-REVIEW-2026-11-17.md` and are still open. Each is marked
with why it cannot close in software:

| id | what it is | why it is still open |
|---|---|---|
| H17 | `DOUT` written by read-modify-write instead of the `DOUT_SET`/`DOUT_CLR` aliases | flash a bit-bang example and see whether the ACK still arrives; the module's IMU on the bus is a real peer |
| H21/H22 | `19_000_000` written where the bus clock is 39 MHz after the radio starts | measure the actual clock on the board -- a baud rate that comes out wrong is the instrument |
| H28 | EM2 entry and exit sequence | flash and watch whether the wake-up happens; the current meter is optional, not required |
| B12 (rest) | `psa_import_key` attributes type/bits not validated | re-pair with the phone after the change; the phone is the instrument |

**How to close them:** one bench session with three instruments: a logic analyser on the bus, the
current meter for EM2, and the phone for pairing. That is a different kind of work from tonight's.

## 3. Four `TODO`s in the source, and what each hides

```
TODO: implement                                  (unqualified)
TODO: implement proper LFXO enable + wait + select sequence
TODO: Store buffer metadata in channel state
+ 1 more
```

The LFXO one matters: the low-frequency oscillator start-up sequence is a wait loop with a timeout,
and getting it wrong is the difference between a 32 kHz clock and no clock. The others are inside
paths the product does not use yet.

## 4. The rules that moved tonight have no test of their own

`compare_target` and `WideTick` are now `crates/efr32mg2x-hal/src/cores/sysrtc.rs`, compiled by both
the HAL and the library harness. They keep 16 tests, which is the good half. The bad half: they are
the only arithmetic in the HAL with that coverage. `bus/bitbang.rs`, `pin.rs` and `emu/` have none,
and those are the three places where this project has already shipped a defect.

## 5. The layering question, which is not a defect but is undocumented debt

`efr32mg24-emlib-sys`, `efr32mg24-radio` and `efr32mg24-bluetooth-sys` all depend on `efr32mg2x-hal`.
A vendor-binding crate depending on a HAL is a rare shape; `nrf-softdevice` deliberately avoids it.
Written up with sources in `04-reference/HAL-LAYERING-QUESTION-2026-11-18.md`, unanswered.

## 6. The publication checklist still has two false lines

```
item 11  the HAL's manifest says efr32mg2x-pac = { version = "0.1.0" } and the published one is 0.1.1
         (resolves, because caret, but it misstates what it uses)
item  8  my own grep for the README install line does not recognise
         { version = "0.1.1", features = ["rt"] }, so it fails correct work
```

Both are one-line fixes and both must be done before the checklist's verdict can be believed.

## What I would do first, if the order were mine

1. Fix the two checklist lines (minutes) and re-run `pre-publish.sh` on the HAL — until its verdict is
   trustworthy there is no honest way to say the HAL is ready.
2. Pin routing for `i2c`/`usart`/`spi`, with an example each. That is the gap that stops the product
   from using the hardware at all; everything else is refinement.
3. The bench session for H17, H21/H22, H28 — with the board, the analyser and the phone.
4. The layering answer from an independent read, and then one paragraph in `docs/02-architecture/`
   that says which way the tree points, so nobody re-opens it.

## What this page does not claim

It does not claim the HAL is close to finished. Six cores moved is a structural win — the crate can
be packaged — and it changed nothing about whether the drivers are correct. The defects that need the
board are still exactly as many as they were this morning.
