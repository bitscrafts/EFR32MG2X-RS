# Independent review of f20 (transmit timeout), 2026-11-28

**Reviewer:** `claude-opus-5-5`, read-only session `3a58d2a6-7ca3-40ea-baf0-89a174327cbf`, 80-turn cap.
**Provenance:** the text below is the reviewer's final message, verbatim. It was asked to write exactly one file
*outside* this repository (`/projects/_tmp/efr32mg24-rs/f20-review/opus-f20.md`); the CLI's write permission was granted
with a malformed pattern (`Edit(///projects/...)`, three slashes), so every edit was refused and the findings arrived
in the transcript instead. Copied here unchanged by the orchestrator.
**What it judges:** the f20 fix (`9621f31`) and the seven instrument attempts recorded in
`F20-TX-TIMEOUT-INSTRUMENT-2026-11-28.md`.

---

I couldn't write the review file: permission to create `docs/04-reference/OPUS-REVIEW-F20-2026-11-28.md` was refused, so nothing was written to the repo. Grant it and I'll write it as drafted. Here are the findings.

**1. Your test hook never disabled the transmitter.** Both hooks write `CMD` with `txen().clear_bit()`. `CMD` only acts on bits written as 1, so writing 0 does nothing. Disabling is a separate bit, `CMD.TXDIS` (bit 3), on both blocks. You can see it in the PAC at `usart0_s/cmd.rs:43` and `eusart0_s/cmd.rs:37`, and in the vendor header (`USART_CMD_TXDIS`, `efr32mg24_usart.h:526`). The hooks are at `src/usart/mod.rs:356` and `src/eusart/mod.rs:212`. Every result in your list fits a transmitter that kept running:
- Attempt 4 accepted every byte because they were actually being sent.
- Attempt 6's "a write to a disabled transmitter does not clear `TXC`" was measured on a live transmitter, so it isn't established.
- No record includes `STATUS.TXENS`, the one bit that would have shown this.

**2. The `Timeout` you saw was a bug in `flush`, not your bound being reached.** `STATUS.TXC` starts at 0 after reset (`STATUS` resets to `0x2040`), so `flush()` on a healthy port that has sent nothing returns `Timeout`. Attempt 5 measured exactly that. It's the same thing as the "broken control" (`control_flush=0`) in your example. The same root cause does more damage elsewhere:
- The `nb` flush (`usart/traits.rs:43`) returns `WouldBlock` forever on a fresh port, so `nb::block!` hangs.
- Spi0's `flush` (`spi/traits.rs:34`) has no bound at all, so it hangs a healthy bus that hasn't transferred yet.
- Fix: a `tx_pending` flag in the driver; `flush` returns `Ok` straight away if nothing was written since the last flush.

**3. Q1: keep the bound on `write`.** "It cannot fire" rests on the hook that did nothing. On EFR32 USARTs the buffer is documented to keep data written while the transmitter is disabled. If that holds here, a real `TXDIS` makes `write` return `Timeout` once the buffer fills. Flow control (CTS held high) is a second real trigger. One boot with `TXDIS` and `txens` in the record settles it.

**4. Q3: one boot can measure both halves.** Run the disabled case first and the control last:
1. Fresh port, `flush()` → `Ok` (after the fix in finding 2).
2. `TXDIS`, then record `txens=0`.
3. Write until refused → `Timeout`.
4. `flush()` → `Timeout`.
5. `TXEN`, then `flush()` → `Ok`.

Writing to `TXDATA` clears `TXC`. `CMD.CLEARTX` empties the buffer; whether it also clears `TXC` is unmeasured. There's no emlib `.c` in the tree that uses `TXC` or `TXDIS`, only the headers.

**5. Q2: counting is the right unit, but the number is too rigid.**
- The "50–100 ms" figure assumes one cycle per iteration. Each iteration reads a peripheral register, so the real bound is at least that and probably a few hundred ms.
- The bigger risk is slow links: the EUSART FIFO holds 16 frames, and at 1200 baud draining it takes about 140 ms. A bounded EUSART `flush` would then report `Timeout` on a healthy link.
- Fix: work out the bound per instance from `clock_hz` and the baud rate. The EUSART driver already stores both.

**6. Q5: waits still unbounded.**

| Location | Wait |
|---|---|
| `eusart/mod.rs:305` | EUSART `flush`, returns `()` (only USART0's `flush` was bounded) |
| `spi/mod.rs:216`, `:226` | Spi0 transmit and receive |
| `spi/mod.rs:362`, `:371` | Spi1 transmit and receive |
| `spi/mod.rs:494`, `:500` | Spi2 transmit and receive |
| `spi/traits.rs:34` | Spi0 `flush` |
| `eusart/mod.rs:209`, `:213` | your test hook's `SYNCBUSY` waits |

These loops are bounded but throw the timeout away and carry on as if it succeeded:
- `eusart/mod.rs:126`, `:156`, `:185`
- `pcnt.rs:114`, `:193`
- `pwm.rs:106`, `timer/mod.rs:120`
- `sysrtc.rs:373`, `:465`, `:486`
- `board.rs:1162`, `:1263`, `:1487`, `:1535`
- `i2c/mod.rs:260`

Two corrections to your own record:
- `flush_receiver` (`usart/mod.rs:304`) isn't a wait; it's a single command write that returns `()`.
- The header of `hal_tx_timeout.rs` still describes `eusart0_unclocked_for_test`, which no longer exists.

**7. The runs that left no record were most likely crashes (faults), not hangs.** Every wait in `new_at` gives up after 100 000 iterations, so the constructor can't sit there forever. A single write to a register can't "never return" either; it completes or faults. The example halts on panic and has no fault handler that writes the mailbox, so a crash looks like `magic 0`. A `HardFault` handler in the shared mailbox module that records `CFSR`, `HFSR`, `BFAR` and the PC would turn these into answers.

**8. Q4: drop the `test-hooks` feature.** Cargo features are merged across the whole dependency tree, so "never enable this in a product" can't be enforced. The hooks' doc comments are also wrong: one says "`CTRL.TXEN`", and both say "never enabled" when `new_at` enabled it. The example should do the fault injection itself, with an `unsafe` PAC write of `txdis()` and a safety comment. Reject the "unbounded waits" option outright. Re-test the `board.rs` `steal()` write once the fault handler exists.

**If it had to ship as 0.1.0, I'd do these first:**
1. Fix the `flush` bug from finding 2 everywhere it appears, and bound the EUSART and SPI waits.
2. Re-run the instrument as the single-boot sequence with `TXDIS`, `TXENS` in the record and a fault handler installed. Until then, take the three "facts" built on the broken hook out of the F20 document and the hook comments.
3. Work out the bound per instance, and replace the feature with the PAC write in the example.
