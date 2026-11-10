/* Memory layout for test application loaded by custom bootloader */

MEMORY
{
  /* Application starts at 0x08004000 (after 16 KB bootloader) */
  FLASH : ORIGIN = 0x08004000, LENGTH = 1520K

  /* RAM: 256 KB total */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Standard cortex-m-rt configuration */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
