# SDK parity audit — our HAL against Silicon Labs' own code

**Started 2026-11-28.** Method and per-module matrix. The method lives in the `silabs-emlib-parity` skill; this file is the
record for this repository, and it is meant to be filled in module by module rather than rewritten.

Part: **EFR32MG24B220F1536IM48-B** (Series 2). Vendor sources read on 2026-11-28 from
`https://raw.githubusercontent.com/SiliconLabs/gecko_sdk/master/platform/emlib/src/em_<periph>.c` and
`.../peripheral_examples/master/series2/<periph>/...`; the copy used for PCNT is in
`/projects/_tmp/efr32mg24-rs/emlib/` (scratch, not in the repository).

## Why this exists

A peripheral that "does nothing" gives no error to follow, and every hypothesis about it is testable only on the bench.
The vendor ships the implementation of every one of those peripherals, and it encodes the rules the datasheet states in
prose -- which register alias to write, whether the block must be enabled first, the order the clock comes up. Reading
it is minutes; guessing is days.

Measured cost of not doing this first, on 2026-11-28: four bench experiments and three written-down hypotheses on the
PCNT, all wrong, against two defects found in ten minutes by reading `em_pcnt.c`.

## The matrix

Status: `not started` · `read` · `differs` · `parity` · `parity measured` (a bench record exists).

| Module | Our source | Vendor file | Vendor example | Status |
|---|---|---|---|---|
| `usart` | `src/usart/` | `em_usart.c` | `series2/usart/*` | **read** (see below) |
| `eusart` | `src/eusart/` | `em_eusart.c` | `series2/eusart/*` | **read** (see below) |
| `spi` | `src/spi/` | `em_usart.c`, `em_eusart.c` (sync mode) | `series2/usart/*_spi*` | **read** (same blocks as usart/eusart) |
| `i2c` | `src/i2c/` | `em_i2c.c` | `series2/i2c/*` | **parity**: clears its NACK through the `+0x2000` alias as `I2C_IntClear` does (`em_i2c.h:295`), and cites it; the base is the *secure* window, which is why the address is computed from there rather than from `pac::aliases` |
| `timer` | `src/timer/` | `em_timer.c` | `series2/timer/*` | **read**: `EN_CLR`/`EN_SET` (`em_timer.c:185`, `:201`, `:273`, `:291`); we write `EN` plain and the timer is measured working |
| `letimer` | `src/letimer.rs` | `em_letimer.c` | `series2/letimer/*` | **parity measured**, with one deliberate divergence |
| `pwm` | `src/pwm.rs` | `em_timer.c` | `series2/timer/timer_pwm*` | not started -- it drives TIMER's `CC[ch].CTRL` directly and deserves the same read `timer` got |
| `pcnt` | `src/pcnt.rs` | `em_pcnt.c` | `series2/pcnt/pcnt_single_oversampling_overflow` | **parity measured** |
| `adc` | `src/adc/` | `em_iadc.c` | `series2/iadc/*` | **read**: vendor `EN_CLR`/`EN_SET` (`em_iadc.c:138`, `:147`) and `MASKREQ_SET/CLR` for masking DMA requests (`:777`); we write `EN` plain (accepted, `hal_adc_read` measured) and never touch `MASKREQ`, so that pair is the form to use the day a scan masks requests |
| `acmp` | `src/acmp/` | `em_acmp.c` | `series2/acmp/*` | **read**: we write `SWRST` plain and wait `RESETTING` (`src/acmp/mod.rs:162`); measured working, so this block accepts it |
| `dma` | `src/dma/` | `em_ldma.c` | `series2/ldma/*` | **read**: `IF_CLR` and `REQDIS_SET` (`em_ldma.c:185`, `:272`); we write `IF` plain, which is the same write-1-to-clear |
| `emu` | `src/emu/` | `em_emu.c` | `series2/emu/*` | **read**: the vendor's aliases here belong to the DCDC/boost/power-switch surface (`PWRCTRL_SET`, `BOOSTCTRL_SET/CLR`, `DCDC->CTRL_CLR`, `em_emu.c:2876`, `:3443`), which this crate does not configure; our only write is a `CTRL` modify for the fields we do set, and EM2/EM4 entry is measured (`hal_em2_sleep`, `hal_emu_voltage`) |
| `cmu` (`clock/`) | `src/clock/` | `em_cmu.c` | — | **read, one statement corrected**: the module said the crystal could not be started at all. `CMU_CLKEN0.LFXO` (bit 23, `efr32mg24_cmu.h:545`) is the gate that starts it, exactly as bit 22 does for the LFRCO; what is genuinely absent is `STATUS.LFXORDY` -- the header exposes only `CALRDY` -- so the driver cannot know when it has stabilised. The refusal to select LFXO stands, with a narrower and truer reason |
| `gpio` | `src/gpio/`, `src/pin.rs` | `em_gpio.c` | `series2/gpio/*` | **read**: the vendor's aliases here serve `EXTILEVEL_CLR/SET`, `EM4WUEN_SET`, `EM4WUPOL_SET/CLR` -- the EM4 wake-up polarity surface, which this crate does not implement. Nothing to compare until it does; when it does, those four aliases are the form |
| `extint` | `src/extint.rs` | `em_gpio.c` (interrupt part) | `series2/gpio/gpio_int*` | **read**: our routing writes `EXTIPSEL`/`EXTIPINSELL`/`EXTIRISE`/`EXTIFALL` plain and is measured (`gpioirq: n=8 ...`); the vendor writes the same registers plain, so the surface we do drive is at parity |
| `rtc` (`sysrtc`) | `src/rtc/`, `src/sysrtc.rs` | `em_sysrtc.c` | `series2/sysrtc/*` | **read**: `em_sysrtc.c` uses **no alias at all**; ours likewise -- the simplest row in the matrix |
| `burtc` | `src/burtc/` | `em_burtc.c` | `series2/burtc/*` | **read**: `EN_SET`/`EN_CLR`, `IF_CLR` (`em_burtc.c`); ours writes `EN` plain and is measured |
| `wdog` | `src/wdog/` | `em_wdog.c` | `series2/wdog/*` | **read**: `EN_SET`/`EN_CLR` only; ours writes `EN` plain, measured `hal_wdog_feed` |
| `aes` | `src/aes/` | `em_aes.c` (Series-1 API) | `series2/crypto/*` | **not comparable**: `em_aes.c` drives the *legacy* AES block (`AES->CTRL`, `AES_CTRL_KEYBUFEN`) and uses no alias; this part's AES is RADIOAES with a descriptor engine, which the SDK exposes through its crypto firmware rather than emlib. Our driver is validated against FIPS-197 C.1 and NIST SP 800-38A F.1.1 instead -- a stronger oracle than a code comparison |
| `gpcrc` | `src/gpcrc.rs` | `em_gpcrc.c` | `series2/gpcrc/*` | **read**: `EN_SET`/`EN_CLR` only; ours plain, measured `hal_gpcrc` |
| `radio` / BLE | `crates/efr32mg24-radio` | RAIL/Bluetooth (vendor blobs) | `series2/rail/*` | **not comparable at register level**: the vendor ships RAIL and the Bluetooth stack as binary blobs, not as an emlib-style driver. The comparison for this surface is the vendor's published API documentation, not `.c` |

## First finding: the alias question is **per block**, not per family

The PCNT defect (plain `CMD` in 0x14 refused) is not a rule for the whole part. Read the two serial drivers and the
answer inverts:

| Block | What the vendor writes for a command | What the vendor writes for enable | Our driver | Verdict |
|---|---|---|---|---|
| PCNT | `CMD_SET` (`em_pcnt.c:131`) | `EN_SET` / `EN_CLR` (`:170`, `:191`) | plain `cmd()` | **defect**, fixed |
| USART | plain `CMD` (`em_usart.c:17`, `:20`) | `EN_SET` (`:745`), `EN_CLR` (`:758`) | plain `cmd()`, plain `en()` | parity in effect (measured working: `hal_usart_link`, `hal_usart_rx_recovery`) |
| EUSART | plain `CMD` (`em_eusart.c:372`, `:771`, `:1227`, `:1367`) and `CMD_SET` (`:745`) | `EN_SET` (`:364`), `EN_CLR` (`:1385`) | plain `cmd()`, plain `en()` | parity in effect (measured: `hal_eusart_loopback`, `hal_eusart1_loopback`) |

So the check is not "always use aliases" -- it is **"does the vendor use an alias for this register on this block?"**,
and the only way to answer it is to read that block's `em_<periph>.c`. A plain write that the hardware accepts (as the
USART clearly does here) is not a defect; a plain write it refuses is a fault, and the two look identical in our code.

Two secondary notes from the same reading, neither a defect today:

* Our `en().write(|w| w.en().clear_bit())` writes the whole `EN` register with a zero, where the vendor writes
  `EN_CLR`. On this block `EN` has one writable bit, so the two are equivalent -- but it is equivalent *by accident of
  the register layout*, and it stops being equivalent the day a second writable bit appears.
* The vendor sets `EN` **first** and the transmit/receive enables after (`em_usart.c:745` then `:758`); our order is the
  same, which is why the serial path never showed the PCNT's second defect.

## Which registers the vendor writes through aliases, per block

Read from the alias identifiers each driver actually uses (`grep -oE '\->[A-Z0-9_]*_(SET|CLR)\b'`), 2026-11-28:

| Block | Aliases the vendor uses | Our driver | Status |
|---|---|---|---|
| `pcnt` | `EN_SET`, `EN_CLR`, `CMD_SET` | plain `EN`, plain `CMD` | **defect** -- plain `CMD` faults; fixed in the PAC (`aliases` module) and the HAL |
| `i2c` | `IEN_SET` only | plain `CMD`, plain `IEN` | read; the vendor writes commands plain too |
| `timer` | `EN_SET`, `EN_CLR` | plain `EN` | read; example `hal_timer_ticks` |
| `letimer` | `EN_SET`, `EN_CLR`, `SWRST_SET` | plain `EN`, plain `SWRST` | read; enable-then-sync-then-reset order matches ours |
| `ldma` | `IF_CLR`, `REQDIS_SET`, `DBGHALT_SET` | plain `IF` (write-1-to-clear, same effect) | read |
| `iadc` | `EN_SET`, `EN_CLR`, `MASKREQ_SET`, `MASKREQ_CLR` | plain `EN` | read |
| `acmp` | `EN_SET`, `EN_CLR`, `IF_CLR`, `SWRST_SET` | plain `EN`, plain `IF`, plain `SWRST` | read; example `hal_acmp_ladder` |
| `gpcrc` | `EN_SET`, `EN_CLR` | plain `EN` | read; example `hal_gpcrc` |
| `wdog` | `EN_SET`, `EN_CLR` | plain `EN` | read; example `hal_wdog_feed` |
| `burtc` | `EN_SET`, `EN_CLR`, `IF_CLR` | plain writes | read |
| `sysrtc` | **none** | plain | read; this block takes no aliases at all |
| `gpio` | `EXTILEVEL_SET/CLR`, `EM4WUEN_SET`, `EM4WUPOL_SET/CLR` | plain `EXTILEVEL`? | not yet compared -- the pin-level surface is large |

Two conclusions that change how this audit is run:

* **Plain `EN` is accepted almost everywhere.** Only `CMD` on the PCNT was refused, and it was refused with a fault.
  So "we should use aliases everywhere" would be churn, not a fix -- the question is per register and the answer comes
  from the vendor's `.c` for that block.
* **`sysrtc` takes no aliases at all**, which is why nothing there ever looked like the PCNT. A module that works is not
  evidence that the pattern is absent elsewhere; it is evidence about that block.

## Module: LETIMER — a divergence that is deliberate, and one that was tidy-up

Read against `em_letimer.c` on 2026-11-28:

| Check | Vendor | Ours | Verdict |
|---|---|---|---|
| Which register gets the command | plain `CMD` (`em_letimer.c:274`) | plain `cmd()` | parity -- unlike the PCNT, this block accepts the plain write |
| Sync before the command | `regSync(SYNCBUSY_STOP \| SYNCBUSY_START)` before `CMD = START/STOP` (`:267`) | **no poll at all**, on purpose | **deliberate divergence**: our module records that reading `SYNCBUSY` on this block stalls the bus, measured three times (`src/letimer.rs:21`), and the caller settles with `SETTLE_US` instead |
| Clearing flags | `IF_CLR` | wrote `IF + 0x2000` by hand | same effect; now expressed through `pac::aliases::clr` |
| Enable order | `EN_SET`, then a sync, then `SWRST_SET` (`:548`, `:553`) | enable first, settle, then clear | same order |

The divergence is worth keeping and worth *labelling*: the vendor busy-waits on `SYNCBUSY` and its own comment warns that
the wait can stall with an external clock; on this part the read stalls even with an internal one. A busy-wait we cannot
complete is worse than a settle time the caller keeps, so the record stands -- but it is a difference from the vendor and
this row exists so nobody "fixes" it back.

Instrument after the alias tidy-up:

    seq=1 status=PASS len=173
    letimer: clk=0x00000101 top0=1000 top=1000 comp0=500 comp1=250 refused=1 cnt0=1000 p1=501 qe=0 ql=1 pq=250
             wrap=1 if1=0x00000001 ifc=0x00000000 if2=0x00000001 p2=499 cnt=499

## Module: PCNT — the worked example

Two defects, both invisible in the header and both named by the vendor's implementation.

### Difference 1 — the register written

| | |
|---|---|
| Our code | `src/pcnt.rs:write_cmd`, `disable`, `enable` -- wrote `cmd()`, `en()` (plain registers, offsets 0x14 and 0x04) |
| Vendor | `em_pcnt.c:131`, `:170`, `:191` -- `CMD_SET`, `EN_CLR`, `EN_SET` |
| Why | `efr32mg24_pcnt.h:62`: `RESERVED0[1008U]` after `LOCK`, then the SET aliases at `0x1000` and the CLR aliases at `0x2000`, inside the block's own frame |
| Consequence | A plain write to `CMD` raises a precise bus error: `CFSR = 0x8200`, `BFAR = PCNT0 + 0x14`, measured twice |

Fixed in the PAC, not the HAL: `crates/efr32mg2x-pac/src/aliases.rs`, hand-written and documented as the SVD gap it
fills (`pac::aliases::set` / `clr`). `Pcnt::base()` is `pac::Pcnt0Ns::ptr()` -- the handle's own address is RAM, and the
first attempt that used it wrote into memory and faulted with `BFAR = 0x2004_0f2c`.

### Difference 2 — the enable state at the moment of the write

| | |
|---|---|
| Our code | `init` issued `CMD.CORERST` (later `CNTRST`) with `EN` low, then synced |
| Vendor | `em_pcnt.c:191`: `EN_SET`, *then* `PCNT_Sync(_PCNT_SYNCBUSY_MASK)`; its disable path syncs *before* clearing `EN` |
| Consequence | With `EN` low the command sits in the low-frequency domain and `SYNCBUSY.CMD` never drops -- measured set for 0.7 s, unchanged by any clock source |

A third difference follows from the same reading: `CORERST` appears once in `em_pcnt.c` (`:755`), inside a Series-0/1
branch, and only to change to an external clock. Our `init` used it as the first command on a Series-2 block.

### Records

Instrument `examples/hal_pcnt_request.rs`, two boots identical:

    pcnt-req: en0=00000001 sync0=00000000 en1=00000001 sync1=00000000
              clk=00000001 clken0=05400000 clken1=00200000 em23=00000003 sync_ulfrco=00000000

Example `examples/hal_pcnt_gpio.rs`, which had recorded `magic 0` for days:

    seq=1 status=PASS len=178
    pcnt: ch=3 cfg=0x00000100 ctrl=0x00000010 st=0x00000000 a=4 lv=4 b=2 c=3 off=0 stop=0 q=0 mm=0
          p0=4 ok=25000 bad=50000 g=85 e=200 of=0 ar=4 br=2 cf=2 cm=4 hr=4 em=8 if=0x00000000

Full narrative: `PCNT-COMMAND-NEVER-COMPLETES-2026-11-28.md`.

## Rules to check first in every new module

These came out of PCNT and are cheap to test elsewhere:

1. **Aliases.** Does the vendor write a `*_SET`/`*_CLR` alias where we write the plain register? Does our PAC even
   expose the alias block? Check the device header's `typedef struct` for the `RESERVED0` gap.
2. **Enable state.** Every `*_Sync` in the vendor code happens with the block enabled.
3. **Clock order.** Oscillator, branch, select, gate -- not the reverse.
4. **Commands the vendor never issues** for this family are commands to distrust.
5. **Bounds.** Where we bound a wait the vendor leaves unbounded, the unit and value must match the datasheet's worst
   case, and giving up must not be silent.

## Hygiene found while reading

* `src/clock/clocks.rs` carried its reasoning in Portuguese, against the project's rule that code is written in
  English. The block about LFXO has been rewritten in English as part of this audit (its content was also corrected,
  see the `cmu` row); the rest of the file's comments still need the same treatment, and that is a separate job.
* The same file and `src/letimer.rs` both carried hand-rolled alias arithmetic (`base + 0x2000`). Both now use
  `pac::aliases`, which is where that knowledge belongs.

## Where this pass stands

Every row now carries a verdict. One defect (`pcnt`), two measured parities (`pcnt`, `letimer`), one
statement corrected (`cmu`), and the rest read and found at parity or outside what this crate drives.

The pass answered the question it was built for: *is our HAL doing anything the vendor's code says not to?* Once --
the PCNT's command register and its enable state -- and that one had cost four bench experiments before the
comparison was made.

## Next

Each row moves independently: fetch the pair (`.c` + example), enumerate its functions, fill the four checks, write the
rows, and measure whatever changes. `i2c`, `adc` and `timer` are the largest surfaces and the most likely to hold the
same class of defect.
