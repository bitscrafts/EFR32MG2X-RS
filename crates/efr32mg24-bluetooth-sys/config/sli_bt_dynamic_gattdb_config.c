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
 * This software is provided 'as-is', without any implied warranty. In no event will the authors be held liable for
 * any damages arising from the use of this software.
 *
 * Permission is granted to anyone to use this software for any purpose, including commercial applications, and to
 * alter it and redistribute it freely, subject to the following restrictions:
 *
 * 1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software.
 * 2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original.
 * 3. This notice may not be removed or altered from any source distribution.
 *
 ******************************************************************************/

/* Written by hand, like `sli_bt_connection_config.c`: in an SDK project the component system generates one of these
 * per feature, and this project links the vendor blobs directly so the generator does not run.
 *
 * **The type is the whole point of this file's second version.** It used to be
 * `sli_bt_dynamic_gattdb_config_t` (`{ uint8_t max_dynamic_gattdbs; }`), taken from the header of that name. The
 * blob disagrees with that header. `bt_dynamic_gattdb_init` is 30 bytes long and reads its argument like this:
 *
 *     0: movs r2, #0
 *     2: ldr  r0, [r0, #0]        ; the first word of the config
 *     6: and.w r1, r0, #1         ; r1 = flags & 1      -> GATT caching
 *     a: ubfx r0, r0, #1, #1      ; r0 = (flags >> 1) & 1 -> include static database
 *     e: strh r2, [r3, #0]        ; .bss.database_lock = 0
 *    10: strh r2, [r3, #2]
 *    12: b.w gatt_db_dynamic_init ; gatt_db_dynamic_init(include_static, caching)
 *
 * So the feature consumes a `sli_bt_gattdb_config_t` -- `{ uint32_t flags; }`, with
 * `SLI_BT_GATTDB_CONFIG_FLAG_ENABLE_GATT_CACHING` (0x01) and `..._INCLUDE_STATIC_DB` (0x02) -- and not the
 * max-count struct. Given the wrong shape, the flags were whatever the byte happened to be, and enabling the
 * component made `sli_bt_system_start_bluetooth` return `SL_STATUS_TIMEOUT` with `system_boot` never arriving.
 *
 * The value is a design decision, stated once: this firmware serves a database built at run time, so there is no
 * static database to include (flag 0x02 off) and nothing to cache yet (flag 0x01 off). Flags are 0.
 */

#include <stdint.h>
#include "sli_bt_config_defs.h"
#include "sli_bt_gattdb_def.h"

SLI_BT_DEFINE_FEATURE_CONFIG(bt, dynamic_gattdb, sli_bt_gattdb_config_t) = {
  .flags = 0,
};
