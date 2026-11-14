#!/usr/bin/env python3
"""Verify the BME680 on a phone's terms: one bluetoothctl session, read all three, subscribe, decode.

    python3 host-env-verify.py <address> [notify-seconds]

Everything happens in **one** bluetoothctl process. A device learned during a scan is temporary in BlueZ: it is dropped
when the process that discovered it exits, so a connect in the next invocation answers "not available" for a board that
is transmitting -- measured here, twice, and each time it looked like a firmware fault.

The decode is the point. `e0 0c` is not a reading; 32.96 °C is. The three standard UUIDs have fixed units and this
script applies them, which is exactly what a phone does when it labels the characteristic.
"""

import re
import subprocess
import sys

TEMP = "00002a6e-0000-1000-8000-00805f9b34fb"
HUM = "00002a6f-0000-1000-8000-00805f9b34fb"
PRESS = "00002a6d-0000-1000-8000-00805f9b34fb"


def run_session(address: str, notify_seconds: int) -> str:
    script = (
        "scan on\n"
        + "@12\n"
        + f"connect {address}\n"
        + "@10\n"
        + "menu gatt\n"
        + "list-attributes " + address + "\n"
        + "@3\n"
    )
    for uuid in (TEMP, HUM, PRESS):
        script += f"select-attribute {uuid}\n@1\nread\n@3\n"
    script += f"select-attribute {TEMP}\n@1\nnotify on\n@{notify_seconds}\nquit\n"

    # bluetoothctl has no `sleep`; a helper feeds lines with the pauses in the *writer*, which is what gives discovery
    # and the link time to settle without a second process.
    feeder = "".join(
        ("sleep %s\n" % line[1:]) if line.startswith("@") else ("printf '%s\\n' \"%s\"\n" % (line, line))
        for line in script.splitlines()
    )
    with open("/tmp/envverify-feed.sh", "w") as handle:
        handle.write(feeder)

    result = subprocess.run(
        [
            "sudo",
            "timeout",
            str(notify_seconds + 60),
            "sh",
            "-c",
            "sh /tmp/envverify-feed.sh | timeout "
            + str(notify_seconds + 55)
            + " bluetoothctl --timeout "
            + str(notify_seconds + 50),
        ],
        capture_output=True,
        text=True,
        errors="replace",
    )
    return result.stdout + result.stderr


def decode(uuid: str, raw: bytes):
    if uuid == TEMP and len(raw) >= 2:
        return f"{int.from_bytes(raw[:2], 'little', signed=True) * 0.01:.2f} °C"
    if uuid == HUM and len(raw) >= 2:
        return f"{int.from_bytes(raw[:2], 'little') * 0.01:.2f} %"
    if uuid == PRESS and len(raw) >= 4:
        return f"{int.from_bytes(raw[:4], 'little') * 0.1 / 100.0:.2f} hPa"
    return "?"


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    address = sys.argv[1].upper()
    hold = int(sys.argv[2]) if len(sys.argv) > 2 else 10

    log = run_session(address, hold)
    with open("/tmp/envverify.log", "w") as handle:
        handle.write(log)

    connected = "Connection successful" in log
    # The listing prints the UUID on the line after "Characteristic (Handle 0xNNNN)". Matching the UUID itself is both
    # simpler and truer: it is what the client would show, whatever this bluetoothctl version's labels look like.
    characteristics = sorted(
        set(re.findall(r"\b(0000(?:2a6[d-f]|181a)-0000-1000-8000-00805f9b34fb)\b", log))
    )
    print(f"connect: {'successful' if connected else 'FAILED'}")
    print(f"characteristics discovered: {len(characteristics)}")
    for uuid in characteristics:
        print(f"  {uuid}")

    # A read prints "Value:" on one line and the bytes on one of the next three, indented.
    hex_lines = []
    lines = log.splitlines()
    for index, line in enumerate(lines):
        if "Value:" in line:
            for follow in lines[index + 1 : index + 4]:
                match = re.match(r"^\s+([0-9a-f]{2}(?: [0-9a-f]{2})*)\s+\S*\s*$", follow)
                if match:
                    hex_lines.append(match.group(1))
                    break
    reads = hex_lines
    print("reads:")
    for uuid, hexbytes in zip((TEMP, HUM, PRESS), reads):
        raw = bytes(int(b, 16) for b in hexbytes.split())
        print(f"  {uuid}  {raw.hex():16s} {decode(uuid, raw)}")

    notifications = hex_lines[3:]
    print(f"notifications: {len(notifications)}")
    if notifications:
        raw = bytes(int(b, 16) for b in notifications[-1].split())
        print(f"  last 0x{raw.hex()}  {decode(TEMP, raw)}")

    ok = connected and len(characteristics) >= 3 and len(reads) >= 3
    print(f"== VERDICT: {'PASS' if ok else 'FAIL'}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
