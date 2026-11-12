//! PAC-level microphone → LED: the Arduino "VU meter" demo, at register level.
//!
//! ## What the official library revealed
//!
//! The mic on the XIAO MG24 **Sense** is not PDM: Seeed's `Seeed_Arduino_Mic`
//! implements the MG24 backend with the **IADC**. From
//! `src/hardware/mg24_adc.{h,cpp}`:
//!
//! - `MIC_ENPIN = PC8` — driven HIGH to power the microphone,
//! - `MIC_INPUTPIN = PC9` — the analog signal (`iadcPosInputPortCPin9`),
//! - `IADCCLK` from **FSRCO**, source clock 20 MHz, ADC clock 10 MHz,
//! - internal 1.2 V reference, 2x high-speed oversampling, 12-bit right aligned,
//! - one channel, continuous conversions, FIFO feeding a ping-pong DMA.
//!
//! ## Why this example is written the long way
//!
//! Three attempts failed before the vendor sequence was followed properly, and
//! each failure is a fact worth keeping:
//!
//! 1. **`CTRL.TIMEBASE = 0` produces no data at all.** The value is not optional:
//!    `timebase = ceil(fsrc / 1 MHz) - 1`, i.e. 19 for a 20 MHz source.
//! 2. **Configuration writes are dropped while the synchroniser is busy.**
//!    `STATUS.SYNCBUSY` and `EN.DISABLING` must be clear before `CTRL`, `SINGLE`
//!    and the FIFO configuration are written; without that wait `SINGLE` read
//!    back as 0 while `CTRL` had landed, which looks like a random failure.
//! 3. **`SINGLE` has its own write handshake**: `STATUS.SINGLEWRITEPENDING` must
//!    clear, exactly as `IADC_reset()` and `IADC_initSingle()` do it.
//!
//! The peripheral also survives a core reset, so a wedged state machine from an
//! earlier image has to be cleared by the same reset sequence the vendor uses —
//! stopping conversions first, then draining the FIFOs.
//!
//! ## Behaviour
//!
//! Polls single conversions, computes an RMS level over a window, and drives the
//! user LED with a software PWM whose duty follows the level — the Arduino demo's
//! behaviour. The mailbox reports RMS and peak, so a quiet room and a clap are
//! distinguishable without watching the LED.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

const PIN_MIC_EN: u32 = 1 << 8; // PC8
const PIN_LED: u32 = 1 << 7; // PA7, active low

/// Samples averaged per level update.
const WINDOW: usize = 128;
/// Software PWM steps per refresh.
const PWM_STEPS: u32 = 100;

fn push_bytes(buf: &mut [u8], mut pos: usize, s: &[u8]) -> usize {
    for b in s {
        if pos < buf.len() {
            buf[pos] = *b;
            pos += 1;
        }
    }
    pos
}

fn push_dec(buf: &mut [u8], mut pos: usize, mut value: u32) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0;
    if value == 0 {
        digits[0] = b'0';
        n = 1;
    }
    while value > 0 {
        digits[n] = b'0' + (value % 10) as u8;
        value /= 10;
        n += 1;
    }
    while n > 0 {
        n -= 1;
        if pos < buf.len() {
            buf[pos] = digits[n];
            pos += 1;
        }
    }
    pos
}

type Iadc = pac::iadc0_s::RegisterBlock;

/// Bounded wait for `STATUS.SYNCBUSY` and `EN.DISABLING` to clear.
fn wait_idle(adc: &Iadc) -> bool {
    let mut spin = 0u32;
    while adc.status().read().syncbusy().bit_is_set() || adc.en().read().disabling().bit_is_set() {
        spin += 1;
        if spin > 2_000_000 {
            return false;
        }
    }
    true
}

/// The vendor's `IADC_reset()`: stop everything, drain, then reset the registers
/// that only accept writes while disabled.
fn reset(adc: &Iadc) -> bool {
    adc.en().write(|w| w.en().set_bit());
    adc.cmd().write(|w| {
        w.singlestop().set_bit().scanstop().set_bit();
        w.timerdis().set_bit()
    });

    // Wait for conversions and queued requests to stop.
    let mut spin = 0u32;
    while adc.status().read().converting().bit_is_set()
        || adc.status().read().scanqueuepending().bit_is_set()
        || adc.status().read().singlequeuepending().bit_is_set()
        || adc.status().read().timeractive().bit_is_set()
    {
        spin += 1;
        if spin > 2_000_000 {
            return false;
        }
    }

    // Reset the two registers that need a handshake.
    adc.maskreq().write(|w| unsafe { w.bits(0) });
    adc.single().write(|w| unsafe { w.bits(0) });
    let mut spin = 0u32;
    while adc.status().read().maskreqwritepending().bit_is_set()
        || adc.status().read().singlewritepending().bit_is_set()
    {
        spin += 1;
        if spin > 2_000_000 {
            return false;
        }
    }

    // Drain both FIFOs, checking the counters as well as the flags (errata
    // IADC_E305: the STATUS bit can lie).
    let mut spin = 0u32;
    while adc.status().read().singlefifodv().bit_is_set() || adc.singlefifostat().read().bits() > 0
    {
        let _ = adc.singlefifodata().read().bits();
        spin += 1;
        if spin > 100_000 {
            break;
        }
    }

    adc.en().write(|w| w.en().clear_bit());
    wait_idle(adc)
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    dp.cmu_s
        .clken0()
        .modify(|_r, w| w.gpio().set_bit().iadc0().set_bit().hfrcoem23().set_bit());
    // HFRCOEM23, not FSRCO. Measured: with IADCCLK selected from FSRCO the
    // converter's synchroniser never finished disabling (EN.DISABLING stayed 1),
    // and the next configuration write faulted at BFAR=0x49004098 (SINGLE).
    // HFRCOEM23 is the oscillator already proven to clock TIMER0 on this board.
    dp.hfrcoem23_s.ctrl().modify(|_r, w| w.forceen().set_bit());
    let mut ready = 0u32;
    while !dp.hfrcoem23_s.status().read().rdy().bit_is_set() {
        ready += 1;
        if ready > 1_000_000 {
            break;
        }
    }
    dp.cmu_s
        .iadcclkctrl()
        .modify(|_r, w| w.clksel().hfrcoem23());

    // PC8 = microphone power, driven high. PC9 stays analog (disabled mode).
    dp.gpio_s
        .portc_modeh()
        .modify(|_r, w| w.mode0().pushpull().mode1().disabled());
    dp.gpio_s
        .portc_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_MIC_EN) });
    dp.gpio_s
        .cdbusalloc()
        .modify(|_r, w| unsafe { w.cdodd0().bits(1) });

    let adc = &dp.iadc0_s;

    let reset_ok = reset(adc);
    if !reset_ok || adc.en().read().disabling().bit_is_set() {
        // Writing configuration into a converter stuck in DISABLING is a bus
        // fault (measured: BFAR=0x49004098). Report and stop instead.
        mailbox::fail(b"adc stuck in DISABLING, not configured");
        loop {
            cortex_m::asm::nop();
        }
    }

    // CTRL: normal warm-up, TIMEBASE = ceil(fsrc/1MHz) - 1 = 19 at ~20 MHz,
    // source clock not divided.
    adc.ctrl().write(|w| {
        w.warmupmode().normal();
        w.adcclksuspend0().prswudis();
        w.adcclksuspend1().prswudis();
        unsafe {
            // timebase = ceil(fsrc/1MHz) - 1; HFRCOEM23 is a few MHz here, so a
            // larger value than strictly needed is safe (it only lengthens the
            // internal timing windows).
            w.timebase().bits(19);
            w.hsclkrate().bits(0);
        }
        w
    });
    // CFG0: normal mode, 2x high-speed OSR, 1x gain, internal 1.21 V reference.
    adc.cfg0().write(|w| {
        w.adcmode()
            .normal()
            .osrhs()
            .hispd2()
            .analoggain()
            .anagain1()
            .refsel()
            .vbgr()
    });
    // Scheduler prescale: 20 MHz source / 10 MHz ADC clock = divide by 2.
    adc.sched0().write(|w| unsafe { w.prescale().bits(1) });
    // Single FIFO: right-aligned 12-bit, valid at one entry.
    adc.singlefifocfg().write(|w| {
        w.alignment().right12();
        w.showid().clear_bit();
        w.dvl().valid1()
    });
    adc.en().write(|w| w.en().set_bit());
    if !wait_idle(adc) {
        mailbox::fail(b"adc never left sync after enable");
        loop {
            cortex_m::asm::nop();
        }
    }

    // SINGLE is a WSYNC register: the vendor's own comment says it "can only be
    // written while enabled". Writing it with EN=0 is what faulted at
    // BFAR=0x49004098 and left the converter stuck in DISABLING -- the single
    // most expensive mistake in this example. Order: static config while
    // disabled, channel config after enabling.
    adc.single().write(|w| {
        w.portpos().portc();
        unsafe {
            w.pinpos().bits(9);
            w.pinneg().bits(0);
        }
        w.cmp().clear_bit();
        w.portneg().gnd();
        w.cfg().config0()
    });
    let mut spin = 0u32;
    while adc.status().read().singlewritepending().bit_is_set() {
        spin += 1;
        if spin > 2_000_000 {
            break;
        }
    }
    // Let the microphone bias settle before measuring. Measured: reporting the
    // first window immediately gave RMS 98 while the same chain measured a few
    // hundred counts later; the difference is settling, not gain.
    cortex_m::asm::delay(300_000);

    // LED as a plain push-pull output, active low.
    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
    dp.gpio_s
        .porta_dout()
        .modify(|r, w| unsafe { w.bits(r.bits() | PIN_LED) });

    let mut peak: u32 = 0;
    let mut reported = false;
    let mut samples_seen: u32 = 0;

    loop {
        let mut sum_sq: u64 = 0;
        let mut window_peak: u32 = 0;
        let mut taken = 0usize;

        while taken < WINDOW {
            adc.cmd().write(|w| w.singlestart().set_bit());
            let mut spin = 0u32;
            while !adc.status().read().singlefifodv().bit_is_set() {
                spin += 1;
                if spin > 20_000 {
                    break;
                }
            }
            if adc.status().read().singlefifodv().bit_is_set() {
                let raw = adc.singledata().read().data().bits();
                let centred = if raw >= 2048 { raw - 2048 } else { 2048 - raw };
                sum_sq += (centred as u64) * (centred as u64);
                if centred > window_peak {
                    window_peak = centred;
                }
                taken += 1;
                samples_seen = samples_seen.wrapping_add(1);
            } else {
                // No sample: report instead of spinning forever. This is what
                // the first three attempts did wrong.
                break;
            }
        }

        let rms = if taken > 0 {
            isqrt((sum_sq / taken as u64) as u32)
        } else {
            0
        };
        if window_peak > peak {
            peak = window_peak;
        }

        // Brightness follows the level: a quiet room leaves the LED almost off,
        // a clap lights it fully.
        let duty = core::cmp::min(PWM_STEPS, (rms * PWM_STEPS) / 200);
        for step in 0..PWM_STEPS {
            if step < duty {
                dp.gpio_s
                    .porta_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() & !PIN_LED) });
            } else {
                dp.gpio_s
                    .porta_dout()
                    .modify(|r, w| unsafe { w.bits(r.bits() | PIN_LED) });
            }
            cortex_m::asm::delay(300);
        }

        if !reported {
            let mut msg = [0u8; 64];
            let mut pos = push_bytes(&mut msg, 0, b"mic");
            pos = push_bytes(
                &mut msg,
                pos,
                if reset_ok { b" rst=ok" } else { b" rst=BAD" },
            );
            pos = push_bytes(&mut msg, pos, b" rms=");
            pos = push_dec(&mut msg, pos, rms);
            pos = push_bytes(&mut msg, pos, b" pk=");
            pos = push_dec(&mut msg, pos, peak);
            pos = push_bytes(&mut msg, pos, b" n=");
            pos = push_dec(&mut msg, pos, samples_seen);
            // A working chain produces samples; silence is a level, not zero
            // samples. Zero samples means the converter never delivered.
            if samples_seen > 0 {
                mailbox::pass(&msg[..pos]);
            } else {
                mailbox::fail(&msg[..pos]);
            }
            reported = true;
        }
    }
}

/// Integer square root, so the example needs no libm.
fn isqrt(value: u32) -> u32 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x
}
