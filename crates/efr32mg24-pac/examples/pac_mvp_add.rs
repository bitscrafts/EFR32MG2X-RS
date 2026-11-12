//! MVP: the on-chip matrix/vector processor adds two fp16 vectors, from the PAC.
//!
//! ## What the MVP is, from the reference manual
//!
//! EFR32xG24 chapter 31: a co-processor with its own sequencer that accelerates
//! floating point matrix work — one complex multiply-accumulate per cycle, four
//! single-precision multipliers and four adders, six times the Cortex-M33 FMAC
//! rate. It holds eight instructions (three 32-bit words each), eight loop
//! controllers and five array controllers, and it reads/writes system memory
//! through three AHB ports.
//!
//! The reference manual also says something worth quoting, because it decides how
//! this example has to be written:
//!
//! > Revision 1.1: "MVP: Removed Registers and Definitions from MVP section - use
//! > of the MVP shouldn't require access to these registers"
//!
//! In other words Silicon Labs does not document the register interface: their
//! supported path is the SDK driver, which builds a `struct` in RAM and memcpys it
//! over `MVP->ALU[0]`. The layout below therefore comes from the SDK's own
//! `sl_mvp.h` (Zlib licensed) — instruction words, array and loop configuration,
//! the ALU opcodes — and the opcode numbers are confirmed by this PAC's
//! `instr0cfg2.aluop()` enum, where `Addr = 74` and `Macr2a = 471`.
//!
//! ## What this example computes
//!
//! `out[k] = a[k] + b[k]` for eight fp16 elements:
//!
//! - three arrays: `a`, `b` in, `out` out;
//! - one loop of eight iterations, incrementing the inner dimension of all three;
//! - one instruction: load `a` into R0, load `b` into R1, `ADDR` (real add) into R2,
//!   store R2.
//!
//! The result is checked against the same sum computed by the CPU, so a wrong
//! program cannot pass: if the MVP did nothing, `out` would stay zero, and if it
//! read the wrong words the values would differ. The mailbox carries both the
//! expected and the observed sums.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efr32mg24_pac as pac;
use panic_halt as _;

#[path = "common/mailbox.rs"]
mod mailbox;

/// Number of elements in each vector.
const N: usize = 8;

/// Element type for the array controllers: `BINARY16` (fp16), value 2 in
/// `sli_mvp_datatype_t`.
const BASETYPE_BINARY16: u32 = 2;

/// `_MVP_INSTRCFG2_ALUOP_ADDR` in this PAC's enum: real addition of two operands.
const ALUOP_ADDR: u32 = 74;

/// Field shifts, from the SDK's instruction/array/loop structs.
const DIM_BASETYPE_SHIFT: u32 = 12;
const DIM_STRIDE_SHIFT: u32 = 16;
const LOOP_NUMITERS_SHIFT: u32 = 0;
const LOOP_INCRDIM_SHIFT: u32 = 12;
const CFG2_ALUOP_SHIFT: u32 = 20;
const CFG2_ENDPROG: u32 = 1 << 31;

/// Vectors must be word aligned: the AHB ports move 32-bit words.
#[repr(align(32))]
struct Aligned([u16; N]);

static mut A: Aligned = Aligned([0; N]);
static mut B: Aligned = Aligned([0; N]);
static mut OUT: Aligned = Aligned([0; N]);

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

/// Encode a small non-negative integer as IEEE binary16.
///
/// Exact for 0..=2047, which is all this example needs, and it keeps the test
/// self-checking: no lookup table of magic constants to get wrong.
fn fp16_from_int(value: i32) -> u16 {
    if value == 0 {
        return 0;
    }
    let negative = value < 0;
    let magnitude = value.unsigned_abs();
    let mut exponent = 0u32;
    while magnitude >= (2u32 << exponent) {
        exponent += 1;
    }
    // magnitude = 2^exponent + remainder
    let remainder = magnitude - (1u32 << exponent);
    let mantissa = (remainder << (10 - exponent)) & 0x3FF;
    let exp_field = (exponent + 15) as u16;
    ((negative as u16) << 15) | (exp_field << 10) | mantissa as u16
}

/// Decode binary16 back to an integer (values in this example are exact).
fn fp16_to_int(raw: u16) -> i32 {
    let negative = raw & 0x8000 != 0;
    let exp_field = ((raw >> 10) & 0x1F) as i32;
    let mantissa = (raw & 0x3FF) as i32;
    if exp_field == 0 && mantissa == 0 {
        return 0;
    }
    let exponent = exp_field - 15;
    let value = if exponent >= 0 {
        (1 << exponent) + (mantissa >> (10 - exponent))
    } else {
        0
    };
    if negative {
        -value
    } else {
        value
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    dp.cmu_s.clken0().modify(|_r, w| w.gpio().set_bit());
    // The MVP's bus clock lives in CLKEN1, bit 30 ("MVP - Enable Bus Clock").
    dp.cmu_s.clken1().modify(|_r, w| w.mvp().set_bit());

    dp.gpio_s.porta_model().modify(|_r, w| w.mode7().pushpull());
    let led = || {
        dp.gpio_s
            .porta_dout()
            .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 7)) });
    };

    // Fill the inputs: a = 1..8, b = 10,20,..,80. Small integers so fp16 is exact.
    for k in 0..N {
        unsafe {
            A.0[k] = fp16_from_int(k as i32 + 1);
            B.0[k] = fp16_from_int(((k as i32 + 1) * 10) as i32);
            OUT.0[k] = 0;
        }
    }

    // Two aliases exist: 0x4D00_0000 (secure) and 0x5D00_0000 (non-secure). Measured on this
    // board: the secure alias reads zero from a non-secure image, and reading FAULTSTATUS (0x24)
    // through the non-secure alias raises a bus error — so the block is reachable only at
    // 0x5D00_0000, and only the registers below are safe to touch.
    let mvp = &dp.mvp_ns;

    // A clock gate is not an enable. `EN` at 0x04 has bit 0 = Enable and bit 1 = Disabling busy,
    // and every configuration write below is dropped while the block is disabled. This write was
    // missing, and its absence is the reason the program never ran.
    mvp.en().write(|w| w.en().set_bit());
    let mut spin = 0u32;
    while mvp.en().read().disabling().bit_is_set() {
        spin += 1;
        if spin > 1_000_000 {
            mailbox::fail(b"mvp stuck disabling");
            loop {
                led();
                cortex_m::asm::delay(2_000_000);
            }
        }
    }

    // Prove the block keeps what is written to it before blaming the program: a configuration
    // register that reads back as zero means the writes are being dropped, and every later
    // symptom (a program that never runs, arrays that stay zero) would be a consequence of that.
    mvp.array0dim2cfg()
        .write(|w| unsafe { w.bits(0x0000_0007) });
    if mvp.array0dim2cfg().read().bits() != 0x0000_0007 {
        let mut msg = [0u8; 64];
        let mut pos = push_bytes(&mut msg, 0, b"mvp writes dropped en=");
        pos = push_dec(&mut msg, pos, mvp.en().read().bits() & 1);
        pos = push_bytes(&mut msg, pos, b" clken1=");
        pos = push_dec(
            &mut msg,
            pos,
            u32::from(dp.cmu_s.clken1().read().mvp().bit()),
        );
        mailbox::fail(&msg[..pos]);
        loop {
            led();
            cortex_m::asm::delay(2_000_000);
        }
    }

    // ---- array controllers: a, b, out -------------------------------------
    // dim0: element type. dim1: one row. dim2: N elements, stride 1.
    // `addr_of!` avoids creating a reference to a `static mut`.
    let bases: [u32; 3] = unsafe {
        [
            core::ptr::addr_of!(A.0) as u32,
            core::ptr::addr_of!(B.0) as u32,
            core::ptr::addr_of!(OUT.0) as u32,
        ]
    };
    for (i, base) in bases.iter().enumerate() {
        let addr = *base;
        match i {
            0 => {
                mvp.array0addrcfg().write(|w| unsafe { w.bits(addr) });
                mvp.array0dim0cfg()
                    .write(|w| unsafe { w.bits(BASETYPE_BINARY16 << DIM_BASETYPE_SHIFT) });
                mvp.array0dim1cfg().write(|w| unsafe { w.bits(0) });
                mvp.array0dim2cfg().write(|w| unsafe {
                    w.bits(((N as u32 - 1) | (1 << DIM_STRIDE_SHIFT)) & 0xFFFF_FFFF)
                });
            }
            1 => {
                mvp.array1addrcfg().write(|w| unsafe { w.bits(addr) });
                mvp.array1dim0cfg()
                    .write(|w| unsafe { w.bits(BASETYPE_BINARY16 << DIM_BASETYPE_SHIFT) });
                mvp.array1dim1cfg().write(|w| unsafe { w.bits(0) });
                mvp.array1dim2cfg()
                    .write(|w| unsafe { w.bits((N as u32 - 1) | (1 << DIM_STRIDE_SHIFT)) });
            }
            _ => {
                mvp.array2addrcfg().write(|w| unsafe { w.bits(addr) });
                mvp.array2dim0cfg()
                    .write(|w| unsafe { w.bits(BASETYPE_BINARY16 << DIM_BASETYPE_SHIFT) });
                mvp.array2dim1cfg().write(|w| unsafe { w.bits(0) });
                mvp.array2dim2cfg()
                    .write(|w| unsafe { w.bits((N as u32 - 1) | (1 << DIM_STRIDE_SHIFT)) });
            }
        }
    }

    // ---- one loop: N iterations, each advancing dim2 (the "width") --------
    // INCRDIM bit 2 is the width; the per-array nibble lives at 4*array.
    let incr_width = 0b0100u32;
    let incrdims = incr_width | (incr_width << 4) | (incr_width << 8);
    mvp.loop0cfg().write(|w| unsafe {
        w.bits(((N as u32 - 1) << LOOP_NUMITERS_SHIFT) | (incrdims << LOOP_INCRDIM_SHIFT))
    });
    mvp.loop0rst().write(|w| unsafe { w.bits(0) });

    // ---- the instruction -------------------------------------------------
    // CFG0: X = R0, Y = R1, A = R2, Z = R2 (no negate/zero masks).
    mvp.instr0cfg0()
        .write(|w| unsafe { w.bits((1 << 8) | (2 << 16) | (2 << 28)) });
    // CFG1: load a into R0, load b into R1, store R2 into out, all advancing width.
    let stream0 = 0u32 | (1 << 3) | (0 << 4) | (1 << 9);
    let stream1 = 1u32 | (1 << 3) | (1 << 4) | (1 << 9);
    let ostream = 2u32 | (1 << 3) | (2 << 4) | (1 << 9);
    mvp.instr0cfg1()
        .write(|w| unsafe { w.bits(stream0 | (stream1 << 10) | (ostream << 20)) });
    // CFG2: opcode, this instruction both opens and closes loop 0, and ends the program.
    mvp.instr0cfg2().write(|w| unsafe {
        w.bits(
            (ALUOP_ADDR << CFG2_ALUOP_SHIFT)
                | (1 << 0)   // loop0 begin
                | (1 << 1)   // loop0 end
                | CFG2_ENDPROG,
        )
    });

    // ---- run -------------------------------------------------------------
    // CMD bit 3 is INIT, described in the SVD as an "Initialization Command/Qualifier": it
    // qualifies the START written in the same access, it is not a separate pass.
    mvp.cmd().write(|w| w.init().set_bit().start().set_bit());
    let mut spin = 0u32;
    while !mvp.status().read().idle().bit_is_set() {
        spin += 1;
        if spin > 20_000_000 {
            // FAULTSTATUS (0x24) is not readable through this alias — it raises a bus error — so
            // the report carries the status bits and whether the instruction word survived, which
            // is enough to tell "never started" from "started and faulted".
            let st = mvp.status().read();
            let mut msg = [0u8; 64];
            let mut pos = push_bytes(&mut msg, 0, b"mvp stuck run=");
            pos = push_dec(&mut msg, pos, u32::from(st.running().bit()));
            pos = push_bytes(&mut msg, pos, b" idle=");
            pos = push_dec(&mut msg, pos, u32::from(st.idle().bit()));
            pos = push_bytes(&mut msg, pos, b" instr2=");
            pos = push_dec(&mut msg, pos, mvp.instr0cfg2().read().bits());
            mailbox::fail(&msg[..pos]);
            loop {
                led();
                cortex_m::asm::delay(2_000_000);
            }
        }
    }

    // ---- verify against the CPU ------------------------------------------
    let mut ok = true;
    let mut sum_expected = 0i32;
    let mut sum_observed = 0i32;
    for k in 0..N {
        let (a, b, out) = unsafe { (A.0[k], B.0[k], OUT.0[k]) };
        let expected = fp16_to_int(a) + fp16_to_int(b);
        let observed = fp16_to_int(out);
        sum_expected += expected;
        sum_observed += observed;
        if expected != observed {
            ok = false;
        }
    }

    let mut msg = [0u8; 64];
    let mut pos = push_bytes(&mut msg, 0, b"mvp sum exp=");
    pos = push_dec(&mut msg, pos, sum_expected as u32);
    pos = push_bytes(&mut msg, pos, b" got=");
    pos = push_dec(&mut msg, pos, sum_observed as u32);

    if ok {
        mailbox::pass(&msg[..pos]);
    } else {
        mailbox::fail(&msg[..pos]);
    }

    loop {
        led();
        cortex_m::asm::delay(2_000_000);
    }
}
