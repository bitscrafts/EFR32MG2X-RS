# SYSRTC: who owns compare channel 0

**Question.** The 2026-11-22 review (`HAL-OPUS-ANALYSIS-2026-11-22.md`, concurrency section) found that the HAL and the
Bluetooth sleep timer both use SYSRTC compare channel 0 and the same interrupt, "so they can't share one image as
written". This records what each side actually programs, and decides the split.

## What was measured

```
HAL   crates/efr32mg2x-hal/src/sysrtc.rs:442,463
      regs.grp0_cmp0value().write(...)            -> compare channel 0
      NVIC::unmask(Interrupt::SYSRTC_APP)         -> the SYSRTC_APP vector

BLE   crates/efr32mg24-bluetooth-sys/src/sleeptimer.rs:69
      efr32mg2x_hal::cores::sysrtc::compare_target(now, next)   -> the same arithmetic
      and the same channel: its own header records that the per-channel flag for
      channel 0 is bit 1 of GRP0_IF (`SYSRTC_GRP0_IF_CMP0`), measured -- "0x22 one count
      after the deadline, 0x44 on channel 1". See SYSRTC-COMPARE-WORKS.md.

PRODUCT  crates/xiao-mg24-bsp/examples/module_ble_dosimeter.rs
      uses neither: it counts seconds in the superloop and drains stack events.
```

So the collision is real **in code** and not live **in the product**. Nothing breaks today because no image uses both
paths; the first image that runs the Embassy time driver (which reaches `schedule_at_global` -> `grp0_cmp0value`)
alongside the Bluetooth stack will have two writers on one compare register, with no error at any layer: the later
write silently moves the other party's deadline, which is precisely the "cadence stops after days" failure mode.

## Decision

**Channel 0 belongs to the Bluetooth sleep timer while the stack is linked.** It is the link-layer deadline: the
stack has to be woken to keep a connection alive, it is the path with measured evidence behind it
(`SYSRTC-COMPARE-WORKS.md`), and moving it would have to be re-verified with a phone on the other end.

**The application takes channel 1 or 2.** These are free, they have the same arming and flag behaviour (the `0x44`
measurement above is channel 1), and moving the HAL's side costs a board test that needs no phone.

**A product image may not use channel 0 for itself.** The HAL's `set_compare` and `schedule_at` must stop being
callable in an image that links the Bluetooth sleep timer.

## Why the decision needs enforcement, not a comment

The project's own rule applies: a rule that lives only in a document is a rule that gets broken -- `OPEN-REGISTER`
row A24 is that rule, learned the hard way in this repository. A comment above `grp0_cmp0value` saying "the stack
also uses this" has exactly the failure mode the row describes: it is invisible at the call site, and the compiler
says nothing when it is ignored.

The shape that fits the rest of this HAL is a **token**:

```rust
impl Sysrtc {
    /// Channel 0, which the Bluetooth sleep timer owns. Returns None when the stack has claimed it.
    pub fn claim_bluetooth_channel(&mut self) -> Option<CompareChannel> { ... }

    /// The application's channel. Never 0 in an image that links the stack.
    pub fn claim_application_channel(&mut self) -> CompareChannel { ... }   // 1, then 2
}
```

`set_compare` then takes a `CompareChannel`, so a caller cannot spell channel 0 without having asked for it. The
Bluetooth crate takes it once, in the same place it starts the sleep timer.

Until that exists, the honest statement is the one in this file: **the split is decided and not yet enforced**, and
no image is broken today because no image uses both.