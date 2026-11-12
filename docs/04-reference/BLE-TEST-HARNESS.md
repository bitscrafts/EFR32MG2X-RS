# Testing the radio: what is actually available

Measured 2026-11-12, on the host that owns the boards (`10.10.10.217`).

## The host has a Bluetooth adapter, and it can play both roles

```
hci0   Type: Primary  Bus: USB   BD Address: 90:DE:80:81:44:56
```

It comes up with the Wi-Fi dongle's companion radio and was `DOWN` until this was measured: the
`bluetooth` service was installed but **disabled**, so nothing had ever used it.

```bash
sudo systemctl start bluetooth      # was: inactive (dead), preset enabled, disabled
sudo hciconfig hci0 up              # was: DOWN
```

With those two, the host becomes a BLE peer that needs no phone and no extra hardware:

**As a transmitter**, which is what the board's examples need to hear:

```bash
printf 'power off\nsystem-alias lnx-ble\npower on\ndiscoverable on\nadvertise on\n' \
  | sudo bluetoothctl
```

Verified at the HCI level rather than by trusting the tool:

```
sudo btmon -r capture.snoop | grep "Advertising Enable"
  LE Set Extended Advertising Enable (0x08|0x0039) ncmd 2
```

The alias only applies after a power cycle of the adapter — setting it while powered leaves the old
name in the advertising data, which is the kind of detail that makes a test look broken when it is not.

**As a receiver**, to check what a board transmits:

```bash
printf 'power on\nscan on\n' | sudo timeout 15 bluetoothctl
```

Note that `bluetoothctl --timeout N scan on` prints nothing useful here; the interactive form with the
commands on stdin does. That cost one confusing round.

## What was on the air

A 14-second scan of the host adapter found **no advertisers at all**. That explains why
nRF Connect showed nothing: the nice!nano is not advertising. Its firmware identifies itself as
`239a:00b3 nice!nano`, and it resets on a 1200-bps touch, but it is not a ZMK build with BLE on — so
there is no keyboard to find.

So the emitter side of the test is solved by the host, and the remaining work is on our side: the XIAO
has to receive. That is the RAIL BLE path, not the Bluetooth stack — see `BLE-BRINGUP-CHAIN.md`.

## Why this is better than a phone

A phone shows a name in a list, which answers "did something arrive". The host adapter answers "what
arrived, on which channel, at which address, with which payload", and it can do it unattended, which
means a test that runs while nobody is looking at the board.
