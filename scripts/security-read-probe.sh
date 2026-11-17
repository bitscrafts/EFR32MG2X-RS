#!/bin/sh
# Does the dosimeter's characteristic refuse an unpaired client? (review/plan step 7)
#
# The device declares BONDED_READ | BONDED_NOTIFY on all three characteristics. The question is answered by two reads of
# the same handle:
#
#   1. with no bond on this host, the read must fail with an ATT error and return no value;
#   2. after pairing, the same read must return the value.
#
# `gatttool --sec-level=low` is the right tool for the first case: it never asks the link to encrypt, which is what an
# unpaired client does. Every step prints its own output, including the empty ones -- the first version of this script
# printed nothing for the unpaired read and left "no output" indistinguishable from "refused".
set -u
DEV="E4:0A:75:13:3E:E0"
HANDLE="0x0003"

# bluetoothctl holds the link if it is connected, and gatttool then fails with "Device or resource busy" -- which is what
# the first run of this script reported instead of an answer.
printf 'disconnect %s\nquit\n' "$DEV" | timeout 20 bluetoothctl >/dev/null 2>&1 || true
sleep 1

echo "--- bond neste host:"
bluetoothctl info "$DEV" 2>/dev/null | grep -E "^\s+(Bonded|Paired):" || echo "  (nenhum)"

echo "--- 1. leitura SEM emparelhar (sec-level=low), saida completa:"
sudo -n timeout 30 gatttool -b "$DEV" --sec-level=low --char-read -a "$HANDLE" 2>&1 \
  | sed 's/^\s*//' | grep -viE "^$" | head -6
echo "    rc=$?"

echo "--- 2. emparelhar:"
{ echo "power on"; sleep 1; echo "scan on"; sleep 5; echo "pair $DEV"; sleep 7; echo "quit"; } | timeout 45 bluetoothctl 2>&1 \
  | grep -Ei "Pairing successful|Failed to pair|Authentication|AlreadyExists" | head -2

echo "--- 3. leitura DEPOIS de emparelhar (sec-level=medium), saida completa:"
printf 'disconnect %s\nquit\n' "$DEV" | timeout 20 bluetoothctl >/dev/null 2>&1 || true
sleep 1
sudo -n timeout 30 gatttool -b "$DEV" --sec-level=medium --char-read -a "$HANDLE" 2>&1 \
  | sed 's/^\s*//' | grep -viE "^$" | head -6
echo "    rc=$?"

echo "--- 4. o dispositivo diz (ler o mailbox):"
