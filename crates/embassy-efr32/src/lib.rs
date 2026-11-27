//! Embassy time driver for the EFR32MG24, on top of the HAL's SYSRTC.
//!
//! ## What this gives you
//!
//! `embassy-time`, and through it `embassy-sync` and `embassy-executor`, on this part. Embassy asks
//! a time driver for two things: a monotonic 64-bit tick, and somewhere to park a waker until a
//! deadline. The second half is in [`alarm`], pure and host-tested.
//!
//! ## Why the platform is a parameter
//!
//! The driver does not reach into the HAL's registers itself. It asks a [`Platform`] for the tick
//! and for arming the comparator, and the BSP supplies that from `efr32mg2x_hal::sysrtc`. The reason
//! is the same one that produced `cores/` in the HAL: arithmetic that can be tested on a host is
//! separated from the two register writes that cannot, and the seam is visible in the type.
//!
//! ```ignore
//! struct SysrtcPlatform;
//! impl embassy_efr32::Platform for SysrtcPlatform {
//!     fn ticks_64() -> u64 { /* the HAL's 64-bit tick */ }
//!     fn arm(value: u32, rearm: bool) { /* the HAL's comparator */ }
//! }
//! embassy_efr32::time_driver!(SysrtcPlatform);
//! ```
//!
//! ## What is proven, and what is not
//!
//! The deadline choice ([`alarm::earliest`]) has host tests. That a comparator interrupt wakes a
//! task on the board does **not** -- it needs the board, a deadline shorter than a second and a
//! waker that prints. This crate is a skeleton that compiles and whose logic is tested; it is not
//! a measured driver, and nothing should claim otherwise until that print exists.
#![deny(missing_docs)]
#![no_std]

pub mod alarm;
pub mod ticks;

use core::marker::PhantomData;
use core::task::Waker;
use embassy_time_driver::Driver;

/// What a BSP must supply: the clock and the comparator. Two functions, deliberately.
pub trait Platform: Send + Sync + 'static {
    /// A monotonic 64-bit tick. The HAL's `WideTick` provides exactly this.
    fn ticks_64() -> u64;

    /// Point the comparator at `value` and enable it. `rearm` means the requested deadline was
    /// further away than the 32-bit counter can express, so the driver must be called again once
    /// this value is reached.
    fn arm(value: u32, rearm: bool);
}

/// The Embassy driver. Stateless: the clock is the platform's, the pending waker is [`alarm`]'s.
pub struct Efr32Driver<P: Platform>(PhantomData<P>);

impl<P: Platform> Efr32Driver<P> {
    /// The driver. Nothing to carry: the clock and the comparator are the platform's, and the pending
    /// waker belongs to [`alarm`].
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P: Platform> Default for Efr32Driver<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Platform> Driver for Efr32Driver<P> {
    fn now(&self) -> u64 {
        P::ticks_64()
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        if let Some(deadline) = alarm::store(at, waker) {
            // The value written to the comparator comes from tested code, not from here: a 64-bit
            // deadline does not fit a 32-bit compare, and the two rules that matter (a past
            // deadline fires now, a far one is clamped and re-armed) live in `ticks`.
            let (value, rearm) = ticks::comparator_value(self.now(), deadline);
            P::arm(value, rearm);
        }
    }
}

/// Call from the platform's interrupt when the comparator fires.
pub fn on_alarm<P: Platform>() {
    alarm::take_expired(P::ticks_64);
}

/// Installs the driver for `embassy-time`. Use once, in the BSP, after the clock is running.
#[macro_export]
macro_rules! time_driver {
    ($platform:ty) => {
        $crate::__install!($platform);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __install {
    ($platform:ty) => {
        embassy_time_driver::time_driver_impl!(
            static DRIVER: $crate::Efr32Driver<$platform> = $crate::Efr32Driver::new()
        );
    };
}
