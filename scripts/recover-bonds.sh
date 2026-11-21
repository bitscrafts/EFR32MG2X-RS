#!/bin/bash
# Recupera a placa quando ela anuncia mas ninguem se lhe liga.
#
# Sintoma: o LED diz que esta' a anunciar, a app nao encontra nada, um scanner tambem nao,
# e nao ha' erro em lado nenhum. Duas ocorrencias medidas (2026-11-17 e 2026-11-21).
#
# O que faz, e pela ordem certa: apaga SO' a base de bonds (NVM3, 5 paginas, com a guarda do
# modulo acima do piso da aplicacao) e repoe o firmware. Sao duas variaveis de uma vez -- e'
# recuperacao, nao diagnostico, e o registo em docs/05-operations/BOND-DATABASE-FAILURES.md
# diz isso com a mesma clareza.
#
# Uso:
#   bash scripts/recover-bonds.sh              # apaga bonds + repoe o firmware
#   bash scripts/recover-bonds.sh --firmware   # so' repoe o firmware (nao apaga bonds)
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
export PATH=/usr/local/cargo/bin:$PATH

FLASH_HOST=${FLASH_HOST:-10.10.10.217}
FLASH_USER=${FLASH_USER:-mvcorrea}
TARGET=thumbv8m.main-none-eabihf
SO_FIRMWARE=0
[ "${1:-}" = "--firmware" ] && SO_FIRMWARE=1

flasha() { # <exemplo> <features>
    local ex=$1 feats=$2 elf=target/$TARGET/release/examples/$1
    echo "== build $ex"
    cargo build --release -p efr32mg2x-xiao-bsp --example "$ex" --features "$feats" \
        --target "$TARGET" 2>&1 | grep -E "^error" -A3 | head -6
    [ -f "$elf" ] || { echo "REFUSE: sem ELF para $ex"; return 1; }
    echo "== flash $ex"
    FLASH_HOST=$FLASH_HOST FLASH_USER=$FLASH_USER bash scripts/flash.sh "$elf" --confirm --run \
        > "/tmp/recover-$ex.log" 2>&1
    local rc=$?
    grep -E "load_min=|Verification" "/tmp/recover-$ex.log" | head -2
    echo "rc=$rc"
    return $rc
}

if [ "$SO_FIRMWARE" -eq 0 ]; then
    echo "### 1/2 apagar a base de bonds (so' o NVM3, guarda activa)"
    flasha module_nvm3_wipe "sense stack" || { echo "FALHOU o wipe"; exit 1; }
    sleep 2
else
    echo "### 1/2 saltado (--firmware: a base de bonds fica como esta')"
fi

echo "### 2/2 repor o firmware do dosimetro"
flasha module_ble_dosimeter "stack sense external-bonding" || { echo "FALHOU o firmware"; exit 1; }

echo
echo "### sonda"
ssh -o BatchMode=yes "$FLASH_USER@$FLASH_HOST" \
    "sudo -n probe-rs list 2>/dev/null | grep -c CMSIS-DAP" | sed 's/^/probes: /'
echo "feito -- abre a app e liga. Se nao ligar, o proximo passo e' registar o motivo da falha."