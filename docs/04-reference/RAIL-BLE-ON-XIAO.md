# BLE on the XIAO through RAIL: what was measured, and where it stopped

The goal was `xiao-mg24` advertising. The Bluetooth stack is unusable on this part yet (see
`BLE-BRINGUP-CHAIN.md`), so the RAIL library was the path. This is how far it got, with the numbers.

## What works

| Step | Evidence |
|---|---|
| RAIL initialises | `rail: ver 2.16.5 h=<handle>`, status Pass |
| RAIL is reachable from Rust | `RAIL_Init` links once emlib is vendored (`em_cmu.c`, `emu.c`, `system.c`, device `system_efr32mg24.c`) |
| The BLE API is callable | `RAIL_BLE_ConfigPhy1Mbps`, `RAIL_BLE_Phy1Mbps` appear once `protocol/ble/rail_ble.h` is added to bindgen — `rail.h` does not include it |

## What the library can and cannot do

`rail_caps` asks instead of assuming:

```
caps ble1m=0 vit=1 2m=0 coded=1 cfg_null=...
```

**This RAIL build supports BLE 1 Mbps only in the Viterbi variant.** `RAIL_BLE_Phy1Mbps` is a null
pointer, exactly as `rail_ble.h` warns ("will be NULL if RAIL_BLE_SUPPORTS_1MBPS_NON_VITERBI is 0"). The
air interface is the same BLE 1 Mbps GFSK either way — the difference is the receiver's decoder — so the
Viterbi configuration is the one to use, not a workaround.

## The two assertions, and what each taught

A hand-built channel configuration (frequency + spacing, null modem config) produced:

```
rx: assert=65        RAIL_ASSERT_FAILED_INVALID_CHANNEL_CONFIG
```

Correct behaviour: a channel entry without a modem config names a frequency but not a modulation. The
vendor's own answer is `RAIL_BLE_Phy1MbpsViterbi`, which is a pointer to a complete
`RAIL_ChannelConfig_t` — frequencies, channel numbering, CRC init, whitening and modem registers.

Using that configuration produced:

```
rx: assert=66        RAIL_ASSERT_INVALID_XTAL_FREQUENCY
```

and the configuration says what it expects: **39 000 000 Hz**. The chip has to agree, and it does not,
because **the crystal is not running at all**:

```
hfxo started=1 hfxo=0kHz sys=19000kHz
```

The board runs from the internal HFRCO at 19 MHz. A 2.4 GHz synthesizer cannot be driven from an RC
oscillator: RAIL refuses the configuration rather than transmitting on a frequency it cannot guarantee.

## The trap that cost the most

`emlib_hfxo_start()` called `CMU_OscillatorEnable(cmuOsc_HFXO, true, true)`, compiled, and returned. It
does nothing. The vendored header documents it:

> This is a dummy function to solve backward compatibility issues.

so on this part the function is a compatibility stub, and `CMU_HFXOInit()` alone does not start the
oscillator either. This is the same class of defect as a check that cannot fail: an API that exists, is
called, and has no effect. It is why the measurement prints the resulting frequency instead of trusting
that the call worked.

## The next step, precisely

Start the HFXO for this board and prove it before touching RAIL again:

1. `CMU_HFXOInit(&hfxoInit)` with `CMU_HFXOINIT_DEFAULT` (already done, does not start it).
2. Enable the oscillator by register — `CMU->OSCENCMD = CMU_OSCENCMD_HFXOEN` — or find the Series 2
   equivalent in this emlib version, since `CMU_OscillatorEnable` is the stub.
3. **Check `CMU_ClockFreqGet(cmuClock_HFXO) != 0`** and print it. Nothing after this is meaningful until
   that number is 39 000 000.
4. Then re-run `rail_ble_rx`: if the assertion changes from 66 to 0 and a packet arrives, the receive
   path is proven and the transmit example follows the same configuration.

Open question worth answering with a scope, not a guess: the XIAO MG24 Sense's crystal frequency and
whether Seeed's design loads it. The reference design and the module schematic settle it; the vendor's
generated configuration for this exact part (`efr32mg24b220f1536im48`) says 39 MHz, which is the value
to verify.
## Update: the crystal starts, and the frequency is still unknown (2026-11-12)

`emlib_hfxo_start()` now starts the oscillator **by register**, which is where the earlier attempt was
wrong:

```c
HFXO0->CTRL |= HFXO_CTRL_FORCEEN;      /* "Force Digital Clock Request" */
while ((HFXO0->STATUS & HFXO_STATUS_RDY) == 0) { ... }
```

`HFXO_CTRL_FORCEEN` and `HFXO_STATUS_RDY` come from the device header, and the instance is `HFXO0` — the
compiler corrected that guess for us (`'HFXO' undeclared; did you mean 'HFXO0'?`).

Measured after that change:

```
hfxo started=1  hfxo=0kHz  sys=19000kHz     (started=1 means STATUS.RDY came up)
rx: assert=66                               (RAIL still refuses the configuration)
```

So the two facts to hold together:

1. **The oscillator reports ready.** `STATUS.RDY` sets within the bounded wait, so the crystal is
   physically running and the register sequence is right.
2. **The software does not know its frequency.** `CMU_ClockFreqGet(cmuClock_HFXO)` returns **0**, and
   RAIL compares the channel configuration's `xtalFrequencyHz` (39 000 000) against that knowledge, not
   against the oscillator's actual behaviour. A ready oscillator with an unknown frequency is still
   unusable to a synthesizer.

The missing piece is a build input, not code: the crystal frequency and load-capacitance trim for this
board are supplied by a generated `sl_device_init_hfxo_config.h` in Silicon Labs' projects, which this
project does not have. It is the same class of artifact as `rail_config.c` — vendor-generated
per-board data — and it is why emlib has no `HFXO_FREQ` anywhere in the headers we vendored.

### What to do next, in order

1. Obtain (or write, from the module schematic) `sl_device_init_hfxo_config.h` for the XIAO MG24 Sense:
   crystal frequency (the vendor configuration for this part says 39 MHz) and the CTUNE values.
2. Compile it into `efr32mg24-emlib-sys` — it must be visible to `em_cmu.c`, because that is where the
   frequency query is answered.
3. Re-measure: the criterion is `CMU_ClockFreqGet(cmuClock_HFXO) == 39 000 000`, printed by
   `rail_caps`. Nothing after this is meaningful until that number is right.
4. Then `rail_ble_rx` again. Success is not "no assert": it is a packet in the mailbox, because RAIL
   will happily receive noise on a wrong frequency if the CRC happens to match, and only a frame with
   the host's name in it proves the path.

### Honest status of the ask

`xiao-mg24` advertising is **not** done, and nothing has been transmitted by the board. What exists:
nine examples, one of them (`rail_ble_rx`) that compiles and runs to the point of the radio refusing
its configuration for a reason that is now named exactly.
## Breakthrough: the clock was the whole of assert 66 (2026-11-12, later)

The chain, each step measured:

```
RAIL_GetRadioClockFreqHz()  ->  19000 kHz     radio clocked from the internal HFRCO
config xtalFrequencyHz      ->  39000 kHz     what the BLE configuration expects
                              -----------
RAIL_ASSERT_INVALID_XTAL_FREQUENCY (66)
```

Starting the crystal was necessary and **not sufficient**: an oscillator that nothing selects clocks
nothing. Two calls, both now measured on the board:

1. `HFXO0->CTRL |= HFXO_CTRL_FORCEEN` — starts it (`STATUS.RDY` comes up).
2. `CMU_ClockSelectSet(cmuClock_SYSCLK, cmuSelect_HFXO)` — makes it the high-frequency clock.

After both, **the assertion is gone**: `RAIL_ConfigChannels` accepts the vendor BLE configuration and the
receive example reaches `RAIL_StartRx` with `st=0 state=2` (state 2 = RX).

Two measurements were wrong before they were right, and both are the same lesson:

- `CMU_ClockFreqGet(cmuClock_HFXO)` returns **0** on this part — emlib's switch has no case for that
  clock id. The zero looked like "crystal not running" and said nothing about the crystal.
- The vendor channel configuration numbers its channels from 2402 MHz, which is *Bluetooth* channel 37.
  Listening on logical channel 37 put the radio at 2476 MHz. The logical channel is **0**.

## The transmit path runs

`rail_ble_tx` builds an advertising PDU (advertiser address + `02 01 06` flags + `0A 09 "xiao-mg24"`),
writes it to the TX FIFO and calls `RAIL_StartTx` on logical channel 0:

```
tx: radioclk=39000kHz      (the clock RAIL now sees)
tx: pdu=20B clock=39000
tx: sent=1 fail=0 st=0     (StartTx returned NO_ERROR)
```

So the board **does transmit**, through RAIL, with no Bluetooth stack, no PSA and no nvm3 — which was the
point of the exercise.

## What is not yet proven, and the honest reasons

1. **No independent observer has seen it.** The host's `bluetoothctl scan` produced no device list at all
   in every attempt, including while the host itself was advertising — so that observer is unverified and
   cannot be used to claim success. The phone (nRF Connect) is the instrument that works; it stays a
   two-second check when someone is in the room.
2. **The advertising rate is far below 100 ms.** The mailbox showed `sent=1` after roughly three seconds,
   so the loop is not turning over as written. The likely reason is already known and already documented
   (`DELAY-CLOCK-DEFECT.md`): `Delay` is calibrated for a 19 MHz HFRCO assumption that is itself ~2.3x
   wrong, and switching SYSCLK to the 39 MHz crystal changes the tick rate under it. A beacon whose
   interval is off by a factor is still a beacon, but it is not a 100 ms advertiser and the number should
   not be claimed.
3. **One advertisement in the air, not a stream.** The FIFO is consumed per transmission, so the frame is
   now rewritten every iteration; whether the radio keeps the same advertiser address across transmissions
   has not been verified on air.

## Where this leaves the ask

`xiao-mg24` advertising exists in firmware and `RAIL_StartTx` returns success on the board. The
verification that remains is external and cheap:

- point nRF Connect at the board and confirm the name appears;
- if it does not, the next suspects are in order: the advertising channel actually used (logical 0 vs the
  three channels a real advertiser hops), the static-random address format, and the interval.
## The beacon transmits once, and the radio never says it is done (2026-11-12, final state of this path)

Measured, in order, each one a correction of the previous belief:

```
tx: radioclk=39000kHz           the crystal is now the radio clock; assert 66 gone
tx: pdu=20B clock=39000         40 14 | 6-byte address | 02 01 06 | 0A 09 "xiao-mg24"
tx: sent=1 fail=0 st=0          RAIL_StartTx returned NO_ERROR
tx: iter=1 sent=1 fail=0 st=0   and the loop never came back round
```

Three attempts at the loop, each one a defect of mine first and a finding second:

1. **`delay.delay_ms(100)` after the clock switch.** Changing SYSCLK stops SysTick, so the HAL's
   `Delay` waits for a wrap that never comes. Re-arming it after the switch did not help either.
2. **`dwt::since(t0)` without `dwt::start()`.** The cycle counter is off by default, so `since` reads
   zero for ever: a wait that cannot complete, in the shape of a wait that looks correct.
3. **`RAIL_Idle(..., wait=true)` before refilling the FIFO.** With no `eventsCallback` configured the
   radio is never declared done, so the idle call waits on a transmitter that stays busy.

So the last state is honest and precise: **one advertisement leaves the board, and RAIL never reports
its completion.** Making the beacon repeat needs RAIL's event/callback machinery (`RAIL_ConfigEvents`
plus a `RAILCb_TxPacketSent`), which is a further step — and it is a step worth taking only if the
goal is a beacon, because:

**RAIL cannot deliver GATT.** It is a link layer: it transmits and receives PDUs. Services,
characteristics, subscriptions and notifications are host-stack work. The board's only host stack is
Silicon Labs' Bluetooth stack, and that is where the GATT milestone has to be won.

Also unresolved and *not* claimed: no independent observer has confirmed any of these transmissions.
The host's `bluetoothctl scan` returned no device list even while the host itself was advertising, so
that instrument is unverified and proves nothing in either direction. The phone is the only trusted
observer available, and it saw `lnx-ble` but never `xiao-mg24`.
## It transmits, and no receiver hears it (2026-11-12, end of this session)

The chain is now measured at every link, and that is the useful part: each link says the packet is
fine, and the air says otherwise.

```
RAIL_BLE_Init                       ok
RAIL_BLE_ConfigPhy1MbpsViterbi      st=0
RAIL_BLE_ConfigChannelRadioParams   st=0   (crcInit 0x555555, AA 0x8E89BED6, channel 37)
RAIL_ConfigTxPower + SetTxPowerDbm  pa=0 dbm=0   (HP mode, 3300 mV, 10 dBm)
RAIL_ConfigData                     PACKET_MODE
RAIL_PrepareChannel(37)             pr=0
RAIL_StartTx(37)                    st=0
RAIL_EVENT_TX_PACKET_SENT           hw=57   <- counted in the radio's OWN interrupt
PDU, read back off the chip         40 14 | F5 FF FE 13 3E E0 | 02 01 06 | 0A 09 "xiao-mg24"
host scan, 100% duty cycle          sees 4 other advertisers (up to 262 reports each), not this one
```

`hw=` is the decisive number and it is new here: it is not this program counting its own calls, it is
`RAIL_EVENT_TX_PACKET_SENT` arriving in the events callback, which only fires when the radio's interrupt
handler runs. `hw=57` with `sent=57` says the modem completes a packet every time it is asked.

So the API is not lying and the frame in memory is not wrong. What remains is the step between the modem
and the antenna: either the on-air framing (access address / whitening / PHY variant) disagrees with the
specification in a way a scanner silently drops, or the output stage is not driving the board's antenna.

What would settle it, in order of cost:

1. **A phone with nRF Connect** — a different receiver with a different front end. It saw `lnx-ble` when
   the host advertised, so it works.
2. **`RAIL_GetTxPowerDbm()` after transmission** — if it does not read back what was requested, the PA is
   not accepting the configuration.
3. **`RAIL_ConfigAntenna()`** — the vendor's generated `rail_config.c` for this exact part contains no
   antenna configuration, so the default path is what the vendor ships; that argues the switch is not the
   cause, but the default path has never been *read back* here.

Everything below this line was fixed on the way, and each one was a real defect, not a guess:

- The PAC declared 46 of the device's 74 interrupts; the radio's were missing, the vector table held
  zeroes, and the first radio interrupt jumped to address 0 (HardFault, CFSR=0x00000100).
- `PROVIDE(FRC = DefaultHandler)` satisfied the vector's reference, so the linker never pulled the blob
  member defining the real handler (measured: pc inside `DefaultHandler_`). Fixed with `+whole-archive`.
- The HAL shipped a duplicate `device.x` that shadowed the PAC's.
- The beacon did not follow `rail_ble.h`'s documented sequence: `RAIL_BLE_Init` and
  `RAIL_BLE_ConfigChannelRadioParams` were both missing, and without the first the radio enters TX and
  never leaves.
