# Binary Blobs Directory

This directory contains Silicon Labs proprietary binary libraries (`.a` files) for the Bluetooth stack.

## Important

**These files are NOT included in the repository** due to MSLA licensing restrictions.

You must obtain them from Gecko SDK yourself.

## How to Obtain

### Automated Setup (Recommended)

Run the setup script from the crate root:

```bash
cd efr32mg24-bluetooth-sys
bash scripts/setup-bluetooth.sh
```

This will:
1. Check/install Git LFS
2. Clone Gecko SDK v4.4.5
3. Extract Bluetooth blob
4. Extract BGAPI headers
5. Clean up temporary files

### Manual Setup

#### 1. Download Gecko SDK

```bash
git clone https://github.com/SiliconLabs/gecko_sdk.git
cd gecko_sdk
git checkout v4.4.5
git lfs install
git lfs pull --include="protocol/bluetooth/lib/*.a"
```

#### 2. Locate Bluetooth Blob

For EFR32MG24B220 (XIAO MG24 Sense):

```bash
# Path in Gecko SDK:
# protocol/bluetooth/lib/libbluetooth_efr32xg24_gcc_release.a
# or
# protocol/bluetooth/bgstack/lib/libbluetooth_efr32xg24_gcc_release.a

find gecko_sdk -name "libbluetooth_efr32xg24_gcc_release.a"
```

#### 3. Copy to This Directory

```bash
# For EFR32MG24B220
cp gecko_sdk/protocol/bluetooth/lib/libbluetooth_efr32xg24_gcc_release.a \
   efr32mg24-bluetooth-sys/blobs/efr32mg24b220/
```

## Directory Structure

```
blobs/
├── README.md (this file)
└── efr32mg24b220/
    └── libbluetooth_efr32xg24_gcc_release.a  (NOT in git, you must add)
```

## License

Binary blobs are licensed under Silicon Labs MSLA only.

See [LICENSE-MSLA](../LICENSE-MSLA) for restrictions.

## File Size

Expected blob size: ~1.5-2 MB

## Verification

To verify the blob is correct:

```bash
file blobs/efr32mg24b220/libbluetooth_efr32xg24_gcc_release.a
# Should show: current ar archive

# Check symbols (requires ARM toolchain)
arm-none-eabi-nm blobs/efr32mg24b220/libbluetooth_efr32xg24_gcc_release.a | grep sl_bt_init
# Should show Bluetooth function symbols
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
