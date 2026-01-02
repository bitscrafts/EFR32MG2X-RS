/* Memory layout for Seeed Studio XIAO MG24 Sense (EFR32MG24B220F1536IM48-B) */

MEMORY
{
  /* Main Flash: 1536 KB (0x180000 bytes) */
  /* Address: 0x0800_0000 - 0x0817_FFFF */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1536K

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
