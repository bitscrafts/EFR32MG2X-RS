#!/usr/bin/env bash
# Orphan bond: the device keeps its bond and the phone forgets it. Before the policy, the device answered the new
# pairing with the old long-term key and the phone cancelled (`0x05 Authentication Failed`), leaving the product
# un-pairable until it was wiped with a debugger.
#
# This measures the three states in order, on the product example:
#   1. pair                      baseline, and the device has a bond
#   2. remove only the host bond  -> the orphan state
#   3. pair again                -> the policy must drop the stale bond and let the new one in
#   4. reset, reconnect           -> and the link must come back encrypted (sec=1) with no pairing
set -u
HOST="${FLASH_HOST:-10.10.10.217}"
USER_="${FLASH_USER:-mvcorrea}"
MAC="${MAC:-E4:0A:75:13:3E:E0}"
STAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

ssh_() { ssh -o BatchMode=yes "${USER_}@${HOST}" "$@"; }

echo "=== orphan bond cycle, ${STAMP}"
echo "--- 1. pair (device clean, host clean)"
ssh_ 'sudo -n sh /tmp/pair-clean.sh' 2>&1 | grep -E "paired on attempt|pairing failed|Bonded" | tail -3

echo "--- 2. remove ONLY the host bond: the orphan state"
ssh_ "sudo -n bluetoothctl remove ${MAC}" 2>&1 | tail -1
ssh_ "sudo -n bluetoothctl info ${MAC}" 2>&1 | grep -cE "Bonded: yes" | sed 's/^/host still bonded (0 expected): /'

echo "--- 3. pair again, with the device still holding the old bond"
ssh_ 'sudo -n sh /tmp/pair-clean.sh' 2>&1 | grep -E "paired on attempt|pairing failed|AuthenticationFailed|Bonded" | tail -4

echo "--- 4. reset, then reconnect without pairing"
ssh_ 'sudo -n probe-rs reset --chip EFR32MG24B220F1536IM48' >/dev/null 2>&1
sleep 8
ssh_ 'sudo -n sh /tmp/ble-reconnect.sh' 2>&1 | grep -iE "Connection successful|Bonded:|Connected:" | tail -3

sleep 65
echo "--- record (stale= and dkeys= are the policy; sec= is the truth about the link)"
bash scripts/read-mailbox.sh "${HOST}" "${USER_}" 2>&1 | tail -1 | grep -oE "sreq=[0-9]+|sec=[0-9]+|mtu=[0-9]+|stale=[0-9]+|dkeys=[0-9]+|windows=[0-9]+|published=[0-9]+|disc=[0-9]+|adv=[0-9]+"
echo "=== end ${STAMP}"
