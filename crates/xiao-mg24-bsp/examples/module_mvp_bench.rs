//! Does the MVP pay for itself? One number, measured both ways — through the BSP.
//!
//! The MVP is a matrix/vector processor that reads and writes system memory. Whether it is worth using for the
//! dosimeter's model is not a matter of opinion: it is the cost of the same work done two ways, measured on
//! this part, with the cycle counter.
//!
//! This example does exactly that, for element-wise addition of two `binary16` vectors of `N` elements:
//!
//! - **MVP**: one `Mvp::add_fp16` call, the whole loop in hardware.
//! - **CPU**: the same loop in Rust, over the same memory, reading and writing the same words.
//!
//! The record carries both cycle counts, the element count and a self-check (every result must be `2.0` in
//! binary16, because every input is `1.0`). A wrong result fails the record even if the timing looks good.
//!
//! ## What this measures, and what it does not
//!
//! It measures the *arithmetic loop*. It does not measure a matrix multiply, and it does not measure the cost
//! of getting a model's coefficients into the arrays — the arrays read memory, so that cost is the CPU's load
//! of the coefficients, which this example does not do. Adding 64 vectors is a smaller job than the dosimeter's
//! GMM; treat this as the floor for the MVP's advantage, not as the answer for the whole model.

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use xiao_mg24_bsp::Module;

use panic_halt as _;
use xiao_mg24_bsp::hal::dwt;

#[path = "common/mailbox.rs"]
mod mailbox;

/// 64 elements: about the size of one band's worth of coefficients in the dosimeter's GMM.
const N: usize = 64;

/// `1.0` in IEEE binary16. The self-check expects `1.0 + 1.0 = 2.0` (`0x4000`) everywhere.
const ONE_FP16: u16 = 0x3C00;
const TWO_FP16: u16 = 0x4000;

struct Text {
    buf: [u8; 160],
    len: usize,
}

impl Text {
    fn new() -> Self {
        Self {
            buf: [0; 160],
            len: 0,
        }
    }
    fn s(&mut self, text: &[u8]) {
        for b in text {
            if self.len < self.buf.len() {
                self.buf[self.len] = *b;
                self.len += 1;
            }
        }
    }
    fn u(&mut self, mut v: u32) {
        if v == 0 {
            self.s(b"0");
            return;
        }
        let mut d = [0u8; 10];
        let mut n = 0;
        while v > 0 {
            d[n] = b'0' + (v % 10) as u8;
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let b = d[n];
            self.s(&[b]);
        }
    }
    fn bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

#[entry]
fn main() -> ! {
    // SAFETY: the mailbox slot is this program's own record, in RAM, written by nothing else.
    unsafe { core::ptr::write_volatile(0x2003_F000 as *mut u32, 0) };

    let module = match Module::take() {
        Some(m) => m,
        None => loop {
            core::hint::spin_loop();
        },
    };

    let mut mvp = match module.mvp() {
        Ok(m) => m,
        Err(_) => {
            mailbox::fail(b"mvp: will not enable (stuck disabling)");
            loop {
                cortex_m::asm::wfi();
            }
        }
    };

    let mut a = [ONE_FP16; N];
    let mut b = [ONE_FP16; N];
    let mut out_mvp = [0u16; N];
    let mut out_cpu = [0u16; N];

    // SAFETY: nothing else in this program touches DWT; it is started once, here, and only read after.
    let t0 = unsafe { dwt::start() }
        .expect("dwt: the cycle counter does not run without a debug session");
    let mvp_result = mvp.add_fp16(&a, &b, &mut out_mvp);
    let mvp_cycles = dwt::since(t0);

    let t1 = dwt::cycles();
    // The software loop over the same memory: read a word from each input, write one to the output.
    // `black_box` on the inputs and on the result keeps the compiler from folding the loop away --
    // measured 2026-11-17: without it the loop "took 2 cycles", which is what an optimised-away loop costs.
    for i in 0..N {
        out_cpu[i] = core::hint::black_box(a[i]).wrapping_add(core::hint::black_box(b[i]));
    }
    let cpu_cycles = dwt::since(t1);
    let cpu_first = core::hint::black_box(out_cpu[0]);

    // Touch the inputs so the compiler cannot fold them away as constants.
    a[0] = a[0].wrapping_add(mvp_cycles as u16 & 1);
    b[0] = b[0].wrapping_add(cpu_cycles as u16 & 1);

    let mut ok = mvp_result.is_ok();
    for v in &out_mvp {
        if *v != TWO_FP16 {
            ok = false;
        }
    }

    let mut t = Text::new();
    t.s(b"mvp: n=");
    t.u(N as u32);
    t.s(b" cycles_mvp=");
    t.u(mvp_cycles);
    t.s(b" cycles_cpu=");
    t.u(cpu_cycles);
    t.s(b" ratio_x100=");
    if mvp_cycles > 0 {
        t.u((cpu_cycles.saturating_mul(100)) / mvp_cycles.max(1));
    } else {
        t.s(b"?");
    }
    t.s(b" mvp_first=");
    t.u(out_mvp[0] as u32);
    t.s(b" cpu_first=");
    t.u(cpu_first as u32);

    if ok {
        mailbox::pass(t.bytes());
    } else {
        mailbox::fail(t.bytes());
    }

    loop {
        cortex_m::asm::wfi();
    }
}
