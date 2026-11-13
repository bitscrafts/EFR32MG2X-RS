# What is still C, and what it would take to be Rust

Measured 2026-11-13, after `sli_psa_context_get_size` and `sli_aes_crypt_ctr_radio` became Rust
(`crates/efr32mg24-bluetooth-sys/src/support.rs`, `libraries/aes-ctr-radio`).

## Decision, and the order it implies (user, 2026-11-13)

> *"documente a analise, apos o ble estiver totalmente funcional e com exemplos correndo iremos fazer essas
> migracoes c->rust!"*

The migrations described here are **deferred, deliberately**. They are not a prerequisite for Bluetooth and
must not become one: rewriting `em_cmu.c` while the link layer is still one symbol from linking would move
the failure surface from "one missing timer" to "a clock tree that nobody has measured". The order is:

1. **BLE fully functional** — the vendor stack links, boots (`system_boot` with a Bluetooth address), the
   board advertises and a GATT client connects and reads a characteristic.
2. **Examples running** — one example per level that exercises that path, on the board, with the observed
   output recorded, not merely compiled.
3. **Then, and only then**, the migrations below, one file at a time, each with the gate green before the
   next one starts and with the previous behaviour measured again after the swap.

What is already Rust (the two support symbols) was not a migration of convenience: without them the stack
does not link at all, so they are on the critical path to step 1 rather than in the deferred list.

The rule this file applies: **C that is ours becomes Rust; vendor binary blobs stay, because they are
licensed binaries and not source.** What follows is every C file the build compiles, ranked by what it would
cost to replace.

## Already Rust

| What | Where | How it is verified |
|---|---|---|
| PSA context sizes | `bluetooth-sys/src/support.rs` | sizes measured with the vendor's own mbedtls headers; rounded **up**, because the value is an allocation size |
| AES-128-CTR for the controller | `libraries/aes-ctr-radio` | FIPS-197 vector, zero-key vector, reversibility, counter carry — 4 host tests |
| PA curves: dBm ↔ raw | *not yet* — currently the vendor plugin (`vendor/pa2`, 1181 lines of C) | the radio's power read-back (99 for 10 dBm) |

## The C we wrote, and its Rust plan

| File | Lines | What it does | Rust plan | Effort |
|---|---|---|---|---|
| `crates/cmsis-stubs/src/cmsis_stubs.c` | 166 | the `CORE_*` helpers the vendor blobs call: interrupt masking, critical sections | straight `#[no_mangle]` port; the PAC already exposes the NVIC/PRIMASK registers | small |
| `crates/efr32mg24-emlib-sys/src/shim.c` | 93 | start the HFXO, select it as SYSCLK, read the unique id, read the core clock | becomes HAL functions over PAC registers (`HFXO0.CTRL.FORCEEN`, `CMU.CLOCKCTRL`, `DEVINFO`) | small |

Both are small because they were written small on purpose: they exist to stop the vendor C from spreading,
not to be a layer.

## The vendor C that is compiled today, and what it really costs

| File | Lines | Why it is compiled | Rust plan | Effort |
|---|---|---|---|---|
| `em_cmu.c` | 11 864 | `RAIL_Init` pulls `rfhal_radio.o`, which calls `CMU_ClockEnable` and friends | replace the handful of functions actually reached (`CMU_ClockEnable`, `CMU_ClockSelectSet`, `CMU_OscillatorEnable`, `CMU_ClockFreqGet`) with HAL functions over PAC registers, then drop the file | medium — the risk is knowing which functions are reached; `--gc-sections` plus the linker's undefined list says exactly that |
| `em_emu.c` | 4 312 | `RAIL_Init` calls `EMU_DCDCSetPFMXTimeoutMaxCtrl` | one register write in the HAL | small |
| `em_system.c` | 433 | `SYSTEM_GetUnique`, `SYSTEM_ChipRevisionGet` | HAL functions over `DEVINFO` | small |
| `system_efr32mg24.c` | 691 | `SystemSYSCLKGet`, used by RAIL's clock setup | HAL function over `CMU` | small |
| `pa_conversions_efr32.c` + `pa_curves_efr32.c` | 1 181 | the dBm ↔ raw mapping and the PA curves | Rust: the curves are **data** (piecewise segments per PA mode), the conversion is a table lookup; both are testable on the host against the read-back | medium |
| `sli_protocol_crypto_radioaes.c` | 900 | AES-CTR on the RADIOAES peripheral | **done in Rust** (`libraries/aes-ctr-radio`, software) | — |

The three small ones (`em_emu`, `em_system`, `system_efr32mg24`) together are a few register writes. The
large one is `em_cmu.c`, and its size is misleading: the linker already tells us which of its 11 864 lines
are reached, because everything else is dropped by `--gc-sections`.

## What must stay binary

The RAIL library and the three Bluetooth blobs. They are MSLA binaries with no source published, and the
radio's sequencing (MODEM/FRC/AGC/RAC) is not something to reimplement from a datasheet in a project that
wants a product. The honest boundary is: **a vendor blob may implement the radio; it may not own anything
that touches our data structures** — which is why the PSA sizes and the AES are now ours, and why the PA
curves should be next.

## The one symbol between here and the vendor stack linking

```
$ cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg24-radio \
      --example ble_beacon --features stack
rust-lld: error: undefined symbol: sl_sleeptimer_restart_timer
          >>> referenced by bglib.c, restart_stack_event_timer
```

Everything else the stack needs is satisfied: twenty undefined symbols, then five, then two, now one.

`sli_sleeptimer_restart_timer` is the stack's timer: `(handle, timeout, callback, data, priority, flags)`.
In pure Rust that means a real HAL piece — drive the SYSRTC peripheral (start the counter, program the
compare channel, route its interrupt, keep a small timer list) instead of vendoring `sl_sleeptimer.c`. That
is the natural next step and it is a HAL-shaped one, not a wrapper: a timer the rest of the project can use
too.

Note the direction of causality, because it is easy to get backwards: implementing this timer **in Rust is
not one of the deferred migrations** — it is the last thing standing between the link and the stack. It
belongs to step 1, and it belongs in the HAL rather than in a `-sys` crate, because a timer is hardware
policy, not vendor glue.

## Definition of done for "BLE fully functional"

Written down so that it can be checked rather than argued:

- [ ] `ble_beacon --features stack` links, with no undefined symbols.
- [ ] Flashed, the stack reports `system_boot` with a version and an address, read from the mailbox.
- [ ] The name it advertises is listed by `scripts/ble-scan-bluez.sh` on the host, not merely captured by
      `btmon`.
- [ ] `ble_gatt` links and a GATT client on the host connects and reads a characteristic back:
      `bluetoothctl connect <addr>` followed by `bluetoothctl gatt.read <handle>` returning the value the
      firmware set.
- [ ] Each of those paths has an example that **ran**, with its output recorded in `docs/03-development/EXAMPLES.md`.

Advertising through RAIL (commit `5092303` onward) satisfies none of these except the third, by a different
route: that is the link layer alone, with no host stack, and it cannot answer a GATT read.
