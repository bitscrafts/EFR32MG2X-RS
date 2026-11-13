//! Link-time wrappers for the `diag` feature.
//!
//! `--wrap=SYM` redirects every *undefined* reference to `SYM` to `__wrap_SYM`, which can then call `__real_SYM`. It
//! works only for symbols that some object references without defining, so the list below holds only the RAIL entry
//! points the controller calls (each has a `U` row in the ABI table of the round 2 report). `usch_sendEventToTask` is
//! deliberately absent: every controller object defines its own weak copy, so there is no undefined reference to
//! redirect.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_DIAG").is_none() {
        return;
    }
    for sym in [
        "RAIL_GetTime",
        "RAIL_ConfigEvents",
        "RAIL_StartTx",
        "RAIL_StartScheduledTx",
        "RAIL_Idle",
        "RAIL_WriteTxFifo",
        // Receiving, because a scanner that reports nothing has two possible causes -- the link layer never asks the
        // radio to receive, or it does and the packets never arrive -- and only a counter tells them apart. Both
        // symbols are declared in `vendor/rail/rail.h` (3749 and 3672) and the controller references both, so both
        // are wrappable. `RAIL_GetRxPacketInfo` is NOT in this list: the controller imports it, but the header this
        // project vendors does not declare it, and a wrapper written against a guessed prototype is the exact defect
        // this project keeps finding in other people's code.
        "RAIL_ScheduleRx",
        "RAIL_ConfigRxOptions",
    ] {
        println!("cargo:rustc-link-arg-examples=--wrap={sym}");
    }
}
