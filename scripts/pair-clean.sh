#!/bin/sh
# Pair the XIAO MG24 probe from this host, from a clean state, with retries.
#
# `bluetoothctl` driven from a pipe is timing-sensitive: a pair attempt can be cancelled by the host before the
# peripheral answers (`AuthenticationCanceled`) or fail because the scan had not seen the device yet. The retry is not
# cosmetic -- it is the difference between a measurement and an accident, and every bond measurement runs through here.
set -u
DEV="E4:0A:75:13:3E:E0"
ATTEMPTS="${ATTEMPTS:-3}"

i=1
while [ "$i" -le "$ATTEMPTS" ]; do
    echo "---- attempt $i/$ATTEMPTS"
    OUT="$(
        {
            echo "remove $DEV"
            sleep 2
            echo "scan on"
            sleep 6
            echo "scan off"
            echo "pair $DEV"
            sleep 14
            echo "info $DEV"
            sleep 2
            echo "quit"
        } | timeout 70 bluetoothctl 2>&1 | sed 's/\r//'
    )"

    if printf '%s\n' "$OUT" | grep -qi "Pairing successful"; then
        printf '%s\n' "$OUT" | grep -iE "Pairing successful|Bonded:|Paired:|Connected:" | tail -4
        echo "(paired on attempt $i)"
        exit 0
    fi

    printf '%s\n' "$OUT" | grep -iE "Failed to pair|auth failed|AuthenticationCanceled" | tail -2
    i=$((i + 1))
    [ "$i" -le "$ATTEMPTS" ] && sleep 3
done

echo "(pairing failed after $ATTEMPTS attempts)"
exit 1
