# EFR32MG24 HAL Structure Plan

**Date**: December 4, 2025
**Status**: Tier 1 Complete - GPIO, CMU, Delay Implemented
**Target Hardware**: Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)

---

## Overview

This document outlines the structure and implementation strategy for the EFR32MG24 Hardware Abstraction Layer (HAL). The HAL provides safe, ergonomic Rust APIs over the low-level PAC register access.

**Current Status**: Tier 1 peripherals (GPIO, CMU, Delay) are fully implemented with hardware register access. All examples compile and build to flashable ARM Cortex-M33 binaries.

## Design Principles

### 1. Type Safety
- Use Rust's type system to prevent misuse at compile time
- Zero-cost abstractions where possible
- Ownership-based resource management (pins, peripherals)

### 2. embedded-hal Compatibility
- Implement `embedded-hal` v1.0 traits for maximum ecosystem compatibility
- Enable use with existing drivers and libraries
- Support both blocking and async operations (async via feature flag)

### 3. Ergonomics
- Builder pattern for peripheral configuration
- Sensible defaults for common use cases
- Clear error types and handling

### 4. No Dynamic Allocation
- `#![no_std]` compatible
- Static resource management
- Compile-time peripheral configuration where possible

### 5. TrustZone-M Awareness
- Support for secure and non-secure peripheral access
- Type-safe security attribute configuration
- Document security implications

## Crate Structure (Current Implementation)

```
crates/efr32mg24-hal/
├── Cargo.toml
├── README.md
├── memory.x                # Linker memory layout (1536 KB Flash, 256 KB RAM)
├── device.x                # Interrupt vector table (77 interrupts)
├── build.rs                # Build script (copies linker scripts)
├── .cargo/config.toml      # Cortex-M33 linker configuration
├── docs/
│   ├── README.md           # Module documentation index
│   ├── STATUS.md           # HAL implementation status
│   ├── PHASE2_PLAN.md      # Phase 2 completion details
│   ├── BUILD_SYSTEM.md     # Build pipeline documentation
│   └── LINKER_SETUP.md     # Linker configuration guide
├── examples/               # ✅ 3 examples implemented
│   ├── 01_clock.rs         # Clock configuration (HFXO/HFRCO/LFXO)
│   ├── 02_delay.rs         # Delay usage (ms/us/ns)
│   └── 03_gpio.rs          # GPIO LED control and button input
└── src/
    ├── lib.rs             # Crate root, re-exports
    ├── prelude.rs         # Common imports
    │
    ├── clock/             # ✅ CMU - Clock Management (Tier 1 COMPLETE)
    │   ├── mod.rs         # Module coordinator (73 lines)
    │   ├── types.rs       # Type definitions (66 lines)
    │   ├── clocks.rs      # Clock configuration (138 lines)
    │   ├── frozen.rs      # FrozenClocks (40 lines)
    │   └── README.md      # CMU documentation
    │
    ├── delay/             # ✅ Delay - SysTick delays (Tier 1 COMPLETE)
    │   ├── mod.rs         # SysTick implementation (~100 lines)
    │   └── README.md      # Delay documentation
    │
    ├── gpio/              # ✅ GPIO - Digital I/O (Tier 1 COMPLETE)
    │   ├── mod.rs         # Module coordinator (104 lines)
    │   ├── types.rs       # Type definitions (155 lines)
    │   ├── pin.rs         # Pin implementation (248 lines)
    │   ├── traits.rs      # embedded-hal traits (110 lines)
    │   └── README.md      # GPIO documentation
    │
    ├── serial.rs          # 🚧 USART/UART (Tier 2 PLANNED)
    ├── i2c.rs             # 🚧 I2C (Tier 2 PLANNED)
    ├── spi.rs             # 🚧 SPI (Tier 2 PLANNED)
    ├── timer.rs           # 🚧 Timers/PWM (Tier 2 PLANNED)
    ├── adc.rs             # ⏳ IADC (Tier 3 PLANNED)
    ├── dma.rs             # ⏳ LDMA (Tier 3 PLANNED)
    ├── rtc.rs             # ⏳ RTC/RTCC (Tier 3 PLANNED)
    ├── prs.rs             # ⏳ Peripheral Reflex System (Tier 3 PLANNED)
    │
    ├── power.rs           # ⏳ EMU (Tier 3 PLANNED)
    ├── watchdog.rs        # ⏳ WDOG (Tier 3 PLANNED)
    ├── flash.rs           # ⏳ Flash operations (Tier 3 PLANNED)
    ├── signature.rs       # ⏳ Chip signature (Tier 3 PLANNED)
    │
    ├── radio/             # ⏳ Radio subsystem (Future)
    │   ├── mod.rs
    │   ├── ble.rs         # Bluetooth LE
    │   ├── ieee802154.rs  # Thread/Zigbee/Matter
    │   └── modem.rs       # Modem configuration
    │
    └── crypto/            # ⏳ Cryptographic hardware (Future)
        ├── mod.rs
        ├── aes.rs         # RADIOAES
        └── trng.rs        # True Random Number Generator
```

**Legend**:
- ✅ Complete - Fully implemented with hardware register access
- 🚧 Planned - Tier 2, next implementation priority
- ⏳ Future - Tier 3 or beyond

## Module Documentation Requirements

Each module MUST have its own `README.md` file (per project requirements) containing:
- Module purpose and capabilities
- Usage examples
- Hardware considerations
- Performance characteristics
- Safety and security notes (where applicable)

## Implementation Status

### ✅ Tier 1 Complete (December 4, 2025)

**GPIO Module** (617 lines across 4 files):
- Hardware register access (MODEL/MODEH, DOUT, DIN)
- Type-safe pin modes with compile-time enforcement
- embedded-hal v1.0 OutputPin/InputPin traits
- Critical section protection for atomic operations
- Module README with hardware register documentation

**CMU Module** (317 lines across 4 files):
- SYSCLKCTRL register for clock source selection
- HFXO/HFRCO/LFXO/LFRCO configuration
- Peripheral consumption pattern
- FrozenClocks for frequency tracking
- Module README with clock tree documentation

**Delay Module** (~100 lines):
- SysTick-based blocking delays
- Millisecond/microsecond/nanosecond precision
- embedded-hal v1.0 DelayNs trait
- Integration with CMU clock frequencies
- Module README with timing accuracy notes

**Examples** (3 working examples):
- 01_clock.rs - Clock configuration
- 02_delay.rs - Delay usage
- 03_gpio.rs - GPIO LED and button control

**Build System**:
- Linker configuration (memory.x, device.x)
- Build script (build.rs)
- All examples build to flashable binaries (707-725 KB)

### 🚧 Tier 2 Next (Planned)

**Priority**: Communication peripherals
- USART/EUSART - Serial communication
- I2C - I2C master mode
- SPI - SPI master mode
- Timers - Timer and PWM functionality

### ⏳ Tier 3 Future

**Priority**: Advanced features
- ADC (IADC) - Analog-to-digital conversion
- DMA (LDMA) - Direct memory access
- EMU - Energy management and low-power modes
- Radio - Bluetooth, Zigbee, Thread, Matter support

---

## Implementation Phases (Original Plan)

### Phase 1: Core Infrastructure (Week 1) - ✅ COMPLETE
**Priority**: Foundation for all other modules

#### 1.1 Clock Management (`clock/`) - ✅ COMPLETE
```rust
pub struct Clocks {
    pub hfclk: Hertz,
    pub lfclk: Hertz,
    pub pclk: Hertz,
}

pub struct ClockConfig {
    // HFXO (High Frequency Crystal Oscillator) configuration
    pub hfxo: Option<HfxoConfig>,
    // LFXO (Low Frequency Crystal Oscillator) configuration
    pub lfxo: Option<LfxoConfig>,
}

impl Clocks {
    pub fn new(cmu: CMU_S, config: ClockConfig) -> Self;
    pub fn freeze(self) -> FrozenClocks;
}
```

**Features**:
- High-frequency clock configuration (HFXO, HFRCO)
- Low-frequency clock configuration (LFXO, LFRCO)
- Peripheral clock enables/disables
- Clock frequency calculation
- Type-safe frozen clocks (cannot be modified after freeze)

**Dependencies**: PAC only

#### 1.2 GPIO (`gpio.rs`)
```rust
pub struct Pin<MODE> {
    pin: u8,
    port: Port,
    _mode: PhantomData<MODE>,
}

// Type states for compile-time mode checking
pub struct Input<MODE> { _mode: PhantomData<MODE> }
pub struct Output<MODE> { _mode: PhantomData<MODE> }
pub struct Analog;
pub struct Alternate<const N: u8>;

// Input modes
pub struct Floating;
pub struct PullUp;
pub struct PullDown;

// Output modes
pub struct PushPull;
pub struct OpenDrain;

impl<MODE> Pin<Input<MODE>> {
    pub fn into_pull_up_input(self) -> Pin<Input<PullUp>>;
    pub fn is_high(&self) -> bool;
    pub fn is_low(&self) -> bool;
}

impl<MODE> Pin<Output<MODE>> {
    pub fn set_high(&mut self);
    pub fn set_low(&mut self);
    pub fn toggle(&mut self);
}
```

**Features**:
- Type-safe pin modes (compile-time enforcement)
- embedded-hal digital I/O traits
- Interrupt configuration
- Drive strength configuration
- Slew rate configuration

**embedded-hal traits**:
- `InputPin`
- `OutputPin`
- `ToggleableOutputPin`

**Dependencies**: PAC, Clocks

#### 1.3 Delay (`delay.rs`)
```rust
pub struct Delay {
    // Uses SYSTICK or TIMER for delays
}

impl Delay {
    pub fn new(syst: cortex_m::peripheral::SYST, clocks: &Clocks) -> Self;
}

// embedded-hal traits
impl DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32);
    fn delay_us(&mut self, us: u32);
    fn delay_ms(&mut self, ms: u32);
}
```

**Features**:
- Microsecond delays using SYSTICK
- Millisecond delays
- embedded-hal `DelayNs` trait

**Dependencies**: cortex-m, Clocks

### Phase 2: Communication Peripherals (Week 2-3)

#### 2.1 Serial/USART (`serial.rs`)
```rust
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

pub struct Tx<USART>;
pub struct Rx<USART>;

pub struct Config {
    pub baudrate: Bps,
    pub parity: Parity,
    pub stopbits: StopBits,
    pub databits: DataBits,
    pub flow_control: FlowControl,
}

impl<USART, PINS> Serial<USART, PINS> {
    pub fn new(usart: USART, pins: PINS, config: Config, clocks: &Clocks) -> Self;
    pub fn split(self) -> (Tx<USART>, Rx<USART>);
    pub fn release(self) -> (USART, PINS);
}
```

**Features**:
- Multiple USART instances (USART0, USART1, etc.)
- Configurable baud rate, parity, stop bits
- Hardware flow control (CTS/RTS)
- DMA support (future)
- embedded-hal serial traits

**embedded-hal traits**:
- `serial::Read`
- `serial::Write`
- `serial::ErrorType`

**Dependencies**: PAC, GPIO, Clocks

#### 2.2 I2C (`i2c.rs`)
```rust
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

pub struct Config {
    pub speed: Speed,
    pub timeout: Option<Timeout>,
}

pub enum Speed {
    Standard,      // 100 kHz
    Fast,          // 400 kHz
    FastPlus,      // 1 MHz
}

impl<I2C, PINS> I2c<I2C, PINS> {
    pub fn new(i2c: I2C, pins: PINS, config: Config, clocks: &Clocks) -> Self;
}
```

**Features**:
- Master mode (standard, fast, fast-plus)
- 7-bit and 10-bit addressing
- embedded-hal I2C traits
- Error handling (NACK, arbitration loss, etc.)

**embedded-hal traits**:
- `i2c::I2c`
- `i2c::ErrorType`

**Dependencies**: PAC, GPIO, Clocks

#### 2.3 SPI (`spi.rs`)
```rust
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

pub struct Config {
    pub mode: Mode,
    pub frequency: Hertz,
    pub bit_order: BitOrder,
}

impl<SPI, PINS> Spi<SPI, PINS> {
    pub fn new(spi: SPI, pins: PINS, config: Config, clocks: &Clocks) -> Self;
}
```

**Features**:
- Master mode
- All SPI modes (0-3)
- Configurable clock frequency
- MSB/LSB first
- embedded-hal SPI traits

**embedded-hal traits**:
- `spi::SpiBus`
- `spi::ErrorType`

**Dependencies**: PAC, GPIO, Clocks

### Phase 3: Timers and ADC (Week 4)

#### 3.1 Timers (`timer.rs`)
```rust
pub struct Timer<TIM> {
    tim: TIM,
}

pub struct Config {
    pub frequency: Hertz,
    pub mode: Mode,
}

pub enum Mode {
    OneShot,
    Periodic,
}

impl<TIM> Timer<TIM> {
    pub fn new(tim: TIM, config: Config, clocks: &Clocks) -> Self;
    pub fn start(&mut self, timeout: Duration);
    pub fn wait(&mut self) -> nb::Result<(), Void>;
    pub fn cancel(&mut self);
}
```

**Features**:
- TIMER0-4 support
- PWM generation
- Input capture
- Output compare
- Interrupt support

**embedded-hal traits**:
- `timer::CountDown`
- `pwm::SetDutyCycle`

**Dependencies**: PAC, Clocks

#### 3.2 ADC (`adc.rs`)
```rust
pub struct Adc {
    iadc: IADC0_S,
}

pub struct Config {
    pub reference: Reference,
    pub resolution: Resolution,
    pub oversample: Oversample,
}

impl Adc {
    pub fn new(iadc: IADC0_S, config: Config, clocks: &Clocks) -> Self;
    pub fn read<PIN>(&mut self, pin: &PIN) -> u16;
}
```

**Features**:
- Single-ended and differential measurements
- Configurable reference voltage
- Oversampling
- Multiple input channels

**embedded-hal traits**:
- `adc::OneShot`

**Dependencies**: PAC, GPIO, Clocks

### Phase 4: Power Management (Week 5)

#### 4.1 Power/EMU (`power.rs`)
```rust
pub struct PowerMode;

impl PowerMode {
    pub fn em0() -> Em0;  // Active
    pub fn em1() -> Em1;  // Sleep
    pub fn em2() -> Em2;  // Deep sleep
    pub fn em3() -> Em3;  // Stop
}

pub fn enter_em1();
pub fn enter_em2(config: Em2Config);
pub fn enter_em3(config: Em3Config);
```

**Features**:
- Energy mode transitions (EM0-EM3)
- Peripheral state preservation
- Wakeup source configuration
- DCDC configuration
- Type-safe energy mode APIs

**Dependencies**: PAC, Clocks

### Phase 5: Advanced Features (Future)

#### 5.1 DMA (`dma.rs`)
- LDMA (Linked DMA) support
- Memory-to-memory transfers
- Peripheral-to-memory transfers
- Scatter-gather operations

#### 5.2 Radio (`radio/`)
- BLE stack integration (future, may use C library bindings)
- IEEE 802.15.4 (Thread/Zigbee/Matter)
- Packet handling
- RF calibration

#### 5.3 Crypto (`crypto/`)
- AES encryption/decryption
- True random number generation
- Secure Vault integration

## embedded-hal Trait Coverage

### Target embedded-hal v1.0 Traits

| Trait | Module | Priority | Status |
|-------|--------|----------|--------|
| `digital::InputPin` | gpio | P0 | ⏳ Planned |
| `digital::OutputPin` | gpio | P0 | ⏳ Planned |
| `digital::ToggleableOutputPin` | gpio | P0 | ⏳ Planned |
| `delay::DelayNs` | delay | P0 | ⏳ Planned |
| `serial::Read` | serial | P1 | ⏳ Planned |
| `serial::Write` | serial | P1 | ⏳ Planned |
| `i2c::I2c` | i2c | P1 | ⏳ Planned |
| `spi::SpiBus` | spi | P1 | ⏳ Planned |
| `pwm::SetDutyCycle` | timer | P2 | ⏳ Planned |
| `adc::OneShot` | adc | P2 | ⏳ Planned |

## Error Handling Strategy

### Error Types

Each module defines its own error type with specific failure modes:

```rust
// Example: I2C errors
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Arbitration loss
    ArbitrationLoss,
    /// No acknowledge received
    Nack,
    /// Bus error
    BusError,
    /// Timeout
    Timeout,
    /// Overrun
    Overrun,
}
```

### embedded-hal Error Mapping

```rust
impl embedded_hal::i2c::Error for Error {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        match self {
            Error::ArbitrationLoss => ErrorKind::ArbitrationLoss,
            Error::Nack => ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown),
            Error::BusError => ErrorKind::Bus,
            Error::Timeout | Error::Overrun => ErrorKind::Other,
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Mock PAC peripherals where possible
- Test configuration builders
- Test type state transitions

### Integration Tests
- Require actual hardware (XIAO MG24 Sense)
- Test peripheral interactions
- Verify timing and electrical characteristics

### Examples as Tests
- Each example should be a minimal working program
- Documented with expected behavior
- Can be used as smoke tests on hardware

## Documentation Requirements

### Module-Level Documentation

Each module must include:

1. **README.md** (per project requirements)
   - Module overview
   - Hardware capabilities
   - Usage examples
   - Performance notes
   - Safety considerations

2. **Inline Documentation**
   - All public APIs documented with `///`
   - Examples in doc comments
   - Safety notes for `unsafe` code
   - Panic conditions documented

### Example Structure

```rust
/// GPIO pin in input mode
///
/// # Examples
///
/// ```no_run
/// # use efr32mg24_hal::gpio::*;
/// let pin = gpioa.pa0.into_pull_up_input();
/// if pin.is_high() {
///     // Pin is high
/// }
/// ```
pub struct Pin<Input<PullUp>> { /* ... */ }
```

## Feature Flags

```toml
[features]
default = []

# Runtime support
rt = ["crates/efr32mg24-pac/rt", "cortex-m-rt/device"]

# Enable async support (requires executor)
async = ["embassy-executor", "embassy-time"]

# Enable specific peripheral groups
radio = []       # Radio subsystem
crypto = []      # Cryptographic hardware
usb = []         # USB (if available)

# Security features
trustzone = []   # TrustZone-M support

# Development features
defmt = ["dep:defmt"]  # defmt logging support
```

## Dependencies

### Core Dependencies
```toml
[dependencies]
efr32mg24-pac = { path = "../efr32mg24-pac", version = "0.1.0" }
cortex-m = "0.7.7"
embedded-hal = "1.0.0"
critical-section = "1.2.0"
nb = "1.1"

[dependencies.cortex-m-rt]
version = "0.7.3"
optional = true

# Async support (optional)
[dependencies.embassy-executor]
version = "0.5"
optional = true

[dependencies.embassy-time]
version = "0.3"
optional = true

# Logging (optional)
[dependencies.defmt]
version = "0.3"
optional = true
```

## Migration Strategy for B220 PAC

When the correct B220 SVD file is obtained and PAC is regenerated:

1. **Identify Breaking Changes**
   - Compare A020 vs B220 register layouts
   - Document peripheral differences
   - List removed/added features

2. **Update HAL Code**
   - Fix broken register accesses
   - Add support for new B220 features
   - Update type definitions if needed

3. **Test on Hardware**
   - Verify all peripherals work correctly
   - Check for timing/electrical differences
   - Update examples if needed

4. **Document Migration**
   - Create migration guide in `docs/migration.md`
   - List all breaking changes
   - Provide upgrade path for users

## Timeline Estimate

**Assuming development without hardware** (hardware testing deferred):

- **Week 1**: Clock + GPIO + Delay (core infrastructure)
- **Week 2**: USART/Serial implementation
- **Week 3**: I2C + SPI implementation
- **Week 4**: Timers + ADC
- **Week 5**: Power management, documentation review
- **Week 6+**: Advanced features (DMA, radio prep)
- **Hardware arrival**: Integration testing, examples, bug fixes

**Total**: 6-8 weeks for basic HAL (without radio/crypto)

## Success Criteria

### Phase 1 Complete When:
- ✅ Clocks configurable and frozen
- ✅ GPIO pins can be configured in all modes
- ✅ Delay implementation works (verified via timing tests on hardware)
- ✅ All modules have README.md documentation
- ✅ Unit tests pass
- ✅ Code compiles with no warnings

### Phase 2 Complete When:
- ✅ USART can send/receive data
- ✅ I2C can communicate with slave devices
- ✅ SPI can transfer data
- ✅ embedded-hal traits implemented for all
- ✅ Examples demonstrate each peripheral
- ✅ Integration tests pass on hardware

### HAL v0.1.0 Ready When:
- ✅ All Phase 1-2 modules complete
- ✅ Comprehensive documentation
- ✅ Working examples for all peripherals
- ✅ Tested on XIAO MG24 Sense hardware
- ✅ No known critical bugs
- ✅ Clean CI/CD pipeline

## Next Steps

1. ✅ Document XIAO hardware details
2. ✅ Create this HAL structure plan
3. ⏳ Obtain B220 SVD file (or proceed with A020 temporarily)
4. ⏳ Begin Phase 1 implementation (clock.rs)
5. ⏳ Create module README.md templates
6. ⏳ Set up CI/CD for HAL crate

---

**Document Version**: 1.0
**Last Updated**: December 3, 2025
**Status**: Planning Complete - Ready for Implementation
