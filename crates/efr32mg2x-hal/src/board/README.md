# board

Peripherals: the single place where the HAL touches the PAC singleton.

## What it is

Peripherals: the single place where the HAL touches the PAC singleton.

The point of this module is that no **example** ends up naming `pac`. An
application asks the HAL for a board, and the board hands out the pieces
(I2C bus, LED) with their bus clocks already enabled — because a missing
`CMU_CLKEN0` bit is a precise bus fault on the first register access, and
that failure belongs in one place, not in every example.

The PAC is still underneath: this is a PAC-based HAL, and it is the PAC that
defines every register this board pokes. What the HAL owns is the sequence
and the defaults that the hardware demands (gate first, then configure).

## API at a glance

| item | kind | what it does |
|---|---|---|
| `Board` | struct | Everything an application needs from the XIAO MG24 Sense + Expansion Base. |
| `Handout` | enum | — |
| `acmp` | fn | The analogue comparator, with its bus gate already set. |
| `adc` | fn | The general-purpose ADC, distinct from the microphone's converter. |
| `aes` | fn | The `RADIOAES` crypto block, with its bus gate on. |
| `allocate_odd_cd_bus` | fn | Give the odd CD bus to `ADC0`, which is what an odd pin needs to reach the converter. |
| `boost_to_38_mhz` | fn | Raise the core clock to the 38 MHz HFRCODPLL band, and report the calibration word it wrote. |
| `boost_to_56_mhz` | fn | Raise the core clock to the **56 MHz** band — the nearest calibrated band to 19 x 3. |
| `boost_to_80_mhz` | fn | Raise the core clock to the **80 MHz** band, the fastest calibrated one under 100. |
| `burtc` | fn | The backup counter, on the LFRCO branch and counting from the first tick after this call. |
| `buzzer_on` | fn | A TIMER0 PWM buzzer on the pin the caller names. |
| `clock_hz` | fn | — |
| `converter` | fn | IADC0 clocked from HFRCOEM23, ready for the microphone driver. |
| `count_edges` | fn | The same channel as [`Self::watch_edges`], with its edges counted in the GPIO interrupt handler. |
| `delay` | fn | A millisecond delay on SysTick. |
| `emu` | fn | The energy management unit, handed out through the board like every other peripheral: an example outside this crate must not name a PAC type, and the gate enforces that. |
| `eusart0_async` | fn | EUSART0 as an **asynchronous** serial port, with its clock branch and its two pads. |
| `eusart0_tx_disabled_for_test` | fn | — |
| `eusart1_async` | fn | EUSART1 as an asynchronous serial port, the second of the two blocks. |
| `gpcrc` | fn | The CRC accelerator, for checking an image, a configuration blob or a frame. |
| `i2c0` | fn | The hardware I2C0, with its clock enabled and its two lines still unrouted. |
| `i2c1` | fn | Hands out `I2C1` — the second bus, with its own clock-enable bit and its own pins. |
| `i2c_on` | fn | A bit-banged I2C master on the two pins the caller names. |
| `i2c_static` | fn | — |
| `input_on` | fn | An input with the internal pull-up on the pin the caller names. |
| `letimer` | fn | The low-energy timer: 24-bit counter, two compare channels, and the only timer here that runs in EM2. |
| `mvp` | fn | The matrix/vector processor, enabled and ready. |
| `output_on` | fn | A push-pull output on the pin the caller names. |
| `pace_microphone` | fn | Start TIMER0 free-running at `hz`, with no pin and no PWM channel, as the microphone's pace. |
| `pcnt` | fn | The pulse counter, with its clock open, counting nothing yet. |
| `prepare_sysrtc_clocks` | fn | Gate the SYSRTC's clocks without starting it. |
| `prs` | fn | The PRS block, with its bus clock open, for a caller that drives its own trigger. |
| `pwm_on` | fn | A PWM channel on the pin the caller names, at `freq_hz`, dark. |
| `route_eusart0_async` | fn | Route EUSART0's transmitter and receiver to two pads, for the asynchronous mode. |
| `route_eusart0_spi` | fn | Route EUSART0 in synchronous mode: MOSI, MISO and the clock. |
| `route_eusart1_async` | fn | Route EUSART1's transmitter and receiver to two pads. |
| `route_eusart1_spi` | fn | Route `EUSART1`'s three SPI signals to the pads the caller names. |
| `route_i2c0` | fn | Points I2C0's SCL and SDA at real pins, and puts those pins in open-drain. |
| `route_usart0` | fn | Route USART0's transmitter and receiver to two pins. |
| `route_usart0_spi` | fn | Route `USART0`'s three SPI signals to the pads the caller names. |
| `select_hfxo` | fn | Run the core from the crystal and report the frequency in force. |
| `sleep` | fn | Entry and exit of EM2, carrying the `EMU` it drives. |
| `spi0_on` | fn | `USART0` in synchronous mode, on three routed pins. |
| `spi2_on` | fn | `EUSART1` in synchronous mode, on three routed pins. |
| `spi_on` | fn | EUSART0 as an SPI master on three routed pins: the board's own SPI pads. |
| `sysrtc` | fn | The tick timer, shared by every protocol stack that may run on this radio. |
| `take` | fn | Take the peripherals and enable the bus clocks this HAL drives. |
| `timer0_counter` | fn | The raw TIMER0 counter, for a probe that has to answer "does the timer run at all". |
| `timer0_free_running` | fn | TIMER0 as a free-running 16-bit counter in the `EM01GRPACLK` group, with that group's source left alone. |
| `timer0_free_running_prescaled` | fn | The same counter with its prescaler chosen by the caller, `divide_log2` being the number of halvings. |
| `timer0_on` | fn | TIMER0 on the pin the caller names, calibrated but undecided about what it means. |
| `timer1` | fn | `TIMER1`, handed out with its own claim so two owners cannot pace the same counter. |
| `timer2` | fn | `TIMER2` — the same driver and the same claim discipline as [`Board::timer1`]. |
| `timer3` | fn | `TIMER3` — the same driver and the same claim discipline as [`Board::timer1`]. |
| `timer4` | fn | `TIMER4` — the same driver and the same claim discipline as [`Board::timer1`]. |
| `usart0` | fn | USART0 with its clock enabled, ready for the pins to be routed. |
| `watch_edges` | fn | Point an external-interrupt channel at `port`/`pin` and have it react to the edges named. |
| `wdog` | fn | The watchdog, configured at construction. |

## Registers it touches

`Acmp0S`, `BurtcNs`, `EmuS`, `Eusart0S`, `Eusart1S`, `GpcrcNs`, `GpioNs`, `GpioS`, `I2c0Ns`, `I2c1S`, `Iadc0S`, `Letimer0Ns`, `MvpNs`, `Pcnt0Ns`, `Peripherals`, `PrsS`, `RadioaesS`, `Timer1S`, `Timer2S`, `Timer3S`, `Timer4S`, `Usart0S`, `Wdog0Ns`

## What was measured

```
    board: gatt c0 notif=0 refused=0 ev=0 se=4/4616 sm=1 h=010603a0 ua=00000000x0 cv=
```

Controls and the full story: [BLE-STATUS.md](../../../../docs/04-reference/BLE-STATUS.md)

## What is not proven

- ~~H28~~ | EM2 entry and exit sequence | **CLOSED 2026-11-21** (`B3`): ten ~1 s naps in EM2 woken by the SYSRTC comparator, intervals 0.92-1.03 s, mean 0.99, total 9.92 s, eleven interrupts. (HAL-GAP-ANALYSIS-2026-11-18.md (`../../../../docs/04-reference/HAL-GAP-ANALYSIS-2026-11-18`))
- Plan to close the coverage gaps (`03-development/PLAN-COVERAGE-2026-11-25`) - what each missing item needs to be *proven*, in phases: no hardware needed, board on the desk, meter-class hardware, and the radio's vendor blobs. (../../../../docs/INDEX.md (`../../../../docs/INDEX`))

## Using it

See `examples/hal_acmp_ladder.rs`, `examples/hal_adc_prs_gpio.rs`, `examples/hal_adc_prs_timer.rs`, `examples/hal_adc_prs_trigger.rs`, `examples/hal_adc_read.rs`, `examples/hal_aes_kat.rs`, `examples/hal_burtc_backup.rs`, `examples/hal_cmu_sources.rs`, `examples/hal_delay_clock.rs`, `examples/hal_delay_handles.rs`, `examples/hal_em2_sleep.rs`, `examples/hal_emu_voltage.rs`, `examples/hal_eusart1_loopback.rs`, `examples/hal_eusart_loopback.rs`, `examples/hal_gpcrc.rs`, `examples/hal_gpio_interrupt.rs`, `examples/hal_hfxo.rs`, `examples/hal_i2c_async.rs`, `examples/hal_i2c_bitbang.rs`, `examples/hal_i2c_hardware_imu.rs`, `examples/hal_i2c_recovery.rs`, `examples/hal_imu_read.rs`, `examples/hal_led_blink.rs`, `examples/hal_letimer_compare.rs`, `examples/hal_mic_level.rs`, `examples/hal_mvp_bench.rs`, `examples/hal_one_of_each.rs`, `examples/hal_pcnt_gpio.rs`, `examples/hal_spi_loopback.rs`, `examples/hal_spi_masters.rs`, `examples/hal_sysrtc_wide.rs`, `examples/hal_sysrtc_wrap.rs`, `examples/hal_timer0_pwm.rs`, `examples/hal_timer0_repacing.rs`, `examples/hal_timer_compare.rs`, `examples/hal_timer_instances.rs`, `examples/hal_timer_ticks.rs`, `examples/hal_tx_timeout.rs`, `examples/hal_usart_link.rs`, `examples/hal_usart_rx_recovery.rs`, `examples/hal_wdog_feed.rs`
