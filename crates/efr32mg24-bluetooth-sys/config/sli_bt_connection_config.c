/***************************************************************************//**
 * @file sli_bt_connection_config.c
 * @brief Configuration instance for the "connection" feature.
 *******************************************************************************
 * # License
 * <b>Copyright 2021 Silicon Laboratories Inc. www.silabs.com</b>
 *******************************************************************************
 *
 * SPDX-License-Identifier: Zlib
 *
 * The licensor of this software is Silicon Laboratories Inc.
 *
 * This software is provided 'as-is', without any express or implied warranty. In no event will the authors be held
 * liable for any damages arising from the use of this software.
 *
 * Permission is granted to anyone to use this software for any purpose, including commercial applications, and to
 * alter it and redistribute it freely, subject to the following restrictions:
 *
 * 1. The origin of this software must not be misrepresented; you must not claim that you wrote the original
 *    software.
 * 2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original
 *    software.
 * 3. This notice may not be removed or altered from any source distribution.
 *
 ******************************************************************************/

/* Written by hand, and it is the only hand-written piece of the vendor's generated configuration.
 *
 * In an SDK project the component system generates one of these per feature, filled from the values in
 * `sl_bluetooth_config.h`. This project builds the stack from the vendor blobs, so the generator is not available
 * and the initialiser's link fails on exactly one symbol: `sli_feature_bt_connection_config`.
 *
 * The values are not invented: both come from the configuration headers in this same vendor tree, and the shape of
 * the assignment is the one the SDK's own macro documents --
 *
 *   SLI_BT_DEFINE_FEATURE_CONFIG(bt, advertiser, sli_bt_advertiser_config_t) = { .max_advertisers = ... };
 *
 * Enabling the connection feature without its configuration object is what produced `undefined symbol:
 * sli_feature_bt_connection_config` at link time.
 */

#include "sl_bluetooth_config.h"
#include "sli_bt_config_defs.h"
#include "sli_bt_connection_config.h"

SLI_BT_DEFINE_FEATURE_CONFIG(bt, connection, sli_bt_connection_config_t) = {
  .max_connections = SL_BT_CONFIG_MAX_CONNECTIONS_SUM,
  .data_len = SL_BT_CONFIG_CONNECTION_DATA_LENGTH,
};
