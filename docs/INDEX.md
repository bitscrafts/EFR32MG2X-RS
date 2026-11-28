# EFR32MG24 Project Documentation Index

Complete documentation index for the EFR32MG24 embedded Rust project.

## Quick Navigation

- **New to the project?** Start with [Getting Started](01-getting-started/README.md)
- **Need to flash firmware?** See [Quick Flash Guide](01-getting-started/QUICK_FLASH_GUIDE.md)
- **Building firmware?** Check [Scripts Documentation](../scripts/README.md)
- **Architecture details?** Read [Architecture Overview](02-architecture/README.md)
- **Development workflow?** Check [Development Guide](03-development/README.md)
- **API reference?** See [Reference Documentation](04-reference/README.md)
- **Troubleshooting?** Consult [Recovery Guide](01-getting-started/RECOVERY_GUIDE.md)

## Documentation Structure

This documentation follows the SSOT (Single Source of Truth) methodology, organizing information by topic rather than duplicating content.

### 01. Getting Started

Core guides for getting up and running with the EFR32MG24 hardware and firmware.

**Main Guide:**
- [Getting Started README](01-getting-started/README.md) - Start here

**Flashing Procedures:**
- [Build here, flash where the probe is](01-getting-started/FLASHING.md) - The current workflow, the guards, and what is not yet verified
- [Quick Flash Guide](01-getting-started/QUICK_FLASH_GUIDE.md) - Fast reference for common flashing operations
- [Comprehensive Flashing Guide](01-getting-started/COMPREHENSIVE_FLASHING_GUIDE.md) - Complete flashing documentation
- [Commander Flash Procedure](01-getting-started/COMMANDER_FLASH_PROCEDURE.md) - Using Simplicity Commander
- [OpenOCD Flash Procedure](01-getting-started/OPENOCD_FLASH_PROCEDURE.md) - Using OpenOCD
- [Hardware Flashing Guide](01-getting-started/HARDWARE_FLASHING_GUIDE.md) - Hardware-level flashing details

- [Flash read-back backups](01-getting-started/BACKUPS.md) - Bootloader + app region dumped from the chip, with hashes

**Bootloader:**
- [Bootloader Quick Reference](01-getting-started/BOOTLOADER_QUICK_REFERENCE.md) - Essential bootloader information

**Safety and Recovery:**
- [Critical Safety Protocol](01-getting-started/CRITICAL_SAFETY_PROTOCOL.md) - Must read before flashing
- [Recovery Guide](01-getting-started/RECOVERY_GUIDE.md) - Device recovery procedures

**Recovery Procedures:**
- [BMP Recovery Guide](01-getting-started/recovery/BMP_RECOVERY_GUIDE.md) - Black Magic Probe recovery
- [Board Recovery Status](01-getting-started/recovery/BOARD_RECOVERY_STATUS.md) - Recovery status tracking
- [PC1 Recovery Plan](01-getting-started/recovery/PC1_RECOVERY_PLAN.md) - Specific PC1 recovery
- [Memory Analysis](01-getting-started/recovery/memory_analysis.md) - Memory debugging for recovery

### 02. Architecture

System architecture, design decisions, and technical terminology.

**Overview:**
- [Architecture README](02-architecture/README.md) - Architecture overview

**Core Architecture:**
- [Radio Architecture](02-architecture/RADIO_ARCHITECTURE.md) - Radio subsystem design
- [Unified Radio Architecture](02-architecture/UNIFIED_RADIO_ARCHITECTURE.md) - Unified radio interface design
- [Identifier Scheme](02-architecture/IDENTIFIER_SCHEME.md) - Naming and identification conventions
- [Crate organisation research](02-architecture/CRATE-ORGANISATION-RESEARCH.md) - How rp-hal, nrf-hal, stm32f4xx-hal and esp-hal split PAC/HAL/BSP, and what this workspace took from each
- [Terminology](02-architecture/TERMINOLOGY.md) - Project terminology and definitions

**HAL (Hardware Abstraction Layer):**
- [HAL Structure Plan](02-architecture/hal/HAL_STRUCTURE_PLAN.md) - HAL organization and design

### 03. Development

Development guides, workflows, CI/CD, and project planning.

**Overview:**
- [Development README](03-development/README.md) - Development guide overview

**Project Management:**
- [Status](03-development/STATUS.md) - Current project status
- [Plan](03-development/PLAN.md) - Project roadmap and planning
- [Backlog](03-development/BACKLOG.md) - Feature backlog and TODOs
- [Log](03-development/LOG.md) - Development log and decisions

**Development Guides:**
- [BSP Plan](03-development/BSP_PLAN.md) - Board Support Package planning
- [Build Optimization](03-development/BUILD_OPTIMIZATION.md) - Build system optimization
- [Bootloader Session](03-development/bootloader-session.md) - Bootloader development notes
- [Refactoring Steps](03-development/refactoring-steps.md) - Code refactoring guidance

**CI/CD:**
- [CI/CD Testing](03-development/CI_CD_TESTING.md) - Continuous integration testing
- [CI/CD Issues](03-development/CI_CD_ISSUES.md) - Known CI/CD issues and solutions

### 04. Reference Documentation

Research, analysis, and reference materials.

**Overview:**
- [Reference README](04-reference/README.md) - Reference documentation overview
- [HAL, radio and BLE review](04-reference/HAL-RADIO-BLE-REVIEW-2026-11-17.md) - independent review at `101114f`: 31 HAL defects, 18 BLE defects, with the status of each after the 2026-11-18 pass
- [HAL completeness and robustness review, 2026-11-22](04-reference/HAL-OPUS-ANALYSIS-2026-11-22.md) - independent read-only review: 12 robustness findings, the gate's dead guard, and the status of every 2026-11-18 finding
- [I2C recovery state](04-reference/I2C-RECOVERY-STATE.md) - a NACK is now reported as one; the bus still does not survive it, with the decode of `IF = 0x109f9` and the next step
- [f20, the transmit timeout](04-reference/F20-TX-TIMEOUT-INSTRUMENT-2026-11-28.md) - the fix is committed and green; four instruments measured nothing, the three negatives are recorded, and the staged-mailbox test that would turn "no record" into a line number is written down
- [f20, the transmit timeout](04-reference/OPUS-REVIEW-F20-2026-11-28.md) - the fix is committed and green; four instruments measured nothing, the three negatives are recorded, and the staged-mailbox test that would turn "no record" into a line number is written down
- [f20, the transmit timeout](04-reference/PCNT-COMMAND-NEVER-COMPLETES-2026-11-28.md) - the fix is committed and green; four instruments measured nothing, the three negatives are recorded, and the staged-mailbox test that would turn "no record" into a line number is written down
- [SYSRTC compare channel ownership](04-reference/SYSRTC-OWNERSHIP.md) - channel 0 belongs to the Bluetooth sleep timer; the application takes 1 or 2, and the token that would enforce it
- [The IMU FIFO, measured](04-reference/IMU-FIFO-MEASURED.md) - why the FIFO is not justified for throughput, and the one thing (contiguity) that would change the answer
- [LETIMER0's top, measured](04-reference/LETIMER-TOP-2026-11-26.md) - `CTRL.CNTTOPEN` is the switch, the top is 24 bits at reset and not the manual's 65535, and the earlier "`TOP` is ignored" was a `u16` truncation
- [PRS, a GPIO pad as a producer](04-reference/PRS-GPIO-PRODUCER-2026-11-26.md) - the pad's level is the signal: 8 moves gave 4 conversions, and `EXTIRISE`/`EXTIFALL` are not in the PRS path at all
- [PCNT0's ceiling is a pulse width](04-reference/PCNT-CEILING-2026-11-26.md) - a 53 ns timer event is invisible to a 32.768 kHz sampler (0 counts at 1 kHz), and a pad driven by its own timer is not on the producer path
- [LETIMER0's compares, measured](04-reference/LETIMER-COMPARE-2026-11-26.md) - the flags are set by a write to `IF` and cleared only through `IF + 0x2000`, the down-counter reaches `COMP0` before `COMP1`, and a `TOP` readback on a stopped timer can return 0
- [The IADC on a PRS edge, measured](04-reference/IADC-PRS-TRIGGER-2026-11-26.md) - `TRIGGER.SINGLETRIGSEL = PRSPOS` converts once per edge on an asynchronous channel (`TRIGGER = 0x300`, 8 pulses -> 8 conversions), and the three controls that make the count mean something: arming alone converts, no pulse converts nothing, an unarmed queue ignores edges
- [A timer's overflow driving the PRS, measured](04-reference/PRS-TIMER-PRODUCER-2026-11-26.md) - TIMER0's overflow on channel 0 gave `ev=8` for `ovf=8` with no CPU write in the loop, the channel word `0x000c2101` (`FNSEL`/`SOURCESEL`/`SIGSEL`), four zero controls, and the 16-bit `CNT` that made the first instrument report a working path as dead
- [A GPIO pad driving the PRS, measured](04-reference/PRS-GPIO-PRODUCER-2026-11-26.md) - a pad moved by its own pull resistor paces one conversion per rising edge (`c=4 up=4` of eight changes, `long=8 upg=8` of sixteen), `EXTIRISE`/`EXTIFALL` are not in the PRS path (`fall=4` with the rising mask cleared), the two-bit port/pin encoding reads back as `sel=0x2000 isel=0x3000`, and three controls stayed at zero
- [A GPIO edge as an interrupt, measured](04-reference/GPIO-INTERRUPT-2026-11-27.md) - one pad change is one interrupt on both vectors (`cnt=8` of `n=8`, `odd=8`/`p0even=8`), `IEN` is the per-pin gate in front of the vector (`dis=0` with the edge masks armed), and the flag latches with the vector masked (`if=0x00000008`) while the core counts nothing
- [Chained DMA descriptors, measured](04-reference/DMA-LINKED-2026-11-27.md) - two legs run from one channel in order (`d1=8 d2=8`), the second descriptor has to live in RAM with `STRUCTREQ` set (`c7` against five channel-register cases that fetch it and never run it), the `SWREQ` control that proves the fetched descriptor was valid (`c6`), and the link-cleared control that moves only the first leg
- [The LDMA done interrupt, measured](04-reference/DMA-IRQ-2026-11-27.md) - a 2047-word transfer ends with the core asleep: one `WFI`, one handler entry and the deadline clock still at zero (`sp=1 irq=1 tk=0`); `IEN.CHDONE` is the delivery gate, since clearing it leaves the data complete and the `DONE` flag latched (`cf=1`) with no handler entry, and the deadline is what ends the wait
- [EUSART0 in asynchronous mode, measured](04-reference/EUSART-ASYNC-2026-11-27.md) - eight distinct bytes out and back through `CFG0.LOOPBK` (`am=8 err=0`) against a control that reads zero (`ctl=0`); `CFG0` cannot be written with `EN` set (precise bus fault at `EUSART0_S+0x08`), and the EUSART's asynchronous divider is `1+DIV/32`, not the USART's `1+DIV/256` -- the USART arithmetic produced 15 650 baud where 115 200 was asked for, and `div=297` produced 23 ticks for 80 bit times on the wire
- [BURTC, the backup counter, measured](04-reference/BURTC-2026-11-27.md) - two counters on one LFRCO agree (`d0=s0=32901`, `r0=1000`), the prescaler is a real divide by 8 (`r1=125`), and with `EM4GRPACLK` pointed at nothing the counter stops dead (`ctl=0`) and resumes when the source is restored; the backup domain survives a core reset, so `CFG` faults while `EN` is set (`BFAR=0x50064008`) and `CMD` faults while `EN` is low (`BFAR=0x5006400c`)
- [The clocks nobody had measured: FSRCO, HFRCOEM23, ULFRCO](04-reference/CMU-MISSING-CLOCKS-2026-11-27.md) - every rate a ratio to the 39 MHz crystal, counted by TIMER0 free-running (`HFRCOEM23` 19.173..19.179 MHz in band 8, `FSRCO` 20.206..20.228 MHz, `ULFRCO` 993.7 Hz through BURTC, `LFRCO` 32.96 kHz), the source-removed control that stops BURTC dead (`ul0=0`) and the restore that starts it again (`ur=988`), and the datasheet table each verdict comes from
- [The ACMP0 ladder as a ruler, measured](04-reference/ACMP-LADDER-2026-11-27.md) - 252 on-chip comparisons with no pin or wire: the ladder swept 1..63 against the same internal node crosses at tap 43 or 44 against the 1.25 V reference and tap 22 against the 2.5 V one (`e=111`, `f=0` for the flat control, `u=0` unstable), placing the node at **about 853 mV** (833..873 across boots, one tap wide per run); swapping the inputs inverts the answer (`sw=01`) and a disabled block follows `CTRL.NOTRDYVAL` (`off=01`); the bus gate is `CMU_CLKEN1.ACMP0` (`BFAR=0x49008004` without it) and `INPUTCTRL` faults with `EN=0` (`BFAR=0x49008014`)
- [RADIOAES against two published known answers](04-reference/AES-KAT-2026-11-27.md) - the AES engine driven by scatter-gather descriptor chains and nothing transmitted: FIPS-197 C.1 and NIST SP 800-38A F.1.1 block 1 come back byte for byte (`ct=69c4e0d8...c55a`, `c2=3ad77bb4...ef97`), the decrypt bit returns the plaintext, and a near-miss key moves all sixteen bytes to a block OpenSSL computes identically; `STATUS.SOFTRSTBSY` is set at reset and never clears, so the driver has no reset step, and a finished chain reports `IF=0x12` (stopped, not end-of-block)
- [The delay, measured against the crystal](04-reference/DELAY-MEASURED-2026-11-27.md) - a `delay_ms(200)` takes 200.279 ms on the reset clock and 201.124 ms with the core on the crystal, `delay_us(100_000)` takes 100.010 ms (`e1=5 eu=0`), one SysTick period of 390 000 ticks is 10.003 ms (`r1=38 988 303`, so the counter runs at the core clock undivided), and a `Delay` told 78 MHz takes 401 ms (`x=401171 ex=2`); the 2.3x-short claim this replaces was DWT cycles divided by an assumed clock, and `emlib_clock` re-run today reads `meas=2659k` against a delay the crystal puts at its full 10 ms
- [`missing_docs` in all 33 crates](04-reference/MISSING-DOCS-2026-11-27.md) - the lint is installed everywhere and measured at 0 undocumented items (`cargo rustc --lib -- -D missing_docs` per crate); the 74 hand-written items that had none included two doc blocks attached to the wrong function (`Module::take`, `enable_pairing_with`, `pump`), the only remaining `allow(missing_docs)` sits on the two modules that `include!` bindgen output and the gate verifies that placement, and a fixture proves the lint fires while a crate-level allow, a module-level allow on hand-written code, a removed deny and a raised floor each fail the check

**Bootloader Research:**
- [Gecko Bootloader Research](04-reference/GECKO_BOOTLOADER_RESEARCH.md) - Silicon Labs bootloader analysis
- [Rust Bootloader Research](04-reference/RUST_BOOTLOADER_RESEARCH.md) - Rust bootloader investigation
- [Rust Bootloader Solution](04-reference/RUST_BOOTLOADER_SOLUTION.md) - Final bootloader implementation
- [Arduino Bootloader Analysis](04-reference/ARDUINO_BOOTLOADER_ANALYSIS.md) - Arduino bootloader reverse engineering

**Radio Research:**
- [Arduino Radio Analysis](04-reference/ARDUINO_RADIO_ANALYSIS.md) - Arduino radio implementation analysis
- [Bluetooth Refactoring Decisions](04-reference/BLUETOOTH_REFACTORING_DECISIONS.md) - Bluetooth stack decisions
- [Radio Implementation Plan](04-reference/RADIO_IMPLEMENTATION_PLAN.md) - Radio implementation strategy

**Bridge and Safety:**
- [SAMD11 Bridge and Safe Flashing](04-reference/SAMD11_BRIDGE_AND_SAFE_FLASHING.md) - SAMD11 bridge debugger

**Hardware:**
- [XIAO MG24 Hardware](04-reference/hardware/XIAO_MG24_HARDWARE.md) - Hardware specifications and pinout

**PAC (Peripheral Access Crate):**
- [B220 vs A020 Comparison](04-reference/pac/B220_VS_A020_COMPARISON.md) - Chip variant comparison
- [SVD Pack Extraction](04-reference/pac/SVD_PACK_EXTRACTION.md) - SVD file extraction process
- [SVD Processing History](04-reference/pac/SVD_PROCESSING_HISTORY.md) - SVD processing log

### 99. Archive

Historical documents, resolved issues, and deprecated information.

**Overview:**
- [Archive README](99-archive/README.md) - Archive documentation

**Archived Documents:**
- [Blink Debug Plan](99-archive/BLINK_DEBUG_PLAN.md) - Historical debug session
- [Clock Config SWD Issue](99-archive/HOW_CLOCK_CONFIG_DISABLED_SWD.md) - Resolved clock issue
- [Phase D0 Complete](99-archive/PHASE_D0_COMPLETE.md) - Completed phase documentation
- [Probe-rs EFR32MG24 Issues](99-archive/PROBE_RS_EFR32MG24_ISSUES.md) - Historical probe-rs issues
- [EFR32MG24 Research Summary](99-archive/EFR32MG24_RESEARCH_SUMMARY.md) - Initial research
- [Findings](99-archive/FINDINGS.md) - Early project findings
- [Project Summary](99-archive/PROJECT_SUMMARY.md) - Historical project summary

## Scripts

The project includes several utility scripts for building and flashing firmware.

See [Scripts Documentation](../scripts/README.md) for detailed information about:
- Container-based build system
- Flashing procedures
- Mass erase and recovery
- Bootloader management

**Quick Links:**
- `build-container.sh` - Build firmware in container
- `flash-mass-erase.sh` - Mass erase and flash
- `flash-bootloader-test.sh` - Flash bootloader + test app
- `flash-arduino-only.sh` - Flash Arduino test firmware

## Additional Resources

- [Complete Handoff Document](../HANDOFF.md) - Comprehensive project handoff
- [Changelog](../CHANGELOG.md) - Version history and changes
- [Project README](../README.md) - Main project README
- [Cargo Workspace](../Cargo.toml) - Workspace configuration

## Documentation Organization Principles

This documentation follows SSOT (Single Source of Truth) principles:

1. **No Duplication:** Information exists in one place only
2. **Clear Categorization:** Documents organized by topic and purpose
3. **Cross-Referencing:** Links connect related information
4. **Version Control:** All changes tracked in git
5. **Living Documents:** Documentation evolves with code

### Finding Information

Use this decision tree to locate documentation:

```mermaid
graph TD
    A[What do I need?] --> B{Type of Info}
    B -->|How to flash| C[01-getting-started]
    B -->|How it works| D[02-architecture]
    B -->|How to develop| E[03-development]
    B -->|Research/Analysis| F[04-reference]
    B -->|Old/Resolved| G[99-archive]

    C --> C1[Quick Flash Guide]
    C --> C2[Recovery procedures]

    D --> D1[Radio Architecture]
    D --> D2[HAL Structure]

    E --> E1[Status & Planning]
    E --> E2[Build & CI/CD]

    F --> F1[Bootloader Research]
    F --> F2[Hardware Specs]

    G --> G1[Historical Issues]
```

## Contributing to Documentation

When adding or updating documentation:

1. **Choose the right location:**
   - Getting started = User-facing guides and procedures
   - Architecture = System design and structure
   - Development = Developer workflow and tools
   - Reference = Research and detailed analysis
   - Archive = Resolved issues and historical docs

2. **Update this index** after adding new files

3. **Use cross-references** instead of duplicating content

4. **Include metadata:**
   - Last updated date
   - Author/maintainer
   - Related documents

5. **Follow naming conventions:**
   - Use UPPER_SNAKE_CASE for major documents
   - Use kebab-case for section documents
   - Use descriptive names

## Document Status

| Section | Files | Last Major Update |
|---------|-------|-------------------|
| 01-getting-started | 13 | 2026-05-16 |
| 02-architecture | 6 | 2026-05-16 |
| 03-development | 11 | 2026-05-16 |
| 04-reference | 13 | 2026-05-16 |
| 99-archive | 8 | 2026-05-16 |
| **Total** | **51** | **2026-05-16** |

## License

See [LICENSE](../LICENSE) file for details.

---

*Documentation organized using SSOT methodology - Last updated: 2026-05-16*

- [Expansion Base example status](03-development/EXPANSION-BOARD-STATUS-2026-11-11.md) — what is verified, what is not, and the ten hardware lessons
- [Fall detection calibration](04-reference/FALL-DETECTION-CALIBRATION.md) - nine real drops, and why the impact threshold cannot separate a fall from a knock
- [I2C0 read defect](04-reference/I2C0-READ-DEFECT.md) — fourteen measurements, closed: the last byte needed a NACK and the flags needed `IF_CLR`

### Session records and catalogues

- [Session 2026-11-11](03-development/SESSION-2026-11-11.md) - IMU found on a different bus, microphone working, RTC/buzzer in the HAL, fall detection, BSP facade, MVP research
- [Example catalogue](03-development/EXAMPLES.md) - every example with the evidence that it works, and the three that are red on purpose
- [Expansion board status](03-development/EXPANSION-BOARD-STATUS-2026-11-11.md) - device by device, plus the RAM and linker findings
- [IMU investigation](04-reference/IMU-INVESTIGATION.md) - the bus lines were swapped; how it was found
- [IADC and microphone notes](04-reference/IADC-MIC-NOTES.md) - `SINGLE` is only writable while the converter is enabled
- [MVP notes](04-reference/MVP-NOTES.md) - the accelerator, and why hand-programming it is not the supported path
- [Fall detection calibration](04-reference/FALL-DETECTION-CALIBRATION.md) - nine real drops, and why the impact threshold cannot separate a fall from a knock
- [I2C after a NACK](04-reference/I2C-RECOVERY-STATE.md) - closed with a control in both directions, and the sequence `ABORT` + `STOP`, then `CLEARPC`, `CLEARTX`
- [HAL maturity and publication](04-reference/HAL-MATURITY-2026-11-25.md) - what is proven per peripheral, what blocks a publish, and the checklist
- [HAL review brief and the path to A](04-reference/HAL-REVIEW-BRIEF-2026-11-25.md) - a self-contained brief for an outside reviewer: the records, the rubric, and what B -> A means per dimension
- [Coverage boundary](04-reference/COVERAGE-BOUNDARY-2026-11-25.md) - every peripheral family the device has, which ones this crate drives with a record, which are absent, and which modes of the driven ones are missing (SPI slave, I2C slave, I2S, capture, differential)
- [Plan to close the coverage gaps](03-development/PLAN-COVERAGE-2026-11-25.md) - what each missing item needs to be *proven*, in phases: no hardware needed, board on the desk, meter-class hardware, and the radio's vendor blobs
- [DWT needs a debug session](04-reference/DWT-NEEDS-A-DEBUG-SESSION.md) - the cycle counter freezes when the probe detaches; use SysTick for standalone timing
- [DMA channel stop](04-reference/DMA-CHANNEL-STOP.md) - `CHSTATUS` not `CHBUSY` is the armed state; `stop()` proven, and a blocking transfer no longer leaves the channel armed
- [PAC clock and timer notes](04-reference/PAC-CLOCK-AND-TIMER-NOTES.md) - the register orders this die demands

- [Session 2026-11-12](03-development/SESSION-2026-11-12.md) -- radio stacks compile and link, layer audit, nine book chapters
- [Session 2026-11-14](03-development/SESSION-2026-11-14.md) -- the radio works, the BME680 reaches a phone, and the four defects that were ours
- [Roadmap Gantt and phase table](../README.md) -- what is done, what is active, what is next
- [Radio notes](04-reference/RADIO-NOTES.md) -- what the two vendor stacks needed, and the three gaps that remain
- [Layer audit 2026-11-12](02-architecture/LAYER-AUDIT-2026-11-12.md) -- seven violations the compiler cannot see

- [BLE: what is missing for gatt](04-reference/BLE-GATT-GAP.md) -- twelve symbols in three groups, and which are ours
- [Testing the radio with an nRF52840](04-reference/BLE-TEST-EQUIPMENT.md) -- sniffer, central, advertiser: what each proves
- [Can the nRF52840 test the radio?](04-reference/NRF52840-AS-INSTRUMENT.md) -- measured: no BLE radio API in either Rust HAL, and the four paths out
- [Testing the radio: what is available](04-reference/BLE-TEST-HARNESS.md) -- the host adapter as transmitter and receiver, measured.
- [Identifiers of every device](04-reference/DEVICE-IDENTIFIERS.md) -- USB, serial, silicon unique id, with provenance.
- [HOST RULE: the radio carries the network](04-reference/HOST-RADIO-SAFETY.md) -- Wi-Fi and Bluetooth are one dongle; what is forbidden.
- [BLE through RAIL on the XIAO](04-reference/RAIL-BLE-ON-XIAO.md) -- caps, both assertions, and the HFXO no-op trap.
- [The path to GATT](04-reference/GATT-PATH.md) -- the four symbols left, two of them not obtainable from public sources.
- [HOST RULE: the radio carries the network](04-reference/HOST-RADIO-SAFETY.md)
- [What is still C, and what it costs to be Rust](04-reference/PURE-RUST-PATH.md)
- [One radio, three protocols](04-reference/RADIO-PROTOCOLS.md)
- [SYSRTC: the counter does not start, and what the measurements rule out](04-reference/SYSRTC-DEFECT.md)

| [`docs/04-reference/SYSRTC-OPUS-REVIEW.md`](docs/04-reference/SYSRTC-OPUS-REVIEW.md) | Review report (SYSRTC-OPUS-REVIEW) |
- [SYSRTC, for the three protocols that will share it](04-reference/SYSRTC-FOR-THREE-PROTOCOLS.md)
- [Reading the vendor blobs (objdump, then a decompiler)](03-development/BLOB-DISASSEMBLY.md)
- [The modem configuration: the data RAIL needs and nobody documents](04-reference/MODEM-CONFIG.md)
- [Bluetooth integration: the symbol ledger](03-development/BLE-INTEGRATION.md)
- [The compare channel works, and three registers were wrong](04-reference/SYSRTC-COMPARE-WORKS.md)
- [Bluetooth: the state, measured](04-reference/BLE-STATUS.md)
| [BLE-BLOB-FINDINGS.md](04-reference/BLE-BLOB-FINDINGS.md) | What the vendor blobs do, from decompilation (no vendor code) |
