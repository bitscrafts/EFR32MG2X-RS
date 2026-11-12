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

| Example | What it proves | How to check it |
|---|---|---|
| `ble_beacon` | the stack boots and a phone can see a name | **nRF Connect for Mobile**: the name appears in the list |
| `ble_advertise_custom` | a framed manufacturer payload inside the 31-byte budget | nRF Connect shows a hex blob under the device; the frame starts `ff e5 02 01 …` |
| `ble_scanner` | the receive path: address, RSSI and payload length of whoever is nearby | the mailbox reports the strongest advertiser; blink per packet |
| `ble_gatt` | a connection, a subscription, notifications in order | **staged**: needs the PSA crypto blobs (see RADIO-NOTES) |
| `rail_version` (rail-sys) | RAIL header and blob agree | mailbox: `rail ver=…` |

Two of those are a pair: flash one board with a beacon and another with the scanner, and the radio is
proven in both directions without a phone. nRF Connect is the third opinion — it is the only thing that
proves the antenna, because it is a receiver built by someone else.

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
