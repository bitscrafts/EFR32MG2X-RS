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
| `i2c` | `src/i2c/` | `em_i2c.c` | `series2/i2c/*` | **read**: aliases only for `IEN_SET` (`em_i2c.c:933`); commands are written plain, as we do |
| `timer` | `src/timer/` | `em_timer.c` | `series2/timer/*` | **read**: `EN_CLR`/`EN_SET` (`em_timer.c:185`, `:201`, `:273`, `:291`); we write `EN` plain and the timer is measured working |
| `letimer` | `src/letimer.rs` | `em_letimer.c` | `series2/letimer/*` | **read**: `EN_SET` then a sync **before** `SWRST_SET` (`em_letimer.c:548`, `:553`) -- the enable-before-sync rule again |
| `pwm` | `src/pwm.rs` | `em_timer.c` | `series2/timer/timer_pwm*` | not started |
| `pcnt` | `src/pcnt.rs` | `em_pcnt.c` | `series2/pcnt/pcnt_single_oversampling_overflow` | **parity measured** |
| `adc` | `src/adc/` | `em_iadc.c` | `series2/iadc/*` | not started |
| `acmp` | `src/acmp/` | `em_acmp.c` | `series2/acmp/*` | **read**: we write `SWRST` plain and wait `RESETTING` (`src/acmp/mod.rs:162`); measured working, so this block accepts it |
| `dma` | `src/dma/` | `em_ldma.c` | `series2/ldma/*` | **read**: `IF_CLR` and `REQDIS_SET` (`em_ldma.c:185`, `:272`); we write `IF` plain, which is the same write-1-to-clear |
| `emu` | `src/emu/` | `em_emu.c` | `series2/emu/*` | not started |
| `cmu` (`clock/`) | `src/clock/` | `em_cmu.c` | — | not started |
| `gpio` | `src/gpio/`, `src/pin.rs` | `em_gpio.c` | `series2/gpio/*` | not started |
| `extint` | `src/extint.rs` | `em_gpio.c` (interrupt part) | `series2/gpio/gpio_int*` | not started |
| `rtc` (`sysrtc`) | `src/rtc/`, `src/sysrtc.rs` | `em_sysrtc.c` | `series2/sysrtc/*` | not started |
| `burtc` | `src/burtc/` | `em_burtc.c` | `series2/burtc/*` | not started |
| `wdog` | `src/wdog/` | `em_wdog.c` | `series2/wdog/*` | not started |
| `aes` | `src/aes/` | `em_aes.c` (Series-1 API) | `series2/crypto/*` | not started |
| `gpcrc` | `src/gpcrc.rs` | `em_gpcrc.c` | `series2/gpcrc/*` | not started |
| `radio` / BLE | `crates/efr32mg24-radio` | RAIL/Bluetooth (vendor blobs) | `series2/rail/*` | not started |

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

## Next

Each row moves independently: fetch the pair (`.c` + example), enumerate its functions, fill the four checks, write the
rows, and measure whatever changes. `i2c`, `adc` and `timer` are the largest surfaces and the most likely to hold the
same class of defect.
