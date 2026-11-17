# BLE on the bench: what works, and how to bring it back

Written 2026-11-17, late, after a night in which the link worked, broke, and was blamed on the wrong layer three
times. Every number here was measured; nothing is inferred from a design.

## The state, in one table

| what | value | how it was measured |
|---|---|---|
| firmware runs, measures, records | `seconds windows published` growing | mailbox, `scripts/read-mailbox.sh` |
| advertisement carries the service | `UUID: Vendor specific (e1d0e000-…)` | `bluetoothctl info`, i.e. BlueZ's own parse |
| bond, inside the window | `Pairing successful`, `Bonded: yes` | `scripts/ble-end-to-end.sh` |
| data behind the bond | `rms 11 00 = 17`, 8 non-zero bands, `status 01 99 09 01 1e` | same, `gatttool` at `sec-level=medium` |
| window policy | bond new only in the first 600 s after power-on | `PAIRING_WINDOW_SECONDS` |
| refusal, outside the window | `cfno` grows, `smrsn=4102` (`0x1006`) on the phone | device record vs host error |

`VERDICT: PASS` from `scripts/ble-end-to-end.sh` on 2026-11-17 at 20:57.

## The failure that cost the night

After a sequence of debugger resets the board stopped booting the BLE firmware: the core sat in
`RAILCb_AssertFailed` (`r1=0x4b`), no record was written, and nothing was advertised. It was **not** the firmware:
that exact image had passed end to end forty minutes earlier, and `module_led_blink` still ran on the same board
(`pc=0x08006304`), so the core, the flash and the clock were fine.

What actually causes it, as far as the measurements can tell:

- A **system reset does not reset the radio peripheral.** `probe-rs reset` and the flash script's `reset + run` leave the
  RAIL state machine where it was, and a stack that initialises into that state asserts.
- **Halting the core while the radio runs** makes it worse. `scripts/wait-for-record.sh` halted every 12 s looking for
  a record, which is exactly the wrong instrument: it kept the state broken while it waited for the state to heal.
- **A power cycle looked like it would clear it, and it does not.** Measured three times after this was first
  written: unplug, five seconds, replug — the core is in `RAILCb_AssertFailed` again within a minute, with the
  120 s firmware and with the 600 s firmware alike. The earlier report that it worked after a replug was true of
  one occasion, and it did not survive being repeated, so it is withdrawn here rather than left standing.

What survives every measurement:

- the core runs non-BLE firmware on the same board at the same time (`module_led_blink` at `0x08006304`, and
  `module_dosimeter_offline` at `0x0800640e` in `core::ptr::read_volatile`), so the silicon, the clock, the flash
  and the debugger are all fine;
- the failing code is the Bluetooth stack's start-up (`RAILCb_AssertFailed`, `r1=0x4b`), and it fails before the
  first 60 s record is published, which is why the mailbox looks empty;
- the host holds `Paired: yes`, `Bonded: yes`, `Connected: no`, so nothing on the host is connecting to the board.

This is where the bench stands until the radio stack's start-up is examined with time to read the vendor's assert
codes, instead of the guesses that cost this night.

## Thursday, in order

```
1  flash the product                  FLASH_HOST=10.10.10.217 FLASH_USER=mvcorrea \
                                      bash scripts/flash.sh <elf> --confirm --run
2  POWER CYCLE the board              unplug USB, 5 s, plug back
3  prove the link from the host       sudo -n /tmp/blevenv/bin/python /tmp/ble_minimo.py
                                      (scan unfiltered, print, connect, read -- no asserts)
4  prove the whole path with checks   bash scripts/ble-end-to-end.sh   -> VERDICT: PASS
5  the phone                          open the app, accept the bond within 10 minutes
```

If step 3 prints no dosimeter, the board is not running: check `reg pc` for `RAILCb_AssertFailed` and go back to the
power cycle. If it prints the dosimeter but the phone cannot see it, the phone side is the problem and the app prints
the permission state and the number of devices it sees.

## What is not claimed

- **dB(A) and dose % are not on the wire.** The NR-15 arithmetic exists and is tested; it is not wired to the stream,
  because A-weighting needs the per-band sum, not the broadband level.
- The calibration is **from the datasheet** (`−38 dBV/Pa`, DC 0.7 V, cross-checked against the measured bias at 0.9 %),
  **not** from a calibrator. Uncertainty ±3.0 dB, and the app says so on every screen.
- Nothing here is a certification.

## The next step, and it is a specific one

The vendor library calls `RAILCb_AssertFailed(errorCode)` — an application-provided callback. We do not define it, so
the vendor's weak default spins. Defining it is the whole next move: capture `errorCode` (`r1` was `0x4b`), and if the
vendor exposes `RAIL_GetAssertInfo()` / the assert file and line, capture those too, and publish them to the mailbox
before halting. That turns "the radio asserts" into "the radio asserts at <file>:<line> with code 75", which is a
defect with an address instead of a night of guesses.

Until that number is read, nothing about the radio should be claimed either way.
