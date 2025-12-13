# Phase 5 Tier 3: Advanced Peripherals Implementation Plan

**Project**: EFR32MG24 HAL  
**Phase**: 5 Tier 3 - Advanced Peripherals  
**Target Timeline**: 62 hours (~8 working days)  
**Prerequisites**: Phase 5 Tier 2 Complete + Critical Fixes Applied

---

## Overview

Phase 5 Tier 3 focuses on implementing advanced peripherals essential for production IoT and industrial applications. These peripherals enable power-efficient operation, high-performance data acquisition, and system reliability.

---

## Peripheral Priority & Timeline

### 1. ADC (IADC) - 12-bit Analog-to-Digital Converter
**Estimated Time**: 16 hours  
**Priority**: HIGH (sensor applications)  
**Complexity**: HIGH

#### Features to Implement
- Single-ended and differential inputs
- 12-bit resolution
- Multiple input channels (up to 16)
- Internal/external voltage references
- Programmable gain amplifier (PGA)
- Scan mode for multiple channels
- FIFO for data buffering

#### Register Implementation
- `IADC_CFG` - Main configuration
- `IADC_SINGLEFIFOCFG` - Single conversion FIFO
- `IADC_SCANFIFOCFG` - Scan mode FIFO
- `IADC_SINGLEFIFODATA` - Read conversion results
- `IADC_TRIGGER` - Trigger configuration
- `IADC_CMD` - Start/stop commands

#### API Design
```rust
pub struct Adc {
    iadc: pac::IadcS,
    config: AdcConfig,
}

pub struct AdcConfig {
    pub reference: VoltageReference,  // VREF, VDD, etc.
    pub resolution: Resolution,        // 12-bit
    pub sample_rate: u32,              // Hz
    pub channels: Vec<AdcChannel>,
}

impl Adc {
    pub fn new(iadc: pac::IadcS, config: AdcConfig, clocks: &FrozenClocks) -> Self;
    pub fn read_channel(&mut self, channel: AdcChannel) -> Result<u16, AdcError>;
    pub fn start_scan(&mut self, channels: &[AdcChannel]) -> Result<(), AdcError>;
    pub fn read_scan_result(&mut self) -> Result<Vec<u16>, AdcError>;
}
```

#### Testing Requirements
- Single-shot conversion accuracy
- Scan mode operation
- Reference voltage switching
- FIFO operation

---

### 2. DMA (LDMA) - Linked Direct Memory Access
**Estimated Time**: 20 hours  
**Priority**: CRITICAL (performance multiplier)  
**Complexity**: VERY HIGH

#### Features to Implement
- Linked descriptor chains
- Memory-to-memory transfers
- Memory-to-peripheral transfers
- Peripheral-to-memory transfers
- Circular buffer mode
- Interrupt on transfer complete
- Integration with USART, I2C, SPI, ADC

#### Register Implementation
- `LDMA_CH_CTRL` - Channel control
- `LDMA_CH_SRC` - Source address
- `LDMA_CH_DST` - Destination address
- `LDMA_CH_LINK` - Link to next descriptor
- `LDMA_CHEN` - Channel enable
- `LDMA_CHSTATUS` - Channel status

#### API Design
```rust
pub struct Dma {
    ldma: pac::LdmaS,
}

pub struct DmaChannel {
    channel_num: u8,
    descriptors: Vec<DmaDescriptor>,
}

pub struct DmaTransfer<'a, B> {
    channel: &'a mut DmaChannel,
    buffer: &'a mut B,
}

impl Dma {
    pub fn new(ldma: pac::LdmaS) -> Self;
    pub fn allocate_channel(&mut self) -> Result<DmaChannel, DmaError>;
}

impl DmaChannel {
    pub fn transfer_m2m(&mut self, src: &[u8], dst: &mut [u8]) -> DmaTransfer;
    pub fn transfer_m2p(&mut self, src: &[u8], peripheral: Peripheral) -> DmaTransfer;
    pub fn setup_circular(&mut self, buffer: &mut [u8], peripheral: Peripheral);
}
```

#### Integration Points
- USART TX/RX with DMA
- I2C TX/RX with DMA
- SPI TX/RX with DMA
- ADC scan results to memory
- Timer update events

#### Testing Requirements
- Memory-to-memory copy
- USART DMA transmission
- Circular buffer operation
- Interrupt handling
- Error conditions

---

### 3. Power Management (EMU) - Energy Management Unit
**Estimated Time**: 12 hours  
**Priority**: HIGH (battery applications)  
**Complexity**: MEDIUM

#### Features to Implement
- Energy modes EM0-EM4
- Peripheral clock gating
- Wake-up from low-power modes
- Voltage scaling
- Power consumption optimization

#### Energy Modes
- **EM0**: Active mode (all peripherals available)
- **EM1**: Sleep mode (CPU stopped, peripherals running)
- **EM2**: Deep sleep (HFRCO off, LFXO/LFRCO available)
- **EM3**: Stop mode (most clocks off)
- **EM4**: Hibernate (minimal power, limited wake sources)

#### API Design
```rust
pub struct PowerManager {
    emu: pac::EmuS,
}

pub enum EnergyMode {
    EM0,  // Active
    EM1,  // Sleep
    EM2,  // Deep Sleep
    EM3,  // Stop
    EM4,  // Hibernate
}

impl PowerManager {
    pub fn new(emu: pac::EmuS) -> Self;
    pub fn enter_mode(&mut self, mode: EnergyMode, wakeup: WakeupConfig);
    pub fn get_current_mode(&self) -> EnergyMode;
    pub fn configure_wakeup(&mut self, sources: &[WakeupSource]);
}
```

#### Testing Requirements
- Mode transitions
- Wake-up latency
- Power consumption measurements
- Peripheral state preservation

---

### 4. RTC (RTCC) - Real-Time Clock and Calendar
**Estimated Time**: 8 hours  
**Priority**: MEDIUM (timekeeping applications)  
**Complexity**: LOW-MEDIUM

#### Features to Implement
- Real-time counter
- Calendar (date/time)
- Alarms and compare values
- Timestamp capture
- Low-power operation (runs in EM2/EM3)

#### API Design
```rust
pub struct Rtc {
    rtcc: pac::RtccS,
}

pub struct DateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Rtc {
    pub fn new(rtcc: pac::RtccS, lfclk: &FrozenClocks) -> Self;
    pub fn set_datetime(&mut self, dt: DateTime);
    pub fn get_datetime(&self) -> DateTime;
    pub fn set_alarm(&mut self, dt: DateTime, callback: impl FnMut());
    pub fn get_timestamp(&self) -> u32;
}
```

#### Testing Requirements
- Time accuracy over 24 hours
- Alarm functionality
- Low-power operation
- Leap year handling

---

### 5. Watchdog (WDOG) - Watchdog Timer
**Estimated Time**: 6 hours  
**Priority**: MEDIUM (system reliability)  
**Complexity**: LOW

#### Features to Implement
- Independent watchdog
- Configurable timeout period
- Window mode (min/max timeout)
- Reset generation on timeout
- Debug mode handling

#### API Design
```rust
pub struct Watchdog {
    wdog: pac::WdogS,
}

pub struct WatchdogConfig {
    pub timeout_ms: u32,
    pub window_mode: Option<WindowConfig>,
    pub debug_run: bool,
}

impl Watchdog {
    pub fn new(wdog: pac::WdogS, config: WatchdogConfig) -> Self;
    pub fn feed(&mut self);
    pub fn is_running(&self) -> bool;
    pub fn get_time_left(&self) -> u32;
}
```

#### Testing Requirements
- Timeout reset verification
- Feed operation
- Window mode
- Debug mode behavior

---

## Implementation Order

### Week 1 (20 hours)
- **Day 1-2**: ADC implementation (16 hours)
- **Day 3**: Watchdog implementation (4 hours)

### Week 2 (20 hours)
- **Day 1-3**: DMA core implementation (15 hours)
- **Day 4**: RTC implementation (5 hours)

### Week 3 (22 hours)
- **Day 1-2**: DMA peripheral integration (10 hours)
- **Day 3-4**: Power Management implementation (12 hours)

---

## Testing & Validation

### Per-Peripheral Tests
- Unit tests for each peripheral
- Example applications
- Hardware validation procedures

### Integration Tests
- DMA + USART streaming
- ADC + DMA continuous sampling
- RTC + Power Management sleep/wake
- All peripherals + Watchdog safety

### Documentation
- Module READMEs for each peripheral
- Hardware setup guides
- Performance benchmarks
- Power consumption data

---

## Success Criteria

1. ✅ All peripherals compile without warnings
2. ✅ All peripherals pass unit tests
3. ✅ Example code for each peripheral
4. ✅ Hardware-validated on XIAO MG24
5. ✅ Documentation complete
6. ✅ Integration with existing Tier 1/2 peripherals
7. ✅ Power consumption optimized

---

## Risks & Mitigations

### Risk 1: DMA Complexity
**Mitigation**: Start with simple M2M transfers, iterate to peripheral integration

### Risk 2: Power Management State Handling
**Mitigation**: Comprehensive state machine testing, clear documentation

### Risk 3: ADC Calibration
**Mitigation**: Reference Silicon Labs calibration procedures, add calibration API

### Risk 4: Hardware Availability
**Mitigation**: Extensive software simulation, coordinate hardware access

---

## Dependencies

### External
- XIAO MG24 hardware board (for validation)
- Oscilloscope (ADC/PWM validation)
- Power analyzer (EMU validation)
- Logic analyzer (DMA timing validation)

### Internal
- Phase 5 Tier 2 complete
- Critical fixes applied (CMU ownership, etc.)
- Test infrastructure in place

---

## Deliverables

1. **Source Code**
   - `src/adc/` - ADC module (4 files)
   - `src/dma/` - DMA module (5 files)
   - `src/power/` - Power management (3 files)
   - `src/rtc/` - RTC module (3 files)
   - `src/watchdog/` - Watchdog (2 files)

2. **Examples**
   - `08_adc.rs` - ADC sampling example
   - `09_dma.rs` - DMA transfer example
   - `10_low_power.rs` - Power management example
   - `11_rtc.rs` - RTC alarm example
   - `12_watchdog.rs` - Watchdog example

3. **Tests**
   - Unit tests for all modules
   - Integration test suite
   - Hardware test procedures

4. **Documentation**
   - Module READMEs
   - Updated main README
   - Updated CLAUDE.md
   - Performance data

---

## Post-Tier 3 Roadmap

After Tier 3 completion, the HAL will be ready for:
1. **Phase 6**: Embassy async support
2. **Phase 7**: Board Support Packages (BSPs)
3. **Phase 8**: Advanced features (USB, Crypto, Radio)
4. **Phase 9**: Publication to crates.io

---

**Status**: Ready to begin upon Phase 5 Tier 2 + Critical Fixes completion  
**Maintainer**: EFR32MG24 HAL Team  
**Last Updated**: December 13, 2025
