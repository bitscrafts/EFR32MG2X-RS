# Expansion Base: the module is unreachable while it is seated — measured 2026-11-15

## What was observed, twice

The same XIAO MG24 Sense was measured with and without the Seeed XIAO Expansion Base. Both probes were tried each time
(the onboard CMSIS-DAP at `2886:0062:28DB6336` and a SEGGER J-Link PLUS at `1366:0101:000260101260`), and both read paths
(probe-rs 0.32 and OpenOCD 0.12 with `scripts/openocd-mg24-mini.cfg`).

| Module seated in the base | J-Link attached | SWD debug port | Flashing | Application |
| --- | --- | --- | --- | --- |
| no | no | answers, DP IDR readable | write + read-back verify OK, then runs | runs (blink at 1 Hz, IMU, timers, MVP all report) |
| yes | yes | **no answer** — `Error connecting DP: cannot read IDR`, `No target responded to the debug port` | **fails** | does not start (the OLED never initialises) |
| yes | **no** | **no answer**, same two errors | fails | does not start |

The third row is the one that matters for the diagnosis: with nothing attached but the USB cable, a module seated in the
base is already unreachable. The fault needs neither the J-Link nor its wiring.

On one of the seated attempts the module also became **hot to the touch** while powered only from USB, with no J-Link
attached. That is the fact that matters most: current was flowing where it should not.

Two independent hosts reproduce it — this container's host (lnx, 10.10.10.217) and a Mac — so the host, the USB port, the
programmer and the firmware are all out of the picture. The heat is felt from a component **on the base itself**, on the
side opposite the module socket.

The base's own power chain, from its schematic (`Seeeduino XIAO Expansion board v1.0`, 2020-08-24):

```
VBUS 5V -> ETA6003  charger / power path, 4.2 V 500 mA
        -> ETA3410  buck  -> SYS_3V3  (the 3V3 for the system and every device: OLED, RTC, SD, Grove)
        -> ETA1038  boost -> VBOOST_5V (5 V servo/sensor rail, 700 mA)
```

A short on `SYS_3V3` explains every symptom at once: the buck overheats, the rail cannot hold, the OLED never initialises,
the module loses its margin and its debug port goes quiet — while the SAMD11 keeps enumerating because it is fed from USB.
Order of suspicion: ETA3410 (buck), ETA1038 (boost), ETA6003 (charger).

Test, with everything unplugged and the module out of the socket: resistance between `SYS_3V3` and `GND` on the base's
headers, and then `SYS_3V3` under power. Roughly zero ohms, or a rail far below 3.3 V, confirms it.

### The photograph, and the component most likely to be at fault

A close-up of the base's underside shows the three converters the schematic lists: `ETA6003` carries its part number in
silkscreen, while the buck and the boost are two SOT-23-6 packages marked only with lot codes (`CEPi`, `AICI`), each with
its inductor and its ceramic capacitors beside it. Which of the two is which cannot be read off the board; it is
established by measuring the rails, not by squinting at the marks. The SWD pogo pins (`CLK`, `GND`, `RES`) sit on the
opposite side of the area from the heat, so a pogo bridging two pads of the module is not what is happening.

The cheapest single explanation, and the first thing to test, is a **shorted MLCC**: ceramic capacitors fail short rather
than open, and a shorted output capacitor puts the rail at zero while the converter beside it does the heating — which
makes the regulator look like the culprit when the culprit is the capacitor.

Order of work:

1. Confirm which rail is shorted, meter only: `SYS_3V3` to `GND`, then `VBOOST_5V` to `GND`, with the module removed.
2. Unplug anything on the Grove sockets that lives on the shorted rail and measure again. A Grove module in short is
   replaceable and gives the base back with no soldering.
3. To find the capacitor: inject 0.5–1 V at 100–300 mA into the shorted rail from a current-limited supply and apply
   isopropyl alcohol to that rail's capacitors; the one that dries first is the short. Never inject more than 1 V, and
   never with the module seated. Measuring capacitors one by one with a meter does not work — on a shorted rail every one
   of them reads near zero, which is why the injection step exists.

**Conclusion: the base is faulty and needs replacing. The module is not** — it flashed, verified by read-back and ran with
the base removed, and its blink was confirmed by eye. Until a replacement base exists, the PAC and HAL examples that need
the base stay without a hardware date; writing dates over their absence would be the exact kind of claim this repository
keeps out of its documents.

## What this rules out, and what it points at

Ruled out by measurement:

- Not the programmer: the CMSIS-DAP enumerates and reports itself ready (`CMSIS-DAP: Interface Initialised (SWD)`) while
  the DP does not answer. The bridge works; the target does not.
- Not the firmware: the same binaries flash and run with the module out of the base.
- Not the J-Link: the fault reproduces with it unplugged. (Its own wiring is separately suspect — see below.)
- Not a lack of power on the board: the SAMD11's USB enumerates, so 3V3 is present on the module.

Pointed at, in order of likelihood:

1. **Insertion orientation.** A XIAO seated 180° in the base puts the module's `3V3` against the base's `GND`: the rail
   is shorted through the board, which heats it and holds the part in a state where the debug port does not answer. This
   reproduces every symptom at once and is checked in seconds.
2. A fault inside the base: the OLED (SSD1306 at 0x3C), the BME680 (0x76) and the RTC (0x51) share the 3V3 rail and the
   I2C bus of the carrier. A short on that rail, or a pulled-down line, produces the same picture.
3. Damage already done to the module by the earlier 5 V event described below, which only shows up under the base's
   additional load.

## Separate hazard found the same day: J-Link pin 19 is a 5 V output

While wiring a J-Link to the back-side SWD pads, the SEGGER documentation was read rather than assumed:

```
pin 1   VTref    input   reference only
pin 7   SWDIO
pin 9   SWCLK
pin 15  nRESET
pin 19  5V-Target supply   OUTPUT, 5 V, up to 300 mA
```

Wiring pin 19 to the module's 3V3 pad feeds 5 V into a 3.3 V rail. Do not do it. VTref is an input; the module stays
powered from its own USB.

The pads on the underside of the module carry two separate SWD groups, and they are easy to confuse:

```
M_DIO / M_CLK / M_R   the MG24
S_DIO / S_CLK / S_R   the SAMD11 that acts as the onboard programmer
```

Wires on the S_* group do not reach the MG24, and can disturb the programmer that the flashing path depends on.

## The rule this leaves behind

Before trusting any measurement that needs the base, establish that the debug port answers **with the module seated**.
If it does not, nothing on the base can be measured, and the board should not be left powered: the heating is not a
software symptom.

## Status after the diagnosis (2026-11-15)

**The base is faulty and away for replacement; nothing already measured with it is withdrawn.** A refund was applied for
and a replacement is expected in about a month. The measurements taken with the carrier while it still worked — the panel
showing the BME680's temperature, pressure and humidity, the I2C scan finding 0x3C/0x51/0x76, the RTC at 0x51 — stand on
their date, because they were measured and not assumed. What changes is only the wording: the examples that need the
carrier say so in their tables (the HAL crate's README has a "needs the base" column; the PAC crate's `docs/examples.md`
shows `--` only for the ones that were already unmeasured for other reasons), so nobody reads a `--` as a defect in code
that is fine.

What the module still proves on its own, measured with the base removed: it flashes, verifies by read-back, and runs --
the LED blink, the IMU, the microphone, the timers and the MVP co-processor all report from the part itself.
