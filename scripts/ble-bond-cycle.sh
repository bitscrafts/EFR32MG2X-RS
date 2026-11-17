#!/usr/bin/env bash
# One complete bond cycle, unattended: clean both sides, pair, reset WITHOUT reflashing, reconnect without pairing,
# and record the verdict with timestamps and evidence.
#
# The feature set is the PRODUCT's (`sense,stack`, the same one `module_ble_dosimeter` requires), not a probe-only set:
# a result measured on a binary the product does not build is not a result about the product.
#
# Why two flashes: the device can only be cleared through firmware (`CLEAR_BONDS`), because with the builtin database
# the delete command exists but the probe is not asked for bonds during a run. Phase A boots with cleaning on, phase B
# boots with cleaning off so that the reset under test does not clear what the pairing wrote.
#
# Exit 0 always: this runs unattended and its product is the log line.
set -u

PROJECT="/projects/efr32mg24-rs"
LOG="${LOG:-${PROJECT}/_tmp/ble-bond-cycle.log}"
ELF="/projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/module_pairing_probe"
SRC="${PROJECT}/crates/xiao-mg24-bsp/examples/module_pairing_probe.rs"
HOST="${FLASH_HOST:-10.10.10.217}"
USER="${FLASH_USER:-mvcorrea}"
FEATURES="${FEATURES:-sense,stack}"
EXTRA="${EXTRA:-}"                      # e.g. "external-bonding" to measure the other database
# Phase A always flashes the probe with CLEAR_BONDS (same NVM3 instance and key ranges as the product, so the wipe is
# equivalent); phase B flashes EXAMPLE, because a result measured on the probe is not a result about the product
# (the probe's loop has nothing else to do, which is why it paired when the product did not).
EXAMPLE="${EXAMPLE:-module_pairing_probe}"
ELF_B="/projects/_cache/targets/thumbv8m.main-none-eabihf/release/examples/${EXAMPLE}"

export PATH="/usr/local/cargo/bin:${PATH}"
cd "${PROJECT}" || exit 0
mkdir -p "$(dirname "${LOG}")"
say() { printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" >> "${LOG}"; }

build() {
    # $1 is the example to build: phase A passes the probe, phase B passes ${EXAMPLE}.
    local example="${1:-module_pairing_probe}"
    touch "${PROJECT}/crates/xiao-mg24-bsp/examples/${example}.rs"
    CARGO_TARGET_DIR=/projects/_cache/targets cargo build --release --target thumbv8m.main-none-eabihf \
        -p efr32mg2x-xiao-bsp --features "${FEATURES}${EXTRA:+,${EXTRA}}" --example "${example}" \
        > /tmp/cycle-build.log 2>&1
}

set_clear() {
    python3 - "$SRC" "$1" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
want = "true" if sys.argv[2] == "on" else "false"
t = p.read_text()
for cur in ("true", "false"):
    t = t.replace(f"const CLEAR_BONDS: bool = {cur};", f"const CLEAR_BONDS: bool = {want};")
p.write_text(t)
PY
}

say "=== cycle start (features: ${FEATURES}${EXTRA:+,${EXTRA}})"

# Phase A -- clean the device's bonds.
set_clear on
if ! build module_pairing_probe; then say "build (clear) FAILED -- see /tmp/cycle-build.log"; set_clear off; exit 0; fi
FLASH_HOST="${HOST}" FLASH_USER="${USER}" bash scripts/flash.sh "${ELF}" --confirm --run > /tmp/cycle-flash-a.log 2>&1 \
    || { say "flash (clear) FAILED"; set_clear off; exit 0; }
sleep 5
say "phase A flashed: device bonds cleared at boot"
A=$(bash scripts/read-mailbox.sh "${HOST}" "${USER}" 2>/dev/null | grep "live:" | head -1)
say "  after clean: ${A}"

# Phase B -- the run under test.
set_clear off
if ! build "${EXAMPLE}"; then say "build (test) FAILED -- see /tmp/cycle-build.log"; exit 0; fi
INTERNAL=$(arm-none-eabi-nm "${ELF_B}" 2>/dev/null | grep -c "ubt_pskey_write" || true)
say "builtin bonding database in the binary (${EXAMPLE}): ${INTERNAL} (0 means the external one is in use)"
FLASH_HOST="${HOST}" FLASH_USER="${USER}" bash scripts/flash.sh "${ELF_B}" --confirm --run > /tmp/cycle-flash-b.log 2>&1 \
    || { say "flash (test) FAILED"; exit 0; }
say "phase B flashed: $(python3 scripts/elf-load-range.py "${ELF_B}" | head -1)"

# Pair from a clean host.
timeout 120 ssh "${USER}@${HOST}" 'sudo -n sh /tmp/pair-clean.sh' > /tmp/cycle-pair.log 2>&1
say "pair: successful=$(grep -ciE 'pairing successful' /tmp/cycle-pair.log || true) bonded_yes=$(grep -ciE 'Bonded: yes' /tmp/cycle-pair.log || true)"
grep -iE "Failed to pair|auth failed" /tmp/cycle-pair.log | head -1 | while read -r l; do say "  host: $(echo "${l}" | tr -d '\r')"; done
say "device after pairing: $(bash scripts/read-mailbox.sh "${HOST}" "${USER}" 2>/dev/null | grep 'live:' | head -1)"

# Reset without reflashing; NVM3 must survive and the host must reconnect without pairing.
timeout 60 ssh "${USER}@${HOST}" "sudo -n probe-rs reset --chip EFR32MG24B220F1536IM48" > /dev/null 2>&1
sleep 7
timeout 120 ssh "${USER}@${HOST}" 'sudo -n sh /tmp/ble-reconnect.sh' > /tmp/cycle-reconnect.log 2>&1
say "reconnect after reset: connected=$(grep -c 'Connection successful' /tmp/cycle-reconnect.log || true) pin_or_key_missing=$(grep -c '0x1006' /tmp/cycle-reconnect.log || true)"
AFTER=$(bash scripts/read-mailbox.sh "${HOST}" "${USER}" 2>/dev/null | grep 'live:' | head -1)
say "device after reset+reconnect: ${AFTER}"
# The device's own account of the link: security_mode 1 is encryption with an unauthenticated key. `Bonded: yes` on the
# host is the host's storage and is printed whether or not this link encrypted, so `sec=` is the line that decides.
say "link security after reset: $(printf '%s' "${AFTER}" | grep -o 'sec=[0-9]*' | head -1)"

# Is the host's long-term key in the device's flash, in either byte order?
timeout 90 ssh "${USER}@${HOST}" "sudo -n probe-rs read b8 0x08176000 40960 --chip EFR32MG24B220F1536IM48" > /tmp/cycle-nvm3.txt 2>&1
KEYHEX=$(timeout 60 ssh "${USER}@${HOST}" 'sudo -n grep -m1 "^Key=" "/var/lib/bluetooth/90:DE:80:81:44:56/E4:0A:75:13:3E:E0/info" 2>/dev/null' | cut -d= -f2 | tr -d '[:space:]' || true)
python3 - "${KEYHEX:-}" >> "${LOG}" <<'PY'
import re, sys
key = sys.argv[1] if len(sys.argv) > 1 else ""
raw = open('/tmp/cycle-nvm3.txt', errors='ignore').read()
data = bytes(int(h, 16) for h in re.findall(r'\b[0-9a-fA-F]{2}\b', raw))
if len(data) < 1024:
    print("flash: the read failed, no bytes to search")
elif len(key) != 32:
    print("key: the host recorded no long-term key for this device")
else:
    needle = bytes.fromhex(key)
    print(f"LTK in flash: straight={data.find(needle)} reversed={data.find(needle[::-1])} bytes={len(data)}")
PY

say "=== cycle end"
