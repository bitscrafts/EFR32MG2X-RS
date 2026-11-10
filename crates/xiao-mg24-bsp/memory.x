/* Memory layout for Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) */

MEMORY
{
  /* Application flash: 0x08006000 - 0x0817FFFF (1512 KB)
     The first 24 KB (0x08000000-0x08005FFF) is reserved for the Gecko
     bootloader and MUST NOT be overwritten by applications. */
  FLASH : ORIGIN = 0x08006000, LENGTH = 1512K

  /* RAM: 256 KB (0x40000 bytes) */
  /* Address: 0x2000_0000 - 0x2003_FFFF */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Entry point (from cortex-m-rt) */
ENTRY(Reset);

/* Stack grows downward from end of RAM */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);

/* Heap configuration (optional, for allocator) */
/* Reserve 32 KB for stack, rest available for heap */
_heap_size = LENGTH(RAM) - 32K;
