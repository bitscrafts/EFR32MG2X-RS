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
