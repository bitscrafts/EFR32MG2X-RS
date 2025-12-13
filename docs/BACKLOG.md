# EFR32MG24 HAL Development Backlog

**Last Updated**: December 13, 2025
**Current Sprint**: Phase B - Communication Peripherals (70% complete)
**Next Sprint**: Phase B - Timers/PWM

---

## Ready (Prioritized for Phase B)

### High Priority
- [ ] **TIMER**: Implement TIMER0 basic functionality @priority(high) @phase(B)
  - Register access (CTRL, CMD, CNT, TOP, IEN, IF)
  - Counter modes (up, down, up-down)
  - Clock prescaler configuration
  - Interrupt support

- [ ] **TIMER**: Implement TIMER1-4 basic functionality @priority(high) @phase(B)
  - Replicate TIMER0 implementation for TIMER1-4
  - Peripheral consumption pattern
  - Critical section protection

- [ ] **PWM**: Add PWM generation support @priority(high) @phase(B)
  - CC (Capture/Compare) channel configuration
  - Duty cycle control
  - Frequency configuration
  - embedded-hal PWM traits

### Medium Priority
- [ ] **Examples**: Create 06_timer.rs example @priority(medium) @phase(B)
  - Basic timer counting
  - Interrupt handling
  - Overflow detection

- [ ] **Examples**: Create 07_pwm.rs example @priority(medium) @phase(B)
  - LED brightness control via PWM
  - Multiple PWM channels
  - Frequency/duty cycle adjustment

- [ ] **Documentation**: Write TIMER module README.md @priority(medium) @phase(B)
  - Hardware register documentation
  - Usage examples
  - Timer modes explanation

### Low Priority
- [ ] **Testing**: Verify all examples build without warnings @priority(low) @phase(B)
- [ ] **Documentation**: Update STATUS.md after Phase B completion @priority(low) @phase(B)

---

## In Progress

Currently no active tasks (documentation cleanup just completed).

---

## Review (Waiting for Validation)

- [ ] **Documentation**: Phase terminology update (Phase 5 → Phase A/B) @review(ready for testing)
  - CLAUDE.md updated
  - docs/PLAN.md updated
  - docs/STATUS.md updated
  - Orchestrator agent updated with pre-commit audit

---

## Blocked

- [ ] **Phase C**: Embassy async support @blocked(Waiting for Phase B completion)
  - Reason: Need stable HAL peripherals before async implementation
  - Blocker: Phase B completion (Timers)

- [ ] **Phase C**: Hardware testing on XIAO MG24 Sense @blocked(Hardware not available)
  - Reason: All code is untested on physical hardware
  - Blocker: Need to acquire XIAO MG24 Sense board

---

## Done (Recent - Last 30 Days)

### Week of December 9-13, 2025

- [x] **Phase B**: I2C master mode implementation @done(2025-12-12)
  - I2C0/I2C1 register access
  - 7-bit addressing
  - Standard (100 kHz) and Fast (400 kHz) modes
  - embedded-hal v1.0 I2c traits
  - Example 05_i2c.rs created
  - 524 lines code, 237 lines documentation

- [x] **Phase B**: SPI master mode implementation @done(2025-12-12)
  - USART in SPI mode
  - Master mode configuration
  - embedded-hal SPI traits

- [x] **Documentation**: Phase terminology standardization @done(2025-12-13)
  - Updated Phase 5 → Phase A/B across all docs
  - Archived 4 obsolete documentation files
  - Enhanced orchestrator with pre-commit documentation audit

### Week of December 2-6, 2025

- [x] **Phase B**: USART serial communication @done(2025-12-04)
  - USART0 register access
  - Configurable baud rates
  - embedded-hal-nb serial traits
  - Example 04_usart.rs created
  - 373 lines code, 237 lines documentation

- [x] **Phase A**: GPIO hardware register access @done(2025-12-04)
  - PORTx_MODEL/MODEH register manipulation
  - Type-safe pin modes
  - Drive strength configuration
  - embedded-hal v1.0 OutputPin/InputPin traits
  - Example 03_gpio.rs created
  - 617 lines code

- [x] **Phase A**: CMU clock configuration @done(2025-12-04)
  - HFXO/HFRCO/LFXO/LFRCO support
  - SYSCLKCTRL register access
  - Peripheral consumption pattern
  - FrozenClocks frequency tracking
  - 317 lines code

- [x] **Phase A**: Delay implementation @done(2025-12-04)
  - SysTick-based blocking delays
  - embedded-hal v1.0 DelayNs trait
  - Example 02_delay.rs created
  - ~100 lines code

---

## Ideas (Unprioritized Future Work)

### Phase C Candidates
- [ ] **EMU**: Energy Management Unit
  - Energy mode transitions (EM0-EM4)
  - Voltage scaling
  - Low-power mode support

- [ ] **LDMA**: Linked DMA Controller
  - Peripheral-to-memory transfers
  - Memory-to-memory transfers
  - Linked descriptor support

- [ ] **IADC**: Analog-to-Digital Converter
  - Single-shot conversion
  - Continuous conversion
  - Multi-channel support

- [ ] **LETIMER**: Low-Energy Timer
  - Real-time clock functionality
  - Wakeup from low-power modes

### Advanced Features
- [ ] **Embassy**: Async runtime support
  - Timer driver for Embassy
  - DMA abstraction
  - Async peripheral drivers

- [ ] **Security**: Cryptographic accelerators
  - AES hardware acceleration
  - SHA hardware acceleration
  - Secure Vault integration

- [ ] **Radio**: 2.4 GHz radio support
  - Basic radio initialization
  - FFI bindings to Silicon Labs libraries
  - Consider pure Rust implementation long-term

### Tooling & Infrastructure
- [ ] **Testing**: Unit tests for HAL modules
  - Mock hardware testing
  - Per-peripheral test suites

- [ ] **CI/CD**: Continuous integration
  - GitHub Actions workflow
  - Automated builds for all examples
  - Documentation generation

- [ ] **Hardware**: XIAO MG24 Sense board acquisition
  - Hardware-in-loop testing
  - Real-world validation
  - Performance benchmarking

### Community & Ecosystem
- [ ] **Publishing**: Publish to crates.io
  - Version 0.1.0 release
  - Documentation polish
  - Examples verification

- [ ] **BSP**: Board Support Package for XIAO MG24 Sense
  - Pin definitions
  - Board-specific peripherals
  - Sensor integration (IMU, microphone)

---

## Metrics

**Phase B Progress**: 70% complete (3 of 5 peripherals done)
- ✅ USART (Complete)
- ✅ I2C (Complete)
- ✅ SPI (Complete)
- ⏳ TIMER (In Ready)
- ⏳ PWM (In Ready)

**Lines of Code (HAL)**:
- GPIO: 617 lines
- CMU: 317 lines
- Delay: ~100 lines
- USART: 373 lines
- I2C: 524 lines
- **Total**: ~1,931 lines (excluding examples)

**Examples**: 5 working examples
- 01_clock.rs
- 02_delay.rs
- 03_gpio.rs
- 04_usart.rs
- 05_i2c.rs

**Time Investment**: ~18 hours total
- Phase 1-4: ~8 hours
- Phase A: ~4 hours
- Phase B: ~6 hours

---

## Notes

**Kanban Columns Explained**:
- **Ready**: Prioritized tasks ready to start (Phase B focus)
- **In Progress**: Currently being worked on
- **Review**: Waiting for validation/testing
- **Blocked**: Cannot proceed until blocker resolved
- **Done**: Completed in last 30 days (archived monthly)
- **Ideas**: Unprioritized future work

**Task Metadata Tags**:
- `@priority(high|medium|low)` - Task priority
- `@phase(A|B|C|...)` - Development phase
- `@started(YYYY-MM-DD)` - Start date
- `@done(YYYY-MM-DD)` - Completion date
- `@blocked(reason)` - Blocking reason
- `@review(status)` - Review status
- `@assignee(name)` - Task owner (usually "orchestrator" for solo dev)

**Update Frequency**:
- Review backlog: Daily (before starting work)
- Update In Progress: Real-time (as tasks move)
- Archive Done items: Monthly (move to LOG.md)
- Reprioritize Ready: Weekly (based on phase goals)

---

**Last Review**: December 13, 2025
**Next Review**: December 14, 2025
**Maintained By**: Rust Embedded Orchestrator
