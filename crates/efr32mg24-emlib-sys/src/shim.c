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

/* Bring the 39 MHz crystal up.
 *
 * The radio needs the HFXO: RAIL refuses a channel configuration whose crystal frequency disagrees with
 * the chip's, and the board had never started its crystal at all — it runs from the internal HFRCO at
 * 19 MHz, which is fine for a blink and useless for a 2.4 GHz synthesizer.
 *
 * `CMU_HFXOINIT_DEFAULT` is the vendor's default configuration for this part; using anything else would
 * be this project inventing oscillator settings.
 */
bool emlib_hfxo_start(void)
{
  CMU_HFXOInit_TypeDef hfxoInit = CMU_HFXOINIT_DEFAULT;
  CMU_HFXOInit(&hfxoInit);

  /* Start the oscillator by register.
   *
   * This is the part that took three attempts. `CMU_OscillatorEnable()` is a documented no-op in this
   * emlib version ("a dummy function to solve backward compatibility issues"), and `CMU_HFXOInit()`
   * only writes the crystal configuration. On Series 2 the HFXO is its own peripheral with a control
   * register, and `HFXO_CTRL_FORCEEN` ("Force Digital Clock Request") is what turns it on. Both the bit
   * and `HFXO_STATUS_RDY` come from the device header, not from a guess.
   */
  HFXO0->CTRL |= HFXO_CTRL_FORCEEN;

  /* Bounded wait: an oscillator that never settles must report false, not spin forever. */
  for (uint32_t i = 0U; i < 2000000U; i++) {
    if ((HFXO0->STATUS & HFXO_STATUS_RDY) != 0U) {
      return true;
    }
  }
  return false;
}

/* The frequency RAIL will compare the channel configuration against. */
uint32_t emlib_hfxo_freq(void)
{
  /* The crystal's own frequency, not the clock the core happens to run from. */
  return CMU_ClockFreqGet(cmuClock_HFXO);
}

/* Point the high-frequency clock at the crystal.
 *
 * Measured before this: `RAIL_GetRadioClockFreqHz()` answered 19000 kHz — the radio was being clocked
 * from the internal HFRCO — while the BLE channel configuration expects 39000000. That mismatch is the
 * whole of RAIL_ASSERT_INVALID_XTAL_FREQUENCY (66). Starting the crystal was necessary and not
 * sufficient: an oscillator that nothing selects does not clock anything.
 */
void emlib_clock_select_hfxo(void)
{
  CMU_ClockSelectSet(cmuClock_SYSCLK, cmuSelect_HFXO);
}
