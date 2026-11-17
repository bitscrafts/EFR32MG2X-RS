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
- A **power cycle** clears it. Measured: after the operator unplugged and replugged the board, the link worked again.

So: **after flashing a firmware that starts the BLE stack, cut the power.** Not reset — unplug. Then leave the
debugger alone while it runs.

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
