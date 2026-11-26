# EFR32MG24 HAL Development Backlog

**Last Updated**: 2026-11-26
**Current sprint**: Phase D2f -- Bluetooth pairing and bonding (the last D2 item)
**Next sprint**: Phase D2g -- BME680 gas resistance, then Phase D3 (Zigbee 3.0)

**State**: the radio works and a phone can use it -- advertising, scanning, connections, GATT read/write/notify, and the
BME680 published as the standard Environmental Sensing service (`base_ble_env_gatt`). Every example of every crate
compiles under `scripts/check-examples-build.sh`, 13 of them run on the board via `scripts/ble-matrix-all.sh`, and
`scripts/gate.sh` reports `VERDICT: PASS`. The state of the tree, counted, is in
[`STATUS.md`](STATUS.md); the Gantt and the phase table are in `README.md`.

This file is open work only. What closed is in [`LOG.md`](LOG.md) and in the crate READMEs, where each closed item
carries the record that closed it.

---

## Open (in order)

### High Priority

- **D2f: pairing fails before the key exchange.** With the security manager enabled the request is accepted and the
  bond fails at a DHKey check with `pg=0 pe=0 pa=0` -- none of the PSA entry points were called -- so the fault is in
  the state machine before key generation, not in the cryptography. The PSA layer (P-256 generate/export/ECDH,
  AES-CMAC/CTR/ECB) is written and verified on the board. Until this closes, the component stays out of the product
  build and the limitation is documented in `docs/04-reference/BLE-STATUS.md`.
- **CI cannot build the vendor-dependent crates.** `**/vendor/**/*.{h,c}` is excluded from git under the Silabs MSLA,
  so a clean clone has no emlib, Bluetooth stack or RAIL sources and the radio and BSP examples cannot compile on the
  runner. The PAC, HAL and the domain libraries do build. Options, in order of preference: fetch the public
  `SiliconLabs/gecko_sdk` in the workflow, commit the vendor trees, or scope the workflow to what builds without them.
- **D2g: BME680 gas resistance.** The driver reads temperature, pressure, humidity and the variant byte; the heater,
  `res_heat` and the `par_g*` curves are missing, so the reading that distinguishes a 680 from a 688 is not implemented.

### Medium Priority

- **The advertising rate is unmeasured.** The only adapter available captures about 8% of what is on the air, so counts
  cannot become a rate without a controllable receiver.
- **Occasional notification refusals** when a client subscribes and the board pushes at 20 Hz against a 30 ms connection
  interval.
- **MTU negotiation and connection parameters** are not addressed.

### Low Priority

- **`psa_get_key_attributes`** writes 36 bytes asserted at compile time, with no C translation unit in the tree to
  measure `sizeof` -- the weakest claim in the PSA layer.
- **Pure Rust, no externs**: emlib, the PA tables, channel configuration, NVM3 and the SE manager still bind to vendor
  code; the MSLA blobs remain at the ABI boundary. Ordered after Bluetooth is closed.
- **The delay clock runs 2.3x short** (`docs/04-reference/DELAY-CLOCK-DEFECT.md`).

---

## Open in the HAL

These are the items that need the part, in the order the night queue takes them. Each one names what would count as
measured, because "it compiles" has never been the bar here.

- **PRS with a GPIO producer at a real rate.** The pin is measured: a pad moved by its own pull resistor paces
  conversions, one per rising pad edge, and `EXTIRISE`/`EXTIFALL` turned out not to be in the PRS path at all --
  [`PRS-GPIO-PRODUCER-2026-11-26.md`](../04-reference/PRS-GPIO-PRODUCER-2026-11-26.md). What is still open is a pad
  driven by a wire, a button or another peripheral, which is the case a product has and the pull resistor is not, and
  the fastest rate the converter can follow such a producer.
- **The root README's project metrics.** It still says "12 peripheral modules", "Examples: 83 files, 26 of them
  checked" and "Crates: 12 total". The tree has 29 public HAL modules and 29 HAL examples, and the example count the
  build script checks is a number that script can print.
- **DMA stages 3 and 4**: linked descriptors and interrupt-driven async. Deferred, not broken.
- **The families nobody has written a driver for**: `pcnt0`, `burtc`, `acmp0`/`acmp1`, `vdac0`/`vdac1`, `keyscan`,
  `radioaes`, `dcdc`, `dpll0`, `ulfrco`, `fsrco`, `hfrcoem23`. What each would cost is in
  [`COVERAGE-BOUNDARY-2026-11-25.md`](../04-reference/COVERAGE-BOUNDARY-2026-11-25.md).

---

## Closed in the HAL, 2026-11-25 and 2026-11-26

Each of these has a record read back off the part; the record string is in the document named.

- `LETIMER0`: `CTRL.CNTTOPEN` is the switch that makes `TOP` mean anything, and the counter is 24 bits, not 16 --
  [`LETIMER-TOP-2026-11-26.md`](../04-reference/LETIMER-TOP-2026-11-26.md).
- `LETIMER0` delivery: 24-bit `counter()`, `TOP` with `CNTTOPEN`, compares refused at or above the top, slot 12 of the
  board claimed -- [`LETIMER-COMPARE-2026-11-26.md`](../04-reference/LETIMER-COMPARE-2026-11-26.md).
- IADC started by a PRS positive edge, with arming, a software trigger and the three controls that make the count mean
  something -- [`IADC-PRS-TRIGGER-2026-11-26.md`](../04-reference/IADC-PRS-TRIGGER-2026-11-26.md).
- PRS as a real producer: TIMER0's overflow through channel 0 converted once per overflow, with no CPU write in the
  loop and four controls at zero -- [`PRS-TIMER-PRODUCER-2026-11-26.md`](../04-reference/PRS-TIMER-PRODUCER-2026-11-26.md).
- A GPIO pad as the PRS producer: `pin::EdgeWatch` plus `Input::set_pull_up` move the pad with its own pull resistor,
  the converter fires once per rising pad edge, and `EXTIRISE`/`EXTIFALL` were measured to sit outside the PRS path --
  [`PRS-GPIO-PRODUCER-2026-11-26.md`](../04-reference/PRS-GPIO-PRODUCER-2026-11-26.md).
- Before those, and already closed: the I2C read defect and its recovery, the USART receiver's refusal, SPI loopback
  with a control, the WDOG feed budget, the CRC-32 check values, the MVP benchmark, the SYSRTC 64-bit path and the
  Embassy time driver. All in `docs/04-reference/` and in the HAL README's measured table.

---

## History

Until 2026-11-26 this file also carried a January-2026 kanban: Ready / In Progress / Review / Blocked columns, task
tags, a metric block with "12 working examples" and `08_adc.rs`, and a last review dated 2025-12-28. Its state lines
had been false for months -- it still had Embassy blocked "waiting for Phase B", and hardware "not available" for a
board that has been on the bench since January. The old text is in git history:
`git show 043c7cc:docs/03-development/BACKLOG.md` (`STATUS.md` and `PLAN.md` beside it), with a timestamped copy in
`.archive/` on the machine that made the change, which is gitignored and therefore does not survive a clone. A backlog
that reports state competes with `STATUS.md` and loses; this one lists work.