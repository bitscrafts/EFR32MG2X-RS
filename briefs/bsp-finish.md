# Brief: finish the BSP, so the board facts live in one place

You are working in `/projects/efr32mg24-rs` (a Rust workspace for the EFR32MG2X / Seeed XIAO MG24 Sense).

## The one job

Today there are **two** board layers:

- `crates/efr32mg2x-hal/src/board.rs` (237 lines) — a `Board` type holding board facts: the LED pin
  (`PA7`), the microphone enable and signal pins (`PC8`/`PC9`), the IMU power pin (`PD5`), the IMU bus
  (`PB2`/`PB3`), plus `adc()`, `emu()`, `wdog()`, `sysrtc()`, `converter()`, `timer0_on()`, `pwm_on()`,
  `buzzer_on()`, `input_on()`, `output_on()`, `i2c_on()`, `delay()`.
- `crates/xiao-mg24-bsp` (package `efr32mg2x-xiao-bsp`) — a `Module` that *wraps* that `Board`
  (`pub fn hal(&self) -> &HalBoard`) and re-exposes `led()`, `imu_bus()`, `imu_power()`, `microphone()`,
  `delay()`.

The HAL is supposed to be board-agnostic. A HAL serves any EFR32MG24; knowing that the LED is on `PA7`
is board knowledge. Make the board layer single:

1. **Move** the board knowledge from `crates/efr32mg2x-hal/src/board.rs` into
   `crates/efr32mg2x-xiao-bsp` (keep the public API shape: `Board::take()`, `led()`, `imu_bus()`,
   `imu_power()`, `microphone()`, `delay()`, `adc()`, `emu()`, `wdog()`, `sysrtc()`, `timer0_on()`,
   `pwm_on()`, `buzzer_on()`, `input_on()`, `output_on()`, `i2c_on()`).
2. **Delete** the `board` module from the HAL, and stop exporting it from `crates/efr32mg2x-hal/src/lib.rs`.
3. **Move** the HAL examples that use `Board` into `crates/efr32mg2x-xiao-bsp/examples/` (with whatever
   support files they need: `examples/common/mailbox.rs`, `memory.x`, linker/defmt build script, target
   settings). They must keep their names and their behaviour.
4. Leave `crates/xiao-expansion-base-bsp` untouched.

## Hard constraints

- **Never** run `git commit`, `git push`, `cargo publish`, or `cargo yank`. Leave the work in the tree.
- **Never** touch `crates/efr32mg2x-pac` — not one line. It is frozen.
- **Never** flash or talk to hardware. There is no board attached to you; every check is a build check.
- Do **not** rename the `efr32mg24-*` crates (that is a later job).
- Do **not** invent hardware facts. The measured ones are: LED `PA7` (active low), microphone enable
  `PC8`, microphone signal `PC9`, IMU power `PD5`, IMU bus `PB2`=SDA / `PB3`=SCL, IMU address `0x6a`,
  IADC clocked from `HFRCOEM23`. Anything else, copy from the existing code rather than guessing.
- Every public item you add needs a doc comment; the workspace denies missing docs.

## Acceptance criteria — all of them, each one checkable

1. `grep -rn "PA7\|PC8\|PC9\|PD5\|PB2\|PB3" crates/efr32mg2x-hal/src/ ` prints **nothing**.
   (Board pins must no longer be named inside the HAL.)
2. `cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg2x-hal --features rt` succeeds.
3. `cargo build --release --target thumbv8m.main-none-eabihf -p efr32mg2x-xiao-bsp --features rt --examples`
   succeeds, and it builds **at least ten** examples (list them in your final answer).
4. `bash scripts/gate.sh` ends with `VERDICT: PASS`.
5. `git status --porcelain` shows your work, and `git log --oneline -1` is unchanged from when you started
   (you committed nothing).

## How to work

- Read before writing: `crates/efr32mg2x-hal/src/board.rs`, `crates/xiao-mg24-bsp/src/module.rs`,
  `crates/efr32mg2x-hal/src/lib.rs`, and one example such as `examples/hal_led_blink.rs`.
- Move code with `git mv` where a file moves wholesale, so the history keeps following it.
- Build after each structural step; do not leave a broken tree while starting the next step.
- If an acceptance criterion cannot be met for a reason you can prove (not a guess), stop and say exactly
  which one and why, with the command output. A truthful partial is worth more than a false PASS.

## Report at the end

Say, in this order: which criteria passed (with the command output that shows it), which failed, the list of
examples that now live in the BSP, and anything you found that a reviewer must look at by hand.
