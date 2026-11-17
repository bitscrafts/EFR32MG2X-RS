#!/bin/bash
# Espera que a placa volte a escrever um registo válido, e diz o que encontrou.
#
# Uso: bash wait-for-record.sh [segundos]
# Serve para o operador desligar e voltar a ligar a alimentação e o agente confirmar, sem adivinhar.
set -uo pipefail
LIMITE="${1:-180}"
ADDR=0x2003F000
INICIO=$(date +%s)

while [ $(( $(date +%s) - INICIO )) -lt "$LIMITE" ]; do
  TEXTO=$(sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg \
    -c 'init; halt' -c "mdw $ADDR 72" -c 'resume; shutdown' 2>/dev/null \
    | sed -n 's/^0x2003f[0-9a-f]*: \(.*\)/\1/p' | tr '\n' ' ')
  if printf '%s' "$TEXTO" | grep -q "seconds="; then
    echo "REGISTO VALIDO:"
    printf '%s\n' "$TEXTO" | tr -s ' ' | cut -c1-400
    exit 0
  fi
  echo "  sem registo ainda ($(( $(date +%s) - INICIO ))s) -- core em $(printf '%s' "$TEXTO" | cut -c1-40)"
  sleep 12
done
echo "SEM REGISTO em ${LIMITE}s -- a placa continua a nao escrever"
exit 1
