# The modem configuration: the data RAIL needs and nobody documents

Living document, started 2026-11-13. It grows as facts appear — the user's instruction was *"pode ir criando
documentação para isso, e conforme mais documentação apareça esse será adicionada"*. Every section says where
its statements come from, and the "still unknown" list is as important as the rest.

Why this file exists: when the question came up of whether the whole radio path could be Rust, the honest
answer was that the language is not the obstacle — **the data is**. RAIL is code we can read, argue about and
eventually replace; the modem configuration it consumes is a generated table whose meaning is spread across a
tool, some headers, and a lot of undocumented constants.

## What the data actually is

The structures are public, in `crates/efr32mg24-rail-sys/vendor/rail/rail_types.h`. Two levels:

```c
typedef struct RAIL_ChannelConfig {
  RAIL_RadioConfig_t phyConfigBase;        /* base radio configuration for the entries below */
  RAIL_RadioConfig_t phyConfigDeltaSubtract;
  const RAIL_ChannelConfigEntry_t *configs;
  uint32_t length;
  uint32_t signature;
  uint32_t xtalFrequencyHz;                /* the crystal this config expects */
} RAIL_ChannelConfig_t;

typedef struct RAIL_ChannelConfigEntry {
  RAIL_RadioConfig_t phyConfigDeltaAdd;
  uint32_t baseFrequency;                  /* Hz */
  uint32_t channelSpacing;                 /* Hz */
  uint16_t physicalChannelOffset;          /* subtracted from the logical channel */
  uint16_t channelNumberStart;
  uint16_t channelNumberEnd;
  RAIL_TxPower_t maxPower;
  RAIL_ChannelConfigEntryAttr_t *attr;
  RAIL_ChannelConfigEntryType_t entryType;
  uint8_t reserved[3];
  const uint8_t *stackInfo;
  ...
} RAIL_ChannelConfigEntry_t;
```

`RAIL_RadioConfig_t` is `const uint32_t *` — an opaque array of modem registers and their values. **That array
is the undocumented part**, and it is generated, not written.

## The mapping that cost a session, now explicit

`physicalChannelOffset` is subtracted from the logical channel to give the physical channel that is relative to
`baseFrequency`. For Bluetooth advertising that means:

```text
baseFrequency        2402 000 000 Hz
channelSpacing           2 000 000 Hz
physicalChannelOffset           37

logical 37 -> physical 0  -> 2402 MHz   (the first advertising channel)
logical 38 -> physical 1  -> 2426 MHz   (offset applies before spacing: see the note below)
logical 39 -> physical 2  -> 2480 MHz
```

The three advertising channels are not evenly spaced, which is why a single base/spacing pair cannot describe
them: the generated file carries several entries, and the correct physical indices on this part are the ones
measured earlier from the RAIL blob and the vendor documentation — `(0, 37)`, `(12, 38)`, `(39, 39)` as
`(physical, logical)` pairs. **The numbers are the same fact as the entry table, seen from the other side** —
that is the point: the mapping is data, and the data is where the mapping lives.

The 802.15.4 file fetched earlier by accident (`/projects/_tmp/efr32mg24-rs/rail_config.c`, base 2450 MHz,
1 MHz spacing, channels 11-26) is the same structure for the other protocol, and it is the artefact to copy
the shape from when the time comes.

## What comes from the blob, and what must come from the application

Measured, not assumed:

| Piece | Where it comes from | Evidence |
|---|---|---|
| `RAIL_BLE_Phy1Mbps` | the blob exports it — and it is **NULL** when `RAIL_BLE_SUPPORTS_1MBPS_NON_VITERBI` is 0 | measured `ble1m=0 vit=1`: only the Viterbi PHY exists |
| `RAIL_BLE_Phy1MbpsViterbi` | blob | works; the only supported 1 Mbps PHY on this part |
| `phyConfigBase` (the modem array) | **the application's generated file** | `RAIL_BLE_ConfigPhy1MbpsViterbi` asserts `RAIL_ASSERT_FAILED_INVALID_CHANNEL_CONFIG` when it is null |
| `xtalFrequencyHz` | the generated file | the config expects 39 MHz; a mismatched value asserts `RAIL_ASSERT_INVALID_XTAL_FREQUENCY` (observed as assert 66) |

So the blob contains the *code* that consumes the table, not the table. Everything the radio needs to be
configured correctly lives in a file produced by Silicon Labs' **Radio Configurator**, which is why
"download Simplicity Studio and extract what matters" stays on the list: it is the generator.

## What is still unknown (the list this document grows from)

1. **The encoding of the `phyConfigBase` array.** It is a sequence of what look like `(address, value)` pairs,
   but the address space, the terminator, and the per-field meaning are not documented in anything we have.
   A next step: parse the 802.15.4 file's array, find its length and shape, and compare two configurations
   that differ in one parameter (e.g. the same PHY at two powers) to see which entries move.
2. **What `phyConfigDeltaAdd`/`DeltaSubtract` change** relative to the base, and when the delta is applied.
3. **The `stackInfo` blob**, whose layout depends on `protocolId` (the header says so and stops there).
4. **Whether a modem config can be transplanted between parts** of the same family (b220 vs a010), which
   matters because the blobs we have are for b220.
5. **How much of it a pure-Rust path would actually need** — a fixed single-PHY configuration for one
   protocol is a much smaller ask than the full generality Radio Configurator provides.

## Where this is going

The sequence this file is meant to serve:

1. document the structures (this section — done, from the headers);
2. extract the *shape* of a real table from a generated file we have (the 802.15.4 one), without copying its
   values into a public repository;
3. obtain one generated file for **our** part and PHY, from Radio Configurator;
4. only then does "advertising in pure Rust" become a question about registers and timing, with the data in
   hand.

Step 2 is available today and is the cheapest way to learn the encoding; step 3 needs the tool.

## Log of additions

- **2026-11-13** — file created. Structures transcribed from `rail_types.h`; the channel-mapping field
  identified as the explanation for the physical/logical confusion measured the day before; the
  blob-provides-code-not-data distinction measured (`RAIL_BLE_Phy1Mbps` NULL, `phyConfigBase` null asserts);
  the five unknowns listed with the experiments that would close them.

## The table exists, and it is in the package — 2026-11-13

The claim above ("only the Radio Configurator produces it") was too strong. The Gecko SDK v4.4.5 package ships the
Radio Configurator's output for BLE on this part, generated and Zlib licensed:

    platform/radio/rail_lib/plugin/rail_util_built_in_phys/efr32xg24/sl_rail_ble_config_39MHz.c

Vendored (reduced to the 1 Mbps Viterbi 39 MHz configuration and the tables it references) under
`crates/efr32mg24-rail-sys/vendor/ble_config/`. Its values, read from the file rather than deduced:

    RAIL_ChannelConfig_t sl_rail_ble_phy_1Mbps_viterbi_39MHz_channelConfig = {
      .phyConfigBase       = sl_rail_ble_phy_1Mbps_viterbi_39MHz_modemConfigBase,
      .phyConfigDeltaSubtract = NULL,
      .configs             = sl_rail_ble_phy_1Mbps_viterbi_39MHz_channels,
      .length              = 3U,
      .signature           = 0UL,
      .xtalFrequencyHz     = 39000000UL,
    };

    entry 0: baseFrequency 2402000000, channelSpacing 2000000, physicalChannelOffset 0,
             channelNumberStart 0, channelNumberEnd 37

Three entries cover the band; the first spans channels 0..37 from 2402 MHz in 2 MHz steps.

**Two things this settles:**

1. **The crystal frequency is 39 MHz**, which is what RAIL asserts about (`RAIL_ASSERT_INVALID_XTAL_FREQUENCY`,
   code 66) when it has no configuration at all. A Bluetooth stack that starts without this table cannot configure
   RAIL, and the default implementation of `RAILCb_AssertFailed` then loops forever — which is exactly the "hang"
   this project spent a session chasing.
2. **The channel numbering is physical**: channel 0 = 2402 MHz, 12 = 2426, 39 = 2480. The advertising example's
   `(0,37)`, `(12,38)`, `(39,39)` pairs are the physical channel for the radio and the Bluetooth logical channel for
   the whitening seed, and this table's first entry confirms the physical side. `physicalChannelOffset` is 0, so no
   offset is applied.

## Why the table is vendored but not compiled

Compiling the file as a separate C unit fails at link with `duplicate symbol`, and the reason is worth recording:
`rail_types.h` includes chip-specific `.c` sources, so any translation unit of ours that includes it also defines
things the RAIL blob already defines (`sli_bootloader_preHook` from `rfhal_pti.c`, plus a pile of `RAILINT_*`).
Including a header should not add functions to the image; here it does.

The intended route is therefore to **emit the table as Rust statics**: the modem configuration is a `uint32_t[]`,
the channel entries are a small `repr(C)` array, and the `RAIL_ChannelConfig_t` is six fields. That keeps the data
(the Radio Configurator's own output) and leaves the C out, which is also this project's stated direction.

## What the table still does not explain

The Bluetooth blobs never call `RAIL_ConfigChannels` (checked: zero imports across all three archives), so the
application has to, in the order the SDK's own flex utility uses: `RAIL_Init`, then `RAIL_ConfigChannels` with this
table. Whether the stack then accepts a RAIL instance it did not create itself is the open question -- and the way
to answer it is a program that does exactly that and reads `rail_assert::last_assert()` afterwards.

