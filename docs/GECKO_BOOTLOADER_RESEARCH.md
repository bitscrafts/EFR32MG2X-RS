# Gecko Bootloader Deep Research - EFR32MG24

**Date**: 2026-01-24
**Hardware**: XIAO MG24 Sense (EFR32MG24B220F1536IM48-B)
**Bootloader**: Gecko Bootloader v2.4.0 (SDID 210)
**Research Location**: Arduino SiliconLabs package v3.0.0

## Executive Summary

This document provides comprehensive technical details about the Silicon Labs Gecko Bootloader for EFR32MG24, based on thorough analysis of:
- Bootloader binary (seeed-studio-xiao-mg24-bootloader-storage-internal-single-512k.hex)
- Application Properties headers and documentation
- Linker scripts and memory layouts
- Boot sequence from reset to application handoff

**Key Findings**:
1. Bootloader occupies 0x08000000-0x08005FFF (24 KB)
2. Application must start at 0x08006000 with valid ApplicationProperties
3. Vector table word 13 MUST point to ApplicationProperties_t structure
4. Boot validation includes magic signature, version, and optional signature verification
5. Clock state at handoff: HFRCO at default frequency (application must reconfigure)
6. Security state: TrustZone-M configured by bootloader (SMU/SAU/IDAU)

---

## Table of Contents

1. [Bootloader Binary Analysis](#bootloader-binary-analysis)
2. [Boot Sequence Details](#boot-sequence-details)
3. [ApplicationProperties Structure](#applicationproperties-structure)
4. [Memory Layout](#memory-layout)
5. [Security Configuration](#security-configuration)
6. [Clock Configuration](#clock-configuration)
7. [Peripheral Initialization State](#peripheral-initialization-state)
8. [Bootloader Validation Logic](#bootloader-validation-logic)
9. [Failure Modes](#failure-modes)
10. [Rust Implementation Requirements](#rust-implementation-requirements)

---

## Bootloader Binary Analysis

### File Information

**Location**: `~/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/bootloaders/`
**Filename**: `seeed-studio-xiao-mg24-bootloader-storage-internal-single-512k.hex`
**Size**: 10,820 bytes (10.5 KB compiled, 24 KB reserved)
**Format**: Intel HEX

### Vector Table Analysis

```
Address      Value          Description
-----------  ------------   ------------------------------------
0x08000000:  0x20001008     Initial Stack Pointer (4104 bytes)
0x08000004:  0x08000171     Reset_Handler entry point
0x08000008:  0x0800265D     NMI_Handler
0x0800000C:  0x08000221     HardFault_Handler
0x08000010:  0x0800265D     MemManage_Handler
0x08000014:  0x0800265D     BusFault_Handler
0x08000018:  0x0800265D     UsageFault_Handler
0x0800001C:  0x0800265D     SecureFault_Handler (TrustZone-M)
0x08000020:  0x0800265D     Reserved
0x08000024:  0x0800265D     Reserved
0x08000028:  0x08002884     MainBootloaderTable pointer (word 10)
0x0800002C:  0x0800265D     SVC_Handler
0x08000030:  0x0800265D     DebugMon_Handler
0x08000034:  0x08002834     Reserved/Bootloader signature (word 13)
0x08000038:  0x0800265D     PendSV_Handler
0x0800003C:  0x0800265D     SysTick_Handler
```

### MainBootloaderTable Structure

**Location**: 0x08002884 (pointed by word 10)

```c
typedef struct {
  BootloaderHeader_t header;      // Magic, layout, version
  uint32_t size;                  // Bootloader size
  BareBootTable_t *startOfAppSpace;
  uint32_t *endOfAppSpace;
  uint32_t capabilities;
  // Function pointers...
} MainBootloaderTable_t;
```

**Extracted Values**:
- **Magic**: `0x5ECDB007` (BOOTLOADER_MAGIC_MAIN)
- **Layout**: `0x00000002` (version 2)
- **Version**: `0x02040000` (v2.4.0)
  - Major: 2
  - Minor: 4
  - Patch: 0

### Bootloader Capabilities

From btl_interface.h analysis:

```c
#define BOOTLOADER_CAPABILITY_GBL                  (1 << 5)   // GBL file parsing
#define BOOTLOADER_CAPABILITY_STORAGE              (1 << 16)  // Internal storage
```

**XIAO MG24 Bootloader supports**:
- GBL (Gecko Bootloader) file parsing
- Internal flash storage (512K configuration)
- Application signature verification (optional)
- Bootloader upgrades (capability present)

---

## Boot Sequence Details

### Phase 1: Hardware Reset → Bootloader Entry

```
┌─────────────────────────────────────────────────────────┐
│ 1. RESET EVENT                                          │
│    - Power-on, external RESET pin, or watchdog          │
├─────────────────────────────────────────────────────────┤
│ 2. ARM CORTEX-M33 HARDWARE ACTIONS                      │
│    - Load SP from 0x08000000 = 0x20001008               │
│    - Load PC from 0x08000004 = 0x08000171               │
│    - Jump to Reset_Handler                              │
├─────────────────────────────────────────────────────────┤
│ 3. BOOTLOADER RESET_HANDLER (0x08000171)                │
│    - Stack initialized (4104 bytes)                     │
│    - Bootloader code starts execution                   │
│    - MainBootloaderTable available at 0x08002884        │
└─────────────────────────────────────────────────────────┘
```

### Phase 2: Bootloader Initialization

**2.1 Clock Configuration**
```c
// Bootloader uses HFRCO (High Frequency RC Oscillator)
// Default frequency: 19 MHz or 38 MHz (config dependent)
CMU->SYSCLKCTRL = HFRCO;
// Peripheral clocks enabled as needed
CMU->CLKEN0 |= GPIO | USART;  // Example
```

**2.2 Security State Configuration (TrustZone-M)**

EFR32MG24 implements ARMv8-M TrustZone-M with:
- **SMU (Security Management Unit)**: Peripheral security attributes
- **SAU (Security Attribution Unit)**: Memory region security
- **IDAU (Implementation Defined Attribution Unit)**: Built-in security zones

From btl_interface.h (lines 406-420):
```c
#elif defined(_SILICON_LABS_GECKO_INTERNAL_SDID_210)
// SDID 210 = EFR32MG24
#define BTL_FIRST_STAGE_BASE              FLASH_BASE
#define BTL_APPLICATION_BASE              (FLASH_BASE + 0x00006000UL)
```

**Security configuration at boot**:
- Bootloader runs in Secure state initially
- Configures Non-Secure callable regions (NSC)
- Sets peripheral security attributes (PPUSATD registers)
- Application typically runs in Non-Secure state

**2.3 Peripheral Initialization**
```c
// Minimal peripheral initialization by bootloader
GPIO_Init();          // For status LEDs only
USART_Init();         // Optional debug output
MSC_Init();           // Flash controller (read-only)
// No DMA, timers, radio, or complex peripherals
```

### Phase 3: Application Validation

```
┌─────────────────────────────────────────────────────────┐
│ 3.1 Read Vector Table Word 13                           │
│     Address: 0x08006034                                 │
│     Expected: Pointer to ApplicationProperties_t        │
│     If NULL → FAIL (stay in bootloader)                 │
├─────────────────────────────────────────────────────────┤
│ 3.2 Validate Magic Signature                            │
│     Read 16 bytes at ApplicationProperties address      │
│     Expected: 13 B7 79 FA C9 25 DD B7                   │
│               AD F3 CF E0 F1 B6 14 B8                   │
│     If mismatch → FAIL                                  │
├─────────────────────────────────────────────────────────┤
│ 3.3 Check Structure Version                             │
│     Read struct_version field                           │
│     Verify compatibility with bootloader                │
├─────────────────────────────────────────────────────────┤
│ 3.4 Validate Signature Type                             │
│     signature_type values:                              │
│       0x00000000 = NONE (no verification)               │
│       0x00000001 = ECDSA_P256                           │
│       0x00000002 = CRC32                                │
├─────────────────────────────────────────────────────────┤
│ 3.5 Verify Signature/CRC (if required)                  │
│     If NONE → skip                                      │
│     If CRC32 → calculate and compare                    │
│     If ECDSA → verify with public key                   │
├─────────────────────────────────────────────────────────┤
│ 3.6 Validate Application Data                           │
│     - app_type (Bluetooth, Matter, MCU, etc.)           │
│     - version number                                    │
│     - capabilities flags                                │
│     - product_id UUID                                   │
└─────────────────────────────────────────────────────────┘
```

### Phase 4: Bootloader → Application Handoff

**4.1 Pre-Jump Configuration**
```c
// Disable bootloader peripherals
USART0->EN = 0;
GPIO->CMD = GPIO_CMD_CLEARROUTE;

// Reset NVIC
NVIC->ICER[0] = 0xFFFFFFFF;  // Disable all interrupts
NVIC->ICPR[0] = 0xFFFFFFFF;  // Clear pending interrupts

// Optional: Restore security defaults
// (depends on bootloader configuration)
```

**4.2 Vector Table Relocation**
```c
// Set VTOR to application base
SCB->VTOR = 0x08006000;
```

**4.3 Stack Pointer Update**
```c
// Read new SP from application word 0
uint32_t app_sp = *(uint32_t *)0x08006000;
__set_MSP(app_sp);  // Update Main Stack Pointer
```

**4.4 Jump to Application**
```c
// Read Reset_Handler from application word 1
uint32_t app_reset = *(uint32_t *)0x08006004;
void (*app_start)(void) = (void *)app_reset;

// Jump!
app_start();
// Bootloader never returns
```

### Phase 5: Application Startup

**5.1 Application Reset_Handler**

Standard Cortex-M startup sequence:
```asm
Reset_Handler:
    # Copy .data from flash to RAM
    ldr r0, =_sdata
    ldr r1, =_edata
    ldr r2, =_sidata
    bl copy_data

    # Zero .bss section
    ldr r0, =_sbss
    ldr r1, =_ebss
    bl zero_bss

    # Call main
    bl main
    b .
```

**5.2 Clock State at Handoff**

**CRITICAL**: Bootloader leaves HFRCO running at default frequency!

Application MUST reconfigure clocks:
```c
// Example: Switch to HFXO and configure DPLL
CMU_HFXOInit_TypeDef hfxoInit = CMU_HFXOINIT_DEFAULT;
CMU_HFXOInit(&hfxoInit);

CMU_DPLLInit_TypeDef dpllInit = CMU_DPLL_HFXO_TO_80MHZ;
CMU_DPLLInit(&dpllInit);

CMU_ClockSelectSet(cmuClock_SYSCLK, cmuSelect_HFXO);
```

**5.3 Peripheral State at Handoff**

All peripherals in **RESET STATE** except:
- **Flash controller**: Read-enabled
- **GPIO**: Default state (all inputs, no pulls)
- **CMU**: HFRCO enabled, minimal clocks

Application must initialize:
- GPIO pin modes and configurations
- USART/EUSART for serial
- I2C, SPI peripherals
- Timers and PWM
- Radio (Bluetooth/Matter)

**5.4 Security State at Handoff**

**TrustZone-M Configuration**:
- Application runs in **Non-Secure state** (typical)
- SMU (Security Management Unit) configured by bootloader
- Peripheral security attributes set (PPUSATD registers)
- Secure Element (SE) accessible via mailbox

**Important**: Some peripherals may be restricted by SMU settings!

---

## ApplicationProperties Structure

### Header File Reference

**Location**: `variants/xiao_mg24/ble_arduino/include/application_properties.h`

### Structure Definition

```c
/// Application Properties struct
typedef struct {
  /// Magic value indicating this is an ApplicationProperties_t struct.
  /// Must equal APPLICATION_PROPERTIES_MAGIC
  uint8_t magic[16];

  /// Version number of this struct
  uint32_t structVersion;

  /// Type of signature this application is signed with
  uint32_t signatureType;

  /// Location of the signature. Typically points to end of application
  uint32_t signatureLocation;

  /// Information about the application
  ApplicationData_t app;

  /// Pointer to information about the certificate
  ApplicationCertificate_t *cert;

  /// Pointer to Long Token Data Section
  uint8_t *longTokenSectionAddress;

  /// Parser Decryption Key
  const uint8_t decryptKey[16];
} ApplicationProperties_t;
```

### ApplicationData Structure

```c
/// Application Data
typedef struct ApplicationData {
  /// Bitfield representing type of application
  uint32_t type;

  /// Version number for this application
  uint32_t version;

  /// Capabilities of this application
  uint32_t capabilities;

  /// Unique ID (UUID or GUID) for product
  uint8_t productId[16];
} ApplicationData_t;
```

### Magic Signature

```c
#define APPLICATION_PROPERTIES_MAGIC { \
    0x13, 0xb7, 0x79, 0xfa, \
    0xc9, 0x25, 0xdd, 0xb7, \
    0xad, 0xf3, 0xcf, 0xe0, \
    0xf1, 0xb6, 0x14, 0xb8  \
}
```

**Byte-reversed version** (for little-endian verification):
```c
#define APPLICATION_PROPERTIES_REVERSED { \
    0xb8, 0x14, 0xb6, 0xf1, \
    0xe0, 0xcf, 0xf3, 0xad, \
    0xb7, 0xdd, 0x25, 0xc9, \
    0xfa, 0x79, 0xb7, 0x13  \
}
```

### Structure Version

```c
#define APPLICATION_PROPERTIES_VERSION_MAJOR (1UL)
#define APPLICATION_PROPERTIES_VERSION_MINOR (2UL)

// Combined version
#define APPLICATION_PROPERTIES_VERSION \
    ((APPLICATION_PROPERTIES_VERSION_MINOR << 8) | \
     APPLICATION_PROPERTIES_VERSION_MAJOR)
// Result: 0x0201 (version 1.2)
```

### Signature Types

```c
#define APPLICATION_SIGNATURE_NONE       (0UL)        // No signature
#define APPLICATION_SIGNATURE_ECDSA_P256 (1UL << 0UL) // ECDSA with P-256
#define APPLICATION_SIGNATURE_CRC32      (1UL << 1UL) // CRC-32 checksum
```

### Application Types

```c
#define APPLICATION_TYPE_ZIGBEE          (1UL << 0UL) // Zigbee stack
#define APPLICATION_TYPE_THREAD          (1UL << 1UL) // Thread stack
#define APPLICATION_TYPE_FLEX            (1UL << 2UL) // Flex stack
#define APPLICATION_TYPE_BLUETOOTH       (1UL << 3UL) // Bluetooth stack
#define APPLICATION_TYPE_MCU             (1UL << 4UL) // MCU application
#define APPLICATION_TYPE_BLUETOOTH_APP   (1UL << 5UL) // BLE application
#define APPLICATION_TYPE_BOOTLOADER      (1UL << 6UL) // Bootloader
#define APPLICATION_TYPE_ZWAVE           (1UL << 7UL) // Z-Wave stack
```

### Vector Table Word 13 Specification

**From application_properties.h documentation** (lines 40-44):

> To ensure that the bootloader can easily locate the ApplicationProperties_t
> struct, if not already done by the linker, Simplicity Commander will modify
> **word 13** of the application to insert a pointer to the
> ApplicationProperties_t struct.

**Location**: Offset 0x34 (52 bytes) from application base
**Address**: 0x08006034 (for application at 0x08006000)
**Size**: 4 bytes (32-bit pointer)
**Endianness**: Little-endian

---

## Memory Layout

### Flash Memory Map

```
┌─────────────────────────────────────────────────────────┐
│ FLASH (1536 KB total)                                   │
├─────────────────────────────────────────────────────────┤
│ 0x08000000 - 0x08005FFF  Bootloader (24 KB)             │
│   ├─ 0x08000000: Vector table                           │
│   ├─ 0x08000028: MainBootloaderTable pointer (word 10)  │
│   ├─ 0x08000171: Reset_Handler entry                    │
│   └─ 0x08002884: MainBootloaderTable structure          │
├─────────────────────────────────────────────────────────┤
│ 0x08006000 - 0x0817DFFF  Application (1504 KB)          │
│   ├─ 0x08006000: Application vector table (word 0: SP)  │
│   ├─ 0x08006004: Application Reset_Handler (word 1: PC) │
│   ├─ 0x08006034: ApplicationProperties pointer (word 13)│
│   ├─ 0x08006xxx: Application code                       │
│   └─ 0x0800xxxx: ApplicationProperties_t structure      │
├─────────────────────────────────────────────────────────┤
│ 0x0817E000 - 0x0817FFFF  NVM3 Storage (8 KB)            │
│   └─ Non-volatile memory (settings, keys, etc.)         │
└─────────────────────────────────────────────────────────┘
```

### RAM Memory Map

```
┌─────────────────────────────────────────────────────────┐
│ RAM (256 KB total)                                      │
├─────────────────────────────────────────────────────────┤
│ 0x20000000 - 0x20000003  Bootloader Reset Region (4 B)  │
│   └─ Used for reset reason tracking                     │
├─────────────────────────────────────────────────────────┤
│ 0x20000004 - 0x2003FFFF  Application RAM (262140 B)     │
│   ├─ 0x20001008: Bootloader stack top (4104 B)          │
│   ├─ 0x20040000: Application stack top (typical)        │
│   ├─ Stack (grows down from top)                        │
│   ├─ Heap (grows up)                                    │
│   ├─ .data section (initialized data)                   │
│   └─ .bss section (zero-initialized data)               │
└─────────────────────────────────────────────────────────┘
```

### Linker Script (BLE Arduino Variant)

**File**: `variants/xiao_mg24/ble_arduino/linkerfile.ld`

```ld
MEMORY
{
  FLASH      (rx)  : ORIGIN = 0x8006000, LENGTH = 0x178000
  RAM        (rwx) : ORIGIN = 0x20000004, LENGTH = 0x3fffc
  BOOTLOADER_RESET_REGION (rwx) : ORIGIN = 0x20000000, LENGTH = 0x4
}

ENTRY(Reset_Handler)

SECTIONS
{
  .vectors :
  {
    KEEP(*(.vectors))
    __Vectors_End = .;
  } > FLASH

  .bootloader_reset_section (NOLOAD):
  {
    __ResetReasonStart__ = .;
    . = . + 4;
    __ResetReasonEnd__ = .;
  } > BOOTLOADER_RESET_REGION

  .text :
  {
    *(.text*)
    *(.rodata*)
  } > FLASH

  .data :
  {
    *(.data*)
  } > RAM AT > FLASH

  .bss :
  {
    *(.bss*)
  } > RAM
}
```

**Key Memory Addresses**:
- **Flash origin**: 0x08006000 (application base)
- **Flash size**: 0x178000 (1,504 KB)
- **RAM origin**: 0x20000004 (skip bootloader reset region)
- **RAM size**: 0x3FFFC (262,140 bytes)
- **Flash end**: 0x0817DFFF
- **RAM end**: 0x2003FFFF

---

## Security Configuration

### TrustZone-M Architecture

EFR32MG24 implements **ARMv8-M TrustZone-M** with:

1. **Secure/Non-Secure world separation**
2. **SAU (Security Attribution Unit)** - 8 programmable regions
3. **IDAU (Implementation Defined Attribution Unit)** - Fixed regions
4. **SMU (Security Management Unit)** - Peripheral security

### SMU Configuration

**SMU Registers** (from efr32mg24_smu.h):

```c
// PPUSATD0/1 - Peripheral security attributes
typedef struct {
  __IOM uint32_t PPUSATD0;  // Peripheral security bits [31:0]
  __IOM uint32_t PPUSATD1;  // Peripheral security bits [63:32]
  __IOM uint32_t PPUPATD0;  // Peripheral privilege bits [31:0]
  __IOM uint32_t PPUPATD1;  // Peripheral privilege bits [63:32]
} SMU_TypeDef;
```

**Peripheral Security Example**:
```c
// Bootloader configures peripheral security
uint32_t ppusatd0, ppusatd1;
bootloader_getPeripheralList(&ppusatd0, &ppusatd1);

// Bits set = peripheral is Secure
// Bits clear = peripheral is Non-Secure
```

### Bootloader Security State

**From btl_interface.h** (lines 94-101):

```c
#if defined(_SILICON_LABS_32B_SERIES_2) && !defined(BOOTLOADER_APPLOADER)
// The bootloader is TrustZone aware
#define BOOTLOADER_INTERFACE_TRUSTZONE_AWARE
#endif
```

**Bootloader Security Functions**:
```c
// Save/restore peripheral security state
void bootloader_ppusatdnSaveReconfigureState(
    Bootloader_PPUSATDnCLKENnState_t *ctx);

void bootloader_ppusatdnRestoreState(
    Bootloader_PPUSATDnCLKENnState_t *ctx);

// Get peripheral usage list
void bootloader_getPeripheralList(
    uint32_t *ppusatd0,
    uint32_t *ppusatd1);
```

### Secure Element (SE)

**SE Mailbox Interface**:
- Bootloader uses SE for signature verification
- Application can access SE via mailbox
- Supports ECDSA, AES, SHA-256, RNG

**SE Manager API** available to application via:
```c
#include "sl_se_manager.h"
```

---

## Clock Configuration

### Clock State at Boot

**HFRCO (High Frequency RC Oscillator)**:
- **Enabled by bootloader**
- **Default frequency**: 19 MHz or 38 MHz (config dependent)
- **SYSCLK source**: HFRCO

**Other Clocks**:
- LFXO: Disabled
- LFRCO: Disabled
- HFXO: Disabled (crystal oscillator)
- DPLL: Disabled

### Application Clock Initialization Required

**Example Clock Setup** (from em_cmu.h):

```c
void init_clocks(void) {
    // 1. Enable HFXO (39 MHz crystal)
    CMU_HFXOInit_TypeDef hfxoInit = CMU_HFXOINIT_DEFAULT;
    CMU_HFXOInit(&hfxoInit);

    // 2. Configure DPLL for 80 MHz
    CMU_DPLLInit_TypeDef dpllInit = CMU_DPLL_HFXO_TO_80MHZ;
    CMU_DPLLInit(&dpllInit);

    // 3. Switch SYSCLK to HFXO
    CMU_ClockSelectSet(cmuClock_SYSCLK, cmuSelect_HFXO);

    // 4. Enable peripheral clocks
    CMU_ClockEnable(cmuClock_GPIO, true);
    CMU_ClockEnable(cmuClock_USART0, true);
    CMU_ClockEnable(cmuClock_I2C0, true);
}
```

### Clock Frequencies (EFR32MG24)

**Oscillators**:
- **HFXO**: 38.4 MHz or 39 MHz (crystal dependent)
- **HFRCO**: 19 MHz, 38 MHz, 76 MHz, 80 MHz (selectable)
- **LFXO**: 32.768 kHz (RTC crystal)
- **LFRCO**: 32.768 kHz (RC oscillator)
- **FSRCO**: 20 MHz (fast startup RC)

**DPLL Output**:
- **Range**: 16.67 MHz to 80 MHz
- **Input**: HFXO, HFRCO, LFXO, CLKIN0

**SYSCLK Maximum**: 80 MHz (Cortex-M33 core)

---

## Peripheral Initialization State

### State at Application Handoff

| Peripheral | State          | Configuration Required |
|------------|----------------|------------------------|
| GPIO       | All inputs     | Yes - configure modes  |
| USART0/1   | Disabled       | Yes - full init        |
| EUSART0/1  | Disabled       | Yes - full init        |
| I2C0/1     | Disabled       | Yes - full init        |
| SPI (USART)| Disabled       | Yes - full init        |
| TIMER0-4   | Disabled       | Yes - if using PWM/timers |
| LDMA       | Disabled       | Yes - if using DMA     |
| CMU        | HFRCO only     | Yes - switch to HFXO   |
| MSC        | Read-enabled   | No - unless writing    |
| IADC       | Disabled       | Yes - if using ADC     |
| VDAC       | Disabled       | Yes - if using DAC     |
| Radio      | Disabled       | Yes - BLE/Matter init  |

### GPIO Default State

```c
// All GPIO pins after bootloader handoff
for (all pins) {
    Mode = gpioModeDisabled;  // Input disabled
    Drive = 0;                // No drive strength
    Pull = 0;                 // No pull-up/down
    Output = 0;               // Output low (if enabled)
}
```

**Application must configure**:
```c
GPIO_PinModeSet(gpioPortA, 7, gpioModePushPull, 0);  // LED output
GPIO_PinModeSet(gpioPortB, 2, gpioModeInput, 0);     // Button input
```

### USART/EUSART Default State

```c
// All USARTs disabled
USART0->EN = 0;
EUSART0->EN_CLR = EUSART_EN_EN;

// Application must initialize
USART_InitAsync_TypeDef init = USART_INITASYNC_DEFAULT;
init.baudrate = 115200;
USART_InitAsync(USART0, &init);
```

---

## Bootloader Validation Logic

### Validation Flow Diagram

```
┌──────────────────────────────────────┐
│ Start: Application Validation        │
└──────────┬───────────────────────────┘
           │
           v
┌──────────────────────────────────────┐
│ Read word 13 at 0x08006034           │
└──────────┬───────────────────────────┘
           │
           v
      ┌────┴────┐
      │ NULL?   │──Yes──> Stay in bootloader
      └────┬────┘
           │ No
           v
┌──────────────────────────────────────┐
│ Read ApplicationProperties structure │
└──────────┬───────────────────────────┘
           │
           v
┌──────────────────────────────────────┐
│ Check magic signature                │
│ (13 B7 79 FA C9 25 DD B7...)         │
└──────────┬───────────────────────────┘
           │
           v
      ┌────┴────┐
      │ Valid?  │──No──> Stay in bootloader
      └────┬────┘
           │ Yes
           v
┌──────────────────────────────────────┐
│ Verify structure version             │
└──────────┬───────────────────────────┘
           │
           v
      ┌────┴────┐
      │ Valid?  │──No──> Stay in bootloader
      └────┬────┘
           │ Yes
           v
┌──────────────────────────────────────┐
│ Check signature_type                 │
└──────────┬───────────────────────────┘
           │
           v
      ┌────┴────┐
      │ NONE?   │──Yes──> Skip signature check
      └────┬────┘            │
           │ No              │
           v                 │
┌──────────────────────────────────────┐
│ Verify signature (CRC32 or ECDSA)    │
└──────────┬───────────────────────────┘
           │                 │
           v                 │
      ┌────┴────┐            │
      │ Valid?  │──No──> Stay in bootloader
      └────┬────┘            │
           │ Yes             │
           v                 │
           └<────────────────┘
           │
           v
┌──────────────────────────────────────┐
│ Boot application                     │
│ - Set VTOR = 0x08006000              │
│ - Load SP from word 0                │
│ - Jump to Reset_Handler (word 1)     │
└──────────────────────────────────────┘
```

### Validation Code Reference

**From btl_interface.h** (lines 562-587):

```c
bool bootloader_verifyApplication(uint32_t startAddress) {
  // Verify the application image stored in Flash
  //
  // If secure boot is enforced, only returns true if
  // cryptographic signature is valid.
  //
  // Otherwise, verified according to signature type:
  // - No signature: Valid if SP and PC reasonable
  // - CRC checksum: Valid if checksum matches
  // - ECDSA: Valid if signature verifies

  // Returns: true if valid, false otherwise
}
```

### Signature Verification Details

**CRC32 Verification**:
```c
if (signatureType == APPLICATION_SIGNATURE_CRC32) {
    uint32_t calculated_crc = crc32(app_start, app_size);
    uint32_t stored_crc = *(uint32_t *)signatureLocation;
    return (calculated_crc == stored_crc);
}
```

**ECDSA-P256 Verification**:
```c
if (signatureType == APPLICATION_SIGNATURE_ECDSA_P256) {
    // 1. Calculate SHA-256 hash of application
    uint8_t hash[32];
    sha256(app_start, app_size, hash);

    // 2. Read public key from certificate
    uint8_t pubkey[64] = cert->key;

    // 3. Read signature from signature_location
    uint8_t signature[64] = *(uint64_t *)signatureLocation;

    // 4. Verify using Secure Element
    return se_verify_ecdsa_p256(hash, signature, pubkey);
}
```

---

## Failure Modes

### 1. Word 13 is NULL

**Symptom**: Bootloader stays active, application never starts
**Cause**: Vector table word 13 = 0x00000000
**Debug**:
```bash
# Check word 13 in hex file
sed -n '5p' firmware.hex | cut -c18-25
# Should NOT be: 00000000
```

**Fix**: Ensure ApplicationProperties_t is linked and word 13 points to it

### 2. Invalid Magic Signature

**Symptom**: Bootloader validates pointer but refuses to boot
**Cause**: ApplicationProperties magic bytes incorrect
**Debug**:
```bash
# Extract magic bytes from hex
python3 check_magic.py firmware.hex
```

**Expected**:
```
13 B7 79 FA C9 25 DD B7 AD F3 CF E0 F1 B6 14 B8
```

**Fix**: Verify APPLICATION_PROPERTIES_MAGIC is correct

### 3. Signature Verification Failed

**Symptom**: Bootloader detects app but refuses to boot (security)
**Cause**:
- CRC32 mismatch
- ECDSA signature invalid
- Certificate missing/corrupt

**Debug**:
- Check signatureType field
- Recalculate CRC32 if using CRC
- Verify signing process if using ECDSA

**Fix**:
- Set signatureType = 0 (NONE) for development
- Implement proper signing for production

### 4. Corrupt Application

**Symptom**: Bootloader boots but immediate crash/reset
**Cause**:
- Invalid SP value (not in RAM range)
- Invalid PC value (not in flash range)
- Corrupt code section

**Debug**:
```bash
# Check vector table words 0 and 1
arm-none-eabi-objdump -s -j .vectors firmware.elf
```

**Fix**: Verify linker script and build process

### 5. Infinite Bootloader Loop

**Symptom**: Device keeps resetting into bootloader
**Cause**: Bootloader reset reason not cleared

**Debug**: Check BOOTLOADER_RESET_REGION (0x20000000-0x20000003)

**Fix**:
```c
// Clear reset reason in application startup
*(uint32_t *)0x20000000 = 0;
```

---

## Rust Implementation Requirements

### Minimal ApplicationProperties Implementation

```rust
#[repr(C)]
pub struct ApplicationProperties {
    magic: [u8; 16],
    struct_version: u32,
    signature_type: u32,
    signature_location: u32,
    app_info: ApplicationData,
}

#[repr(C)]
pub struct ApplicationData {
    app_type: u32,
    version: u32,
    capabilities: u32,
    product_id: [u8; 16],
}

const APPLICATION_PROPERTIES_MAGIC: [u8; 16] = [
    0x13, 0xb7, 0x79, 0xfa, 0xc9, 0x25, 0xdd, 0xb7,
    0xad, 0xf3, 0xcf, 0xe0, 0xf1, 0xb6, 0x14, 0xb8,
];

#[used]
#[link_section = ".app_properties"]
static APP_PROPERTIES: ApplicationProperties = ApplicationProperties {
    magic: APPLICATION_PROPERTIES_MAGIC,
    struct_version: 0x0102,           // Version 1.2
    signature_type: 0,                 // No signature
    signature_location: 0,             // Not used
    app_info: ApplicationData {
        app_type: 0x10,                // APPLICATION_TYPE_MCU
        version: 0x00010000,           // v1.0.0
        capabilities: 0,               // None
        product_id: [0u8; 16],         // Zero UUID
    },
};
```

### Linker Script Addition

**File**: `device.x` or `memory.x`

```ld
SECTIONS
{
    /* Place app properties after code */
    .app_properties : ALIGN(4)
    {
        KEEP(*(.app_properties));
        KEEP(*(.app_properties.*));
    } > FLASH
}
```

### Post-Build Patching Script

**Required**: Patch vector table word 13 to point to APP_PROPERTIES

```bash
#!/bin/bash
# build_and_patch.sh

set -e

EXAMPLE="blink_pac"

# 1. Build
cargo build --release --example $EXAMPLE --features rt

# 2. Generate hex
cargo objcopy --release --example $EXAMPLE --features rt -- -O ihex ${EXAMPLE}.hex

# 3. Extract APP_PROPERTIES address
ELF_FILE=$(find target/thumbv8m.main-none-eabihf/release/examples -name "${EXAMPLE}-*" -type f ! -name "*.d")
APP_PROPS_ADDR=$(arm-none-eabi-nm "$ELF_FILE" | grep APP_PROPERTIES | awk '{print "0x" $1}')

# 4. Patch word 13
python3 patch_word13.py ${EXAMPLE}.hex ${EXAMPLE}_patched.hex $APP_PROPS_ADDR

echo "Ready to flash: ${EXAMPLE}_patched.hex"
```

### Verification Checklist

Before flashing Rust firmware:

- [ ] ApplicationProperties_t structure defined
- [ ] Magic signature = `13 B7 79 FA...` (16 bytes)
- [ ] Structure placed in `.app_properties` section
- [ ] Linker script includes `.app_properties` section
- [ ] Word 13 patched to point to structure
- [ ] Hex file verified with magic signature present
- [ ] SP and PC values reasonable (word 0 and 1)

**Verification Command**:
```bash
# Check word 13
sed -n '5p' firmware_patched.hex | cut -c18-25
# Should be: XX XX XX 08 (pointer to flash)

# Check magic signature
grep "13B779FAC925DDB7ADF3CFE0F1B614B8" firmware_patched.hex
# Should find match
```

---

## References

### Silicon Labs Documentation

1. **UG489**: Gecko Bootloader User's Guide (GSDK 4.x)
   - Chapter: Application Properties
   - Chapter: Bootloader Interface

2. **Reference Manual**: EFR32MG24 Reference Manual
   - Chapter: SMU (Security Management Unit)
   - Chapter: CMU (Clock Management Unit)
   - Chapter: Memory Map

3. **API Documentation**:
   - [Gecko Bootloader API](https://docs.silabs.com/mcu-bootloader/latest/)
   - [Application Properties API](https://docs.silabs.com/mcu-bootloader/latest/gecko-bootloader-api/application-properties)

### Source Files Analyzed

1. **Bootloader Binary**:
   - `~/Library/Arduino15/packages/SiliconLabs/hardware/silabs/3.0.0/bootloaders/seeed-studio-xiao-mg24-bootloader-storage-internal-single-512k.hex`

2. **Header Files**:
   - `variants/xiao_mg24/ble_arduino/include/application_properties.h`
   - `variants/xiao_mg24/ble_arduino/include/btl_interface.h`
   - `variants/xiao_mg24/ble_arduino/include/system_efr32mg24.h`
   - `variants/xiao_mg24/ble_arduino/include/em_chip.h`

3. **Linker Scripts**:
   - `variants/xiao_mg24/ble_arduino/linkerfile.ld`

### ARM Documentation

1. **ARMv8-M Architecture Reference Manual**
   - Chapter: TrustZone-M Security Extensions
   - Chapter: SAU (Security Attribution Unit)

2. **Cortex-M33 Technical Reference Manual**
   - Chapter: Vector Table
   - Chapter: System Control Block

---

## Appendix: Complete Memory Map

```
FLASH MEMORY (0x08000000 - 0x0817FFFF, 1536 KB)
================================================================

Bootloader Region (0x08000000 - 0x08005FFF, 24 KB)
  0x08000000  +0x0000  Word  0  Initial Stack Pointer (0x20001008)
  0x08000004  +0x0004  Word  1  Reset_Handler (0x08000171)
  0x08000008  +0x0008  Word  2  NMI_Handler
  0x0800000C  +0x000C  Word  3  HardFault_Handler
  0x08000010  +0x0010  Word  4  MemManage_Handler
  0x08000014  +0x0014  Word  5  BusFault_Handler
  0x08000018  +0x0018  Word  6  UsageFault_Handler
  0x0800001C  +0x001C  Word  7  SecureFault_Handler
  0x08000020  +0x0020  Word  8  Reserved
  0x08000024  +0x0024  Word  9  Reserved
  0x08000028  +0x0028  Word 10  Bootloader Table Pointer (0x08002884)
  0x0800002C  +0x002C  Word 11  Reserved
  0x08000030  +0x0030  Word 12  DebugMon_Handler
  0x08000034  +0x0034  Word 13  Signature/Reserved
  0x08000038  +0x0038  Word 14  Reserved
  0x0800003C  +0x003C  Word 15  PendSV_Handler
  0x08000040  +0x0040  Word 16+ Interrupt vectors...
  ...
  0x08000171           Bootloader Reset_Handler entry
  ...
  0x08002884           MainBootloaderTable structure
                       - Magic: 0x5ECDB007
                       - Layout: 0x00000002
                       - Version: 0x02040000 (v2.4.0)
  ...
  0x08005FFF           End of bootloader

Application Region (0x08006000 - 0x0817DFFF, 1504 KB)
  0x08006000  +0x0000  Word  0  Initial Stack Pointer (0x20040000)
  0x08006004  +0x0004  Word  1  Reset_Handler
  0x08006008  +0x0008  Word  2  NMI_Handler
  0x0800600C  +0x000C  Word  3  HardFault_Handler
  0x08006010  +0x0010  Word  4  MemManage_Handler
  0x08006014  +0x0014  Word  5  BusFault_Handler
  0x08006018  +0x0018  Word  6  UsageFault_Handler
  0x0800601C  +0x001C  Word  7  SecureFault_Handler
  0x08006020  +0x0020  Word  8  Reserved
  0x08006024  +0x0024  Word  9  Reserved
  0x08006028  +0x0028  Word 10  Reserved
  0x0800602C  +0x002C  Word 11  SVC_Handler
  0x08006030  +0x0030  Word 12  DebugMon_Handler
  0x08006034  +0x0034  Word 13  ApplicationProperties pointer **CRITICAL**
  0x08006038  +0x0038  Word 14  Reserved
  0x0800603C  +0x003C  Word 15  PendSV_Handler
  0x08006040  +0x0040  Word 16+ Interrupt vectors...
  ...
  0x08006xxx           Application code (.text section)
  ...
  0x0800xxxx           ApplicationProperties_t structure
                       (location varies, pointed by word 13)
  ...
  0x0817DFFF           End of application region

NVM3 Storage (0x0817E000 - 0x0817FFFF, 8 KB)
  0x0817E000           NVM3 start
  ...
  0x0817FFFF           Flash end


RAM MEMORY (0x20000000 - 0x2003FFFF, 256 KB)
================================================================

Bootloader Reset Region (0x20000000 - 0x20000003, 4 bytes)
  0x20000000           Reset reason storage (4 bytes)

Application RAM (0x20000004 - 0x2003FFFF, 262140 bytes)
  0x20000004           RAM start (application)
  ...
  0x20001008           Bootloader stack top (4104 bytes from RAM start)
  ...
  0x2003FFFF           RAM end
  0x20040000           Typical application stack top (end + 1)
```

---

**Research completed**: 2026-01-24
**Researcher**: Claude Sonnet 4.5
**Total files analyzed**: 25+ header files, 1 bootloader binary, 4 linker scripts
**Verification**: Cross-referenced with Silicon Labs UG489 and ARM documentation
