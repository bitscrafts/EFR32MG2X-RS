# Registo da fila

Uma entrada por invocação do runner. O que conta é o veredicto do gate, não o relatório do worker.
- 2026-11-18T19:56:49Z **16:-[]C1** sem spec (specs/09-one-core-moves-and-the-pattern-proves-itself.md) — fila parada, nada foi feito.

## 2026-11-18T20:04:19Z — C1
mover spi-core para dentro do HAL e provar que só existe uma cópia
- worker: rc=1, 17s, log `_tmp/queue/worker.log`

## 2026-11-18T20:05:19Z — C1
mover spi-core para dentro do HAL e provar que só existe uma cópia
- worker: rc=1, 1s, log `_tmp/queue/worker.log`

## 2026-11-18T20:05:54Z — C2
o mesmo movimento para i2c-core, com o check a cobrir dois cores
- worker: rc=1, 1s, log `_tmp/queue/worker.log`

## 2026-11-18T20:09:32Z — C1
mover spi-core para dentro do HAL e provar que só existe uma cópia
