#!/usr/bin/env python3
"""Validate the whole BLE path to the dosimeter, in one small file.

Why Python and not the shell scripts: the shell path grew a scan, a `bluetoothctl` session, a `gatttool` session and
three retries, and each of those is a place where a measurement can be wrong for reasons that have nothing to do with the
firmware. This is the same test as an application would do it -- discover by service UUID, connect, read -- and every
value it prints is asserted.

Run on the host that owns the adapter (lnx):

    python3 ble_pair_check.py

Exit code 0 only when every check passes.
"""

import asyncio
import sys

from bleak import BleakClient, BleakScanner

SERVICO = "e1d0e000-0000-4000-8000-00805f9b34fb"
CHAR_RMS = "e1d0e001-0000-4000-8000-00805f9b34fb"
CHAR_BANDS = "e1d0e002-0000-4000-8000-00805f9b34fb"
CHAR_STATUS = "e1d0e003-0000-4000-8000-00805f9b34fb"

# Calibration constants: the same arithmetic the app does, so a reader can compare the two.
LSB_VOLTS = 1.21 / 4096.0
V_PER_PASCAL = 0.012589  # -38 dBV/Pa

falhas = []


def check(ok: bool, what: str, detail: str = "") -> None:
    print(("PASS " if ok else "FAIL ") + what + (f"  [{detail}]" if detail else ""))
    if not ok:
        falhas.append(what)


async def main() -> int:
    print("== 1. descoberta por UUID de servico (o que qualquer telefone faz)")
    achado = None
    async with BleakScanner(service_uuids=[SERVICO]) as scanner:
        await asyncio.sleep(6.0)
        for dispositivo, anuncio in scanner.discovered_devices_and_advertisement_data.values():
            achado = dispositivo
            print(f"  encontrado: {dispositivo.name} {dispositivo.address}")
            print(f"  UUIDs no anuncio: {anuncio.service_uuids}")
            break
    if achado is None:
        check(False, "descoberta por servico", "nenhum dispositivo anunciou o UUID")
        return 1
    check(True, "descoberta por servico", achado.address)

    print("== 2. ligar (o BlueZ emparelha quando a caracteristica exige)")
    async with BleakClient(achado) as cliente:
        check(cliente.is_connected, "ligacao estabelecida")
        try:
            servicos = cliente.services
            nomes = [s.uuid for s in servicos]
            check(SERVICO in [str(s).lower() for s in nomes], "servico do dosimetro presente", str(len(nomes)) + " servicos")
        except Exception as e:  # noqa: BLE001 - queremos ver a mensagem real
            check(False, "descoberta de servicos", repr(e))
            return 1

        print("== 3. ler as tres caracteristicas (a leitura exige bond)")
        try:
            rms_bytes = await cliente.read_gatt_char(CHAR_RMS)
            bands_bytes = await cliente.read_gatt_char(CHAR_BANDS)
            status_bytes = await cliente.read_gatt_char(CHAR_STATUS)
        except Exception as e:  # noqa: BLE001
            check(False, "leitura das caracteristicas", repr(e))
            return 1

        rms = int.from_bytes(rms_bytes, "little")
        status = bytes(status_bytes)
        check(len(status) == 5, "status tem 5 bytes", f"{len(status)}: {status.hex(' ')}")
        if len(status) >= 5:
            flags, janelas, fonte, inc = status[0], int.from_bytes(status[1:3], "little"), status[3], status[4]
            check(bool(flags & 1), "bit0: calibrado", f"flags={flags}")
            check(fonte == 1, "fonte da calibracao = datasheet", f"fonte={fonte}")
            check(inc == 30, "incerteza declarada", f"±{inc / 10:.1f} dB")
            print(f"  janelas contadas: {janelas}")

        check(rms > 0, "rms > 0 (ha sinal)", f"rms={rms}")
        if rms > 0:
            volts = rms * LSB_VOLTS
            spl = 94.0 + 20.0 * __import__("math").log10(volts / V_PER_PASCAL)
            print(f"  nivel: {spl:.1f} dB SPL (±3.0 dB, calibracao de datasheet)")
            check(20.0 < spl < 130.0, "nivel dentro do plausivel", f"{spl:.1f} dB SPL")

        potencias = [int.from_bytes(bands_bytes[i : i + 4], "little") for i in range(0, 32, 4)]
        print(f"  bandas: {potencias}")
        check(any(p > 0 for p in potencias), "pelo menos uma banda com energia")

    print()
    if falhas:
        print(f"VERDICT: FAIL -- {len(falhas)} verificacao(oes): {', '.join(falhas)}")
        return 1
    print("VERDICT: PASS -- descoberta por UUID, bond, e as tres caracteristicas com valores coerentes")
    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
