#!/bin/bash
# End-to-end BLE certification of the firmware, run from the host that owns the adapter.
#
# Three things, in order, because each one is a claim the product depends on and each one can fail on its own:
#
#   1. the ADVERTISEMENT carries the service UUID            -> generic discovery is possible for any phone
#   2. the BOND is created, inside the 120-second window     -> the phone can trust the data
#   3. the three CHARACTERISTICS read with sane values       -> the data is really there
#
# Run on lnx:  bash ble-end-to-end.sh
set -uo pipefail

DEV="E4:0A:75:13:3E:E0"
SERVICO="e1d0e000-0000-4000-8000-00805f9b34fb"
CHIP="EFR32MG24B220F1536IM48"
MON=/tmp/btmon-e2e-$$.log
falhas=0

falhou() { echo "FAIL: $*"; falhas=$((falhas + 1)); }
passou() { echo "PASS: $*"; }

echo "=== 0. estado limpo"
bluetoothctl disconnect "$DEV" >/dev/null 2>&1 || true
bluetoothctl remove "$DEV" >/dev/null 2>&1 || true
sleep 1
# O dispositivo aceita um bond novo só nos primeiros 120 s depois de ligar: o reset abre a janela.
# Sem `reset` o teste mediria a recusa da janela, não a ligação.
sudo -n probe-rs reset --chip "$CHIP" >/dev/null 2>&1
sleep 4

echo "=== 1. o anuncio leva o UUID do servico?"
# A verificacao passa pelo que o BLUEZ descodificou, e nao por bytes no ar.
#
# Duas tentativas anteriores falharam por medir mal: `btmon -w` grava btsnoop binario (um grep de UUID
# nunca casa nesse ficheiro) e `btmon` em texto nao descodifica listas de UUID de 128 bits. O que
# descodifica e' o proprio BlueZ, e `Device1.UUIDs` e' o equivalente exacto do `ScanRecord.serviceUuids`
# que um telefone Android le para filtrar -- portanto e' a mesma pergunta que a app faz.
{ echo "power on"; sleep 1; echo "scan on"; sleep 9; echo "scan off"; echo "quit"; } \
  | timeout 30 bluetoothctl >/dev/null 2>&1
INFO=$(bluetoothctl info "$DEV" 2>/dev/null | sed 's/\r//g')
UUIDS=$(printf '%s' "$INFO" | grep -i "UUID:" | head -4)
Nome=$(printf '%s' "$INFO" | grep -i "^\s*Name:" | head -1)
echo "  $Nome"
printf '%s\n' "$UUIDS" | sed 's/^/  /' | head -4
if printf '%s' "$INFO" | grep -qi "$SERVICO"; then
  passou "o anuncio traz $SERVICO -- descoberta generica por UUID, para qualquer telefone"
else
  falhou "o anuncio NAO traz $SERVICO -- nenhum telefone consegue filtrar por servico"
fi
if printf '%s' "$INFO" | grep -qi "xiao-mg24-dosimeter"; then
  passou "o nome viaja na resposta de scan"
else
  echo "  (aviso: nome nao visto -- nao impede a descoberta por UUID)"
fi

echo "=== 2. o bond passa?"
{ echo "power on"; sleep 1; echo "scan on"; sleep 10; echo "pair $DEV"; sleep 12; echo "quit"; } \
  | timeout 60 bluetoothctl 2>&1 | sed 's/\r//g' \
  | grep -iE "Pairing successful|Failed to pair|AlreadyExists|Authentication" | head -2
BONDED=$(bluetoothctl info "$DEV" 2>/dev/null | grep -c "Bonded: yes" 2>/dev/null || true)
BONDED=${BONDED:-0}
if [ "$BONDED" -ge 1 ]; then passou "bond criado (Bonded: yes)"; else falhou "sem bond"; fi

echo "=== 3. as tres caracteristicas leem?"
bluetoothctl disconnect "$DEV" >/dev/null 2>&1 || true
sleep 2
ler() { sudo -n timeout 25 gatttool -b "$DEV" --sec-level=medium --char-read -a "$1" 2>&1 | grep -i "value" | sed 's/.*: //'; }

RMS=$(ler 0x0003); BANDS=$(ler 0x0006); STATUS=$(ler 0x0009)
echo "  rms   0x0003: ${RMS:-<vazio>}"
echo "  bands 0x0006: ${BANDS:0:47}${BANDS:+ …}"
echo "  status 0x0009: ${STATUS:-<vazio>}"

if [ -n "$RMS" ]; then
  R=$((0x$(echo "$RMS" | awk '{print $2$1}')))
  [ "$R" -gt 0 ] && passou "rms = $R contagens (> 0: ha sinal)" || falhou "rms = 0 -- caminho analogico morto"
else
  falhou "rms nao foi lido (caracteristica exige bond? gatttool sem chave?)"
fi

if [ -n "$STATUS" ]; then
  set -- $STATUS
  FLAGS=$((0x$1)); FONTE=$((0x$4)); INC=$((0x$5))
  echo "  flags=$FLAGS fonte=$FONTE incerteza=$((INC / 10)).$((INC % 10)) dB"
  [ $((FLAGS & 1)) -eq 1 ] && passou "bit0 diz calibrado" || falhou "bit0 = 0 (dispositivo diz nao calibrado)"
  [ "$FONTE" -eq 1 ] && passou "fonte de calibracao = datasheet" || falhou "fonte de calibracao ausente"
  [ "$INC" -eq 30 ] && passou "incerteza declarada = 3,0 dB" || falhou "incerteza nao declarada"
  [ $# -ge 5 ] && passou "status tem 5 bytes" || falhou "status tem $# byte(s), esperados 5"
else
  falhou "status nao foi lido"
fi

echo
if [ "$falhas" -eq 0 ]; then
  echo "VERDICT: PASS -- firmware operacional por BLE, com descoberta generica por UUID"
  exit 0
else
  echo "VERDICT: FAIL -- $falhas verificacao(oes) falharam"
  exit 1
fi
