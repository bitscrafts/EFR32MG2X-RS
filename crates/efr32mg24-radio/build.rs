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
    ] {
        println!("cargo:rustc-link-arg-examples=--wrap={sym}");
    }
}
