#!/bin/sh
# The three cases of the pairing window (2026-11-17).
#
#   1. the device holds no bond                  -> accept (cfwhy=1)
#   2. it holds one, and we are inside 120 s     -> accept (cfwhy=2)
#   3. it holds one, and we are past 120 s       -> refuse (cfwhy=3, cfno=1)
#
# This script covers 2 and 3. Case 1 needs a device whose NVM3 holds no bond, which is a separate flash with the probe's
# CLEAR_BONDS.
set -u
DEV="E4:0A:75:13:3E:E0"

pair_now() {
  printf 'disconnect %s\nquit\n' "$DEV" | timeout 15 bluetoothctl >/dev/null 2>&1 || true
  { echo "power on"; sleep 1; echo "scan on"; sleep 5; echo "pair $DEV"; sleep 6; echo "quit"; } \
    | timeout 40 bluetoothctl 2>&1 | grep -Ei "Pairing successful|Failed to pair|AlreadyExists|Authentication" | head -2
}

echo "=== caso 2: dentro da janela (o dispositivo ja tem um bond)"
pair_now
echo "--- bond no host:"; bluetoothctl info "$DEV" 2>/dev/null | grep -cE "^\s+Bonded: yes" || true

echo "=== a esperar que a janela feche (>120 s desde o reset)"
sleep 130
echo "=== caso 3: fora da janela, com um bond na mao"
printf 'remove %s\nquit\n' "$DEV" | timeout 20 bluetoothctl >/dev/null 2>&1 || true
sleep 2
echo "--- bond no host (0 esperado):"; bluetoothctl info "$DEV" 2>/dev/null | grep -cE "^\s+Bonded: yes" || true
pair_now
echo "--- bond no host DEPOIS (0 esperado: o dispositivo recusou guardar):"
bluetoothctl info "$DEV" 2>/dev/null | grep -cE "^\s+Bonded: yes" || true
