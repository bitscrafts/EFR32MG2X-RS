#!/usr/bin/env python3
"""Connect to the board over BlueZ, exercise read/write/notify, and judge it from the client side.

Run this on the host that owns the dongle, not in the container:

    sudo python3 scripts/host-gatt-probe.py [address] [hold-seconds]

It is the other half of `docs/04-reference/BLE-STATUS.md`: the firmware's mailbox says what the board believes, and
this says what a real client sees. Neither is evidence alone.

Two traps it exists to avoid, both of which produced false verdicts here:
  * `gdbus call --method StartNotify` runs as its own short-lived process and BlueZ drops the subscription when that
    client goes away, writing the CCC descriptor back to 0. The connection has to stay open, so this uses python-dbus.
  * BlueZ only creates a device object after its own scan has seen it (a raw `hcitool lescan` does not count), and a
    scan client killed by a timeout leaves discovery stuck: the next attempt answers `org.bluez.Error.InProgress` for
    ever. `sudo hciconfig hci0 reset` then `bluetoothctl --timeout 13 scan on` clears it. Reset touches the BLE side of
    the dongle only -- its Wi-Fi side carries the network, and nothing here may power the controller down.
  * a read-back can be answered from BlueZ's cache. The verdict that matters is the echo: the firmware repeats the
    bytes the client wrote inside its notifications, and bytes that come back can only have crossed the link.

    python3 gatt-probe.py [address] [hold-seconds]

Why this is not built on `gdbus` any more: `gdbus call --method StartNotify` runs as its own short-lived process, and
BlueZ releases a notification subscription when the dbus client that asked for it goes away -- it writes the CCC
descriptor back to 0 about 150 ms later. That looked exactly like firmware that stops notifying after three packets.
The subscription has to be held by a connection that stays open, so this script keeps one python-dbus connection alive
for the whole window and counts notifications from `PropertiesChanged` on `Value`.

What it prints, in order: the device object, every characteristic with its flags, the notification acquisition, and
then the values received with a timestamp for each.
"""
import sys
import time

import dbus
from dbus.mainloop.glib import DBusGMainLoop

ADDR = sys.argv[1] if len(sys.argv) > 1 else "E4:0A:75:13:3E:E0"
HOLD = float(sys.argv[2]) if len(sys.argv) > 2 else 12.0
DEV = "/org/bluez/hci0/dev_" + ADDR.replace(":", "_")
BLUEZ = "org.bluez"

received = []
# Set by the write below; the echo verdict reads it, so it must not depend on probing the local scope.
want_hex = None


def unwrap(v):
    """python-dbus gives dbus typed values; print them as plain ones."""
    if isinstance(v, dbus.Array):
        return [unwrap(x) for x in v]
    if isinstance(v, (dbus.Byte, dbus.UInt16, dbus.Int16, dbus.UInt32, dbus.Int32)):
        return int(v)
    return v


def main() -> int:
    DBusGMainLoop(set_as_default=True)
    bus = dbus.SystemBus()
    om = dbus.Interface(bus.get_object(BLUEZ, "/"), "org.freedesktop.DBus.ObjectManager")

    def managed():
        return om.GetManagedObjects()

    def find_device():
        try:
            adapter = dbus.Interface(bus.get_object(BLUEZ, "/org/bluez/hci0"), "org.bluez.Adapter1")
            adapter.StartDiscovery()
        except dbus.exceptions.DBusException as e:
            print(f"   discovery: {e.get_dbus_name()}")
        for _ in range(12):
            time.sleep(1)
            for path, ifaces in managed().items():
                dev = ifaces.get("org.bluez.Device1")
                if dev and str(dev.get("Address", "")).upper() == ADDR.upper():
                    try:
                        adapter.StopDiscovery()
                    except dbus.exceptions.DBusException:
                        pass
                    return path
        return None

    tree = managed()
    if DEV in tree:
        dev = DEV
    else:
        print(f"== find {ADDR}")
        dev = find_device()
        if dev is None:
            print("   not discovered in 12 s")
            return 1
    print(f"== device {dev}")

    device = dbus.Interface(bus.get_object(BLUEZ, dev), "org.bluez.Device1")
    props = dbus.Interface(bus.get_object(BLUEZ, dev), "org.freedesktop.DBus.Properties")
    try:
        device.Connect()
    except dbus.exceptions.DBusException as e:
        if "AlreadyConnected" not in str(e):
            print(f"   connect: {e.get_dbus_name()} {e}")
            return 1
    print("   connected")

    # Wait for BlueZ to finish walking the attribute table.
    chars = {}
    for _ in range(20):
        time.sleep(1)
        tree = managed()
        if not props.Get("org.bluez.Device1", "ServicesResolved"):
            continue
        chars = {
            path: ifaces["org.bluez.GattCharacteristic1"]
            for path, ifaces in tree.items()
            if path.startswith(dev) and "org.bluez.GattCharacteristic1" in ifaces
        }
        if chars:
            break

    print(f"== {len(chars)} characteristic(s)")
    for path, c in chars.items():
        print(f"   {unwrap(c.get('UUID'))}  {path.split('/')[-1]}  flags={','.join(unwrap(c.get('Flags')))}")
    if not chars:
        return 1

    def on_props(interface, changed, _invalidated):
        if interface != "org.bluez.GattCharacteristic1" or "Value" not in changed:
            return
        now = time.monotonic()
        received.append((now, [hex(b) for b in unwrap(changed["Value"])]))
        print(f"   [{len(received):3d}] t={now - start:6.2f}s value={received[-1][1]}")

    bus.add_signal_receiver(
        on_props,
        dbus_interface="org.freedesktop.DBus.Properties",
        signal_name="PropertiesChanged",
    )

    target = next(
        (p for p, c in chars.items() if {"notify", "indicate"} & set(unwrap(c.get("Flags")))),
        None,
    )
    if target is None:
        print("   no characteristic supports notify")
        return 1

    char = dbus.Interface(bus.get_object(BLUEZ, target), "org.bluez.GattCharacteristic1")
    start = time.monotonic()
    char.StartNotify()
    print(f"== StartNotify {target.split('/')[-1]}, holding {HOLD:.0f}s")

    # **Read and write, not only subscribe.** Three different claims: the value can be read, a client can write it,
    # and both survive a round trip through the attribute (a write that lands in a callback reads back unchanged).
    for path, c in chars.items():
        if "write" in [str(f) for f in unwrap(c.get("Flags"))]:
            writable = dbus.Interface(bus.get_object(BLUEZ, path), "org.bluez.GattCharacteristic1")
            try:
                before = unwrap(writable.ReadValue(dbus.Dictionary({}, signature="sv")))
                print(f"== Read {path.split('/')[-1]} before -> {[hex(b) for b in before]}")
            except dbus.exceptions.DBusException as e:
                print(f"== Read before failed: {e.get_dbus_name()}")
            # A value that differs every run, so "read back the same bytes" cannot pass by having read what was
            # already there. This is the control for the read: same value before and after proves nothing.
            tag = int(time.time()) & 0xFFFF
            want = [0x5A, 0xA5, tag & 0xFF, (tag >> 8) & 0xFF]
            want_hex = [hex(b) for b in want]
            payload = dbus.Array([dbus.Byte(b) for b in want], signature="y")
            try:
                writable.WriteValue(payload, dbus.Dictionary({}, signature="sv"))
                print("== WriteValue a5 5a 11 22")
            except dbus.exceptions.DBusException as e:
                print(f"== WriteValue failed: {e.get_dbus_name()}")
                try:
                    writable.Write(payload, dbus.Dictionary({}, signature="sv"))
                    print("== Write a5 5a 11 22 (deprecated path)")
                except dbus.exceptions.DBusException as e2:
                    print(f"== Write failed: {e2.get_dbus_name()}")
            import time as _t
            _t.sleep(1)
            try:
                after = unwrap(writable.ReadValue(dbus.Dictionary({}, signature="sv")))
                got = [int(b) for b in after]
                verdict = "PASS" if got == want else "FAIL"
                print(f"== Read {path.split('/')[-1]} after  -> {[hex(b) for b in got]}  wanted {[hex(b) for b in want]}  {verdict}")
            except dbus.exceptions.DBusException as e:
                print(f"== Read after failed: {e.get_dbus_name()}")
            break

    try:
        char.StopNotify()
    except dbus.exceptions.DBusException:
        pass
    try:
        device.Disconnect()
    except dbus.exceptions.DBusException:
        pass

    # Pump the bus so PropertiesChanged signals are delivered, and read the value directly too: a read proves the
    # read path, and the signals prove the notify path. They are different claims.
    from gi.repository import GLib

    context = GLib.MainContext.default()
    while time.monotonic() - start < HOLD:
        while context.pending():
            context.iteration(False)
        time.sleep(0.02)
    # **The non-circular proof.** A read-back can be answered from the host's cache; a notification carrying the bytes
    # this client wrote can only exist if they crossed the link and the board stored them.
    if want_hex is not None:
        echoes = [v for _, v in received if len(v) >= 8 and v[-4:] == want_hex]
        print(
            f"== echo of the written bytes in notifications: {len(echoes)}  "
            f"{'PASS' if echoes else 'FAIL'} (wanted tail {want_hex}, last seen "
            f"{received[-1][1][-4:] if received else 'none'})"
        )
    print(f"== done: {len(received)} notification(s) in {HOLD:.0f}s")
    return 0


if __name__ == "__main__":
    sys.exit(main())
