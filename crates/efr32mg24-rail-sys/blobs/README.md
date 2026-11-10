# Binary Blobs Directory

This directory contains Silicon Labs proprietary binary libraries (`.a` files) for RAIL.

## Important

**These files are NOT included in the repository** due to MSLA licensing restrictions.

You must obtain them from Gecko SDK yourself.

## How to Obtain

### 1. Download Gecko SDK

Download from Silicon Labs:
```bash
git clone https://github.com/SiliconLabs/gecko_sdk.git
cd gecko_sdk
git checkout v4.4.5  # Or latest stable version
```

### 2. Locate RAIL Blob

For EFR32MG24B220 (XIAO MG24 Sense):
```bash
# Path in Gecko SDK:
# platform/radio/rail_lib/autogen/librail_release/

find gecko_sdk -name "librail_efr32xg24_gcc_release.a"
```

### 3. Copy to This Directory

```bash
# For EFR32MG24B220
cp gecko_sdk/platform/radio/rail_lib/autogen/librail_release/librail_efr32xg24_gcc_release.a \
   efr32mg24-rail-sys/blobs/efr32mg24b220/
```

## Directory Structure

```
blobs/
├── README.md (this file)
└── efr32mg24b220/
    └── librail_efr32xg24_gcc_release.a  (NOT in git, you must add)
```

## License

Binary blobs are licensed under Silicon Labs MSLA only.

See [LICENSE-MSLA](../LICENSE-MSLA) for restrictions.

## File Size

Expected blob size: ~500 KB

## Verification

To verify the blob is correct:

```bash
file blobs/efr32mg24b220/librail_efr32xg24_gcc_release.a
# Should show: current ar archive

# Check symbols
arm-none-eabi-nm blobs/efr32mg24b220/librail_efr32xg24_gcc_release.a | grep RAIL_Init
# Should show RAIL function symbols
```

## .gitignore

Binary blobs are excluded via `.gitignore`:
```
*.a
*.o
*.so
*.lib
```

This prevents accidental commits of MSLA-licensed files.
