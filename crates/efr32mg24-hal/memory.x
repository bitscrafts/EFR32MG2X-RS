/* Memory layout for EFR32MG24B220F1536IM48-B (XIAO MG24 Sense) */
MEMORY
{
  /* Application flash: 0x08006000 - 0x0817FFFF (1512 KB)
     The first 24 KB (0x08000000-0x08005FFF) is reserved for the Gecko
     bootloader and MUST NOT be overwritten by applications. */
  FLASH : ORIGIN = 0x08006000, LENGTH = 1512K
  /* SRAM - starts at 0x20000000 */
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
