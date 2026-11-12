/* Two questions Rust can ask emlib without knowing its enums.
 *
 * `CMU_ClockFreqGet` takes a `CMU_Clock_TypeDef`, whose values are built from shift/mask macros in
 * em_cmu.h. Re-deriving that value in Rust would be exactly the kind of copy that drifts silently the
 * day the vendor renumbers a branch, so the enum stays in C and Rust calls a wrapper.
 */
#include "em_cmu.h"
#include "em_device.h"

uint32_t emlib_sysclk_hz(void)
{
  return SystemSYSCLKGet();
}

uint32_t emlib_core_hz(void)
{
  return CMU_ClockFreqGet(cmuClock_CORE);
}

/* Identity, read from the part. */
#include "em_system.h"

uint64_t emlib_unique_id(void)
{
  return SYSTEM_GetUnique();
}

/* Packed so one 32-bit word can carry the whole revision: major, minor, part number or family.
 * The two variants of SYSTEM_ChipRevision_TypeDef are why this wrapper exists at all — the struct's
 * third field is 16 bits on parts that report a part number and 8 bits on parts that report a family,
 * and that difference must not leak into Rust. */
uint32_t emlib_chip_revision(void)
{
  SYSTEM_ChipRevision_TypeDef rev;
  SYSTEM_ChipRevisionGet(&rev);
#if defined(_SYSCFG_CHIPREV_PARTNUMBER_MASK)
  return ((uint32_t)rev.major << 24) | ((uint32_t)rev.minor << 16) | (uint32_t)rev.partNumber;
#else
  return ((uint32_t)rev.major << 24) | ((uint32_t)rev.minor << 16) | (uint32_t)rev.family;
#endif
}
