# EFR32MG24 SVD Files

## Source

**Silicon Labs Gecko Platform Device Family Pack (DFP)**
- **Pack**: SiliconLabs.GeckoPlatform_EFR32MG24_DFP
- **Version**: 2025.6.2
- **Date**: October 6, 2025
- **Download**: https://www.keil.arm.com/packs/geckoplatform_efr32mg24_dfp-siliconlabs/devices/

## Device Coverage

Total: **41 SVD files**
- **A-series**: 27 variants
- **B-series**: 14 variants

### A-Series Variants (27)

#### A010 Family (8 devices)
- EFR32MG24A010F768IM40.svd
- EFR32MG24A010F768IM48.svd
- EFR32MG24A010F1024IM40.svd
- EFR32MG24A010F1024IM48.svd
- EFR32MG24A010F1536GM40.svd
- EFR32MG24A010F1536GM48.svd
- EFR32MG24A010F1536IM40.svd
- EFR32MG24A010F1536IM48.svd

#### A020 Family (7 devices)
- EFR32MG24A020F768IM40.svd
- EFR32MG24A020F1024IM40.svd
- EFR32MG24A020F1024IM48.svd
- EFR32MG24A020F1536GM40.svd
- EFR32MG24A020F1536GM48.svd
- EFR32MG24A020F1536IM40.svd
- EFR32MG24A020F1536IM48.svd

#### A021 Family (1 device)
- EFR32MG24A021F1024IM40.svd

#### A110/A111/A120/A121 Family (4 devices)
- EFR32MG24A110F1024IM48.svd
- EFR32MG24A110F1536GM48.svd
- EFR32MG24A111F1536GM48.svd
- EFR32MG24A120F1536GM48.svd
- EFR32MG24A121F1536GM48.svd

#### A410/A420 Family (4 devices)
- EFR32MG24A410F1536IM40.svd
- EFR32MG24A410F1536IM48.svd
- EFR32MG24A420F1536IM40.svd
- EFR32MG24A420F1536IM48.svd

#### A610/A620 Family (2 devices)
- EFR32MG24A610F1536IM40.svd
- EFR32MG24A620F1536IM40.svd

### B-Series Variants (14)

#### B010 Family (3 devices)
- EFR32MG24B010F1024IM48.svd
- EFR32MG24B010F1536IM40.svd
- EFR32MG24B010F1536IM48.svd

#### B020 Family (3 devices)
- EFR32MG24B020F1024IM48.svd
- EFR32MG24B020F1536IM40.svd
- EFR32MG24B020F1536IM48.svd

#### B110/B120 Family (3 devices)
- EFR32MG24B110F1536GM48.svd
- EFR32MG24B110F1536IM48.svd
- EFR32MG24B120F1536IM48.svd

#### B210/B220 Family (3 devices)
- EFR32MG24B210F1536IM40.svd
- EFR32MG24B210F1536IM48.svd
- **EFR32MG24B220F1536IM48.svd** ⭐ (Seeed Studio XIAO MG24 Sense)

#### B310 Family (1 device)
- EFR32MG24B310F1536IM48.svd

#### B610 Family (1 device)
- EFR32MG24B610F1536IM40.svd

## Key Differences: A-Series vs B-Series

### A-Series
- Earlier silicon revision
- Original EFR32MG24 family
- Widely available in evaluation kits

### B-Series
- Newer silicon revision
- Improvements and bug fixes over A-series
- Used in newer development boards (like Seeed Studio XIAO MG24)
- May have additional features or peripheral changes

## Target Hardware

**Seeed Studio XIAO MG24 Sense**
- **Chip**: EFR32MG24B220F1536IM48-B
- **SVD File**: `EFR32MG24B220F1536IM48.svd`
- **Memory**: 1536 KB Flash, 256 KB RAM
- **Package**: QFN48 (IM48)
- **TX Power**: 19.5 dBm (B220 = high power variant)

## Naming Convention

Format: `EFR32MG24{Series}{Revision}F{Flash}{Package}.svd`

- **Series**: A or B
- **Revision**: 010, 020, 110, 120, 210, 220, etc.
- **Flash**: 768, 1024, or 1536 (KB)
- **Package**:
  - **GM**: QFN48 with specific pin mapping
  - **IM**: QFN48 with standard pin mapping
  - Last digit (40/48): Number of pins

### Revision Codes

Common patterns:
- **x10**: Base variant (e.g., A010, B010)
- **x20**: Enhanced variant (e.g., A020, B020)
- **x21**: Specialized variant
- **110/111**: With TrustZone high security
- **120/121**: With additional features
- **210**: Medium power radio (10 dBm)
- **220**: High power radio (19.5 dBm)
- **310**: Advanced features
- **410/420**: Industrial temperature range
- **610/620**: Extended features

## Usage

### Generating PAC for Specific Device

```bash
# For Seeed Studio XIAO MG24 Sense
svd2rust -i svd/EFR32MG24B220F1536IM48.svd \
         --target cortex-m \
         -o src

# For generic A020 variant
svd2rust -i svd/EFR32MG24A020F1536GM48.svd \
         --target cortex-m \
         -o src
```

### Multi-Device Support

To support multiple devices in one crate, use Cargo features:

```toml
[features]
# B-series (newer, for XIAO MG24 and similar boards)
efr32mg24b220f1536 = []
efr32mg24b210f1536 = []

# A-series (older, for Silicon Labs dev kits)
efr32mg24a020f1536 = []

# Default to XIAO MG24 variant
default = ["efr32mg24b220f1536"]
```

## PAC Generation Status

| Variant | Generated | Tested | Notes |
|---------|-----------|--------|-------|
| EFR32MG24A020F1536GM48 | ✅ | ⏳ | Initial PAC (before B-series discovered) |
| EFR32MG24B220F1536IM48 | ⏳ | ⏳ | Target for XIAO MG24 Sense |
| Others | ⏳ | ⏳ | Available for generation |

## References

- **Keil Pack Portal**: https://www.keil.arm.com/packs/
- **Silicon Labs EFR32MG24**: https://www.silabs.com/wireless/zigbee/efr32mg24-series-2-socs
- **CMSIS-SVD Specification**: https://www.keil.com/pack/doc/CMSIS/SVD/html/

---

**Last Updated**: December 3, 2025
**Pack Version**: 2025.6.2 (October 6, 2025)
**Total Devices**: 41 variants
