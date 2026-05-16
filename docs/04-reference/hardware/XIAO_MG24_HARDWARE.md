# XIAO MG24 Sense Hardware Details

**Date**: December 3, 2025
**Status**: Hardware on order, awaiting delivery

---

## Overview

The Seeed Studio XIAO MG24 Sense is the target hardware platform for testing and development of this Rust HAL. It provides a compact, affordable development board based on the EFR32MG24 SoC.

## Exact Chip Variant

**MCU**: **EFR32MG24B220F1536IM48-B**

### Key Differences from Current PAC

**Current PAC Status**: ⚠️ **Mismatch Identified**

- **Current PAC Generated From**: EFR32MG24**A020**F1536GM48 (A-series, revision 020)
- **XIAO MG24 Sense Uses**: EFR32MG24**B220**F1536IM48 (B-series, revision 220)

**Impact**:
- Different chip series (A vs B)
- Different chip revision (020 vs 220)
- May have peripheral differences, register changes, or errata fixes
- **We need the correct SVD file for the B220 variant**

## Chip Specifications

### Core Processor
- **CPU**: ARM Cortex-M33 with FPU @ 78 MHz
- **Architecture**: ARMv8-M with DSP instructions
- **Features**: TrustZone-M, FPU, DSP

### Memory
- **Flash**: 1536 KB
- **RAM**: 256 KB
- **GPIO**: 32 available (19 exposed on XIAO board)
- **External Flash**: 4 MB SPI flash on XIAO board

### Radio
- **Frequency**: 2.4 GHz
- **TX Power**: +19.5 dBm max
- **RX Sensitivity**: -105.4 dBm
- **Protocols**: Matter, Thread, Zigbee, BLE 5.3, Bluetooth Mesh

### Security
- **Security Level**: Secure Vault High
- **Features**: Hardware crypto acceleration, secure boot

### Special Features
- **AI/ML**: Matrix Vector Processor (MVP) hardware accelerator
- **ADC**: No High-Speed/High-Accuracy IADC
- **Package**: QFN48

## XIAO MG24 Sense Board Features

### Form Factor
- **Size**: 21 x 17.5 mm (thumb-sized)
- **Mounting**: Breadboard-friendly, castellated pads
- **USB**: USB-C connector

### On-Board Sensors (Sense variant)
- **IMU**: 6-axis accelerometer + gyroscope (LSM6DS3TR-C)
- **Microphone**: PDM digital microphone

### Power
- **Operating Voltage**: 3.3V
- **Sleep Current**: 1.95 μA ultra-low power
- **Battery**: JST 1.25mm connector for LiPo battery
- **Charging**: On-board battery charging circuit

### Connectivity
- **Antenna**: On-board PCB antenna
- **GPIO**: 19 GPIO pins exposed
- **Interfaces**: UART, I2C, SPI

### Debug
- **SWD**: Exposed debug pins for programming/debugging

## Product Lifecycle

Silicon Labs targets a **minimum 10-year lifecycle** and expects to continue full production of the EFR32MG24B220 until at least **April 2032**.

## SVD File Acquisition

### Current Status: ✅ SVD File Obtained

The B220 variant SVD file has been successfully extracted from the Silicon Labs Gecko Platform DFP pack.

### Source

**Silicon Labs Gecko Platform Device Family Pack (DFP)**
- **Pack**: SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2
- **Pack Date**: October 6, 2025
- **Download URL**: https://www.keil.arm.com/packs/geckoplatform_efr32mg24_dfp-siliconlabs/devices/
- **SVD File**: `efr32mg24-pac/svd/EFR32MG24B220F1536IM48.svd`

### Complete Device Coverage

The pack includes SVD files for **41 EFR32MG24 variants**:
- **27 A-series** variants (original silicon)
- **14 B-series** variants (newer silicon revision)

See `efr32mg24-pac/svd/README.md` for complete device list and details.

## Next Steps

### Before Hardware Arrival
1. ✅ Identify exact chip variant (EFR32MG24B220F1536IM48)
2. ✅ Obtain B220 SVD file from Silicon Labs
3. ✅ Generate new PAC for B220 variant
4. ⏳ Begin HAL development (GPIO, CMU, USART, etc.)
5. ⏳ Plan XIAO-specific BSP (Board Support Package)

### After Hardware Arrival
1. Test basic peripherals with B220 PAC and HAL
2. Verify register/peripheral implementations
3. Create XIAO MG24 BSP crate
4. Implement sensor drivers (IMU, microphone)
5. Create examples for XIAO board
6. Validate low-power modes and battery operation

## XIAO MG24 Board Support Package (Future)

### Planned Features

**Crate**: `xiao-mg24-bsp`

**Pin Definitions**:
```rust
pub mod pins {
    // Digital I/O
    pub const D0: Pin = PA08;
    pub const D1: Pin = PA09;
    // ... (all 19 GPIO pins mapped)

    // I2C (for IMU)
    pub const SDA: Pin = D4;
    pub const SCL: Pin = D5;

    // SPI
    pub const MOSI: Pin = D10;
    pub const MISO: Pin = D9;
    pub const SCK: Pin = D8;

    // UART
    pub const TX: Pin = D6;
    pub const RX: Pin = D7;

    // Special
    pub const LED_USER: Pin = PA04;  // Example
}
```

**Board-Level Abstractions**:
- IMU driver integration (LSM6DS3TR-C)
- Microphone PDM interface
- Battery monitoring
- USB serial
- On-board LED control

## References

### Hardware Documentation
- **Product Page**: https://www.seeedstudio.com/Seeed-XIAO-MG24-Sense-p-6248.html
- **Wiki**: https://wiki.seeedstudio.com/xiao_mg24_getting_started/
- **Zephyr Docs**: https://docs.zephyrproject.org/latest/boards/seeed/xiao_mg24/doc/index.html

### Silicon Labs Resources
- **Chip Page**: https://www.silabs.com/wireless/zigbee/efr32mg24-series-2-socs/device.efr32mg24b220f1536im48
- **Datasheet**: EFR32MG24 Wireless SoC Family Data Sheet
- **Community**: https://community.silabs.com

### SVD File Resources
- **CMSIS-SVD**: https://cmsis.arm.com (requires EULA acceptance)
- **Silicon Labs Docs**: https://siliconlabs.github.io/Gecko_SDK_Doc/CMSIS/SVD/html/

## Comparison: Current PAC vs XIAO Hardware

| Feature | Current PAC (A020) | XIAO Hardware (B220) | Compatible? |
|---------|-------------------|---------------------|-------------|
| Core | Cortex-M33 @ 78MHz | Cortex-M33 @ 78MHz | ✅ Yes |
| Flash | 1536 KB | 1536 KB | ✅ Yes |
| RAM | 256 KB | 256 KB | ✅ Yes |
| Package | GM48 (QFN48) | IM48 (QFN48) | ⚠️ Different variant |
| Series | A-series | B-series | ❌ Different series |
| Revision | 020 | 220 | ❌ Different revision |
| MVP | Unknown | Yes (confirmed) | ⚠️ Unknown for A020 |
| IADC HS/HA | Unknown | No (confirmed) | ⚠️ Unknown for A020 |

**Recommendation**: **Obtain B220 SVD file** to ensure full compatibility and access to all hardware features.

---

**Document Version**: 1.1
**Last Updated**: December 3, 2025
**Status**: B220 PAC generated, awaiting hardware delivery
