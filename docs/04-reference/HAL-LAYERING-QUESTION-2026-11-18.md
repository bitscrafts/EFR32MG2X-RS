# Does the radio stack depend on the HAL? This tree says yes; the ecosystem says no

**Written 2026-11-18, for an independent analysis. Read-only; nothing here asks for a change.**

The question looks small — where does a 120-line deadline calculation live — and it is not. The
answer depends on which way the layers point, and this repository points a different way from every
ecosystem it resembles.

## What this tree does, measured

```
crates/efr32mg24-bluetooth-sys/Cargo.toml:17
    efr32mg2x-hal = { package = "efr32mg2x-hal", path = "../efr32mg2x-hal", features = ["rt"] }
crates/efr32mg24-radio/Cargo.toml:20        same
crates/efr32mg24-emlib-sys/Cargo.toml:16    same
```

The radio stack, the radio driver crate and the vendor-binding crate all depend on the HAL. There is
no configuration of this repository in which `efr32mg24-bluetooth-sys` is built without
`efr32mg2x-hal`: the dependency is unconditional, not behind a feature.

So the HAL here is not the top layer. It is the layer everything above the register access stands on,
including the crates that in other ecosystems stand on nothing but the PAC. `emlib-sys`, a binding
crate whose whole job is to be a thin wrapper over vendor headers, depending on a HAL is the unusual
part; a HAL that is itself a dependency of the vendor bindings is a shape worth naming.

## What the ecosystems do, measured the same day

```
embassy-nrf/Cargo.toml
    embassy-hal-internal = { version = "0.5.0", path = "../embassy-hal-internal" }
    embassy-time         = { version = "0.5.1", path = "../embassy-time" }
    embassy-sync         = { version = "0.8.0", path = "../embassy-sync" }
    embassy-time-driver  = { version = "0.2.2", path = "../embassy-time-driver" }

nrf-softdevice/Cargo.toml
    embassy-sync, cortex-m -- and no nrf52840-hal, no embassy-nrf
```

Two facts, both load-bearing:

**The `version` and `path` idiom.** Each shared helper is a small published crate carried by both
`version` and `path`. The path serves local development; the version is what makes it publishable.
This is the ecosystem's answer to "pure logic that two layers need".

**The stack does not depend on the HAL.** `nrf-softdevice` binds the radio and talks to registers
itself. It takes `embassy-sync` and `cortex-m` and nothing else from the HAL side. The Nordic
ecosystem avoided exactly the edge this repository has.

## Why the question came up

`sysrtc-core` holds `compare_target` (a deadline already in the past must fire now, not after the
36.4 h counter wrap) and `WideTick` (a 64-bit tick fed by the ISR, five host tests). Two callers:

```
crates/efr32mg2x-hal/src/sysrtc.rs                 compare_target(self.now(), deadline)
crates/efr32mg24-bluetooth-sys/src/sleeptimer.rs   WideTick, compare_target(now(), next)
```

Moving that source into the HAL, as was done for the other five `libraries/*-core`, requires the
radio stack to reach into the HAL. That looked like an inversion to be avoided, until the manifest
above showed the inversion is already the house style.

## The questions worth an independent opinion

1. **Is a HAL that `emlib-sys`, `radio` and `bluetooth-sys` all depend on a deliberate layering or
   accumulated debt?** What does it cost, concretely, the first time someone wants the radio without
   the HAL — or wants to publish `bluetooth-sys` on its own?
2. **What is the smallest change that keeps one implementation of a rule which has already produced
   two defects** (a past deadline waiting 36.4 h; a `deadline == now` that never fired), given that
   the two callers are in different crates and the shared ancestor is a generated PAC?
3. **Does the Nordic pattern apply here?** `nrf-softdevice` reimplements its own register access
   rather than depending on the HAL. Copying that shape would mean `bluetooth-sys` owning its
   deadline arithmetic. Is duplicating a subtle rule better than an already-existing layer edge?
4. **If the answer is "the edge is fine, keep it"**, what should be written down so the next person
   does not re-litigate it — and what check, if any, should guard it?

## Rules for the answer

- Cite `path:line` for every claim about this repository. Measure; do not infer.
- Say where this document is wrong. Two earlier briefs in this session were wrong in ways only
  measurement caught: one had the wrong library count, one assumed the HAL could not build on the
  host.
- Judge on what breaks in two years, not on what is quickest tonight.
- One answer, no longer than 200 lines.
