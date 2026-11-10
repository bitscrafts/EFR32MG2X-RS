# EFR32MG24 PAC notes: clocking and timer behaviour

Facts established on the bench on 2026-11-10, each one with the measurement that
produced it. They are here because every one of them costs an evening to
rediscover, and because three of them are *not* what the register names suggest.

---

## 1. Bus-clock gates are mandatory, and the failure is a bus fault

`CMU_CLKEN0` has one bit per peripheral. Touching a register of an ungated
peripheral raises a **precise data bus error**, and `cortex-m-rt` parks the core
in `HardFault_` — the image "flashes fine" and does nothing.

Measured, SYSRTC0 without `CMU_CLKEN0.SYSRTC0`:

```
CFSR = 0x00008200   (PRECISERR | BFARVALID)
BFAR = 0x400A8020   (SYSRTC0_S + 0x20 = LOCK)
```

Bit positions that matter on this part (from the SVD, verified against the
peripheral bases):

| bit | gate | bit | gate |
|---|---|---|---|
| 4 | TIMER0 | 22 | LFRCO |
| 12 | LETIMER0 | 26 | GPIO |
| 19 | HFRCOEM23 | 30 | SYSRTC0 |

`CMU_CLKEN1` gates the radio and miscellaneous blocks; nothing in the current
examples needs it.

## 2. Oscillator selection is per clock branch, not global

| branch | register | options seen |
|---|---|---|
| `EM01GRPACLK` (TIMER0-4, USART0) | `CMU_EM01GRPACLKCTRL` @ 0x120 | HFRCODPLL, HFXO, FSRCO, HFRCOEM23 (+retimed) |
| `EM23GRPACLK` (LETIMER0) | `CMU_EM23GRPACLKCTRL` @ 0x140 | LFRCO, LFXO, ULFRCO |
| `SYSRTC0CLK` | `CMU_SYSRTC0CLKCTRL` @ 0x240 | LFRCO, LFXO, ULFRCO |

There is **no LFRCO option on `EM01GRPACLK`**, so a low-frequency time base
cannot be given to TIMER0. That is why the 1 Hz blink ends up on TIMER0 +
HFRCOEM23 (≈19.6 MHz measured) rather than on a 32.768 kHz source.

`LFRCO` needs no explicit enable on this die: `LFRCO_S.STATUS` reads
`0x00010001` (RDY=1, ENS=1) with `CLKEN0.LFRCO` set. `HFRCOEM23` does: set
`CTRL.FORCEEN` and poll `STATUS.RDY` before using it.

## 3. TIMER: writes are refused in the wrong enable state

Both faults below are `CFSR=0x00008200`, i.e. precise bus errors, and both land
the core in `HardFault_`:

| write | EN state | BFAR | register |
|---|---|---|---|
| `CFG` | EN=1 | 0x40048004 | CFG |
| `TOP` | EN=0 | 0x4004801C | TOP |

Working order:

```
CFG  (disabled)  ->  EN=1  ->  TOP  ->  CMD.START
```

`TOP` is 32-bit on this part (`Bits 0:31`), not 16-bit.

## 4. TIMER: `IF.OF` cannot be cleared

After an overflow, `IF` reads `0x1` and stays `0x1`:

- writing `1` to the `OF` bit: still `0x1`;
- writing `0xFFFFFFFF` to the whole register: still `0x1`;
- three reads with the core running between them: `0x1` every time.

A loop that polls `IF.OF` therefore spins at CPU speed. Measured on the first
attempt at this example: **21 352 toggles per second** where 2 were expected.
The periodic event has to come from the counter itself — `CNT < previous` is one
wrap — which is what `pac_timer_blink.rs` does.

## 5. LETIMER: counts once, then stops

With `CTRL.CNTTOPEN=1`, `COMP0=0x4000`, `REP0=0`:

- `CNT` runs down from `0x4000` to 0 in ~0.5 s, raises `UF`, and **stops**;
- `IF.COMP0` never sets (`IF` read back `0x6` = `COMP1|UF` at every check);
- software reload of `CNT` on `UF` produced a busy loop, because `UF` stayed
  latched (`IF` again `0x6` after clearing).

Also: a `CNT` write issued while `SYNCBUSY.CNT` is set is dropped — the counter
then sits at 0 forever. Whatever the intended periodic recipe is, none of the
obvious ones worked here, and the timer that did work is TIMER0.

## 6. SYSRTC: `GRP0_IF.CMP0` polling is not a period

With the bus gate open and `SYSRTC0CLK=LFRCO`, `CMP0=16000`:

- the first version produced ~29 000 toggles/s instead of 2/s — the compare
  condition re-arms far faster than a polling loop can clear the flag;
- the integer maths in that version also truncated: `(32768/1000)*500 = 16000`,
  not 16384, which is a 2.4 % error before any clock error is considered.

For a periodic *output*, drive the pin from the peripheral or use a timer that
wraps by itself; do not poll a compare flag from a busy loop.

---

## What the NOP-loop blink really measured

`blink_xiao` / `pac_blink_base` toggle as fast as the CPU executes the loop, so
their rate is a clock measurement, not a blink:

| image | measured | implied SYSCLK |
|---|---|---|
| `pac_blink_base` (1 000 000 NOP/phase) | 6.30 toggle/s | ≈19 MHz (FSRCO) |
| `pac_blink_double` (500 000/phase, gap) | 6.16 toggle/s over the group | ≈19 MHz (FSRCO) |

That is the clock the core runs on when `probe-rs reset` starts the image
directly. After a power cycle the bootloader configures HFXO/HFRCODPLL and the
same image runs 2-4× faster — which is exactly why the 1 Hz blink moved to a
hardware timer.
