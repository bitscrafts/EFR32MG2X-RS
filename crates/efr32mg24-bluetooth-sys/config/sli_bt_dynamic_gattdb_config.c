/***************************************************************************//**
 * @file sli_bt_dynamic_gattdb_config.c
 * @brief Configuration instance for the "dynamic_gattdb" feature.
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

/* Written by hand, like `sli_bt_connection_config.c`, and for the same reason: in an SDK project the component
 * system generates one of these per feature. This project links the vendor blobs directly, so the generator does not
 * run, and a feature enabled in the catalogue without its configuration object fails at link time with
 * `undefined symbol: sli_feature_bt_dynamic_gattdb_config`.
 *
 * The value is a design decision, not a hardware constant: this firmware serves **one** dynamic database, the one
 * `efr32mg24_bluetooth_sys::gatt_db::add_service` builds. The SDK would derive it from the components in the .slcp;
 * there is no .slcp here, so the number is stated once, here, next to what it sizes.
 */

#include <stdint.h>
#include "sli_bt_config_defs.h"
#include "sli_bt_dynamic_gattdb_config.h"

SLI_BT_DEFINE_FEATURE_CONFIG(bt, dynamic_gattdb, sli_bt_dynamic_gattdb_config_t) = {
  .max_dynamic_gattdbs = 1,
};
