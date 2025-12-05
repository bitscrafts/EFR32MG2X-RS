/* Memory layout for EFR32MG24B220F1536IM48-B (XIAO MG24 Sense) */
MEMORY
{
  /* Main Flash - starts at 0x08000000 on EFR32MG24 */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1536K
  /* SRAM - starts at 0x20000000 */
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
