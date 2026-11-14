// Minimal em_device.h stub for RAIL bindings generation
// Defines EFR32MG24 device macros needed by rail_features.h
//
// IMPORTANT: This stub is configured for EFR32MG24 (Series 2 Config 4).
// For other EFR32 variants, adjust _SILICON_LABS_32B_SERIES_2_CONFIG:
//   - EFR32MG24: CONFIG = 4
//   - EFR32MG25: CONFIG = 5
//   - EFR32MG27: CONFIG = 7
//   - EFR32MG28: CONFIG = 8
//
// For multi-device support, these defines should be generated dynamically
// in build.rs based on target feature flags.

#ifndef EM_DEVICE_H
#define EM_DEVICE_H

#include <stdint.h>
#include <stdbool.h>

// Define EFR32MG24 series (Series 2 Config 4)
// This configuration is specific to EFR32MG24B220F1536IM48-B
#define EFR32MG24 1
#define _EFR32_MIGHTY_FAMILY 1
#define _SILICON_LABS_32B_SERIES 2
#define _SILICON_LABS_32B_SERIES_2 1
#define _SILICON_LABS_32B_SERIES_2_CONFIG 4
#define _SILICON_LABS_32B_SERIES_2_CONFIG_4 1

// IRQ numbers (minimal set)
typedef enum IRQn {
    Reset_IRQn                    = -15,
    NonMaskableInt_IRQn          = -14,
    HardFault_IRQn               = -13,
    MemoryManagement_IRQn        = -12,
    BusFault_IRQn                = -11,
    UsageFault_IRQn              = -10,
    SVCall_IRQn                  = -5,
    DebugMonitor_IRQn            = -4,
    PendSV_IRQn                  = -2,
    SysTick_IRQn                 = -1,
} IRQn_Type;

// Core definitions
#define __CM33_REV                0x0001U
#define __MPU_PRESENT             1U
#define __VTOR_PRESENT            1U
#define __NVIC_PRIO_BITS          4U
#define __Vendor_SysTickConfig    0U
#define __FPU_PRESENT             1U
#define __DSP_PRESENT             1U

#endif // EM_DEVICE_H
