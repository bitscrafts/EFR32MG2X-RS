//! Pure logic, testable on the host, compiled into the HAL that owns it.
//!
//! Each file here is also compiled by the matching crate under `libraries/`
//! through `#[path]`, so the host tests keep running and there is one copy.

pub mod spi;
// These sources were written under the library crates, where the lint budget was different;
// documenting them as HAL API is a separate item (QUEUE backlog). The behaviour is covered by
// the host tests that the library crates still run.
#[allow(missing_docs)]
pub mod i2c;
