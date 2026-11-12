/**
 * @file cmsis_stubs.c
 * @brief Minimal CMSIS runtime stubs for Silicon Labs Bluetooth stack
 *
 * The Bluetooth stack binary blobs expect certain CMSIS runtime functions
 * to be provided by the application. This file provides minimal stub
 * implementations.
 *
 * IMPORTANT: These are minimal stubs for linking only. For production use,
 * proper CMSIS implementations should be used.
 */

#include <stdint.h>
#include <stdbool.h>

/**
 * @brief Check if currently executing in interrupt context
 *
 * The Bluetooth stack uses this to determine if it's safe to perform
 * certain operations (e.g., disabling interrupts).
 *
 * Implementation: Always returns false (not in IRQ).
 * This is safe for bare-metal applications where the Bluetooth stack
 * is only called from main thread context.
 *
 * @return false - Not in interrupt context
 */
bool CORE_InIrqContext(void)
{
    /* Minimal stub: Assume never in IRQ context
     *
     * For production:
     * - Read IPSR register to check if in handler mode
     * - Return (IPSR & 0x1FF) != 0
     */
    return false;
}

/**
 * @brief Enter critical section (disable interrupts)
 *
 * The Bluetooth stack uses this for thread-safe access to shared data.
 *
 * Implementation: Disable all configurable interrupts by setting BASEPRI.
 *
 * @return Previous BASEPRI value for restoration
 */
uint32_t CORE_EnterCritical(void)
{
    uint32_t basepri;

    /* Read current BASEPRI */
    __asm__ volatile ("mrs %0, basepri" : "=r" (basepri));

    /* Disable all interrupts with priority >= 1
     * Priority 0 (highest) is not maskable by BASEPRI
     */
    __asm__ volatile ("msr basepri, %0" :: "r" (1 << 5)); /* Priority 1 */
    __asm__ volatile ("isb");

    return basepri;
}

/**
 * @brief Exit critical section (restore interrupts)
 *
 * Restores interrupt state saved by CORE_EnterCritical().
 *
 * @param basepri Previous BASEPRI value from CORE_EnterCritical()
 */
void CORE_ExitCritical(uint32_t basepri)
{
    /* Restore previous BASEPRI */
    __asm__ volatile ("msr basepri, %0" :: "r" (basepri));
    __asm__ volatile ("isb");
}

/**
 * @brief Declare critical section variable
 *
 * This is typically a macro in CMSIS, but the stack expects it as a function.
 * We provide a no-op stub.
 */
void CORE_DECLARE_IRQ_STATE(void)
{
    /* No-op: This is handled by EnterCritical/ExitCritical */
}

/**
 * @brief Atomic compare and set operation
 *
 * Atomically compares *ptr with oldValue and if equal, sets it to newValue.
 *
 * @param ptr Pointer to value
 * @param oldValue Expected current value
 * @param newValue New value to set
 * @return true if swap occurred, false otherwise
 */
bool CORE_AtomicCompareAndSwap(uint32_t *ptr, uint32_t oldValue, uint32_t newValue)
{
    uint32_t result;

    /* Use ARM exclusive load/store instructions
     * LDREX: Load exclusive
     * STREX: Store exclusive (returns 0 if successful)
     */
    __asm__ volatile (
        "1: ldrex %0, [%1]\n"       /* Load exclusive */
        "   cmp %0, %2\n"            /* Compare with oldValue */
        "   bne 2f\n"                /* If not equal, exit */
        "   strex %0, %3, [%1]\n"   /* Store exclusive newValue */
        "   cmp %0, #0\n"            /* Check if store succeeded */
        "   bne 1b\n"                /* Retry if failed */
        "2: \n"
        : "=&r" (result)
        : "r" (ptr), "r" (oldValue), "r" (newValue)
        : "memory", "cc"
    );

    return (*ptr == newValue);
}

/**
 * @brief Yield processor (hint for power saving)
 *
 * The Bluetooth stack may call this in busy-wait loops.
 *
 * Implementation: ARM WFE (Wait For Event) instruction.
 */
void CORE_Yield(void)
{
    __asm__ volatile ("wfe");
}

/**
 * @brief Enter an atomic section (PRIMASK save + disable)
 *
 * The Bluetooth stack calls this around its buffer manipulation. It is *not* the same as
 * CORE_EnterAtomic in some CMSIS versions that only raise BASEPRI: masking with PRIMASK also
 * blocks priority 0, and the stack's assumption is that nothing preempts the buffer it is
 * editing. Getting this wrong does not fail to link — it corrupts the attribute buffer under
 * interrupt load, which is a bug that only appears once a connection is active.
 *
 * @return Previous PRIMASK value, to be passed to CORE_ExitAtomic()
 */
uint32_t CORE_EnterAtomic(void)
{
    uint32_t primask;

    __asm__ volatile ("mrs %0, primask" : "=r" (primask));
    __asm__ volatile ("cpsid i" ::: "memory");
    __asm__ volatile ("isb");

    return primask;
}

/**
 * @brief Exit an atomic section (restore PRIMASK)
 *
 * @param primask Previous PRIMASK value from CORE_EnterAtomic()
 */
void CORE_ExitAtomic(uint32_t primask)
{
    __asm__ volatile ("msr primask, %0" :: "r" (primask) : "memory");
    __asm__ volatile ("isb");
}
