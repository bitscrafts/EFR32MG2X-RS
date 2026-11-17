/* Memory layout for Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) */

MEMORY
{
  /* Application flash: 0x08006000 - 0x08175FFF (1504 KB)
     The first 24 KB (0x08000000-0x08005FFF) is reserved for the Gecko
     bootloader and MUST NOT be overwritten by applications.
     The last 40 KB (0x08176000-0x0817FFFF) is the NVM3 instance: `build.rs` points NVM3_BASE at 0x08176000 with
     NVM3_DEFAULT_NVM_SIZE 40960, so it runs to 0x08180000. The previous length of 1512K ended at 0x08180000 and
     therefore *included* the NVM3 region: the comment claimed a reservation the numbers did not make. Harmless while
     the image stayed under 214 KB, and a silent bond wipe the moment it grew. */
  FLASH : ORIGIN = 0x08006000, LENGTH = 1472K

  /* RAM: 252 KB usable of 256 KB. The top 4 KiB page (0x2003F000..0x2003FFFF) holds the mailbox and the trace
 * buffers, and the stack grows down from 0x2003F000: claiming 256K here makes the first deep call frame overwrite
 * the record the instrument reads. Same reservation as the HAL. */
  /* Address: 0x2000_0000 - 0x2003_FFFF */
  RAM : ORIGIN = 0x20000000, LENGTH = 252K
}

/* Entry point (from cortex-m-rt) */
ENTRY(Reset);

/* Stack grows downward from end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* Heap configuration (optional, for allocator) */
/* Reserve 32 KB for stack, rest available for heap */
_heap_size = LENGTH(RAM) - 32K;

