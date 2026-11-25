//! Pure logic, testable on the host, compiled into the HAL that owns it.
//!
//! `allow(dead_code)` because this layer carries everything a driver *or its host twin* may call: a build that reaches
//! half of it is a build, not a defect, and deleting what a feature-gated path needs is how a library loses the tests
//! that guard it.
//!
//! Each file here is also compiled by the matching crate under `libraries/`
//! through `#[path]`, so the host tests keep running and there is one copy.

#![allow(dead_code)]

pub mod spi;
// These sources were written under the library crates, where the lint budget was different;
// documenting them as HAL API is a separate item (QUEUE backlog). The behaviour is covered by
// the host tests that the library crates still run.
pub mod adc;
pub mod dma;
pub mod gpcrc;
pub mod i2c;
pub mod slots;
pub mod sysrtc;
pub mod usart;
