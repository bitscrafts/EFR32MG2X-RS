#!/usr/bin/env bash
# Halt the target and report where the core is, then resume it.
#
#   scripts/where.sh [host] [user]
#
# A board that advertises but writes no diagnostic record is a board whose main loop died after starting the radio --
# the radio runs from interrupts, so the advertising continues and looks like health. The program counter says where.
set -uo pipefail
HOST="${1:-10.10.10.217}"
USER_NAME="${2:-mvcorrea}"
CFG=/tmp/openocd-mg24-mini.cfg

ssh -n -o ConnectTimeout=10 "${USER_NAME}@${HOST}" \
  "sudo -n openocd -f interface/cmsis-dap.cfg -f ${CFG} \
     -c 'init; halt' -c 'reg pc' -c 'reg lr' -c 'reg sp' -c 'resume; shutdown'" 2>&1 |
  grep -E "pc \(|lr \(|sp \(|halted|target state"
