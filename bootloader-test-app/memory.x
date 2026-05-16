/* Memory layout for application loaded by custom bootloader */
MEMORY
{
  /* Application starts after 16 KB bootloader at 0x08000000 */
  FLASH : ORIGIN = 0x08004000, LENGTH = 1520K
  RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}
