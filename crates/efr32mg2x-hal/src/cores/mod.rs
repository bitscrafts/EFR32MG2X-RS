//! Pure logic, testable on the host, compiled into the HAL that owns it.
//!
//! Each file here is also compiled by the matching crate under `libraries/`
//! through `#[path]`, so the host tests keep running and there is one copy.

pub mod spi;
