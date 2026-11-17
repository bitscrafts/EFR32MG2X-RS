#!/usr/bin/env bash
# Do the stack's bond writes reach flash, and do they stay there?
#
# The question came from a bond that is visible during its session (NVM3 objects 3 -> 8) and gone after a reset
# (8 -> 3). Two explanations survive: the stack deletes stale bonds while initialising, or the writes never leave
# the NVM3 cache. Three dumps of the NVM3 region separate them:
#
#   A before pairing, B during the same session after pairing, C after a reset
#   B != A  -> the writes reached flash
#   C == B  -> and stay there, so the loss happens while the stack initialises (it rejects what it finds)
#   C == A  -> they never reached flash
#
# Read-only: nothing is written to the device except by the pairing itself.
set -u
HOST="${FLASH_HOST:-10.10.10.217}"
USER_="${FLASH_USER:-mvcorrea}"
CHIP="EFR32MG24B220F1536IM48"
BASE=0x08176000
SIZE=40960
OUT="${1:-/tmp/bondflash}"
mkdir -p "${OUT}"

dump() {
  ssh "${USER_}@${HOST}" "sudo -n probe-rs read b8 ${BASE} ${SIZE} --chip ${CHIP} 2>/dev/null" > "${OUT}/$1.txt" 2>&1
}

live() {
  bash scripts/read-mailbox.sh "${HOST}" "${USER_}" 2>&1 | grep "live:" | head -1
}

echo "== A: antes de emparelhar"
dump A
live

echo "== emparelhar (o lnx esquece primeiro, senao responde AlreadyExists)"
ssh "${USER_}@${HOST}" 'sudo -n sh /tmp/unpair-probe.sh' >/dev/null 2>&1
ssh "${USER_}@${HOST}" 'sudo -n sh /tmp/pair-probe.sh' 2>&1 | grep -iE "Pairing successful|Failed to pair" | head -2
sleep 2
echo "-- device depois do pairing:"
live

echo "== B: mesma sessao, depois do pairing"
dump B

echo "== reset"
ssh "${USER_}@${HOST}" "sudo -n probe-rs reset --chip ${CHIP} >/dev/null 2>&1"
sleep 8
echo "-- device depois do reset:"
live

echo "== C: depois do reset"
dump C

python3 - "${OUT}" <<'PY'
import hashlib, pathlib, sys
out = pathlib.Path(sys.argv[1])
def read(name):
    raw = []
    for line in (out / f"{name}.txt").read_text().splitlines():
        if ':' not in line:
            continue
        _, rest = line.split(':', 1)
        for tok in rest.split():
            try:
                raw.append(int(tok, 16))
            except ValueError:
                pass
    return bytes(raw)
a, b, c = read('A'), read('B'), read('C')
def erased(d):
    return sum(1 for x in d if x == 0xFF)
for name, d in (('A', a), ('B', b), ('C', c)):
    print(f"{name}: {len(d)} bytes, {erased(d)} apagados (0xFF), sha256 {hashlib.sha256(d).hexdigest()[:16]}")
print()
print("B != A  (as escritas chegaram a' flash)?", "SIM" if b != a else "NAO")
print("C == B  (sobreviveram ao reset)?       ", "SIM" if c == b else "NAO")
print("C == A  (voltou ao estado inicial)?    ", "SIM" if c == a else "NAO")
PY
