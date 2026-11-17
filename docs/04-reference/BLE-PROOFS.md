# BLE: what is proven, with the numbers that prove it

Measured 2026-11-17 on the XIAO MG24 Sense (EFR32MG24B220F1536IM48) from the lnx host over the dongle, with the
firmware's own counters read over SWD at the end of the same run. Nothing here is inferred from "it seemed to work".

## The GATT data path (E1-BLE.2): 60 s at 1 Hz, received by a real subscriber

```
service        e1d0e000-...    three characteristics, found with `gatttool --characteristics`
  e1d0e001     value handle 0x0003   props 0x12 (read + notify)   rms, u16, counts
  e1d0e002     value handle 0x0006                                bands, 8 x u32
  e1d0e003     value handle 0x0009                                status, u8 flags + u16 window count
CCCD at 0x0004 written with 0100 to enable notifications

host received        56 notifications on 0x0003 in 55 s, values 0x0041 / 0x003f / 0x0042 (RMS counts), no errors
firmware counted     seconds=60  published=60  notifies=56  windows=3780
```

The two counts agree to the unit, and they are produced by different code: one by the host's GATT stack, the other by
the firmware's own send path. `published=60` is one per second over sixty seconds; `notifies=56` is what the stack
accepted to send. That agreement is the point of the measurement -- a subscriber with no notifications and a
firmware with no subscriber would both look "green" on their own.

Reproduce with `scripts/gatt-subscribe.sh <address> 55`.

## Advertising survives a connection

A peripheral that does not restart advertising after a disconnect becomes invisible after the first attempt, and the
host then answers "Device not available" for a device that is right there on the air (measured earlier the same day,
twice). After the run above -- one connection, one disconnect -- the device is found by the next scan.

## Pairing

Pairing completes and the peer reports it: `Pairing successful`, `Paired: yes`, `Bonded: yes`, with the device
recording `sm_bonded` (sev=3) and `ec=1,1,1 gen=2`. Reproduce with `scripts/pair-probe.sh` (pair) and
`scripts/unpair-probe.sh` (forget first: `pair` against a known device answers AlreadyExists and never touches the
link).

## What is NOT proven, and why it matters for the product

The bond does not survive a reset with the builtin database: the record is written and loaded, the address matches,
and the long-term key is not in the store, so the first encrypted reconnect is answered `0x1006 PIN or Key Missing`.
The external database that would fix it is wired and stores to NVM3 (`src/bonding_ext.rs`, feature
`external-bonding`), but with it the pairing itself fails authentication, so the feature stays off and the builtin
database stays as it was.

Consequence, unchanged until that closes: the dosimeter example does not call `enable_pairing`. A device that pairs
and then refuses to reconnect is worse for the app than one that does not pair at all.

## Cost of the same run, for the record

`cycles=2350470467` for sixty seconds of audio at 19 MHz is 206% of real time -- the open item written up in
`APP-LOOP-WITH-BLE-OPEN.md`, and the reason the capture cannot yet be called real-time with the stack running.
