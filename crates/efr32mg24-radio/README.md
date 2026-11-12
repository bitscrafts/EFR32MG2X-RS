# efr32mg24-radio

Safe radio layer for the EFR32MG24: Bluetooth Low Energy through the Silicon Labs BGAPI stack, and
— staged — RAIL for proprietary and 802.15.4 traffic.

## Why this crate exists

The two vendor stacks arrive as **binary blobs** (MSLA licensed) plus headers, wrapped by
`efr32mg24-bluetooth-sys` and `efr32mg24-rail-sys`. Those crates are honest about what they are:
generated bindings, `unsafe`, one-to-one with the C API.

An application should not have to use them directly. It wants a name, an advertising packet, a
connection handle and a notification — not `sl_status_t` and an event queue. This crate is that
level:

```rust
use efr32mg24_radio::Bluetooth;

let mut ble = Bluetooth::beacon(&mut delay, "xiao-mg24")?;
loop {
    if let Some(event) = ble.poll_event() {
        // Connected / Disconnected / GattWrite / ...
    }
}
```

## What it owns

| Concern | Where it lives |
|---|---|
| Bindings, blobs, link flags | `efr32mg24-bluetooth-sys` |
| Boot order, event decoding, advertising data builder | re-exported from the bindings crate |
| Clock and delay the stack asks for at boot | `efr32mg24-hal`, passed in by the caller |
| Which pin the antenna switch is on, whether the radio is powered | a BSP — **not here** |

## Examples

| Example | What it proves | Hardware |
|---|---|---|
| `ble_beacon` | the stack boots and a phone can see a name | XIAO MG24 (any variant) |
| `ble_gatt` | a connection, a subscription, and notifications that arrive in order | XIAO MG24 (any variant) |

```bash
cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg24-radio --examples
```

Both examples report through the SWD mailbox rather than a serial port, so the result can be read
with the board in a drawer, and a phone is only needed to confirm the antenna.

## The blobs are not in the repository

`blobs/` in the `-sys` crate holds three `.a` files that are **not** committed: MSLA licensing, and
~1.5 MB. They are gitignored, which means a clean checkout does not link. See
`crates/efr32mg24-bluetooth-sys/blobs/README.md` for where they come from, and note that the header
shipped beside them must be the **same SDK version** — a mismatch produces functions that exist in
the blob and not in the bindings, which reads like a typo and is not.

## RAIL

`--features rail` is deliberately a `compile_error!` for now: the headers exist in the archived
project but the blob does not, so the feature would promise a link that cannot happen. The plan is in
`docs/04-reference/RADIO-NOTES.md`.
