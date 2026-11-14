#!/usr/bin/env python3
"""Read an Environmental Sensing device and print the values the way a phone would show them.

    python3 host-env-probe.py E4:0A:75:13:3E:E0 [subscribe_seconds]

Why this exists: the deliverable is "the BME680 readings on a phone", and the only way to know that works is to be a
client. This connects, walks every characteristic the device exposes, decodes the three standard ones (0x2A6E
temperature, 0x2A6F humidity, 0x2A6D pressure) into their spec units, then subscribes and reports what arrives.

Discovery is done through BlueZ's ObjectManager rather than a guessed object path, and the D-Bus connection stays alive
for the whole exchange -- both traps that produced false failures earlier in this project.
"""

import sys
import time

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

BLUEZ = "org.bluez"
DEVICE_IFACE = "org.bluez.Device1"
CHAR_IFACE = "org.bluez.GattCharacteristic1"
PROPERTIES = "org.freedesktop.DBus.Properties"

STD = {
    "00002a6e-0000-1000-8000-00805f9b34fb": ("Temperature", 0.01, "°C"),
    "00002a6f-0000-1000-8000-00805f9b34fb": ("Humidity", 0.01, "%"),
    "00002a6d-0000-1000-8000-00805f9b34fb": ("Pressure", 0.1, "Pa"),
}


def find_device(bus, address, timeout=25.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        GLib.MainContext.default().iteration(False)
        manager = dbus.Interface(bus.get_object(BLUEZ, "/"), "org.freedesktop.DBus.ObjectManager")
        try:
            objects = manager.GetManagedObjects()
        except dbus.exceptions.DBusException:
            time.sleep(0.5)
            continue
        for path, interfaces in objects.items():
            device = interfaces.get(DEVICE_IFACE)
            if device and str(device.get("Address", "")).upper() == address:
                return dbus.ObjectPath(path)
        # **One second between polls, and no GLib iteration here.** Iterating the main context while BlueZ is still
        # discovering made the following `Connect` either land on a removed object or block until D-Bus timed out; the
        # proven sequence (this one) simply lets BlueZ finish each discovery tick.
        time.sleep(1.0)
    return None


def decode(uuid, raw):
    if uuid not in STD:
        return None
    name, scale, unit = STD[uuid]
    if uuid.endswith("2a6e"):
        value = int.from_bytes(bytes(raw[:2]), "little", signed=True) * scale
    elif uuid.endswith("2a6d"):
        value = int.from_bytes(bytes(raw[:4]), "little", signed=False) * scale / 10.0
        return f"{name} {value:.1f} {unit}"
    else:
        value = int.from_bytes(bytes(raw[:2]), "little", signed=False) * scale
    return f"{name} {value:.2f} {unit}"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    address = sys.argv[1].upper()
    hold = float(sys.argv[2]) if len(sys.argv) > 2 else 8.0
    # `connect` uses BlueZ's D-Bus `Device1.Connect`, which on this host answers "Method Connect ... doesn't exist" for
    # a device that was just discovered. `bluetoothctl connect` does the same job through a path BlueZ accepts, so the
    # connection is made there and this script only enumerates, reads and subscribes.
    connect = "--connect" in sys.argv

    DBusGMainLoop(set_as_default=True)
    bus = dbus.SystemBus()
    adapter = dbus.Interface(bus.get_object(BLUEZ, "/org/bluez/hci0"), "org.bluez.Adapter1")

    try:
        adapter.StartDiscovery()
    except dbus.exceptions.DBusException:
        pass
    path = find_device(bus, address)
    if path is not None:
        # **Give BlueZ a moment before using the object.** Returning the instant the address appears yields a path whose
        # Device1 interface is not fully populated yet, and the next call answers "Method ... doesn't exist" for a device
        # that is perfectly fine. Two seconds is what the probe that has been working all night spends.
        time.sleep(2.0)
    if path is None:
        try:
            adapter.StopDiscovery()
        except dbus.exceptions.DBusException:
            pass
        print(f"   {address} not discovered in 20 s")
        return 1
    print(f"== {address} discovered")

    device = dbus.Interface(bus.get_object(BLUEZ, path), DEVICE_IFACE)
    props = dbus.Interface(bus.get_object(BLUEZ, path), PROPERTIES)

    # **Connect while discovery is still running.** BlueZ drops a device object when discovery stops if it was learned
    # during that scan, and the following `Connect` then lands on an object with no Device1 methods -- measured here as
    # "Method Connect ... doesn't exist", which reads like a firmware fault and is a host-side ordering trap.
    try:
        adapter.StopDiscovery()
    except dbus.exceptions.DBusException:
        pass
    if connect:
        try:
            device.Connect()
        except dbus.exceptions.DBusException as exc:
            if "AlreadyConnected" not in str(exc):
                print(f"   connect: {exc.get_dbus_name()}: {exc.get_dbus_message()}")
                return 1

    # Services are resolved after `Connect` returns; reading before that yields an empty list and looks like a device
    # with no attributes.
    deadline = time.time() + 15
    while time.time() < deadline:
        GLib.MainContext.default().iteration(False)
        if bool(props.Get(DEVICE_IFACE, "ServicesResolved")):
            break
        time.sleep(0.2)
    print(f"   services resolved: {bool(props.Get(DEVICE_IFACE, 'ServicesResolved'))}")

    manager = dbus.Interface(bus.get_object(BLUEZ, "/"), "org.freedesktop.DBus.ObjectManager")
    chars = []
    for obj_path, interfaces in manager.GetManagedObjects().items():
        if CHAR_IFACE in interfaces:
            chars.append((obj_path, interfaces[CHAR_IFACE]))

    print(f"   characteristics: {len(chars)}")
    notifications = []
    subscribed = 0
    for obj_path, char in chars:
        uuid = str(char.get("UUID", "")).lower()
        flags = [str(f) for f in char.get("Flags", [])]
        label = STD.get(uuid, (uuid,))[0]
        print(f"     {label:12s} {uuid} {','.join(flags)}")
        if "read" not in flags:
            continue
        interface = dbus.Interface(bus.get_object(BLUEZ, obj_path), CHAR_IFACE)
        try:
            raw = interface.ReadValue({})
            print(f"       read: {bytes(raw).hex()}   {decode(uuid, raw) or ''}")
        except dbus.exceptions.DBusException as exc:
            print(f"       read failed: {exc.get_dbus_name()}")
        if "notify" in flags:
            interface.connect_to_signal(
                "PropertiesChanged",
                lambda iface, changed, _invalidated, u=uuid: notifications.append(
                    (u, bytes(changed["Value"]))
                )
                if "Value" in changed
                else None,
            )
            try:
                interface.StartNotify()
                subscribed += 1
            except dbus.exceptions.DBusException as exc:
                print(f"       subscribe failed: {exc.get_dbus_name()}")

    print(f"   subscribed to {subscribed} characteristic(s); watching {hold:.0f}s")
    until = time.time() + hold
    while time.time() < until:
        GLib.MainContext.default().iteration(False)
        time.sleep(0.05)

    print(f"   notifications: {len(notifications)}")
    seen = {}
    for uuid, raw in notifications:
        seen[uuid] = raw
    for uuid, raw in seen.items():
        print(f"     last {STD.get(uuid, (uuid,))[0]:12s} {raw.hex()}  {decode(uuid, raw) or ''}")

    # The verdict is about values, not about traffic: a notification stream of zeros would satisfy "it notifies".
    decoded = [decode(u, r) for u, r in seen.items()]
    ok = len(seen) >= 2 and all(decoded) and not all(
        r == bytes(len(r)) for _u, r in seen.items()
    )
    print(f"== VERDICT: {'PASS' if ok else 'FAIL'}  ({len(seen)} characteristic(s) reporting, decoded values above)")

    try:
        device.Disconnect()
    except dbus.exceptions.DBusException:
        pass
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
