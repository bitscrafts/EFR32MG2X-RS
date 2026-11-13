# SYSRTC: two offsets found, and the one question left

Status 2026-11-13 (evening). The Bluetooth stack needs one symbol to link — `sl_sleeptimer_restart_timer` —
and the plan is a HAL timer over SYSRTC. Two real defects were found and fixed; the counter still does not
advance. This file is the complete measurement record, so the next attempt (or a review) starts from facts.

## Fixed, measured, and worth remembering

Both offsets were taken from the vendor headers themselves, by compiling `offsetof` against
`efr32mg24_cmu.h` and `efr32mg24_sysrtc.h` — not from the PAC, not from the reference manual:

```text
CMU.SYSRTC0CLKCTRL   0x240     the PAC's accessor writes 0x1DC  --  a different register entirely
SYSRTC.EN_SET        0x1004    the alias page mirrors the layout  --  0x1000 is IPVERSION_SET (read-only)
```

What those two bugs looked like from outside, and why they took so long:

- Writing the clock select to `0x1DC` left `SYSRTC0CLKCTRL` at **0 = disabled**. A peripheral with no clock
  cannot accept a synchronised write, so `CMD = START` came back as `CFSR = 0x00008200` (**PRECISERR**) with
  `BFAR = 0x400A8010` — a bus error pointing at the register being written. That reads as "the driver's
  logic is wrong", and it sent several rounds into the wrong place.
- Writing `1` to `0x1000` hit `IPVERSION_SET`, which is read-only, so `EN` stayed **0**. The module was never
  enabled and every write after it was meaningless.

After both fixes, on **both** the secure (`0x400A8000`) and non-secure (`0x500A8000`) instance:

```text
gates S=0x40400000 NS=0x40400000 same=true s30=1 ns30=1 | clkctrl=0x1 clksel=1 | configured | en=0x1 enabled=1 | cmd=start accepted
```

So: clock gates set (SYSRTC0 bit 30 and LFRCO bit 22), the S and NS CMU views alias the same register, the
clock select reads back as LFRCO (1), `EN` reads 1, and `CMD = START` is accepted without fault. That is a
long way from where this started — and it is all reusable.

## What still fails

The counter does not advance, and the failure is now in the **synchronisation protocol**:

- reading `SYSRTC0->CNT` (`+0x18`) after the start **stalls the bus** — the program does not reach its next
  statement, so it is not a fault that reports, it is a read that never returns;
- polling `SYSRTC0->SYNCBUSY` (`+0x1C`) with a bounded loop never publishes its result either, which means the
  *read of SYNCBUSY itself* is what stalls, not the loop.

The vendor does both of these plainly (`while (SYSRTC0->SYNCBUSY != 0U) {}` in `sl_hal_sysrtc_start`, and a
plain `CNT` read in `sl_hal_sysrtc_get_counter`), which is the reason this is still an open question rather
than a conclusion: either something earlier in the sequence is still wrong, or this part needs a step the
vendor's device init performs and this project does not (Simplicity's startup runs `sl_device_init_clocks`,
and the stack's own `sl_system_init` is not executed here).

## Everything ruled out, by test rather than by argument

| Suspect | Test | Result |
|---|---|---|
| The PAC's register layout for SYSRTC | repeated with raw pointers at the header's offsets, read `IPVERSION` | same behaviour — PAC offsets for the block are right |
| The PAC's `SYSRTC0CLKCTRL` accessor | compared against `offsetof` from the vendor header | **wrong: 0x1DC vs 0x240**; fixed by addressing raw |
| The `EN_SET` alias address | computed `offsetof` | **wrong: 0x1000 vs 0x1004**; fixed |
| The secure vs non-secure instance | ran the whole sequence on both | identical |
| The secure vs non-secure CMU views | enabled gates through both, read both back | identical value (aliases) |
| The LFRCO bus gate | enabled bit 22, as emlib's `CMU_OscillatorEnable` does | no change on its own |
| The register offsets of the block | compared with `SYSRTC_TypeDef` | correct |
| The init order | followed `sl_hal_sysrtc_init` + `sleeptimer_hal_init_timer` | reached `EN=1`, `CMD` accepted |
| NVIC unmask with a pending compare | removed the unmask from init | not the hang |

## The instruments this produced (kept in the tree)

- `crates/efr32mg24-pac/examples/pac_sysrtc_gates.rs` — publishes a **cumulative** log (each fact appended and
  the whole text republished) so a later fault or hang cannot erase the earlier ones, plus a `HardFault`
  handler that writes stacked PC, `CFSR` and `BFAR` into a **second** mailbox slot.
- `crates/efr32mg24-pac/examples/pac_sysrtc_raw.rs` — raw-pointer variant, used to separate PAC bugs from
  hardware behaviour.
- The rule these encode: a fault that reports is a diagnosis; a fault that loops is a lost evening. And a
  marker that gets overwritten is not a marker.

## The question

Given a SYSRTC that reads `EN=1`, accepts `CMD = START`, and stalls on the first `CNT` or `SYNCBUSY` access
afterwards — what is missing? The candidates worth arguing:

1. a clock-domain step the vendor's `sl_device_init_clocks` performs and this project skips;
2. a required `SYNCBUSY`-free path for the *first* access after `START` (some Series 2 peripherals need a
   read of `STATUS` to latch the domain);
3. something about the LFRCO being selected but not *stabilised* (a ready wait the vendor does elsewhere);
4. a hardware erratum on SYSRTC when started from a warm reset without `SWRST`.

Evidence for and against each is in the table above; the alternative to resolving it is to implement the
timer over TIMER0, which is proven on this board, and accept no EM2 sleep until SYSRTC works.
