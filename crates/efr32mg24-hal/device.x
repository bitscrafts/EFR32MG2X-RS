/* Device-specific linker script for EFR32MG24 */
/* This file is included by link.x from cortex-m-rt */

/* Provides weak aliases for device-specific interrupt handlers */
/* The PAC crate (efr32mg24-pac) provides the actual interrupt vectors */

/* Gecko Bootloader Application Properties Support */
/* The bootloader requires vector table word 13 (offset 0x34) to point to ApplicationProperties_t */
/* Note: Word 13 must be patched post-build using patch_app_properties.sh script */
SECTIONS
{
    /* Place application properties in flash after code */
    .app_properties : ALIGN(4)
    {
        KEEP(*(.app_properties));
        KEEP(*(.app_properties.*));
    } > FLASH
}

/* All interrupt handlers default to DefaultHandler if not explicitly defined */
PROVIDE(SMU_SECURE_IRQHandler = DefaultHandler);
PROVIDE(SMU_S_PRIVILEGED_IRQHandler = DefaultHandler);
PROVIDE(SMU_NS_PRIVILEGED_IRQHandler = DefaultHandler);
PROVIDE(EMU_IRQHandler = DefaultHandler);
PROVIDE(TIMER0_IRQHandler = DefaultHandler);
PROVIDE(TIMER1_IRQHandler = DefaultHandler);
PROVIDE(TIMER2_IRQHandler = DefaultHandler);
PROVIDE(TIMER3_IRQHandler = DefaultHandler);
PROVIDE(TIMER4_IRQHandler = DefaultHandler);
PROVIDE(USART0_RX_IRQHandler = DefaultHandler);
PROVIDE(USART0_TX_IRQHandler = DefaultHandler);
PROVIDE(EUSART0_RX_IRQHandler = DefaultHandler);
PROVIDE(EUSART0_TX_IRQHandler = DefaultHandler);
PROVIDE(EUSART1_RX_IRQHandler = DefaultHandler);
PROVIDE(EUSART1_TX_IRQHandler = DefaultHandler);
PROVIDE(ICACHE0_IRQHandler = DefaultHandler);
PROVIDE(BURTC_IRQHandler = DefaultHandler);
PROVIDE(LETIMER0_IRQHandler = DefaultHandler);
PROVIDE(SYSCFG_IRQHandler = DefaultHandler);
PROVIDE(MPAHBRAM_IRQHandler = DefaultHandler);
PROVIDE(LDMA_IRQHandler = DefaultHandler);
PROVIDE(LFXO_IRQHandler = DefaultHandler);
PROVIDE(LFRCO_IRQHandler = DefaultHandler);
PROVIDE(ULFRCO_IRQHandler = DefaultHandler);
PROVIDE(GPIO_ODD_IRQHandler = DefaultHandler);
PROVIDE(GPIO_EVEN_IRQHandler = DefaultHandler);
PROVIDE(I2C0_IRQHandler = DefaultHandler);
PROVIDE(I2C1_IRQHandler = DefaultHandler);
PROVIDE(ACMP0_IRQHandler = DefaultHandler);
PROVIDE(ACMP1_IRQHandler = DefaultHandler);
PROVIDE(WDOG0_IRQHandler = DefaultHandler);
PROVIDE(WDOG1_IRQHandler = DefaultHandler);
PROVIDE(HFXO0_IRQHandler = DefaultHandler);
PROVIDE(HFRCO0_IRQHandler = DefaultHandler);
PROVIDE(HFRCOEM23_IRQHandler = DefaultHandler);
PROVIDE(CMU_IRQHandler = DefaultHandler);
PROVIDE(AES_IRQHandler = DefaultHandler);
PROVIDE(IADC_IRQHandler = DefaultHandler);
PROVIDE(MSC_IRQHandler = DefaultHandler);
PROVIDE(DPLL0_IRQHandler = DefaultHandler);
PROVIDE(EMUEFP_IRQHandler = DefaultHandler);
PROVIDE(DCDC_IRQHandler = DefaultHandler);
PROVIDE(PCNT0_IRQHandler = DefaultHandler);
PROVIDE(SW0_IRQHandler = DefaultHandler);
PROVIDE(SW1_IRQHandler = DefaultHandler);
PROVIDE(SW2_IRQHandler = DefaultHandler);
PROVIDE(SW3_IRQHandler = DefaultHandler);
PROVIDE(KERNEL0_IRQHandler = DefaultHandler);
PROVIDE(KERNEL1_IRQHandler = DefaultHandler);
PROVIDE(M33CTI0_IRQHandler = DefaultHandler);
PROVIDE(M33CTI1_IRQHandler = DefaultHandler);
PROVIDE(FPUEXH_IRQHandler = DefaultHandler);
PROVIDE(SEMBRX_IRQHandler = DefaultHandler);
PROVIDE(SEMBTX_IRQHandler = DefaultHandler);
PROVIDE(SYSRTC_APP_IRQHandler = DefaultHandler);
PROVIDE(KEYSCAN_IRQHandler = DefaultHandler);
PROVIDE(VDAC0_IRQHandler = DefaultHandler);
PROVIDE(VDAC1_IRQHandler = DefaultHandler);
PROVIDE(AHB2AHB0_IRQHandler = DefaultHandler);
PROVIDE(AHB2AHB1_IRQHandler = DefaultHandler);
PROVIDE(EMUDG_IRQHandler = DefaultHandler);
PROVIDE(AGC_IRQHandler = DefaultHandler);
PROVIDE(BUFC_IRQHandler = DefaultHandler);
PROVIDE(FRC_PRI_IRQHandler = DefaultHandler);
PROVIDE(FRC_IRQHandler = DefaultHandler);
PROVIDE(MODEM_IRQHandler = DefaultHandler);
PROVIDE(PROTIMER_IRQHandler = DefaultHandler);
PROVIDE(RAC_RSM_IRQHandler = DefaultHandler);
PROVIDE(RAC_SEQ_IRQHandler = DefaultHandler);
PROVIDE(HOSTMAILBOX_IRQHandler = DefaultHandler);
PROVIDE(SYNTH_IRQHandler = DefaultHandler);
PROVIDE(SYSRTC_SEQ_IRQHandler = DefaultHandler);
PROVIDE(RFECA0_IRQHandler = DefaultHandler);
PROVIDE(RFECA1_IRQHandler = DefaultHandler);
