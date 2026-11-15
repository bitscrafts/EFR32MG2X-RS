/* Memory layout for EFR32MG24B220F1536IM48-B (XIAO MG24 Sense) */
MEMORY
{
  /* Application flash: 0x08006000 - 0x0817FFFF (1512 KB)
     The first 24 KB (0x08000000-0x08005FFF) is reserved for the Gecko
     bootloader and MUST NOT be overwritten by applications. */
  FLASH : ORIGIN = 0x08006000, LENGTH = 1512K
  /* SRAM - starts at 0x20000000. 252K of the 256K: the top 4 KB page (0x2003F000..0x2003FFFF) is
     not the stack's. The SWD mailbox lives there, and measured frames reached 0x2003F5AC while the
     stack still started at 0x20040000, so any fixed address in that page could be overwritten by the
     stack at any moment (several readings in earlier rounds were exactly that). */
  RAM   : ORIGIN = 0x20000000, LENGTH = 252K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
