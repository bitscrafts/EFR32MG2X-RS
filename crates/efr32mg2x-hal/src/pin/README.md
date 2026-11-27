# pin

Pin-level primitives: the HAL's lowest layer above the registers.

## What it is

Pin-level primitives: the HAL's lowest layer above the registers.

Nothing here knows which board it is on. A pin is a port, a bit and a polarity; which
pins a board uses is the BSP's business. That separation is what lets the same driver
serve two boards — and it is the rule this crate broke for months by naming PC03, PC01,
PC04, PD5 and PA7 in its own board module.

## API at a glance

| item | kind | what it does |
|---|---|---|
| `DISABLED` | const | Input buffer off; the pull-up applies if `DOUT` is set. |
| `EdgeWatch` | struct | One GPIO external-interrupt channel, bound to the pin it watches and the edges it reacts to. |
| `INPUT` | const | Input enabled, no pull. |
| `INPUT_PULL` | const | Input enabled with a pull-up (the pull-up is selected by `DOUT = 1`). |
| `Input` | struct | An input pin with the internal pull-up enabled. |
| `Output` | struct | A push-pull output pin with a polarity. |
| `PUSH_PULL` | const | Push-pull output. |
| `Port` | enum | — |
| `WIRED_AND` | const | Open-drain output: pulls low, releases high; the pull-up brings it back. |
| `channel` | fn | The external-interrupt channel this watch drives, which is also the PRS `SIGSEL` for `SOURCE = GPIO`. |
| `clear` | fn | Clear this channel's flag through the alias window. |
| `flags` | fn | The whole `IF` word, for a probe that reports what the hardware latched rather than what it expected. |
| `is_high` | fn | True while the pad reads high. |
| `is_low` | fn | True while the pin is pulled low — the pressed state of a button to ground. |
| `off` | fn | Release the pin. |
| `on` | fn | Assert the pin. |
| `pending` | fn | True while this channel's flag is latched. |
| `selection` | fn | The port and pin select words this channel sits in, as they read back. |
| `set` | fn | Assert or release the pin, honouring the polarity it was built with. |
| `set_edges` | fn | Arm or disarm the two edge masks, leaving the channel's binding and its flag alone. |
| `set_pull_up` | fn | Choose which way the internal pull resistor drives the pin. |
| `toggle` | fn | Invert the pin. |

## Registers it touches

None directly: this module composes other drivers and reaches no `pac::` peripheral of its own.

## What was measured

```
    gpioirq: n=8 if=0x00000008 cnt=8 off=0 dis=0 msk=0 odd=8 even=0 pin0=8 p0odd=0 p0even=8 a=0 up=4 sel=0x00002000 isel=0x00003000
    device: sev=0x2005 (disconnected), rsn=0x1006 = HCI "PIN or Key Missing"
    adcgpio: ch=3 sel=0x00002000 isel=0x00003000 din=1 a=0 b=4 c=4 mv=8 up=4 off=0 other=0 pin0=4 up0=4 long=8 upg=8 fall=4 ifb=0x00000009 ifc=0x00000001 v0=0x00000231 vl=0x0000023a
    adcgpio: ch=3 sel=0x00002000 isel=0x00003000 din=1 a=0 b=4 c=4 mv=8 up=4 off=0 other=0 pin0=4 up0=4 long=8 upg=8 fall=4 ifb=0x00000009 ifc=0x00000001 v0=0x00000233 vl=0x0000023a
```

Controls and the full story: [GPIO-INTERRUPT-2026-11-27.md](../../../../docs/04-reference/GPIO-INTERRUPT-2026-11-27.md), [PAIRING-D2F-STATE.md](../../../../docs/04-reference/PAIRING-D2F-STATE.md), [PRS-GPIO-PRODUCER-2026-11-26.md](../../../../docs/04-reference/PRS-GPIO-PRODUCER-2026-11-26.md)

## What is not proven

- So the MCU side is configured; what is not proven is that the *pin* moves and. (EXPANSION-BOARD-STATUS-2026-11-11.md (`../../../../docs/03-development/EXPANSION-BOARD-STATUS-2026-11-11`))
- ⏳ `GpioPin` encoded and never driven (open item); synchronous channels and other consumers untouched. (PLAN.md (`../../../../docs/03-development/PLAN`))
- `prs` | peripheral reflex system: routes an event from one peripheral to another without the CPU | ~250 lines, of which the crate now drives a channel: `prs::Source` names the producers it encodes (`Off`, `Timer0Overflow`, `Timer0Compare0`, `Timer1Overflow`, `GpioPin`), `Prs::connect` writes `ASYNC_CHn_CTRL` whole and `Prs::channel_ctrl` reads it back, and `Prs::pulse` is the software edge. (COVERAGE-BOUNDARY-2026-11-25.md (`../../../../docs/04-reference/COVERAGE-BOUNDARY-2026-11-25`))

## Using it

See `examples/hal_adc_prs_gpio.rs`, `examples/hal_adc_prs_timer.rs`, `examples/hal_adc_prs_trigger.rs`, `examples/hal_em2_sleep.rs`, `examples/hal_eusart_loopback.rs`, `examples/hal_gpio_interrupt.rs`, `examples/hal_i2c_async.rs`, `examples/hal_i2c_bitbang.rs`, `examples/hal_i2c_hardware_imu.rs`, `examples/hal_i2c_recovery.rs`, `examples/hal_imu_read.rs`, `examples/hal_led_blink.rs`, `examples/hal_mic_level.rs`, `examples/hal_pcnt_gpio.rs`, `examples/hal_spi_loopback.rs`, `examples/hal_sysrtc_wide.rs`, `examples/hal_sysrtc_wrap.rs`, `examples/hal_timer0_pwm.rs`, `examples/hal_usart_link.rs`, `examples/hal_usart_rx_recovery.rs`
