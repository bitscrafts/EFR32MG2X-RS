# Silicon Labs EFR32MG24 SVD Pack Extraction Procedure

**Date**: December 3, 2025
**Status**: ✅ Complete - All 41 SVD files extracted

---

## Overview

This document describes the procedure used to obtain and extract all EFR32MG24 SVD files from the official Silicon Labs Gecko Platform Device Family Pack (DFP).

## Pack Information

### Source

**Download Page**: https://www.keil.arm.com/packs/geckoplatform_efr32mg24_dfp-siliconlabs/devices/

### Pack Details

- **Name**: Silicon Labs Gecko Platform EFR32MG24 DFP
- **Full Name**: `SiliconLabs.GeckoPlatform_EFR32MG24_DFP`
- **Version**: 2025.6.2
- **Release Date**: October 6, 2025
- **File Size**: 21 MB
- **Format**: ZIP archive (with .pack extension)

### Package Contents

- **SVD Files**: 41 device variants
- **Header Files**: C header files for all variants
- **Startup Code**: Device initialization code
- **Linker Scripts**: Memory layout definitions
- **Documentation**: Device reference documentation
- **Flash Algorithms**: Programming algorithms for development tools

## Download Procedure

### Step 1: Navigate to Pack Download Page

1. Visit: https://www.keil.arm.com/packs/geckoplatform_efr32mg24_dfp-siliconlabs/devices/
2. The page lists all 41 EFR32MG24 device variants in the pack
3. Scroll down to find the download link

### Step 2: Download Pack File

**Direct Download Link**:
```
https://www.keil.arm.com/packs/SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack
```

**Download Command** (if using wget or curl):
```bash
# Using wget
wget https://www.keil.arm.com/packs/SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack

# Using curl
curl -O https://www.keil.arm.com/packs/SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack
```

**Downloaded to**: `efr32mg24-pac/SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack`

### Step 3: Verify Pack File

```bash
# Check file size and type
ls -lh SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack
# Output: 21M

# Verify it's a ZIP archive
file SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack
# Output: Zip archive data, at least v2.0 to extract
```

## Extraction Procedure

### Understanding .pack File Format

ARM CMSIS Packs use the `.pack` extension but are standard ZIP archives. They can be extracted using any ZIP utility.

### Step 1: Inspect Pack Contents

```bash
# List all SVD files in the pack
unzip -l SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack | grep "\.svd"

# Count SVD files
unzip -l SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack | grep "\.svd" | wc -l
# Output: 41
```

### Step 2: Extract SVD Files

**Command Used**:
```bash
cd efr32mg24-pac

# Extract all SVD files to svd/ folder
# -o: overwrite files without prompting
# -j: junk paths (extract without directory structure)
# -d: destination directory
unzip -o -j SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack \
      "SVD/EFR32MG24/*.svd" \
      -d svd/
```

**Extraction Output**:
```
Archive:  SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack
  inflating: svd/EFR32MG24A010F1024IM40.svd
  inflating: svd/EFR32MG24A010F1024IM48.svd
  ... (39 more files)
  inflating: svd/EFR32MG24B220F1536IM48.svd  ⭐ (XIAO MG24 Sense)
  inflating: svd/EFR32MG24B310F1536IM48.svd
  inflating: svd/EFR32MG24B610F1536IM40.svd
```

### Step 3: Verify Extraction

```bash
# Count extracted files
ls -1 svd/*.svd | wc -l
# Output: 41

# Count by series
ls -1 svd/ | grep "^EFR32MG24A" | wc -l  # A-series: 27
ls -1 svd/ | grep "^EFR32MG24B" | wc -l  # B-series: 14

# Verify XIAO MG24 variant exists
ls svd/EFR32MG24B220F1536IM48.svd
# Output: svd/EFR32MG24B220F1536IM48.svd ✅
```

### Step 4: Verify SVD File Integrity

```bash
# Check file size (should be ~5-6 MB)
ls -lh svd/EFR32MG24B220F1536IM48.svd
# Output: -rw-r--r-- 1 user 5.7M Oct  6 19:08 ...

# Verify it's valid XML
file svd/EFR32MG24B220F1536IM48.svd
# Output: XML 1.0 document text, ASCII text, with CRLF line terminators

# Check XML structure (first few lines)
head -5 svd/EFR32MG24B220F1536IM48.svd
```

Expected output:
```xml
<?xml version="1.0" encoding="utf-8" standalone="no"?>
<device schemaVersion="1.1" ...>
  <name>EFR32MG24B220F1536IM48</name>
  <description>...</description>
  ...
```

## Extracted Files Inventory

### Complete Device List (41 variants)

#### A-Series (27 devices)

**A010 Family** (8 devices):
- EFR32MG24A010F768IM40.svd (768KB Flash, IM40)
- EFR32MG24A010F768IM48.svd (768KB Flash, IM48)
- EFR32MG24A010F1024IM40.svd (1MB Flash, IM40)
- EFR32MG24A010F1024IM48.svd (1MB Flash, IM48)
- EFR32MG24A010F1536GM40.svd (1.5MB Flash, GM40)
- EFR32MG24A010F1536GM48.svd (1.5MB Flash, GM48)
- EFR32MG24A010F1536IM40.svd (1.5MB Flash, IM40)
- EFR32MG24A010F1536IM48.svd (1.5MB Flash, IM48)

**A020 Family** (7 devices):
- EFR32MG24A020F768IM40.svd
- EFR32MG24A020F1024IM40.svd
- EFR32MG24A020F1024IM48.svd
- EFR32MG24A020F1536GM40.svd
- EFR32MG24A020F1536GM48.svd (originally used for PAC generation)
- EFR32MG24A020F1536IM40.svd
- EFR32MG24A020F1536IM48.svd

**A021 Family** (1 device):
- EFR32MG24A021F1024IM40.svd

**A110/A111/A120/A121 Family** (5 devices):
- EFR32MG24A110F1024IM48.svd
- EFR32MG24A110F1536GM48.svd
- EFR32MG24A111F1536GM48.svd
- EFR32MG24A120F1536GM48.svd
- EFR32MG24A121F1536GM48.svd

**A410/A420 Family** (4 devices):
- EFR32MG24A410F1536IM40.svd
- EFR32MG24A410F1536IM48.svd
- EFR32MG24A420F1536IM40.svd
- EFR32MG24A420F1536IM48.svd

**A610/A620 Family** (2 devices):
- EFR32MG24A610F1536IM40.svd
- EFR32MG24A620F1536IM40.svd

#### B-Series (14 devices)

**B010 Family** (3 devices):
- EFR32MG24B010F1024IM48.svd
- EFR32MG24B010F1536IM40.svd
- EFR32MG24B010F1536IM48.svd

**B020 Family** (3 devices):
- EFR32MG24B020F1024IM48.svd
- EFR32MG24B020F1536IM40.svd
- EFR32MG24B020F1536IM48.svd

**B110/B120 Family** (3 devices):
- EFR32MG24B110F1536GM48.svd
- EFR32MG24B110F1536IM48.svd
- EFR32MG24B120F1536IM48.svd

**B210/B220 Family** (3 devices):
- EFR32MG24B210F1536IM40.svd (10 dBm TX power)
- EFR32MG24B210F1536IM48.svd (10 dBm TX power)
- **EFR32MG24B220F1536IM48.svd** ⭐ (19.5 dBm TX power - Seeed Studio XIAO MG24 Sense)

**B310 Family** (1 device):
- EFR32MG24B310F1536IM48.svd

**B610 Family** (1 device):
- EFR32MG24B610F1536IM40.svd

## SVD File Specifications

### File Format

- **Format**: XML (CMSIS-SVD format)
- **Version**: CMSIS-SVD 1.1
- **Encoding**: UTF-8
- **Line Endings**: CRLF (Windows-style)

### File Sizes

- **A-series**: ~5.2 MB each
- **B-series (B010-B120)**: ~5.2-5.3 MB each
- **B-series (B210-B310)**: ~6.0 MB each (larger due to additional features)

### SVD Content

Each SVD file contains:
- **Device Information**: Name, description, version
- **CPU Configuration**: Cortex-M33, clock frequency, FPU, MPU
- **Memory Map**: Flash, RAM, peripheral base addresses
- **Peripheral Definitions**: All peripheral registers, fields, enumerations
- **Interrupt Definitions**: All interrupt vectors

**Example Peripheral Count**:
- A020 variants: ~133 peripherals
- B220 variants: ~133+ peripherals (may have additional features)

## Target Device Verification

### Seeed Studio XIAO MG24 Sense

**Confirmed Chip**: EFR32MG24B220F1536IM48-B

**Verification Sources**:
1. **Schematic**: https://files.seeedstudio.com/wiki/XIAO_MG24/Getting_Start/XIAO_MG24_SCH.pdf
2. **Datasheet**: https://files.seeedstudio.com/wiki/XIAO_MG24/Getting_Start/mg24-group-datasheet.PDF
3. **Zephyr Docs**: https://docs.zephyrproject.org/latest/boards/seeed/xiao_mg24/doc/index.html

**SVD File**: `svd/EFR32MG24B220F1536IM48.svd` ✅

## Usage

### Generating PAC from B220 SVD

```bash
cd efr32mg24-pac

# Generate PAC for XIAO MG24 Sense
svd2rust -i svd/EFR32MG24B220F1536IM48.svd \
         --target cortex-m \
         -o src

# Expected output: src/lib.rs (~22 MB, ~221,000 lines)
```

### Generating PACs for Other Variants

```bash
# For any A-series variant
svd2rust -i svd/EFR32MG24A020F1536GM48.svd \
         --target cortex-m \
         -o src

# For any B-series variant
svd2rust -i svd/EFR32MG24B210F1536IM48.svd \
         --target cortex-m \
         -o src
```

## File Organization

### Current Structure

```
efr32mg24-pac/
├── svd/
│   ├── README.md                           # SVD inventory documentation
│   ├── EFR32MG24A010F1024IM40.svd         # A-series variants (27 files)
│   │   ... (26 more A-series files)
│   ├── EFR32MG24B010F1024IM48.svd         # B-series variants (14 files)
│   │   ... (13 more B-series files)
│   └── EFR32MG24B220F1536IM48.svd         # XIAO MG24 Sense ⭐
├── src/
│   └── lib.rs                              # Generated PAC (currently A020)
├── Cargo.toml
├── build.rs
├── memory.x
└── SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack  # Original pack (kept for reference)
```

## Pack Management

### Keep Pack File?

**Decision**: ✅ **Keep the pack file**

**Reasons**:
1. **Reference**: Easy access to other resources (headers, linker scripts, docs)
2. **Verification**: Can always re-extract if SVD files are corrupted
3. **Updates**: Compare with future pack versions
4. **Documentation**: Shows exact source and version

**Location**: `efr32mg24-pac/SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack`

### Future Updates

To update SVD files when new pack versions are released:

```bash
# Download new pack
cd efr32mg24-pac
wget https://www.keil.arm.com/packs/SiliconLabs.GeckoPlatform_EFR32MG24_DFP.YYYY.M.V.pack

# Extract to temporary folder
mkdir svd_new
unzip -o -j SiliconLabs.GeckoPlatform_EFR32MG24_DFP.YYYY.M.V.pack \
      "SVD/EFR32MG24/*.svd" \
      -d svd_new/

# Compare with current SVD files
diff -r svd/ svd_new/

# If changes found, update svd folder
mv svd/ svd_old/
mv svd_new/ svd/
```

## Troubleshooting

### Issue: Cannot Download Pack

**Solution**: Use alternative download locations:
- ARM Keil website: https://www.keil.arm.com/packs/
- Silicon Labs GitHub: Check for community-maintained copies
- CMSIS-Pack Registry: https://www.keil.arm.com/cmsis

### Issue: Extraction Fails

**Solution**:
```bash
# Verify pack file integrity
unzip -t SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack

# Re-download if corrupted
rm SiliconLabs.GeckoPlatform_EFR32MG24_DFP.2025.6.2.pack
wget https://www.keil.arm.com/packs/...
```

### Issue: SVD File Invalid

**Verification**:
```bash
# Check XML validity
xmllint --noout svd/EFR32MG24B220F1536IM48.svd

# Test with svd2rust
svd2rust -i svd/EFR32MG24B220F1536IM48.svd --target cortex-m -o /tmp/test
```

## Next Steps

1. ✅ All 41 SVD files extracted successfully
2. ✅ B220 variant confirmed for XIAO MG24 Sense
3. ⏳ Generate new PAC for EFR32MG24B220F1536IM48
4. ⏳ Compare B220 vs A020 peripheral differences
5. ⏳ Plan multi-device support strategy
6. ⏳ Update HAL for B220 compatibility

## References

- **Pack Download**: https://www.keil.arm.com/packs/geckoplatform_efr32mg24_dfp-siliconlabs/devices/
- **CMSIS-Pack Specification**: https://open-cmsis-pack.github.io/Open-CMSIS-Pack-Spec/main/html/index.html
- **CMSIS-SVD Specification**: https://www.keil.com/pack/doc/CMSIS/SVD/html/
- **svd2rust**: https://github.com/rust-embedded/svd2rust

---

**Document Version**: 1.0
**Last Updated**: December 3, 2025
**Status**: Complete - All SVD files successfully extracted
**Total SVD Files**: 41 (27 A-series + 14 B-series)
