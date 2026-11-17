#!/bin/bash
# A placa reinicia? Amostra o registo cinco vezes e verifica se `seconds` cresce ou recomeca.
#
# O sintoma que isto investiga: o operador viu "o LED fica ligado por um tempo e desliga", e o
# telemóvel não encontra o dispositivo. Uma placa que reinicia explica os dois: deixa de anunciar
# durante o arranque e volta a contar segundos do zero.
set -uo pipefail
ADDR=0x2003F000

ler() {
  sudo -n openocd -f interface/cmsis-dap.cfg -f /tmp/openocd-mg24-mini.cfg \
    -c 'init; halt' -c "mdw $ADDR 72" -c 'resume; shutdown' 2>/dev/null \
    | sed -n 's/^0x2003f[0-9a-f]*: \(.*\)/\1/p' | tr '\n' ' '
}

anterior=""
for i in 1 2 3 4 5; do
  TEXTO=$(ler)
  SEQ=$(printf '%s' "$TEXTO" | awk '{print strtonum("0x"$2)}')
  MSG=$(printf '%s' "$TEXTO" | tr -d '\0' | grep -o "seconds=[0-9]*" | head -1)
  SECS=$(printf '%s' "$MSG" | cut -d= -f2)
  echo "amostra $i: seq=$SEQ ${MSG:-<registo sem seconds>}"
  if [ -n "$SECS" ] && [ -n "$anterior" ] && [ "$SECS" -lt "$anterior" ]; then
    echo "  *** seconds DIMINUIU ($anterior -> $SECS): a placa REINICIOU ***"
  fi
  [ -n "$SECS" ] && anterior="$SECS"
  sleep 25
done
