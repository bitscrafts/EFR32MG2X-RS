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
  FLASH : ORIGIN = 0x08006000, LENGTH = 1512K

  /* SRAM - 256 KB total */
  /* Match Arduino RAM configuration */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Required by cortex-m-rt */
/* Match Arduino's exact stack pointer location */
_stack_start = 0x20001208;
