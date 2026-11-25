# Plan: closing the coverage gaps

Written 2026-11-25, from [`docs/04-reference/COVERAGE-BOUNDARY-2026-11-25.md`](../04-reference/COVERAGE-BOUNDARY-2026-11-25.md):
the inventory says what is missing, this file says in what order and what "done" would look like for each item.

## The rule this plan follows

**An item is done when an instrument on the part says so, and a control makes that instrument mean something.** Writing
a driver is not done; "the register writes compile" is the state that shipped a false claim last time. So every row
below names the proof, not the code — and where the proof needs hardware the row says which hardware, because that is
the only reason to defer it.

Two classes are cheap and can be done with no board at all, because the logic is arithmetic that runs on the host:
the decision layer of a mode (which clock edge, which ACK, which overrun) and the arithmetic of a divider or a CRC.
Those are worth writing while the bench is unavailable — they are also the part where defects actually live, as the
USART divider and the receiver's error handling both showed this week.

## Phase 0 — no hardware needed

| # | Item | Proof | Cost |
|---|---|---|---|
| 0.1 | `hal_usart_link` prints `rxerr=` and `last=` next to the byte count | the floating-RX run shows `n=0 rxerr=N last=ferr` instead of `n=271`; the wired run shows `n=8 rxerr=0` | 1 h — **code now, measurement when the board is back**, and the dated record in the README must be re-taken then |
| 0.2 | ~~SPI **slave** decision layer~~ **Done, 2026-11-25.** `cores::spi` gains `Mode` (the CPOL/CPHA table as a function, so the driver cannot compute it two ways) and `SlaveRx` (a one-deep slot with the overrun rule) | 4 host tests, one of which flips the slot back to overwriting and goes red: an overrun is reported **and the unread word survives**, because the byte the master already moved past is the one it expects to have been understood. **Not** written: the driver path — it cannot be proven without a master, and an unproven slave driver is the claim this repository keeps having to retract | 4 h |
| 0.3 | ~~I2C **slave** decision layer~~ **Done, 2026-11-25.** `cores::i2c::slave` — `Slave` with `address_byte`, `receive`, `transmit`, `queue`, and the three `Response`s | 6 host tests: the direction bit is not part of the address (0x6A read and 0x6A written are the same device), another address is refused both ways, a full receive buffer NACKs **and keeps what it had**, an empty transmit buffer stretches instead of returning a filler byte, and a STOP does not leak into the next transaction. Control measured: comparing the whole address byte turns the direction test red. **Not** written: the driver path (10-bit addressing, general call, and the stretch's timing are hardware) | 6 h |
| 0.4 | ~~`gpcrc` driver~~ **Done, 2026-11-25.** `PASS gpcrc: poly=crc32 raw=340bc6d9 zlib=cbf43926 want=cbf43926 diff=1 seed=1` | the accelerator's result, complemented, is the published `zlib` check value; `diff=1` and `seed=1` are the controls that a constant or a driver ignoring `CMD.INIT` would fail. 10 host tests in `libraries/gpcrc-core`, both conventions pinned to their published vectors | ~3 h |
| 0.5 | Doc-tests for the remaining core crates | `cargo test --doc` reports a non-zero count per crate in the gate | 4 h, mechanical |
| 0.6 | ~~`timer` input **capture** decision layer~~ **Done, 2026-11-25.** `cores::timer` gains `elapsed`/`elapsed_with_wraps` (16-bit wrapping, widened after) and `CaptureSlot` | 6 host tests: an interval across the wrap is 32 ticks and not a negative number; two edges before a read are reported as a miss **and the newest is kept** (the opposite of a UART's receive slot, and worth stating); a value read twice does not invent an edge. Control measured: a slot that keeps the first edge turns the miss test red. **Not** written: the driver path, for the same reason as the SPI slave — no instrument yet | 3 h |

## Phase 1 — needs the board that is on the desk

| # | Item | Proof | Cost |
|---|---|---|---|
| 1.1 | SPI **slave** driver, proven against a real master | the CH340/FT232H as master sends three bytes; the record names them and the count. **The adapter that does not enumerate on this host is the blocker**, not the driver | 1 day |
| 1.2 | `LETIMER0` driver: two compare channels, run in EM2 | a record taken after `EM2` shows the compare fired **and** a SYSRTC delta that only EM2 can produce | 2 days |
| 1.3 | `PRS` driver: route Timer0 overflow → ADC trigger, ADC done → DMA | a record whose count the CPU never touched: the DMA count is non-zero while the loop only sleeps | 3 days; this is the item that makes the HAL usable for a battery product |
| 1.4 | `hal_spi_loopback` with a device on the pads instead of the jumper | an external witness for the SPI path — the same gap as the USART's | 2 h once a device exists |
| 1.5 | I2C **slave** driver + a master on the other side | the same adapter (I2C mode) reads what the part writes | 1 day |

## Phase 2 — needs hardware that is not on the desk

| # | Item | Needs | Why it waits |
|---|---|---|---|
| 2.1 | `VDAC0`/`VDAC1` | a meter | a DAC with no meter is a register write with a story attached |
| 2.2 | `ACMP0`/`ACMP1` | a reference voltage | same |
| 2.3 | `PCNT0` | a signal source | a pulse counter with nothing to count proves nothing |
| 2.4 | `KEYSCAN` | a keypad | — |
| 2.5 | `BURTC` | a way to cut power | "it kept time with the main supply off" cannot be shown any other way |
| 2.6 | `IADC0` differential and scan | a differential source, or a divider pair to build one | the refusal is honest today; measuring it needs two known inputs |

## Phase 3 — the radio, which is not this crate's HAL work

The radio gaps are in [`crates/efr32mg24-radio`](../../crates/efr32mg24-radio/README.md), and they are not driver work —
they are vendor components that have not been vendored:

| # | Item | What it unblocks | Cost |
|---|---|---|---|
| 3.1 | **PSA crypto** blobs | the `gatt` example: a connection with notifications; and the cryptographic half of bonding | unknown until the blob's link requirements are read — the `gatt` link already prints the missing `psa_*` symbols |
| 3.2 | **nvm3** | bonding that survives a power cycle, which is what a real peripheral needs | 2–3 days of binding work |
| 3.3 | **sleeptimer** | the stack sleeping between events — without it the BLE examples never leave EM0 | 2 days |
| 3.4 | **the AES radio helper** | link-layer encryption | 1 day |
| 3.5 | **RAIL + a radio configuration** | proprietary frames and 802.15.4; today `--features rail` is a `compile_error!` | 1 week, and it needs emlib vendored first |

## What stays out, and why that is a decision and not an omission

- **`DCDC`** — a power-mode choice, not a driver: the reset default is the LDO and it works. Enabling the DC-DC is a
  board decision that costs a measurement nobody has taken.
- **`SMU`** — configuration of a safety unit; nothing in this crate's scope depends on it.
- **`WDOG1`** — a second watchdog instance with nothing needing two.
- **`SYSCFG`, `DMEM`, `BURAM`, `SCRATCHPAD`, `HOSTMAILBOX`, `ICACHE0`** — infrastructure that the generated PAC already
  reaches where it is needed.
- **Thread / Zigbee / Matter** — stacks above RAIL, not radio drivers.

## The release decision this feeds

For 0.1.0 the proposal in the boundary document stands: claim exactly the rows with a dated record, name "compiled, not
measured" as such, and name every absence above by family and by mode. Phase 0 and Phase 1 do not have to be finished
first — a first release is allowed to be small, as long as it is specific about how small.

The operator's constraint, recorded here so the plan cannot drift from it: **publication waits for the hardware to
work.** The deferred bench items are listed in `docs/05-operations/RESUME-REGISTER.md`.