#!/usr/bin/env python3
"""Pair with a device over BlueZ's D-Bus API, and report what BlueZ said.

    python3 pair-probe.py E4:0A:75:13:3E:E0 [hold_seconds]

Why this exists: `bluetoothctl --timeout N pair` needs the device to be in BlueZ's cache, and it silently does
nothing when it is not. The first version of this project's pairing tests read that silence as a firmware failure.
This script discovers the device itself, waits for the adapter to resolve it, and only then asks it to pair, so
"nothing happened" can never be mistaken for "the board refused".

Keep the D-Bus connection alive for the whole exchange -- the same trap the GATT probe hit: a `gdbus call` runs in a
process that exits immediately, and BlueZ tears down anything that client set up.
"""

import sys
import time

import dbus
from dbus.mainloop.glib import DBusGMainLoop
from gi.repository import GLib

BLUEZ = "org.bluez"
ADAPTER = "/org/bluez/hci0"
ADAPTER_IFACE = "org.bluez.Adapter1"
DEVICE_IFACE = "org.bluez.Device1"
PROPERTIES = "org.freedesktop.DBus.Properties"


def find_device(bus, address: str, timeout: float):
    """Walk BlueZ's object tree for the device with this address.

    The path is not something to guess: `dev_E4_0A_...` looked right and resolved to an object with no Device1
    interface, which produced `Method "Connect" doesn't exist`. The ObjectManager knows what exists.
    """
    deadline = time.time() + timeout
    while time.time() < deadline:
        GLib.MainContext.default().iteration(False)
        manager = dbus.Interface(bus.get_object(BLUEZ, "/"), "org.freedesktop.DBus.ObjectManager")
        try:
            objects = manager.GetManagedObjects()
        except dbus.exceptions.DBusException:
            time.sleep(0.5)
            continue
        for obj_path, interfaces in objects.items():
            device = interfaces.get(DEVICE_IFACE)
            if device is None:
                continue
            if str(device.get("Address", "")).upper() == address:
                return dbus.ObjectPath(obj_path)
        time.sleep(0.5)
    return None


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    address = sys.argv[1].upper()
    hold = float(sys.argv[2]) if len(sys.argv) > 2 else 20.0

    DBusGMainLoop(set_as_default=True)
    bus = dbus.SystemBus()
    adapter = dbus.Interface(bus.get_object(BLUEZ, ADAPTER), ADAPTER_IFACE)
    props = dbus.Interface(bus.get_object(BLUEZ, ADAPTER), PROPERTIES)

    # Discovery first: BlueZ only creates the Device1 object during its own scan.
    try:
        adapter.StartDiscovery()
    except dbus.exceptions.DBusException as exc:
        print(f"   discovery: {exc.get_dbus_name()}")
    path = find_device(bus, address, 15)
    try:
        adapter.StopDiscovery()
    except dbus.exceptions.DBusException:
        pass

    if path is None:
        print(f"   {address} not discovered in 15 s -- the board is not on air, or BlueZ is wedged")
        return 1
    print(f"== {address} discovered")

    device = dbus.Interface(bus.get_object(BLUEZ, path), DEVICE_IFACE)
    dev_props = dbus.Interface(bus.get_object(BLUEZ, path), PROPERTIES)

    print("   connecting")
    try:
        device.Connect()
    except dbus.exceptions.DBusException as exc:
        print(f"   connect: {exc.get_dbus_name()}: {exc.get_dbus_message()}")
        return 1

    # BlueZ exposes the link's own view: if this flips to true, the link is encrypted, which is the real outcome of a
    # successful pairing -- "Pair() returned" is not the same thing.
    for _ in range(30):
        GLib.MainContext.default().iteration(False)
        if bool(dev_props.Get(DEVICE_IFACE, "Connected")):
            break
        time.sleep(0.2)

    print("   pairing")
    started = time.time()
    result = "unknown"
    try:
        device.Pair()
        result = "Pair() returned without error"
    except dbus.exceptions.DBusException as exc:
        result = f"{exc.get_dbus_name()}: {exc.get_dbus_message()}"
    print(f"   pair: {result}  ({time.time() - started:.1f}s)")

    # Let the stack finish and any bond be written before reading the result back.
    until = time.time() + hold
    while time.time() < until:
        GLib.MainContext.default().iteration(False)
        time.sleep(0.1)

    for key in ("Paired", "Bonded", "Connected", "Trusted"):
        try:
            print(f"   {key}: {bool(dev_props.Get(DEVICE_IFACE, key))}")
        except dbus.exceptions.DBusException as exc:
            print(f"   {key}: {exc.get_dbus_name()}")

    try:
        device.Disconnect()
    except dbus.exceptions.DBusException:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
