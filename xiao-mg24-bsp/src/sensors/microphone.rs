//! MSM381ACT001 Analog MEMS Microphone Driver
//!
//! Driver for the MSM381ACT001 analog MEMS microphone on XIAO MG24 Sense.
//!
//! # Hardware
//!
//! - **Model**: MSM381ACT001 (Analog MEMS Microphone)
//! - **Power Control**: PC8 (GPIO output, active high)
//! - **Analog Input**: PC9 (IADC channel)
//! - **Sensitivity**: -26 dBV (@ 1 kHz, 94 dB SPL)
//! - **Frequency Range**: 100 Hz - 10 kHz
//! - **SNR**: 58 dB (A-weighted)
//!
//! # Architecture
//!
//! ```text
//! MSM381ACT001 → PC9 (Analog) → IADC0 → DMA → Circular Buffer
//!      ↑
//!     PC8 (Power Enable)
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use xiao_mg24_bsp::{Board, sensors::Microphone};
//! use efr32mg24_hal::dma::Dma;
//!
//! let mut board = Board::take().unwrap();
//! let dma = Dma::new(board.peripherals.ldma_s);
//! let mut delay = board.into_delay();
//!
//! // Create microphone driver (16 kHz, 512 samples)
//! let mut mic = Microphone::new(
//!     board.pins.pc8,  // Power control
//!     board.pins.pc9,  // Analog input
//!     &board.clocks,
//!     dma,
//!     512
//! );
//!
//! // Start continuous capture
//! mic.start(16_000)?;  // 16 kHz sampling
//!
//! // Read samples
//! loop {
//!     if let Some(samples) = mic.read() {
//!         // Process audio data
//!         process_audio(samples);
//!     }
//!     delay.delay_ms(10);
//! }
//!
//! // Stop when done
//! mic.stop();
//! ```
//!
//! # Sampling Rates
//!
//! Supported rates: 8 kHz, 16 kHz, 32 kHz, 48 kHz
//!
//! # Buffer Size
//!
//! - Larger buffers: Lower CPU overhead, higher latency
//! - Smaller buffers: Higher CPU overhead, lower latency
//! - Recommended: 256-1024 samples for speech, 128-256 for voice control

use crate::pins::{Pc8, Pc9};
use efr32mg24_hal::dma::{Dma, PeripheralRequest};
use efr32mg24_hal::pac;
use embedded_hal::digital::OutputPin;

/// Microphone driver error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicError {
    /// Invalid sampling rate (must be 8k, 16k, 32k, or 48k Hz)
    InvalidSampleRate,
    /// Invalid buffer size (must be 64-2047 samples)
    InvalidBufferSize,
    /// DMA error
    DmaError,
    /// Microphone already running
    AlreadyRunning,
    /// Microphone not running
    NotRunning,
}

/// MSM381ACT001 Microphone Driver
///
/// Provides continuous audio capture using IADC + DMA.
pub struct Microphone {
    /// Power control pin (PC8)
    power_pin: Pc8<efr32mg24_hal::gpio::Output<efr32mg24_hal::gpio::PushPull>>,

    /// DMA controller
    dma: Dma,

    /// Sample buffer (heap allocation required)
    buffer: &'static mut [u16],

    /// Sampling rate in Hz
    sample_rate: u32,

    /// Running state
    running: bool,

    /// Last read position
    last_read_pos: usize,
}

impl Microphone {
    /// Create new microphone driver
    ///
    /// # Arguments
    ///
    /// * `power_pin` - PC8 pin for microphone power control
    /// * `_analog_pin` - PC9 pin (analog input, consumed for type safety)
    /// * `clocks` - Clock configuration (for IADC timing)
    /// * `dma` - DMA controller
    /// * `buffer_size` - Number of samples in circular buffer (64-2047)
    ///
    /// # Errors
    ///
    /// Returns `MicError::InvalidBufferSize` if buffer size is out of range.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let mut mic = Microphone::new(
    ///     board.pins.pc8,
    ///     board.pins.pc9,
    ///     &board.clocks,
    ///     dma,
    ///     512
    /// )?;
    /// ```
    pub fn new(
        mut power_pin: Pc8<efr32mg24_hal::gpio::Output<efr32mg24_hal::gpio::PushPull>>,
        _analog_pin: Pc9<efr32mg24_hal::gpio::Analog>,
        _clocks: &efr32mg24_hal::clock::FrozenClocks,
        ldma: pac::LdmaS,
        buffer_size: usize,
    ) -> Result<Self, MicError> {
        // Validate buffer size
        if buffer_size < 64 || buffer_size > 2047 {
            return Err(MicError::InvalidBufferSize);
        }

        // Allocate buffer (requires heap - leak for 'static lifetime)
        // TODO: Use static allocation or proper heap allocator
        let buffer = unsafe {
            // For now, use a static buffer
            // In production, use proper allocator or const generics
            static mut MIC_BUFFER: [u16; 1024] = [0; 1024];
            &mut MIC_BUFFER[..buffer_size]
        };

        // Power off microphone initially
        power_pin.set_low().ok();

        // Create DMA controller
        let dma = Dma::new(ldma);

        Ok(Self {
            power_pin,
            dma,
            buffer,
            sample_rate: 0,
            running: false,
            last_read_pos: 0,
        })
    }

    /// Start continuous audio capture
    ///
    /// Powers on the microphone, configures IADC, and starts DMA transfer
    /// to circular buffer.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sampling frequency in Hz (8000, 16000, 32000, or 48000)
    ///
    /// # Errors
    ///
    /// Returns `MicError::InvalidSampleRate` for unsupported rates.
    /// Returns `MicError::AlreadyRunning` if already capturing.
    ///
    /// # Example
    ///
    /// ```no_run
    /// mic.start(16_000)?;  // 16 kHz sampling
    /// ```
    pub fn start(&mut self, sample_rate: u32) -> Result<(), MicError> {
        // Validate sample rate
        match sample_rate {
            8_000 | 16_000 | 32_000 | 48_000 => {}
            _ => return Err(MicError::InvalidSampleRate),
        }

        if self.running {
            return Err(MicError::AlreadyRunning);
        }

        // Power on microphone
        self.power_pin.set_high().ok();

        // Wait for microphone to stabilize (10ms typical)
        // TODO: Use proper delay
        cortex_m::asm::delay(780_000); // ~10ms at 78 MHz

        // Configure IADC for analog input
        self.configure_iadc(sample_rate);

        // Start DMA transfer
        let iadc_data_addr = 0x4808_4028; // IADC0.SCANFIFODATA
        let mut ch0 = self.dma.channel0();

        ch0.start_peripheral_to_memory(
            iadc_data_addr,
            self.buffer,
            PeripheralRequest::IadcScan,
        )
        .map_err(|_| MicError::DmaError)?;

        self.sample_rate = sample_rate;
        self.running = true;
        self.last_read_pos = 0;

        Ok(())
    }

    /// Stop audio capture
    ///
    /// Stops DMA, disables IADC, and powers off microphone.
    ///
    /// # Example
    ///
    /// ```no_run
    /// mic.stop();
    /// ```
    pub fn stop(&mut self) {
        if !self.running {
            return;
        }

        // Stop DMA
        let mut ch0 = self.dma.channel0();
        ch0.stop();

        // Disable IADC
        unsafe {
            let iadc = &(*pac::Iadc0S::ptr());
            iadc.en().write(|w| w.en().clear_bit());
        }

        // Power off microphone
        self.power_pin.set_low().ok();

        self.running = false;
    }

    /// Read available audio samples
    ///
    /// Returns samples captured since last read. Non-blocking.
    ///
    /// # Returns
    ///
    /// - `Some(&[u16])` - Slice of new samples (0 if none available)
    /// - `None` - Microphone not running
    ///
    /// # Example
    ///
    /// ```no_run
    /// loop {
    ///     if let Some(samples) = mic.read() {
    ///         if samples.len() > 0 {
    ///             process_audio(samples);
    ///         }
    ///     }
    ///     delay.delay_ms(10);
    /// }
    /// ```
    pub fn read(&mut self) -> Option<&[u16]> {
        if !self.running {
            return None;
        }

        // TODO: Get current DMA position
        // For now, return empty slice (position tracking not yet implemented)
        Some(&self.buffer[0..0])
    }

    /// Get current sampling rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Check if microphone is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get buffer size
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }

    /// Configure IADC for microphone input
    fn configure_iadc(&self, sample_rate: u32) {
        unsafe {
            let cmu = &(*pac::CmuS::ptr());
            let iadc = &(*pac::Iadc0S::ptr());

            // Enable IADC clock
            cmu.clken0().modify(|_, w| w.iadc0().set_bit());

            // Enable IADC peripheral
            iadc.en().write(|w| w.en().set_bit());

            // Wait for IADC to be ready
            while !iadc.status().read().adcwarm().bit_is_set() {}

            // Configure IADC for audio sampling
            // - 12-bit resolution
            // - Internal 1.21V reference
            // - Continuous scan mode
            iadc.cfg0().write(|w| {
                w.adcmode()
                    .normal()
                    .osrhs()
                    .hispd2() // 2x oversampling
                    .analoggain()
                    .anagain1() // 1x gain
                    .refsel()
                    .vbgr() // 1.21V reference
                    .digavg()
                    .avg1() // No averaging
            });

            // Calculate IADC clock divider for desired sample rate
            // IADC clock needs to be ~13x sample rate for 12-bit conversion
            // With 78 MHz PCLK:
            // - 16 kHz → 208 kHz IADC clock → divider = 375
            // - 48 kHz → 624 kHz IADC clock → divider = 125
            let iadc_clock = sample_rate * 13;
            let pclk = 78_000_000; // 78 MHz
            let divider = (pclk / iadc_clock).max(1).min(4096) as u16;

            // Set prescaler
            iadc.cfg1().write(|w| w.presc().bits(divider - 1));

            // Configure scan table entry 0: PC9 (single-ended)
            // PC9 = Port C, Pin 9
            iadc.scantable().write(|w| w.bits(0x09));

            // Configure scan FIFO
            iadc.scanfifocfg().write(|w| {
                w.alignment()
                    .right12() // Right-align 12-bit data
                    .dmawufifosingle()
                    .set_bit() // DMA wake-up on single entry
            });

            // Start continuous scanning
            iadc.cmd().write(|w| w.scanstart().set_bit());
        }
    }
}

impl Drop for Microphone {
    fn drop(&mut self) {
        self.stop();
    }
}
