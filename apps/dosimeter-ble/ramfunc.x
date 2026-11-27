/* RAM-resident functions: the flash controller cannot write or erase while code runs from flash.
 * The emlib marks MSC_WriteWord and friends `__RAMFUNC`, which puts them in `.ram`. This fragment gives that section a
 * RAM address with its load image in flash, and `Module::take` copies it before anything touches NVM3.
 * It is a separate script from memory.x because cortex-m-rt's link.x includes memory.x *before* its own SECTIONS
 * block, which discarded a SECTIONS of our own (measured: `undefined symbol: __ramfunc_start`). */
SECTIONS
{
  .ramfunc : ALIGN(4)
  {
    __ramfunc_start = .;
    *(.ram)
    *(.RamFunc*)
    *(.ramfunc*)
    __ramfunc_end = .;
  } > RAM AT > FLASH
  __ramfunc_load = LOADADDR(.ramfunc);
}
