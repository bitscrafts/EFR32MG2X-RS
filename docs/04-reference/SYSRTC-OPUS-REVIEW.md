<!-- Filed 2026-11-13T05:18:27Z from an Opus plan-mode analysis run.
     Produced by: Claude Code + claude-opus-5-5, plan mode, READ-ONLY (no project writes).
     Brief: /projects/_tmp/efr32mg24-rs/sysrtc-brief.md
     Repo HEAD at analysis time: ac51757 (tree: 1 changed paths)
     Run: session 6ab46f66-e4dc-45b1-8099-d478c2409da4, stop_reason end_turn, turns 34, cost $2.18

     STATUS: NOT a spec and NOT a plan of record. Nothing in this document is authorised work.
     Every proposal needs its own pre-registered gate before implementation, and any idea previously
     closed by a measurement stays closed until a NEW measurement reopens it.

     Anything marked "asserted-from-memory" is NOT verified — treat it as a lead, not a fact. -->

# SYSRTC stall after START: ranked hypotheses and the checks that decide them

## Context

The Bluetooth stack fails to link on `sl_sleeptimer_restart_timer`, and the plan is to provide it from a HAL
timer over SYSRTC. On the board, SYSRTC reads `EN=1` and `CLKSEL=LFRCO`, and the `CMD=START` write does not
fault. The next SYSRTC access (`SYNCBUSY` or `CNT`) never returns. This is a read-only analysis: hypotheses
ranked by likelihood, each with the check that confirms or rules it out. Sources are the vendored SDK in
`crates/efr32mg24-emlib-sys/vendor/`, the stack blobs in `crates/efr32mg24-bluetooth-sys/blobs/efr32mg24b220/`,
and `crates/efr32mg24-hal/src/sysrtc.rs`.

## Three corrections to the record (from the material itself)

1. **The clock-select reset value is already LFRCO.** `efr32mg24_cmu.h:1040`:
   `_CMU_SYSRTC0CLKCTRL_RESETVALUE 0x00000001` (CLKSEL=1=LFRCO). Writing 1 to 0x240 writes the reset value
   back, so on its own that fix changed nothing. The earlier PRECISERR is better explained by `EN=0`: the EN_SET
   bug was present at the same time, and the brief separately measured that "CMD before EN faults." The
   "stayed at 0" reading only makes sense if something before `main` wrote 0 (for example a bootloader).
   Check: halt at the reset vector and read `0x40008240`.
2. **"CMD=START accepted" is not established.** The Cortex-M33 buffers stores. A marker written after the
   `CMD` store can be published while the store is still waiting at the peripheral. If so, the first later
   SYSRTC access is just where the core runs into it. The data is consistent with this: `SYNCBUSY` reads
   *before* START return (the `wait_sync()` after the clock select and after `EN_SET` in `sysrtc.rs`), and
   the first read *after* START hangs.
3. **The stack needs ten sleeptimer functions, not one.** `nm -u` on the blobs:
   - `bglib.c.obj`: `restart_timer`, `get_clock_accuracy`
   - `timer.c.obj`: `init`, `get_tick_count`, `get_tick_count64`, `ms_to_tick`, `ms32_to_tick`,
     `tick64_to_ms`
   - `bgapi_system.c.obj`: `get_timer_frequency`

   Only `restart_timer` is undefined today because of what the current example pulls in. Larger apps will
   need the rest. Also, the brief cites GSDK v4.4.5, but the vendored HAL is the newer
   `sl_clock_manager` / `sl_hal_sysrtc` generation.

## Ranked hypotheses

**H1 (most likely): the selected LF clock is not reaching SYSRTC, so the first synchronised access waits
forever.** Writes to EN, CFG and GRP0 complete without the LF clock, and so do reads of SYNCBUSY. START (and
everything after it) needs the LF domain to clock. A clock that is selected but not running would show up as
a bus wait, not an error. Unselected (CLKSEL=0) and module disabled (EN=0) are the cases that fault. That
matches every measurement. Checks, cheapest first:
- **a. Select ULFRCO (CLKSEL=3) and change nothing else.** ULFRCO is always on. If `CNT` then advances at
  about 1 kHz, the sequence is correct and the LFRCO path is the fault. If it still hangs, H1 is out and H3
  or H4 move up. This is the single most decisive flash.
- **b. Read `LFRCO->STATUS` (`0x40024008`) just before START.** Bit 0 is RDY, bit 16 is ENS, bit 31 is
  LOCK. If ENS=0, no on-demand request reached the oscillator. This register is the canonical one: on xG24,
  `CMU->STATUS` only has CALRDY, WDOGLOCK and LOCK, with no oscillator-ready bits.
- **c. Read `LFRCO->CTRL` (`+0x04`) and `LFRCO->CFG` (`+0x24`).**
  - If CTRL.DISONDEMAND (bit 1) is set, or CTRL is locked, something before `main` disabled on-demand.
    The reset value is 0.
  - If CFG.HIGHPRECEN (bit 0) is set, LFRCO is in precision mode, which needs HFXO running.
- **d. Force it.** Write `LFRCO->CTRL = FORCEEN (1)`, wait for STATUS.RDY with a bounded loop, then run the
  sequence. If this works, on-demand is the broken path. Keep FORCEEN as a workaround until the cause is
  understood.

**H2: the CMD write is the access that hangs (this sharpens H1).**
- Check: put a `dsb` right after the `CMD` store and the marker after the `dsb`. A missing marker means the
  write itself is stuck.
- Second check: after START, read an unrelated peripheral (for example `CMU->IPVERSION`) instead of SYSRTC.
  If that also hangs, the bus is held by the outstanding SYSRTC write. If it returns, only SYSRTC is
  waiting.

**H3: leftover state from a warm reset.** This is a real defect whether or not it causes the hang.
`sysrtc.rs` "disables" a running module with `write_en_set(0)`, which does nothing. The vendor disables with
`EN_CLR` (`0x2004`) = 1, then waits for `EN.DISABLING` (bit 1) to clear (`sl_hal_sysrtc_init`). If the
debugger's reset does not reset SYSRTC, CFG/GRP0 are written while the module is enabled and possibly
running. Check: the very first instruction in `main`, before any init, records EN, STATUS and SYNCBUSY. If
EN=0 at entry, H3 is out as the cause, but the write should be fixed anyway.

**H4: debug freeze.** `CFG=0` means DEBUGRUN is off, so the counter freezes while the core is halted.
Measuring with probe-rs halts and resets the core. Check: set `CFG=1` once, and separately do one cold run
(power cycle, probe attached afterwards without reset) and read the mailbox. Low probability, cheap.

**H5: no `DSB` after the bus-clock enable.** The vendor puts `__DSB()` after `enable_bus_clock(SYSRTC0)`
(`sl_sleeptimer_hal_sysrtc.c:86`). Low probability, since the earlier register accesses work. Add it anyway.

**Ruled out from the material:** a sequence difference from the vendor. The vendored sleeptimer HAL never
writes `CMD` directly; START must be inside `sl_hal_sysrtc_enable()`, whose header is not vendored, so its
exact body is **unknown**. Its next call is `set_counter()`, which waits on SYNCBUSY first. Either way the
vendor's next access after START is the same SYNCBUSY read that hangs here, so its code would hang the same
way in this state.

## Answers to the seven questions

1. **Why does the read stall instead of returning?** SYSRTC's bus interface hands synchronised writes
   (CMD, CNT) and their SYNCBUSY handshake across to the LF clock domain. With no LF clock edges the handshake
   never finishes, so the bus wait never ends. The likely waiter is the START write still pending in the
   synchroniser (see H2). That this exact mechanism is SYSRTC's internal design is **inferred, not
   documented** in the material; H1a and H2 test it.
2. **Is an oscillator-ready wait needed after CLKSEL?** The vendor does none. `sleeptimer_hal_init_timer`
   has no LFRCO wait, and the vendored emlib only writes `LFRCO->CTRL` inside `CMU_LFRCOSetPrecision`. In the
   vendor's model the oscillator starts on demand, and a synchronised access simply waits for it. If a status
   bit is needed, use `LFRCO->STATUS` (RDY, ENS), not the CMU.
3. **What does device init do for SYSRTC?** For an LFRCO-sourced SYSRTC in default precision, nothing beyond
   setting CLKSEL to LFRCO, which is also the reset value. It does matter if precision mode is enabled
   (needs HFXO), or for LFXO (`sl_device_init_lfxo` waits for LFXO RDY). `CHIP_Init` has no xG24 SYSRTC or
   LFRCO fixups in the vendored `em_chip.h` (it only has Series 2 Config 1 blocks). `sl_device_init_clocks`
   is not vendored, so its exact contents for this board config are **unknown**.
4. **Is SWRST required first?** The vendored HAL does not call it. `SWRST.RESETTING` probably also completes
   in the LF domain, which would explain why the earlier experiment with no clock or EN=0 hung or faulted. It
   is not required. Once H1 is resolved, `SWRST` followed by polling `RESETTING` is itself a test of whether
   the LF clock is alive.
5. **Errata?** **Unknown.** Nothing in the tree covers it. Check the EFR32xG24 errata sheet for SYSRTC and
   LFRCO entries before treating any as relevant.
6. **Is TIMER0 an acceptable substitute?** Yes, and the vendor sanctions it: `sl_sleeptimer_hal_timer.c`
   is an official backend. It needs a 32-bit TIMER (TIMER0 on xG24), prescale 1024, and it registers an
   **EM1 requirement** (lines 34 and 385-390). What changes:
   - **Energy modes:** no EM2/EM3, so the stack's deep-sleep scheduling is unavailable.
   - **Frequency:** about 38 kHz from a 39 MHz HFXO, not 32768 Hz. That is acceptable only because the stack
     asks for the rate through `get_timer_frequency` and converts through `ms_to_tick`, so those must report
     the real rate.
   - **Clock accuracy:** `get_clock_accuracy` should then report the HFXO's ppm.
   - **Reentrancy:** the same as with SYSRTC. The callback is invoked in the timer ISR, and the stack's
     callback just tail-calls `gecko_stack_update_callback`. `restart_timer` can be called from that context,
     so the timer list must be safe for that.

   Acceptable as a stopgap behind the same Rust API. Not acceptable as the destination, given the decision
   that Zigbee and Thread share SYSRTC (and RAIL's EM2 timer sync uses SYSRTC over PRS).
7. **Does the stack need anything beyond the timer driver?** From the disassembly,
   `restart_stack_event_timer(timeout)` calls
   `restart_timer(&sleep_timer_handle, timeout, sleeptimer_timer_callback, NULL, priority 0, flags 0)`.
   `timeout == 0xFFFFFFFF` takes a separate branch; what that branch does is not established. So:
   - **Ticks:** the timeout arrives in ticks the stack computed, so a consistent `get_timer_frequency`,
     `ms_to_tick` and `tick*_to_ms` set is required.
   - **Priority:** `priority` only orders timers that expire on the same tick. It is not an NVIC mapping.
   - **Option flags:** 0, so there is no power-manager interaction.
   - **Handle:** the handle lives in the stack's `.bss`. Treat it as storage the implementation owns, laid
     out per `sl_sleeptimer.h:77`, and do not write past that struct.
   - **Init:** `sl_sleeptimer_init` is imported by `timer.c.obj` and will be called by the stack itself.
   - **Interrupt priority:** atomic sections here are PRIMASK-based
     (`crates/cmsis-stubs/src/cmsis_stubs.c:146`), so they mask everything. If they are ever switched to
     BASEPRI, the timer IRQ priority must be numerically at or above `CORE_ATOMIC_BASE_PRIORITY_LEVEL`
     (`sl_bt_api.h:15600`).

## Recommended next flash session (one variable per flash)

1. Reset-entry snapshot: `CMU 0x240`, `LFRCO CTRL/STATUS/CFG`, `SYSRTC EN/STATUS/SYNCBUSY`. Tests
   corrections 1, H1c and H3.
2. The current sequence with `dsb` after the CMD store, then a CMU read. Tests H2.
3. CLKSEL=ULFRCO. Tests H1a.
4. Only if step 3 counts: LFRCO FORCEEN and wait for RDY, back on LFRCO. Tests H1d.
5. Independent of the result: fix the `EN_CLR` disable path and add the bus-clock `DSB` (H3, H5).

If 3 and 4 both still hang, fall back to the TIMER0 backend (answer 6) behind the same HAL API, and record
SYSRTC as open with these measurements in `docs/04-reference/SYSRTC-DEFECT.md`.
