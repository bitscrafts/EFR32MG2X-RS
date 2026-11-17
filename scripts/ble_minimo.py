#!/usr/bin/env python3
"""The smallest possible BLE check: scan everything, print what is there, connect to ours, read.

No filter, no assertions, no retries. Every line it prints is a fact from the adapter. Once this shows the device and
reads a value, the checks go back in — the other way round produced a test that failed for its own reasons.

    sudo /tmp/blevenv/bin/python ble_minimo.py
"""

import asyncio

from bleak import BleakClient, BleakScanner

CHAR_RMS = "e1d0e001-0000-4000-8000-00805f9b34fb"
CHAR_BANDS = "e1d0e002-0000-4000-8000-00805f9b34fb"
CHAR_STATUS = "e1d0e003-0000-4000-8000-00805f9b34fb"


async def main() -> None:
    print("== tudo o que o adaptador ve em 8 s")
    vistos = await BleakScanner.discover(timeout=8.0, return_adv=True)
    for endereco, (dispositivo, anuncio) in vistos.items():
        print(f"  {endereco}  nome={dispositivo.name!r}  rssi={anuncio.rssi}  uuids={anuncio.service_uuids}")

    alvo = None
    for endereco, (dispositivo, _) in vistos.items():
        if dispositivo.name and "xiao" in dispositivo.name.lower():
            alvo = dispositivo
            break

    if alvo is None:
        print("\n== nao apareceu nenhum dispositivo com 'xiao' no nome")
        return

    print(f"\n== a ligar a {alvo.name} {alvo.address}")
    async with BleakClient(alvo) as cliente:
        print("  ligado:", cliente.is_connected)
        for nome, uuid in (("rms", CHAR_RMS), ("bands", CHAR_BANDS), ("status", CHAR_STATUS)):
            try:
                valor = await cliente.read_gatt_char(uuid)
                print(f"  {nome:6s}: {bytes(valor).hex(' ')}")
            except Exception as e:  # noqa: BLE001
                print(f"  {nome:6s}: ERRO {e!r}")


if __name__ == "__main__":
    asyncio.run(main())
