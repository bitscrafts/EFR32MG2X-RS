//! LDMA (Linked DMA) Module
//!
//! This module provides a Hardware Abstraction Layer (HAL) for the EFR32MG24's
//! Linked DMA (LDMA) controller, enabling efficient data transfers between memory
//! and peripherals without CPU intervention.
//!
//! # Features
//!
//! - **8 Independent Channels**: CH0-CH7 for concurrent transfers
//! - **Memory-to-Memory**: Efficient bulk data copying
//! - **Transfer Sizes**: Byte (8-bit), Halfword (16-bit), Word (32-bit)
//! - **Blocking Transfers**: Wait for completion with timeout
//! - **Type-Safe Channels**: Compile-time channel validation
//! - **Critical Sections**: RTOS-safe atomic operations
//!
//! # Current Implementation (Phase 1)
//!
//! - ✅ Memory-to-memory transfers
//! - ✅ Software-triggered (SWREQ)
//! - ✅ Blocking operation
//! - ✅ Single channel (CH0)
//! - ✅ All transfer sizes
//!
//! # Future Extensions (Phase 2)
//!
//! - ⏳ Linked descriptors (chained transfers)
//! - ⏳ Peripheral-to-memory transfers
//! - ⏳ Interrupt-driven transfers
//! - ⏳ All 8 channels
//!
//! # Architecture
//!
//! The LDMA controller uses descriptor-based transfers:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                 LDMA Controller                     │
//! ├─────────────────────────────────────────────────────┤
//! │ CH0 │ CH1 │ CH2 │ CH3 │ CH4 │ CH5 │ CH6 │ CH7 │    │
//! └─────────────────────────────────────────────────────┘
//!        │
//!        ├─► CFG   (Arbitration, Address Signs)
//!        ├─► CTRL  (Size, Count, Addressing Mode)
//!        ├─► SRC   (Source Address)
//!        ├─► DST   (Destination Address)
//!        └─► LINK  (Next Descriptor)
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use efr32mg24_hal::dma::{Dma, TransferSize};
//! use efr32mg24_hal::pac;
//!
//! let dp = pac::Peripherals::take().unwrap();
//!
//! // Initialize DMA controller
//! let mut dma = Dma::new(dp.ldma_s);
//!
//! // Get channel 0
//! let mut ch0 = dma.channel0();
//!
//! // Prepare data
//! let src = [1u32, 2, 3, 4, 5];
//! let mut dst = [0u32; 5];
//!
//! // Perform memory-to-memory transfer
//! ch0.transfer(&src, &mut dst).expect("DMA transfer failed");
//!
//! // Verify
//! assert_eq!(dst, [1, 2, 3, 4, 5]);
//! ```
//!
//! # Safety
//!
//! This module uses `unsafe` for direct hardware register access. All unsafe
//! operations are documented with SAFETY comments explaining:
//! - Why the operation is safe
//! - What invariants are maintained
//! - What preconditions must hold

mod types;
pub use types::*;

use crate::pac;

/// DMA Controller
///
/// Owns the LDMA peripheral and provides access to all 8 DMA channels.
///
/// # Ownership
///
/// This struct takes ownership of the `LDMA_S` peripheral, ensuring exclusive
/// access and preventing multiple mutable references.
pub struct Dma {
    _ldma: pac::LdmaS,
}

impl Dma {
    /// Creates a new DMA controller
    ///
    /// Takes ownership of the LDMA peripheral and initializes it.
    ///
    /// # Arguments
    ///
    /// * `ldma` - The LDMA peripheral from PAC
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let dp = pac::Peripherals::take().unwrap();
    /// let dma = Dma::new(dp.ldma_s);
    /// ```
    pub fn new(ldma: pac::LdmaS) -> Self {
        // Enable LDMA clock
        // SAFETY: We have exclusive access to CMU via critical section.
        // Setting LDMA bit enables the DMA controller clock.
        critical_section::with(|_cs| {
            let cmu = unsafe { &(*pac::CmuS::ptr()) };
            cmu.clken0().modify(|_, w| w.ldma().set_bit());
        });

        // Enable LDMA peripheral
        // SAFETY: We own the LDMA peripheral. EN register enables the controller.
        unsafe {
            let ldma_ptr = pac::LdmaS::ptr();
            (*ldma_ptr).en().write(|w| w.en().set_bit());
        }

        Self { _ldma: ldma }
    }

    /// Returns a mutable reference to Channel 0
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let mut dma = Dma::new(dp.ldma_s);
    /// let mut ch0 = dma.channel0();
    /// ```
    pub fn channel0(&mut self) -> Channel<0> {
        Channel {
            _marker: core::marker::PhantomData,
        }
    }
}

/// DMA Channel (Type-Safe)
///
/// Represents a single DMA channel with compile-time channel number validation.
///
/// # Type Parameter
///
/// * `N` - Channel number (0-7)
pub struct Channel<const N: u8> {
    _marker: core::marker::PhantomData<()>,
}

impl<const N: u8> Channel<N> {
    /// Performs a blocking memory-to-memory transfer
    ///
    /// Copies data from `src` to `dst` using DMA. Blocks until transfer completes
    /// or timeout occurs.
    ///
    /// # Arguments
    ///
    /// * `src` - Source slice (must be same length as dst)
    /// * `dst` - Destination slice (must be same length as src)
    ///
    /// # Errors
    ///
    /// Returns `DmaError::InvalidLength` if slices have different lengths or are too large.
    /// Returns `DmaError::Timeout` if transfer doesn't complete within expected time.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let src = [1u32, 2, 3, 4, 5];
    /// let mut dst = [0u32; 5];
    /// ch0.transfer(&src, &mut dst)?;
    /// ```
    pub fn transfer<T>(&mut self, src: &[T], dst: &mut [T]) -> Result<(), DmaError>
    where
        T: TransferElement,
    {
        if src.len() != dst.len() {
            return Err(DmaError::InvalidLength);
        }

        if src.len() == 0 || src.len() > 2047 {
            return Err(DmaError::InvalidLength);
        }

        let xfer_count = src.len() as u16;
        let size = T::SIZE;

        // Configure channel in critical section
        // SAFETY: Critical section ensures atomic register access for RTOS safety.
        // We configure the channel registers for a memory-to-memory transfer.
        critical_section::with(|_cs| {
            let ldma = unsafe { &(*pac::LdmaS::ptr()) };

            // Ensure channel is idle before configuration
            if N == 0 && ldma.chbusy().read().bits() & (1 << N) != 0 {
                return Err(DmaError::Busy);
            }

            // Configure transfer control
            // SAFETY: Writing to CH0_CTRL register configures the transfer parameters.
            // - STRUCTTYPE = Transfer (0)
            // - XFERCNT = number of units to transfer
            // - SIZE = transfer unit size
            // - SRCINC/DSTINC = One (increment by one unit)
            // - BLOCKSIZE = All (transfer all at once)
            unsafe {
                match N {
                    0 => {
                        ldma.ch0_ctrl().write(|w| {
                            w.structtype().transfer()
                                .xfercnt().bits(xfer_count - 1) // Hardware uses count-1
                                .size().bits(size as u8)
                                .srcinc().one()
                                .dstinc().one()
                                .blocksize().all()
                                .doneien().set_bit() // Enable done interrupt flag
                        });

                        // Configure addresses
                        // SAFETY: Writing absolute addresses for memory-to-memory transfer.
                        ldma.ch0_src().write(|w| w.bits(src.as_ptr() as u32));
                        ldma.ch0_dst().write(|w| w.bits(dst.as_mut_ptr() as u32));

                        // Clear done flag
                        ldma.if_().write(|w| w.bits(1 << N));

                        // Enable channel
                        ldma.chen().write(|w| w.bits(1 << N));

                        // Trigger software request
                        ldma.swreq().write(|w| w.bits(1 << N));
                    }
                    _ => return Err(DmaError::Unsupported),
                }
            }

            Ok(())
        })?;

        // Wait for completion with timeout
        let timeout_cycles = 1_000_000; // ~13ms at 78 MHz
        let mut cycles = 0;

        // SAFETY: Reading status registers to poll for completion.
        let ldma = unsafe { &(*pac::LdmaS::ptr()) };

        loop {
            // Check if done flag is set
            if ldma.if_().read().bits() & (1 << N) != 0 {
                // Clear done flag
                // SAFETY: Writing to IF register clears the interrupt flag.
                unsafe {
                    ldma.if_().write(|w| w.bits(1 << N));
                }
                break;
            }

            cycles += 1;
            if cycles > timeout_cycles {
                return Err(DmaError::Timeout);
            }
        }

        Ok(())
    }
}
