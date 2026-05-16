/* Direct flash - NO BOOTLOADER - for testing only! */
MEMORY
{
  /* Full flash starting at 0x08000000 */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1536K
  
  /* SRAM - 256 KB total */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K - 8
}

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
