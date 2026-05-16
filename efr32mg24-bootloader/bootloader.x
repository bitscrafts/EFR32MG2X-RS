/* Memory layout for EFR32MG24 Minimal Bootloader */

MEMORY
{
  /* Bootloader in first 16 KB of flash */
  FLASH : ORIGIN = 0x08000000, LENGTH = 16K

  /* RAM: 256 KB total, bootloader uses minimal amount */
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

/* Stack size for bootloader (minimal - 2 KB) */
_stack_size = 2K;

/* Sections */
SECTIONS
{
  /* Vector table at start of flash */
  .vector_table ORIGIN(FLASH) :
  {
    /* Stack pointer */
    LONG(_stack_start);

    /* Reset handler */
    LONG(Reset | 1);  /* OR with 1 for Thumb mode */

    /* Exception handlers (minimal set) */
    LONG(DefaultHandler | 1);  /* NMI */
    LONG(HardFault | 1);       /* HardFault */
    LONG(DefaultHandler | 1);  /* MemManage */
    LONG(DefaultHandler | 1);  /* BusFault */
    LONG(DefaultHandler | 1);  /* UsageFault */
    LONG(DefaultHandler | 1);  /* SecureFault */
    LONG(0);                   /* Reserved */
    LONG(0);                   /* Reserved */
    LONG(0);                   /* Reserved */
    LONG(DefaultHandler | 1);  /* SVCall */
    LONG(DefaultHandler | 1);  /* DebugMon */
    LONG(0);                   /* Reserved */
    LONG(DefaultHandler | 1);  /* PendSV */
    LONG(DefaultHandler | 1);  /* SysTick */

    /* Pad to 512 bytes (128 IRQ slots for EFR32MG24) */
    . = ORIGIN(FLASH) + 0x200;
  } > FLASH

  /* Code section */
  .text :
  {
    *(.text .text.*);
    *(.rodata .rodata.*);
  } > FLASH

  /* Data section (initialized data) */
  .data : AT(LOADADDR(.text) + SIZEOF(.text))
  {
    _sdata = .;
    *(.data .data.*);
    _edata = .;
  } > RAM

  /* BSS section (zero-initialized data) */
  .bss :
  {
    _sbss = .;
    *(.bss .bss.*);
    *(COMMON);
    _ebss = .;
  } > RAM

  /* Stack (grows downward from end of RAM) */
  .stack (NOLOAD) :
  {
    . = . + _stack_size;
    _stack_start = .;
  } > RAM

  /* Discard debug info to save space */
  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*);
    *(.ARM.extab .ARM.extab.*);
  }
}

/* Application start address (immediately after bootloader) */
APP_START = ORIGIN(FLASH) + LENGTH(FLASH);  /* 0x08004000 */
