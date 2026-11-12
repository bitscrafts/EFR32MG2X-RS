#![no_std]
#![doc = include_str!("../README.md")]

//! This crate has no Rust API on purpose.
//!
//! It exists so that the C file it compiles — the `CORE_*` helpers the Silicon Labs radio blobs
//! call — is compiled **once**, with one definition of each symbol, for every crate in the
//! workspace that links a vendor blob. A dependency on this crate is a dependency on those
//! symbols; that is the whole contract.
//!
//! The functions themselves are in `src/cmsis_stubs.c`, commented there, because they are
//! assembly and comments belong next to the instructions.
