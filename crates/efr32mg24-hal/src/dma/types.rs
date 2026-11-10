//! DMA Type Definitions
//!
//! This module contains type definitions, enums, and traits used by the DMA HAL.

/// DMA Transfer Size
///
/// Specifies the unit size for each DMA transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransferSize {
    /// 8-bit byte transfers
    Byte = 0,
    /// 16-bit halfword transfers
    Halfword = 1,
    /// 32-bit word transfers
    Word = 2,
}

/// DMA Error Types
///
/// Errors that can occur during DMA operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DmaError {
    /// Source and destination slices have different lengths, or length is invalid (0 or >2047)
    InvalidLength,
    /// Channel is already busy with another transfer
    Busy,
    /// Transfer did not complete within timeout period
    Timeout,
    /// Operation not supported (e.g., channel number >0 in Phase 1)
    Unsupported,
    /// Invalid buffer address or alignment
    InvalidAddress,
}

/// Peripheral DMA Request Source
///
/// Defines which peripheral can trigger DMA transfers.
/// These values map to LDMAXBAR SOURCESEL/SIGSEL fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PeripheralRequest {
    /// No peripheral request (software triggered only)
    None = 0,

    /// IADC Single conversion complete
    /// SOURCESEL = 0x30 (IADC0), SIGSEL = 0 (IADC_SINGLE)
    IadcSingle = 1,

    /// IADC Scan conversion complete
    /// SOURCESEL = 0x30 (IADC0), SIGSEL = 1 (IADC_SCAN)
    IadcScan = 2,

    /// USART0 RX Data Valid
    /// SOURCESEL = 0x0C (USART0), SIGSEL = 0 (RXDATAV)
    Usart0Rx = 3,

    /// USART0 TX Buffer Level
    /// SOURCESEL = 0x0C (USART0), SIGSEL = 1 (TXBL)
    Usart0Tx = 4,
}

impl PeripheralRequest {
    /// Returns (SOURCESEL, SIGSEL) for LDMAXBAR configuration
    pub(crate) fn to_xbar_config(self) -> (u8, u8) {
        match self {
            PeripheralRequest::None => (0, 0),
            PeripheralRequest::IadcSingle => (0x30, 0),
            PeripheralRequest::IadcScan => (0x30, 1),
            PeripheralRequest::Usart0Rx => (0x0C, 0),
            PeripheralRequest::Usart0Tx => (0x0C, 1),
        }
    }
}

/// Trait for types that can be transferred via DMA
///
/// This trait defines types that have a valid DMA transfer size.
pub trait TransferElement: Sized {
    /// The DMA transfer size for this type
    const SIZE: TransferSize;
}

impl TransferElement for u8 {
    const SIZE: TransferSize = TransferSize::Byte;
}

impl TransferElement for i8 {
    const SIZE: TransferSize = TransferSize::Byte;
}

impl TransferElement for u16 {
    const SIZE: TransferSize = TransferSize::Halfword;
}

impl TransferElement for i16 {
    const SIZE: TransferSize = TransferSize::Halfword;
}

impl TransferElement for u32 {
    const SIZE: TransferSize = TransferSize::Word;
}

impl TransferElement for i32 {
    const SIZE: TransferSize = TransferSize::Word;
}
