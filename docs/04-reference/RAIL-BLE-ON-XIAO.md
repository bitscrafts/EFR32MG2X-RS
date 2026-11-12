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
