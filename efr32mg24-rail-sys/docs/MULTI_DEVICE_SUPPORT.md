# Multi-Device Support Plan

## Overview

This document outlines the strategy for extending `efr32mg24-rail-sys` to support multiple EFR32 device variants beyond the initial EFR32MG24B220.

## Current State

**Supported Device**: EFR32MG24B220F1536IM48-B (Series 2 Config 4)

**Hardcoded Components**:
1. `include/em_device.h` - Device-specific macros
2. `blobs/efr32mg24b220/` - Device-specific RAIL blob
3. `build.rs` - Single blob path

## Target Devices (Future)

Silicon Labs EFR32 Series 2 family:

| Device | Series 2 Config | Flash | RAM | Radio | Notes |
|--------|----------------|-------|-----|-------|-------|
| **EFR32MG24** | 4 | 1536 KB | 256 KB | 2.4 GHz | **Current** |
| EFR32MG25 | 5 | 1536 KB | 256 KB | 2.4 GHz + Sub-GHz | Dual-band |
| EFR32MG27 | 7 | 1024 KB | 256 KB | 2.4 GHz | Lower power |
| EFR32MG28 | 8 | 1024 KB | 256 KB | 2.4 GHz | Ultra low power |

## Implementation Strategy

### Phase 1: Feature Flags (Cargo Features)

Add device-specific feature flags to `Cargo.toml`:

```toml
[features]
default = ["efr32mg24"]

# Device variants
efr32mg24 = []
efr32mg25 = []
efr32mg27 = []
efr32mg28 = []
```

### Phase 2: Dynamic Device Configuration

Generate `em_device.h` dynamically in `build.rs`:

```rust
fn generate_device_header(device: &str) -> String {
    let (config, config_macro) = match device {
        "efr32mg24" => (4, "_SILICON_LABS_32B_SERIES_2_CONFIG_4"),
        "efr32mg25" => (5, "_SILICON_LABS_32B_SERIES_2_CONFIG_5"),
        "efr32mg27" => (7, "_SILICON_LABS_32B_SERIES_2_CONFIG_7"),
        "efr32mg28" => (8, "_SILICON_LABS_32B_SERIES_2_CONFIG_8"),
        _ => panic!("Unknown device: {}", device),
    };

    format!(r#"
// Auto-generated device header for {}
#define _SILICON_LABS_32B_SERIES_2_CONFIG {}
#define {} 1
"#, device, config, config_macro)
}
```

### Phase 3: Multi-Blob Support

Restructure blob directory:

```
blobs/
├── efr32mg24/
│   └── librail_efr32xg24_gcc_release.a
├── efr32mg25/
│   └── librail_efr32xg25_gcc_release.a
├── efr32mg27/
│   └── librail_efr32xg27_gcc_release.a
└── efr32mg28/
    └── librail_efr32xg28_gcc_release.a
```

Update `build.rs` to select blob based on feature:

```rust
let device = if cfg!(feature = "efr32mg24") {
    "efr32mg24"
} else if cfg!(feature = "efr32mg25") {
    "efr32mg25"
} else if cfg!(feature = "efr32mg27") {
    "efr32mg27"
} else if cfg!(feature = "efr32mg28") {
    "efr32mg28"
} else {
    panic!("No device feature selected");
};

let blob_dir = manifest_dir.join(format!("blobs/{}", device));
```

### Phase 4: SHA256 Verification Table

Maintain a table of expected hashes for each device variant:

```rust
const BLOB_HASHES: &[(&str, &str)] = &[
    ("efr32mg24", "072f0ff0eb708091e49a3fd536c45f4f0568ed6f163c2889f5a8536b234c66a2"),
    ("efr32mg25", "TBD_HASH_FOR_MG25"),
    ("efr32mg27", "TBD_HASH_FOR_MG27"),
    ("efr32mg28", "TBD_HASH_FOR_MG28"),
];
```

### Phase 5: Documentation Updates

Update README.md to document multi-device support:

```markdown
## Device Selection

Select your target device using Cargo features:

```toml
[dependencies]
efr32mg24-rail-sys = { version = "0.1", features = ["efr32mg25"] }
\```

Available devices:
- `efr32mg24` (default) - EFR32MG24 (Series 2 Config 4)
- `efr32mg25` - EFR32MG25 (Series 2 Config 5) - Dual-band
- `efr32mg27` - EFR32MG27 (Series 2 Config 7) - Lower power
- `efr32mg28` - EFR32MG28 (Series 2 Config 8) - Ultra low power
```

## Migration Path

For users currently on single-device support:

1. **No breaking changes** - `efr32mg24` is default feature
2. **Opt-in** - New devices require explicit feature flag
3. **Backward compatible** - Existing code continues to work

## Testing Strategy

1. **Unit tests** - Test device header generation
2. **Integration tests** - Verify blob linking for each device
3. **CI/CD** - Test all device features in parallel
4. **Documentation tests** - Ensure examples compile for all devices

## Timeline

- **Phase 1**: Add feature flags (1 day)
- **Phase 2**: Dynamic device configuration (2 days)
- **Phase 3**: Multi-blob support (1 day)
- **Phase 4**: SHA256 verification (1 day)
- **Phase 5**: Documentation (1 day)

**Total**: ~1 week for complete multi-device support

## Dependencies

- Gecko SDK for each device variant
- RAIL blobs for MG25, MG27, MG28
- Testing hardware for validation (optional)

## Risk Assessment

**Low Risk**:
- Feature flags are standard Rust practice
- Backward compatible design
- Incremental implementation

**Medium Risk**:
- Obtaining RAIL blobs for all variants
- SHA256 hash verification
- Testing without hardware

**Mitigation**:
- Start with MG24 (already working)
- Add devices incrementally as blobs become available
- Use build.rs warnings for missing blobs

## Success Criteria

✅ All device variants compile successfully
✅ SHA256 verification passes for all blobs
✅ Documentation covers all devices
✅ CI/CD tests all features
✅ No breaking changes for existing users

---

**Status**: Planning Phase
**Target**: Phase D (Radio & Security)
**Priority**: Medium (after basic RAIL wrapper works)

<!-- META: last_updated=2025-12-28 version=0.1.0 phase=D1-RAIL status=planning -->
