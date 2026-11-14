/* Reduced from the SDK's sl_rail_ble_config_39MHz.c (Zlib, Silicon Laboratories).
 *
 * Only the 1 Mbps Viterbi 39 MHz configuration. **`rail_ble.h` is deliberately not included**: it drags in
 * definitions the RAIL blob already provides, and the link fails with `duplicate symbol: RAILINT_...` naming an
 * rfhal module -- including a header should not add a function to the image, and here it did.
 *
 * Only the 1 Mbps Viterbi 39 MHz configuration and the tables it references are kept: the full file pulls in
 * symbols that collide with the RAIL blob at link time. The values are carried across unchanged by
 * /projects/_tmp/efr32mg24-rs/reduce-ble-config.py -- this is generated data, not a transcription.
 */
#ifndef SL_RAIL_BLE_1MBPS_39MHZ_H
#define SL_RAIL_BLE_1MBPS_39MHZ_H

#include <stdint.h>
#include "rail_types.h"

#ifdef __cplusplus
extern "C" {
#endif

extern const RAIL_ChannelConfig_t sl_rail_ble_phy_1Mbps_viterbi_39MHz_channelConfig;

#ifdef __cplusplus
}
#endif

#endif
