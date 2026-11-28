/* Memory layout for EFR32MG24B220F1536IM48-B
 *
 * Chip: EFR32MG24B220F1536IM48-B (XIAO MG24 Sense)
 * Flash: 1536 KB total
 * RAM: 256 KB
 *
 * Flash Memory Map (NOTE: Flash is at 0x08000000 on EFR32MG24):
 * 0x0800_0000 - 0x0800_5FFF: Gecko Bootloader (24 KB) - Arduino uses 24 KB!
 * 0x0800_6000 - 0x0817_FFFF: Application flash (1512 KB)
 *
 * RAM Memory Map:
 * 0x2000_0000 - 0x2003_FFFF: SRAM (256 KB)
 *
 * CRITICAL: Arduino bootloader uses 24 KB (0x6000), NOT 16 KB!
 * This was discovered by analyzing Arduino hex files which start at 0x08006000.
 */

MEMORY
{
  /* Application flash - starts after 24 KB Gecko bootloader */
  /* Bootloader occupies 0x08000000 - 0x08005FFF (first 24 KB) */
  /* MUST match Arduino's bootloader layout for compatibility */
  /* 1472K, not 1512K: the top 40 KB (0x0817_6000 - 0x0819_FFFF) is left to NVM3, which is where the Bluetooth stack
   * keeps bonds, pairing keys and its own page bookkeeping. 1512K is "all the flash after the bootloader" and is the
   * wrong default for a product on this part: a program that grows into that window does not fail to link, it wipes
   * the pairing state of every phone that had bonded. A consumer that genuinely needs the whole flash passes its own
   * `-Tmemory.x` -- a visible decision, in one place.
   *
   * This file is the only memory layout in the workspace. Until 2026-11-27 the HAL, the embassy runtime and both BSPs
   * each shipped their own copy, and the linker took whichever came first on the search path, which was this one. */
  FLASH : ORIGIN = 0x08006000, LENGTH = 1472K

  /* SRAM - 256 KB total, but the last 4 KB page is reserved as the SWD mailbox
   * (0x2003_F000), so the linker gets 252 KB. */
  RAM : ORIGIN = 0x20000000, LENGTH = 252K
}

/* Required by cortex-m-rt.
 *
 * This used to be 0x20001208 — the stack pointer the Arduino core happens to
 * start with — which left the linker only 4.5 KB of RAM in total. Any example
 * with a buffer bigger than a few kilobytes failed to link with
 * "stack end address is not below stack start", which reads like a bug in the
 * example and is really an accident of that constant.
 *
 * The value below puts the stack top just under the mailbox page. The
 * bootloader does not care where the application's stack is: the vector table
 * carries SP and the core loads it on reset.
 */
_stack_start = 0x2003F000;
