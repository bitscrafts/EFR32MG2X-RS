/* Component catalogue for this firmware.
 *
 * Generated files of this name list which software components are compiled in; the Bluetooth stack's initialiser
 * reads it to decide which features to register and which BGAPI classes to expose. This project builds the stack
 * from the vendor blobs, so the list is chosen to match what those blobs contain rather than produced by a
 * configurator:
 *
 *   system, GAP, connections, legacy advertising, GATT and GATT server, and NVM for the bonding database.
 *
 * Nothing about scanning, extended or periodic advertising, direction finding, channel sounding, PAwR, L2CAP or
 * the connection analyzer is enabled: those classes exist in the blob but this firmware does not use them, and a
 * feature table that claims them would make the host expect resources the application never provides.
 */
#ifndef SL_COMPONENT_CATALOG_H
#define SL_COMPONENT_CATALOG_H

#define SL_CATALOG_BLUETOOTH_FEATURE_SYSTEM_PRESENT 1
#define SL_CATALOG_BLUETOOTH_FEATURE_NVM_PRESENT 1
#define SL_CATALOG_BLUETOOTH_FEATURE_GAP_PRESENT 1
#define SL_CATALOG_BLUETOOTH_FEATURE_CONNECTION_PRESENT 1
// The "advertiser" component, and *not* legacy_advertiser/extended/periodic: `sl_bt_stack_init.c` enables the
// advertiser's backwards-compatibility feature only when none of those superseding components is present, and it is
// that feature that makes `sl_bt_advertiser_create_set` and friends do anything. Measured: with both present,
// `create_set` answered SL_STATUS_NOT_SUPPORTED (0x000F).
#define SL_CATALOG_BLUETOOTH_FEATURE_ADVERTISER_PRESENT 1
#define SL_CATALOG_BLUETOOTH_FEATURE_GATT_PRESENT 1
#define SL_CATALOG_BLUETOOTH_FEATURE_GATT_SERVER_PRESENT 1
// Security manager and bonding: pairing, encryption, and a key store that survives a reboot.
// **Off, deliberately.** Enabling the security manager made every pairing attempt end in SMP reason 8 with the
// arena's free list damaged (see docs/04-reference/BLE-STATUS.md). Pairing is not required for a working GATT service,
// and carrying a security manager that corrupts the heap puts every *other* example at risk. The PSA layer it needs
// stays in the crate, tested and verified on the board (P-256 key generation, export, ECDH), so re-enabling is a
// catalogue change plus the free-list defect -- not a rewrite.
//
// **Off in the product configuration.** The heap defect that used to make this fatal is fixed (see the slack in
// `libraries/bt-heap`), and with the fix a pairing attempt gets past the allocation and dies with SMP reason 0x0B -- a
// DHKey-check failure reported with **zero** elliptic-curve calls made (the counters in `psa_ec` say generate=0,
// export=0, agree=0), so the peer is the one declaring the failure and the security manager never reached the crypto.
// Pairing is not needed for the GATT product that is delivered and verified, and a component that fails halfway should
// not be in the product build.
//
// #define SL_CATALOG_BLUETOOTH_FEATURE_SM_PRESENT 1
// #define SL_CATALOG_BLUETOOTH_FEATURE_BUILTIN_BONDING_DATABASE_PRESENT 1
/* The dynamic GATT database: enabled, and the reason it was blocked for a day.
 *
 * Turning it on made `sli_bt_system_start_bluetooth` return `SL_STATUS_TIMEOUT` and `system_boot` never arrived, which
 * looked like "this component cannot work in this build". It could: the feature's configuration instance had the wrong
 * type. `bt_dynamic_gattdb_init` is thirty bytes long and reads `flags & 1` (GATT caching) and `(flags >> 1) & 1`
 * (include the static database) from the first word of its argument, so it wants a `sli_bt_gattdb_config_t`, not the
 * `sli_bt_dynamic_gattdb_config_t` its own header advertises. With the wrong shape the flags were garbage and the
 * feature failed during stack start.
 *
 * `config/sli_bt_dynamic_gattdb_config.c` now carries the right type with flags 0 -- no static database to include,
 * nothing to cache -- and the stack boots with a database built at run time. */
#define SL_CATALOG_BLUETOOTH_FEATURE_DYNAMIC_GATTDB_PRESENT 1
// Scanning. The header comment above used to say this was deliberately absent, on the grounds that a feature table
// must not claim resources the application never provides. That reasoning was backwards for this component: the
// scanner's state is static in the link layer, it asks for no per-feature memory from `ll_initAllocMem` (which takes
// only connections, advertising sets and whitelist entries), and without the entry the scanner class is never
// registered -- so `sl_bt_scanner_start` answers 0 and no report ever arrives, which is indistinguishable from a dead
// receiver. Measured on the board before this line existed: the scanner example ran, reported that it had started,
// and saw nothing while the host was advertising at -57 dBm.
#define SL_CATALOG_BLUETOOTH_FEATURE_SCANNER_PRESENT 1

#endif /* SL_COMPONENT_CATALOG_H */
