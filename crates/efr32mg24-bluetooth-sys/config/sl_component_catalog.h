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

#endif /* SL_COMPONENT_CATALOG_H */
