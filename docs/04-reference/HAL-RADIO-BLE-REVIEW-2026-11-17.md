<!--
Provenance: independent read-only review by claude-fable-5-2 (Claude Code, plan mode) of the HAL, radio and
Bluetooth crates at 101114f, on 2026-11-17. Nothing here is authorised work: each item needed its own gate
before being touched, and the state of each is recorded below.

The report was produced in _tmp/review-hal-fable/ and moved here on 2026-11-18, because a review that is the
evidence for thirty-odd defects cannot live in scratch space -- the same lesson as B18, whose generator was
one cleanup away from being lost.
-->

# HAL, radio and Bluetooth review at `101114f` (2026-11-17)

**State of this document: the defect list is as written; the status table below is added.** The body is
preserved unedited so the evidence can be re-read against the code, including where the review was wrong.

## What has since been fixed (2026-11-18)

| # | Commit | What was actually wrong |
|---|---|---|
| H25 | `d4245b2` | a deadline already gone never fired until the counter wrapped (36.4 h) |
| B8 | `9f5fb46` | the same, in the Bluetooth stack's timer backend; both now share one tested function |
| H12 | `ad9e09c` | the MVP accepted 65536 elements against 10-bit fields |
| H30 | `ad9e09c` | `gpio_static` used the non-secure base for the secure register type |
| H31 | `ad9e09c` | the HAL linker script still claimed 1512K, covering the NVM3 pages |
| H27 | `a638e5a` | **the review's premise was wrong** (`WDOG_LOCK` resets unlocked), but it was hiding a real defect: the watchdog was never re-locked after configuration |
| B3 | `17c8128` | a local `PSA_ALG_CMAC` shadowed the specification's value, so the GATT hash could never be computed |
| B4 | `17c8128` | the key id was read as "the Nth used slot", so a MAC could use the wrong key and report success |
| B5 | `3964dc8` | two timer functions returned the value where the contract returns a status and writes through a pointer |
| B10 | `3964dc8` | milliseconds became ticks by ×32 instead of ×32768/1000, 2.4 % short |
| B14 | `3964dc8` | a failed GATT session was abandoned without aborting it |
| B7 | `5d798ba` | the 64-bit tick was the 32-bit counter widened; it now counts the wraps |
| B9 | `d357c68` | the timer callback was called with one argument instead of `(handle, data)` |
| B12 | `a7c0ccb` | **partial**: a MAC over a truncated message is refused instead of reported as good |
| B18 | `7db09c6`, `e215335` | the generator of the BLE config table lived in `_tmp`; it is in `scripts/` and the gate now regenerates and compares |

## What is still open, and why

The live list with the reason per item is `docs/05-operations/NIGHT-QUEUE-2026-11-18.md`. In short: H17 needs an
acknowledging device on the bit-bang bus, H21/H22 is a six-driver refactor in one piece, H28 and the rest of B12
need the board to measure the result, and B16 (moving the link layer off PendSV) is the highest-risk change in
the review and has no safe window yet.

---

Path shorthand: `hal/` = `crates/efr32mg2x-hal/src/`, `bt/` = `crates/efr32mg24-bluetooth-sys/src/`, `vendor/` = `crates/efr32mg24-emlib-sys/vendor445/`.

Evidence grades:
- **C** = I read both the code and the vendor line.
- **D** = disassembly transcribed by a sub-agent; I confirmed the Rust side, and the header side where one exists.
- **P** = plausible, needs the measurement named.

## 1. HAL defect table (ranked by severity)

| # | Where | What is wrong | Evidence | Severity | Smallest fix |
|---|---|---|---|---|---|
| H1 | `hal/dma/mod.rs:289` | The "circular descriptor" address is `LDMA base + 0x50`, commented as `CH0_CTRL`. `0x50` is `LDMA->IF`; `CH[0].CTRL` is at `0x64`. The link reloads the descriptor from IF/IEN/reserved/CFG. | `vendor/device/efr32mg24_ldma.h` struct order: 22 words before `CH[8]`, then `RESERVED0, CFG, LOOP, CTRL` (C) | silent corruption | Link to a real `#[repr(C, align(4))]` descriptor in RAM; never to the register block. |
| H2 | `hal/dma/mod.rs:241-335` | A safe function hands `&mut [T]` to the DMA and returns; the borrow ends while the hardware keeps writing. `peripheral_addr: u32` is an arbitrary read address from safe code. | The project's own rule: no `unsafe` leaking through the public API (C) | silent corruption | Take `&'static mut [T]` or return a guard that holds the borrow and stops the channel on drop. |
| H3 | 9 sites: `hal/dma/mod.rs:324,415,500,528`, `hal/timer/mod.rs:413`, `hal/i2c/mod.rs:261,402`, `hal/wdog/mod.rs:297`, `hal/rtc/mod.rs:246`, `hal/sysrtc.rs:214-216` | Flags are "cleared" by writing 1 to `IF`. On this family a flag is cleared through `IF_CLR` (base + 0x2000). The project already knows this: `hal/sysrtc.rs:299-304` does it correctly. | `vendor/inc/em_ldma.h:2718-2721`, `em_i2c.h:294-297`; `*_HAS_SET_CLEAR` in every device header (C) | wrong result / spurious events | One `fn clear_flags(base, off, mask)` writing `+0x2000`; replace all nine. |
| H4 | `hal/usart/mod.rs:85` | Enables `CLKEN0` bit 1, commented "USART0 clock enable". Bit 1 is `LDMAXBAR`; USART0 is bit 9. The next line touches an ungated peripheral. | `vendor/device/efr32mg24_cmu.h:435,475` (C) | bus fault on construction | `w.usart0().set_bit()`, as `hal/spi/mod.rs:96` does. |
| H5 | `hal/usart/mod.rs:120-123` | `256 * pclk` overflows `u32` at 19 MHz (4.86e9). The result is then written into the `DIV` field, which sits at bit 3, so it is also shifted left by 3. | PAC `usart0_s/clkdiv.rs` `DivW::new(self, 3)`; `_USART_CLKDIV_DIV_MASK 0x7FFFF8` (C) | wrong baud; panic in debug | Compute in `u64`, write with `w.bits()`. |
| H6 | `hal/timer/mod.rs:90,105,113,123` | `EN=1` then `CFG` is written. The project measured that this faults (`hal/board.rs:244-246`). | `vendor/inc/em_timer.h:878-879` waits `DISABLING` before config (C) | bus fault on construction | Disable, wait `EN.DISABLING`, write `CFG`, enable, then `TOP`. `hal/pwm.rs:82-92` is the working order. |
| H7 | `hal/spi/mod.rs:238-263, 368-393` | EUSART: `EN` first, then `CFG0`/`CFG2`; the bit rate goes to `CLKDIV`, but the SPI divider is `CFG2.SDIV`. | `_EUSART_CFG2_SDIV_MASK 0xFF000000`, `vendor/device/efr32mg24_eusart.h:566` (C); the fault on a CFG write while enabled is by analogy with TIMER/IADC (P) | fault or wrong clock | Configure disabled, set `SDIV`, enable, wait sync. |
| H8 | `hal/spi/mod.rs:103-124` | USART SPI never issues `CMD.MASTEREN`; `transfer_byte` (`:209`) then waits for `RXDATAV` with no timeout. | `USART_CMD_MASTEREN`, `vendor/device/efr32mg24_usart.h:531` (C) | hang | Issue `MASTEREN`; bound every wait. |
| H9 | `hal/spi/traits.rs:68,102` | `flush` loops while `TXFL` is set. `TXFL` is set whenever the FIFO has room, i.e. when idle. | `EUSART_STATUS_TXFL` vs `TXIDLE`/`TXC`, `efr32mg24_eusart.h:886-911` (C) | hang | Wait for `TXIDLE`. |
| H10 | `hal/board.rs:148-200` | The band change omits four vendor steps: the PCLK prescaler (limit 50 MHz), the radio-clock prescaler `RHCLKPRESC` (limit 40 MHz), the clamp detection, and restoring the MSC lock. Wait states stay at WS3 at every band. | `vendor/src/em_cmu.c:98,101`, `:2489` onward (`pclkDivMax`, `rhclkPrescMax`, range compare), `:4882` `mscLocked` (C) | out-of-spec PCLK at 56/80 MHz; radio clock out of spec if BLE runs boosted | Mirror `pclkDivOptimize`/`rhclkPrescOptimize`; compare `FREQRANGE` after the write; re-lock. |
| H11 | `hal/board.rs:98-101` | The doc says VSCALE is "asserted here rather than assumed". No code checks it. | Body of `apply_hfrco_cal` (C) | misleading | Read `EMU.STATUS.VSCALE` and return `None` unless it is Vscale2. |
| H12 | `hal/mvp/mod.rs:96` | Accepts `n <= 65_536`; `SIZE` and `NUMITERS` are 10-bit fields. Above 1024 the length spills into neighbouring fields. | `_MVP_ARRAYDIM0CFG_SIZE_MASK 0x3FF`, `_MVP_LOOPCFG_NUMITERS_MASK 0x3FF` (C) | silent corruption | Reject `n > 1024`, or chunk. |
| H13 | `hal/adc/mod.rs:210-219`, `adc/types.rs:35` | Every channel is hard-wired to Port A, the analog bus is never allocated, and `Gnd = 15` selects PA15. | `hal/board.rs:354-362` records that an unallocated bus reads flat zero (C) | wrong number | Channel = (port, pin); write `ABUSALLOC`/`CDBUSALLOC`; ground = `PORTPOS_GND`. |
| H14 | `hal/microphone.rs:377-403` | `level()` measures deviation from a fixed 2048, not from the window mean. The recorded `rms=593 pk=615` (`:360`) is almost pure DC offset. | Peak ≈ RMS only for a constant (C on the arithmetic, P on the cause) | wrong number | Accumulate sum and sum of squares; report variance about the mean. |
| H15 | `hal/microphone.rs:320-327` | The comment says the budget is "two sample periods"; `BUDGET = 2_500` counts loop iterations, roughly ten periods, and scales with the core clock. | Code vs comment (C) | misleading; slow failure | Budget in DWT cycles. |
| H16 | `hal/microphone.rs:341-353` | FIFO overflow is never read, so a dropped sample is invisible; the returned count stays full. | No `SINGLEFIFOOF` read in the file (C) | silent corruption of the signal | Check and clear the overflow flag per window. |
| H17 | `hal/pin.rs:140-166`, `hal/bus/bitbang.rs:237-250` | `DOUT` is read-modify-written across the whole port. Any interrupt that touches the same port in between is lost. | PAC has no `DOUT_SET/CLR` aliases (grep: only `radioaes` has `if_clr`) (C) | lost pin update | Write the `+0x1000`/`+0x2000` aliases; also halves the bit-bang cost. |
| H18 | `hal/bus/bitbang.rs:257-262, 357` | `start()` emits low/low, SCL high, SDA rising (a STOP), then SDA falling. So the "repeated START, no STOP in between" is a STOP followed by a START one half-bit later. | Trace of the four `set()` calls (C) | wrong protocol; comment false | Release SDA while SCL is low, raise SCL, then drop SDA. |
| H19 | `hal/bus/bitbang.rs:161, 233` | Timing is a cycle count. After the BLE stack starts the core runs at 39 MHz (section 6), at 80 MHz after a boost. Released lines are never read back, so a stuck or unpowered bus reads as ACK. | `HALF_BIT_CYCLES = 60`; no `DIN` check after release (C); link to the recorded false ACK (P) | wrong result | Derive the delay from the live clock; return `BusError::Bus` if SDA or SCL stays low when released. |
| H20 | `hal/i2c/mod.rs` (all waits), `:602-614`, `traits.rs:855-872` | No wait has a timeout. `write_read` and `transaction` emit STOP between operations, which breaks the `embedded-hal` contract. The clock source is taken as `hfclk`. | `:151,166,190,200,217,250,256` (C); I2C0 being on the half-rate clock is from memory (P) | hang | Bound the waits; one START…STOP per transaction. |
| H21 | `hal/clock/clocks.rs:78-128` | `Clocks::new` configures nothing and returns whatever frequency the caller states. Every divider in usart/spi/i2c/timer trusts it. | Its own comment at `:84-96` (C) | wrong number everywhere | Read `SYSCLKCTRL.CLKSEL` and the band; or delete the type. |
| H22 | `hal/board.rs:286-289`, `pwm.rs:121`, `board.rs:259` | `19_000_000` is a constant in three places. It is false after `boost_*` and after `Bluetooth::new`. | `bt/stack_init.rs:327-328` (C) | wrong delays and tones | One runtime clock value owned by `Board`. |
| H23 | `hal/board.rs:53-66, 83-85, 398-401` | `emu()`, `wdog()`, `mvp()`, `adc()` steal on every call; the comments say "exactly once". `Converter` is `Copy` (`microphone.rs:37`). | Signatures take `&self` (C) | aliasing | `Option::take` on fields of `Board`. |
| H24 | `hal/sysrtc.rs:104-106` | `unsafe impl Send` is justified by "a second value is prevented"; `Board::sysrtc(&self)` (`board.rs:72`) returns a new one each call. | (C) | claim not earned | Hand it out once, or drop the impl. |
| H25 | `hal/sysrtc.rs:282-286` | A deadline already in the past never fires until the counter wraps (36.4 h). | No re-check after programming the compare (C) | hang of the waiter | Re-read the counter after arming; fire immediately if passed. |
| H26 | `hal/board.rs:373-378, 414-419`; `sysrtc.rs:242-250` | Oscillator and sync timeouts `break` and carry on. | (C) | swallowed error | Return the error. |
| H27 | `hal/wdog/mod.rs:126` | `LOCK` is written while the module is disabled, before `CFG`. By the project's own TIMER finding, a non-config register written disabled is the faulting kind. | `board.rs:256-258` (P) | bus fault | Read BFAR: the `LOCK` offset or the `CFG` offset in `efr32mg24_wdog.h` decides it. |
| H28 | `hal/emu/mod.rs:175-224, 246-287` | EM2 is entered with a bare `wfi` and no pending-event check; EM01 voltage can be lowered at any clock. | Vendor `EMU_VScaleEM01ByClock`, called from `em_cmu.c` band set (C) | missed wake; undervolt | `sleep_if(cond)` with interrupts masked; refuse Vscale1 above 40 MHz. |
| H29 | `hal/rtc/mod.rs:102-126` | A second SYSRTC driver with none of the fixes in `sysrtc.rs` (no clock select, no sync waits). | `sysrtc.rs:12-20` describes exactly this failure (C) | bus fault | Delete it; re-export `sysrtc`. |
| H30 | `hal/board.rs:452` | `gpio_static` casts `0x5003_C000` to the `gpio_s` type; that address is `GPIO_NS_BASE`. | `vendor/device/efr32mg24b220f1536im48.h:537,588` (C) | cosmetic today | `pac::GpioS::ptr()`. |
| H31 | `crates/efr32mg2x-hal/memory.x` | `FLASH LENGTH = 1512K` still covers the NVM3 pages; the BSP's file was corrected to 1472K. | `crates/xiao-mg24-bsp/memory.x:8-12` (C) | latent | Same length in both. |

**Pattern:** H4 to H9, H1, H21 and H29 all sit in modules that no example in the workspace uses. A grep for `timer::Timer`, `usart::`, `spi::Spi`, `dma::`, `rtc::Rtc`, `i2c::I2c[01]` and `Clocks::new` across both example directories returns nothing. Every driver without an example has a defect on its first register access.

## 2. HAL missing work

| Missing | Cost for this product |
|---|---|
| GPIO interrupts (`EXTI`) | Button and IMU interrupt lines must be polled; no wake from EM2 on motion. **Confirmed from the vendor schematic (2026-11-21)**: `XIAO_MGM240S_KICAD_Prj.pdf` sheet 06 marks the IMU's INT1 (pin 4) and INT2 (pin 9) as no-connects, so there is no line to poll *from* and no wake to wire -- the button lives on the expansion carrier. Wake sources this board has: the SYSRTC comparator and the radio. |
| PRS routing | No hardware trigger path between timers, the IADC and the radio; needed for 802.15.4 timing. |
| LDMA from the IADC FIFO | The capture loop busy-waits about 99 % of each sample period (`board.rs:116`). |
| EM2/EM3 entry with oscillator restore, EM4 | Types exist (`emu/types.rs`), entry does not. Battery life. |
| Hardware I2C | Everything is bit-banged, blocking, clock-dependent. |
| USART/EUSART pin routing, async EUSART, LEUART-style low-energy UART | No serial console at all. |
| LETIMER, BURTC, PCNT, ACMP, VDAC, LESENSE, KEYSCAN | None needed for the dosimeter today; ACMP/LESENSE would give wake-on-sound. |
| Flash (MSC) driver in the HAL | Flash is only reachable through `emlib-sys` C. |
| `embedded-hal` `SetDutyCycle` | `timer/traits.rs:8-9` says the trait is not stable; it has been since 1.0. |
| MVP beyond `add_fp16` (MAC, dot product) | The band arithmetic stays in software. |
| WDOG warning interrupt, window mode | `warning_only()` configures it; nothing handles it. |
| PAC corrections | `SYSRTC0CLKCTRL` accessor points at `0x1DC` (`sysrtc.rs:12`); no SET/CLR aliases. |
| Host-side tests | Two BCD tests in `rtc/pcf8563.rs`; nothing for divider maths, which is where H5 lives. |

## 3. HAL optimisation

| Path | Cost now | Alternative | Gain | Proof |
|---|---|---|---|---|
| `capture_window` | 1177 of 1187 cycles per sample idle (`board.rs:116`) | LDMA drains the IADC FIFO into a buffer | CPU free for features or sleep | Same window, `dwt::since` around the work only. |
| Flash wait states after a boost | WS3 at all bands (`board.rs:162`) | WS0 up to 40 MHz, WS1 above (`em_cmu.c:58-59`) | Faster fetches at 38 MHz | `hal_mvp_bench` software path, before and after. |
| Bit-bang `set()` | Port read, two branches, port write, 60-cycle delay per edge | SET/CLR alias writes | About half the non-delay cost | `module_speed_probe` around a 100-byte write. |
| `Ssd1306::flush` (`display/ssd1306.rs:141-155`) | 8 pages × 135 bytes; I estimate about 150 ms blocking at 19 MHz, during which the stack is not pumped | Dirty-page mask; one 1025-byte transfer | 5-8× on typical updates | DWT around `flush`. |
| `delay_ms` (`delay/mod.rs:204-208`) | One SysTick arm/disarm per millisecond | One reload for up to 2^24 ticks | Removes per-ms jitter | Compare against `Sysrtc::now`. |
| `critical_section` around single register writes (`timer/mod.rs:220,363,393`) | Mask/unmask per call | Plain write | Small | Code size diff. |
| `Sysrtc::now` (`sysrtc.rs:261-266`) | Two sync waits per read | One, as the vendor does | Small | n/a |

## 4. HAL API review

- **`embedded-hal`:** implemented for the three bus drivers that do not work (I2C, SPI, serial). `pin::Output`/`Input`, `Pwm`, `BitBangI2c` and `Delay` via `Board` are what the product uses, and only `Delay` has a trait. `bus/mod.rs:10-13` promises an adapter "in a few lines"; it does not exist, so no ecosystem driver can be used.
- **Pins are numbers:** `output_on(port, pin: u8, ..)` validates nothing in release. `set_mode` for B/D pins above 7 is a `debug_assert` and a silent no-op (`pin.rs:67-69`). Two `Output`s on one pin compile.
- **Type-state:** the docs describe type-state GPIO; there is no `gpio` module (`lib.rs:88` links to one).
- **Errors:** none implement `core::error::Error`. Several variants are never produced (`timer::Error::Locked`, `spi::Error::Overrun`, `usart::Error::*`, `emu::Error::Locked`). `MicError::StuckDisabling` is returned for `hz == 0` (`microphone.rs:240`); `wdog` returns `Locked` for a timeout (`wdog/mod.rs:320`).
- **Truncation:** `pace_from_timer` casts the period to `u16` (`microphone.rs:256`).
- **`#[must_use]`:** absent on `capture_window`, `pace_microphone`, `boost_*`. One is stranded on a doc comment at `board.rs:293`.
- **`unsafe` in the public surface:** `dwt::start` (`dwt.rs:25`).
- **Docs that do not compile against the code:** `clock/mod.rs:33-41`, `delay/mod.rs:14,52` (`dp.CMU_NS`, `.freeze()` with no argument), `lib.rs` status block (says 78 MHz and "production-ready").
- **Duplication:** I2c0/I2c1 and Spi1/Spi2 are copy-paste; `timer` already shows the macro.

## 5. Radio and Bluetooth: defects

| # | Where | What is wrong | Evidence | Severity | Fix |
|---|---|---|---|---|---|
| B1 | `bt/bonding_ext.rs:152-157, 235-240` | `store_data` and `get_data` take `(handle, type, data, length)`. The stack passes `(handle, type, length, data)`. Every store returns `0x406` before the error counter; every get writes through address 16. | `smp_try_send_keys+0xa0`: `mov r3,r6; movs r2,#16; movs r1,#5; bl`; same in `gatt_db_save_client_conf+0x14` (D) | pairing fails with the external database | Swap the last two parameters. Re-check with `scripts/audit-blobs.sh callers sli_bt_bonding_store_data`. |
| B2 | `bt/psa.rs:324-328` | `psa_reset_key_attributes` zeroes 232 bytes. The stack allocates 64 for that object. | `smp_verify_remote_public_key+0x54` `bg_zalloc(64)`; `bt/support.rs` size table (D) | heap overrun of about 168 bytes on every pairing | Zero the size `sli_psa_context_get_size` reports for attributes. |
| B3 | `bt/psa.rs:552` | A local `PSA_ALG_CMAC = 0x03C0_0000` shadows the right value. The stack passes `0x03C00200`. | Literal at `gatt_db_calculate_hash+0x1c0` (D); the PSA value of CMAC is `0x03c00200` | GATT database hash always fails | Delete the local constant. |
| B4 | `bt/psa.rs:557-568` | `psa_mac_sign_setup` treats the key id as "the Nth used slot"; every other function matches the stored id. | (C) | wrong key after any destroy | Use `slot_for`. |
| B5 | `bt/sleeptimer.rs:351, 361` | `ms32_to_tick` and `tick64_to_ms` return the value. The contract returns a status and writes through a pointer. | `sl_sleeptimer.h:707-708, 741-742` (C); `timer_get_time+0x8` reads `*ms` from its stack (D) | host reads uninitialised time | Match the prototypes. |
| B6 | `bt/sleeptimer.rs:101-125, 180, 370` | Callbacks run from the SYSRTC interrupt and from the main loop, over a `static mut` table with no critical section. The doc at `:367-369` says interrupt context is unsafe for them. | (C) | table corruption, lost timer | Mask around table edits; pick one context for callbacks. |
| B7 | `bt/sleeptimer.rs:329-331` | The 64-bit tick is the 32-bit counter widened; it wraps after 36.4 h. | (C) | time goes backwards after a day and a half | Extend with an overflow count. |
| B8 | `bt/sleeptimer.rs:63-66` | Arming a deadline that is already past never fires. | Same as H25 (C) | stalled stack timer | Check after arming. |
| B9 | `bt/sleeptimer.rs:143, 405` | Callback is called with one argument; the contract is `(handle, data)`. | `sl_sleeptimer.h:74` (C); harmless for the one host callback seen, unknown for RAIL's (D) | latent | Pass the handle. |
| B10 | `bt/sleeptimer.rs:135-136, 341-343` | 32 ticks/ms is 2.4 % short; accuracy is reported as 500 ppm while `:4` records 0.6 % (6000 ppm). | (C) | wrong margins once sleep is used | 64-bit conversion; report the measured figure or use LFXO. |
| B11 | `bt/psa.rs:535`, `:1023`, `:822-824`, `psa_ec.rs:279-284` | Unknown id returns −140 where the stack accepts only 0 or −136; table-full returns −143 (insufficient *data*); short buffers return −135 instead of −138. | `sli_bt_bonding_delete+0x10` (D) | bonding delete raises a system error | Three constants. |
| B12 | `bt/psa.rs:397-402, 828` | `psa_import_key` ignores type, algorithm and usage; `psa_mac_compute` truncates input to 128 bytes and reports success. | (D), source side (C) | wrong MAC on long input | Honour attributes; reject over-length. |
| B13 | `bt/psa.rs` key ids | The stack uses fixed persistent ids (`0x40037` for the IRK). The shim never honours them, so a new IRK is made each boot and one slot of 16 leaks per reset. | `smp_get_irk_id+0xec` (D) | pairing dies after about 16 resets | Honour the id in the attributes for persistent keys. |
| B14 | `bt/blocking/gatt_db.rs:106-157` | A failed step returns without `sl_bt_gattdb_abort`, leaving the session open. `handles.push` beyond 8 is discarded (`:146`). | (C) | next `new_session` fails; lost handle | Abort on error; return an error at capacity. |
| B15 | `bt/blocking/stack.rs:906-914` | Every external-bonding request is answered with zero length. | (C) | a bond can never be restored | See section 8. |
| ~~B16~~ | `bt/stack_init.rs:113, 140` | The link layer is served from PendSV. | (C) | blocks any RTOS (section 7) | **DONE 2026-11-21**: served from NVIC 17 (`BURTC`) instead. The vendor default is `ll_task_default_callback` (`SCB->ICSR = PENDSVSET`, disassembled from the host archive) and `ll_task_init(cb)` is the seam that replaces it. Measured on the board: `adv=0 gatt=0 ll=22`, `PendSV` delivered **once** (the start resets the seam internally; `efr32mg24_radio::ble` redirects that one kick, `ble::pendsv_redirects()` counts it). The `kernel` feature leaves `PendSV` to a kernel. |
| B17 | `bt/heap.rs:3, 49, 103` | Header says 4 KiB; the arena is 64 KiB. Diagnostics read the heap outside the critical section. | (C) | misleading | Fix the header; mask the readers. |
| B18 | `crates/efr32mg24-rail-sys/src/ble_config.rs:3, 102` | The generator is outside the repository; the once-guard hands a second caller the table before it is filled. | (C) | not reproducible | Commit the generator. |

**PSA ABI completeness:** all 17 `psa_*` imports are defined, all key ids are `u32`, and each signature matches one transcribed call site. No `u64` remains in the PSA path. The remaining ABI errors are B1, B5 and B9, outside it.

**Imports with no definition, if their members are linked** (checked only against `ble_boot_probe`): `sli_process_ble_rpa` (privacy), `sli_pending_btctrl_events` (HCI reset), `sl_power_manager_unsubscribe_em_transition_event`.

**Event pump:** forgetting `poll_event` stops host events and GATT; the link layer keeps running from its own interrupt (line 17, `BURTC` -- it was `PendSV` until 2026-11-21) and timers from the SYSRTC interrupt. Notifications have no flow control: `notify` (`stack.rs:595-620`) returns the stack's refusal and nothing retries or waits for `notification_tx_completed`.

## 6. The 206 % figure

It is a unit error, not a load. `bt/stack_init.rs:327-328` calls `emlib_clock_select_hfxo`, which is `CMU_ClockSelectSet(cmuClock_SYSCLK, cmuSelect_HFXO)` (`crates/efr32mg24-emlib-sys/src/shim.c:92`). The crystal is 39 MHz (`ble_config.rs:173`). After `Bluetooth::new` the core runs at 39 MHz and the cycle counts were divided by 19 MHz.

| Run | Cycles | ÷ 19 MHz | ÷ 39 MHz |
|---|---|---|---|
| 5 s of audio | 195 870 529 | 10.31 s | 5.02 s |
| 60 s of audio | 2 350 470 467 | 123.7 s | 60.27 s |

Per window: 621 810 cycles against 304 186 without the stack, a ratio of 2.044; 39/19 is 2.053. The sample timer runs from HFRCOEM23 and did not change, which is why none of the four experiments in `APP-LOOP-WITH-BLE-OPEN.md` moved the number.

I did not measure this on the board. To confirm, read `CMU_SYSCLKCTRL.CLKSEL` after `Bluetooth::new`, or count DWT cycles across 32 768 SYSRTC ticks.

The same switch halves `Board::delay`, the bit-bang half-bit and the microphone settle time.

## 7. Zigbee and Thread

1. **Radio ownership:** the only RAIL archive is `librail_efr32xg24_gcc_release.a`, the single-protocol build. The controller calls `RAIL_Init` itself and a second caller gets NULL (`stack_init.rs:357-361`). A second protocol needs the multiprotocol RAIL archive and the vendor's dynamic-multiprotocol scheduler. That scheduler expects a kernel, and the kernel owns PendSV (B16).
2. **What the HAL has and lacks:** it has the SYSRTC split (group 0 application, group 1 RAIL, `sysrtc.rs:45-48`) and the HFXO start. It lacks PRS, a correct EM2 path, interrupt-priority configuration, and a power-manager that honours requirements (`bt/vendor_support.rs:42-65` are stubs). `RHCLKPRESC` (H10) is wrong for any radio use above 40 MHz.
3. **Shared services:** `sl_sleeptimer` has 8 slots (`sleeptimer.rs:179`), and the host is configured for 8 on its own (`stack_init.rs:450`). The PSA shim implements exactly what BLE imports; 802.15.4 stacks need CCM, SHA-256, HMAC and, for Thread commissioning, EC-JPAKE.
4. **Memory, from source only:**

| Item | Size | Source |
|---|---|---|
| RAM linked | 252 KB | `xiao-mg24-bsp/memory.x:18` |
| Heap arena (`.bss`) | 64 KB | `bt/heap.rs:49` |
| Host buffers (inside the arena) | 16 KB | `stack_init.rs:443` |
| PSA key table | 1.3 KB | 16 × 80 bytes |
| NVM3 | 40 KB at `0x08176000` | `build.rs:224-225` |
| Application flash | 1472 KB | `memory.x:12` |

   Flash is not the constraint. RAM cannot be judged without `.bss`/`.data`; run `arm-none-eabi-size -A` on the dosimeter image. I have no vendor figures for end-device versus router in the tree and will not quote them from memory.
5. **Cheap now:** make the radio crate own a `RadioClock` token that `Bluetooth::new` consumes, so a second stack cannot start HFXO behind it. Keep `sleeptimer`, `heap` and `psa` in a crate that is not named Bluetooth. Give busy-radio its own error variant rather than a BGAPI code. Do not add a "the radio" singleton to `Board`.

## 8. External bonding: what the path will demand

- **Argument order first (B1).** Until it is fixed nothing else on this path can be judged; the recorded "store errors 0" is consistent with every store failing, because the early return precedes the counter.
- **Sizes come from the callers:** key pieces are 16 bytes with type 5 (`smp_try_send_keys`), the client configuration is 16 bytes with type 11 (`gatt_db_save_client_conf`). List the rest with `audit-blobs.sh callers` rather than from the header.
- **A zero-length answer means "no bond".** It is correct for a first pairing. After a reset it makes the device answer `0x1006` exactly as the builtin database does. The answer must come from NVM3, keyed by the peer's identity address, and resolvable addresses need the IRK, which depends on B13.
- **`sli_bt_bonding_delete`** destroys key `0x40037` and accepts only 0 or −136 (B11).

## 9. The five changes to make first

1. **B1, B2, B3, B5** — four signature or constant errors that overrun the heap, break the GATT hash and block pairing; each is a few lines and is proven by disassembly.
2. **H3** — one helper for flag clearing; nine silent misbehaviours disappear and the rule becomes impossible to forget.
3. **H21/H22 with section 6** — one runtime clock value; it removes the 206 % question, the short delays and the wrong tones together.
4. **Gate `usart`, `spi`, `timer`, `dma`, `rtc::Rtc`, `i2c` behind an `unproven` feature** until each has an example. This is cheaper than fixing six drivers nobody uses, and it stops crates.io from shipping them as working.
5. **B6, B7, B8** — the timer backend must be correct before a second stack shares it.

## 10. Wrong premises in the brief

| Brief says | What the tree shows |
|---|---|
| `adc` writes `portpos=8` into a two-bit field | The field is four bits and 8 is `PORTA` (`_IADC_SINGLE_PORTPOS_MASK 0xF000`, `efr32mg24_iadc.h:964,971`; PAC width 4). The real defect is H13. |
| The 206 % is unexplained | Section 6. |
| `sleeptimer.rs` answers by polling | It has been interrupt-driven since `SYSRTC_APP_IRQHandler` was added (`:101-125`). Its header (`:3-11`), `poll()`'s doc and `stack.rs:849-853` still say otherwise. |
| HAL is 10 235 lines over ~45 files | 8 656 lines in 48 `.rs` files (`wc -l`). |
| Flash writes are a HAL path | The HAL has no flash driver; it touches MSC only for wait states. |
| Each driver is `mod` + `types` + `traits` | `dma`, `mvp`, `delay`, `bus` have no `traits.rs`; `adc`, `emu`, `rtc`, `timer`, `wdog` have stubs of 9-23 lines; `display/types.rs` does not exist. |
| `usart` "does not route pins" | It never gets that far (H4, H5). |
| `ble_config.rs` is the radio configuration to trace | The product path does not use it. Only `rail_ble_config_probe.rs:135` and `ble_rail_then_stack.rs:88` call it; the controller uses its own internal configuration (`stack_init.rs:363-366`). |
| The heap is "an arena" of unstated size | 64 KiB, with a header that says 4 (B17). |

## Not covered

| File | Reason |
|---|---|
| `bt/psa.rs`, `psa_ec.rs`, `crypto.rs`, `support.rs` | Audited by sub-agent against disassembly; I read only the lines cited. |
| `bt/se_trng.rs`, `rail_assert.rs`, `vendor_support.rs`, `blocking/types.rs`, `error.rs` | Not read. |
| `bt/blocking/events.rs`, `stack.rs:1-560, 670-780` | Outline and length handling only. |
| `efr32mg24-radio/src/*`, `efr32mg24-rail-sys/src/lib.rs`, `libraries/bt-heap` | Not read. |
| `hal/sensors/lsm6ds3trc.rs`, `hal/rtc/pcf8563.rs`, `hal/wdog/types.rs` | Skimmed. |
| HAL `README.md`, `_tmp/review-ble-external-db/*`, most of `docs/04-reference/` | Not read; only the I2C, app-loop and BLE-proofs records were. |
| Reference manual, datasheet | No PDF reader available. |