//! The MVP — the on-chip matrix/vector processor — driven through a safe API.
//!
//! The MVP holds eight instructions of three 32-bit words, eight loop controllers and five array
//! controllers, and it reads and writes *system memory*: an operand is a slice you hand it, not a copy you
//! push. That is the whole point of it here — a matrix multiply costs the CPU the loop, not the arithmetic.
//!
//! ## What this driver supports, and what it does not
//!
//! One operation, done properly: `add_fp16` — element-wise addition of two `binary16` vectors into a third.
//! That is the operation whose encoding is proven on this board (the PAC example `pac_mvp_add` measured
//! `exp=396 got=396`), and it is the one the dosimeter's GMM could use as a building block.
//!
//! Everything else — MAC with accumulation, transposed operands, other base types — needs the instruction
//! encodings from `sl_mvp.h`, which are **not** in this repository for those cases. Guessing them would
//! produce a driver that compiles, runs, and returns wrong numbers, which is worse than a driver that says
//! what it does not do. When a case is added, it comes with a self-checking example in the same change.
//!
//! ## The traps this driver encodes, all of them measured
//!
//! - `EN` must be set and `DISABLING` must clear before any configuration write; writes are dropped silently
//!   while the block is coming up.
//! - Configuration writes read back as zero if the CMU gate is off, so `take()` turns the gate on first.
//! - `CMD.INIT` goes in its own access: writing INIT and START together left the sequencer IDLE with the
//!   output array untouched, which is what a run of zero instructions looks like.
//! - The loop passes `NOINCR`: the instruction's streams advance the arrays. Incrementing a dimension in the
//!   loop as well skipped every element twice.
//! - The opcode is `ADDC` (334), not the `ADDR` written in some vendor examples; and the register set is
//!   `X = R1, A = R2, Z = R3`.

use crate::pac;

/// Element type for the array controllers: `BINARY16`, value 2 in the register encoding.
const BASETYPE_BINARY16: u32 = 2;
/// The opcode the vendor driver uses for a vector add.
const ALUOP_ADDC: u32 = 334;

const DIM_BASETYPE_SHIFT: u32 = 12;
const DIM_STRIDE_SHIFT: u32 = 16;
const LOOP_NUMITERS_SHIFT: u32 = 0;
const CFG2_ALUOP_SHIFT: u32 = 20;
const CFG2_ENDPROG: u32 = 1 << 31;

/// Stream/register encoding for `load A`, `load B`, `store out` with `INCRDIM2`, from
/// `SLI_MVP_STREAM(s, r, a, i) = ((i << 7) | (a << 4) | 0x8 | r) << (s * 10)`.
const INSTR0CFG0: u32 = (1 << 0) | (2 << 16) | (3 << 28);
const INSTR0CFG1: u32 = 0x22B8_6A09;

/// What can go wrong, named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The block never left `DISABLING`, so no configuration write would land.
    StuckDisabling,
    /// The sequencer never reported `IDLE` after `START`.
    NeverIdle,
    /// The two inputs are not the same length, or the output does not match them.
    LengthMismatch,
    /// More elements than the loop controller can encode, or an empty vector.
    LengthOutOfRange,
}

/// The MVP, with the borrow of its register block.
pub struct Mvp {
    reg: pac::MvpNs,
}

/// Largest element count the MVP can encode: `SIZE` and `NUMITERS` are 10-bit fields (`0x3FF`).
pub const MVP_MAX_LEN: usize = 1024;

impl Mvp {
    /// Take the block, turn its clock gate on, enable it and wait for the enable to settle.
    ///
    /// `cmu` is the non-secure CMU the board hands out; the gate is on `CLKEN1.MVP`, not `CLKEN0`.
    pub fn take(cmu: &pac::CmuNs, mvp: pac::MvpNs) -> Result<Self, Error> {
        cmu.clken1().modify(|_, w| w.mvp().set_bit());
        mvp.en().write(|w| w.en().set_bit());
        let mut spin = 0u32;
        while mvp.en().read().disabling().bit_is_set() {
            spin += 1;
            if spin > 1_000_000 {
                return Err(Error::StuckDisabling);
            }
        }
        Ok(Self { reg: mvp })
    }

    /// Is the sequencer idle? True before the first run and after every completed one.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.reg.status().read().idle().bit_is_set()
    }

    /// Element-wise addition of two `binary16` vectors into `out`.
    ///
    /// All three slices must be the same length; `out` is where the result is written, so it must not alias
    /// the inputs. The three lengths are checked before anything is handed to the hardware, because a
    /// mismatch would have the MVP read past the end of a slice.
    pub fn add_fp16(&mut self, a: &[u16], b: &[u16], out: &mut [u16]) -> Result<(), Error> {
        let n = a.len();
        // `SIZE` and `NUMITERS` are **10-bit** fields (`_MVP_ARRAYDIM0CFG_SIZE_MASK 0x3FF`,
        // `_MVP_LOOPCFG_NUMITERS_MASK 0x3FF`), so 1024 is the largest length that fits without spilling into the
        // neighbouring fields of the register. The bound used to be 65_536, which is ten times more than the
        // hardware can encode: anything above 1024 wrote into the fields next door and the result was silence or
        // corruption rather than an error.
        if n == 0 || n > MVP_MAX_LEN {
            return Err(Error::LengthOutOfRange);
        }
        if b.len() != n || out.len() != n {
            return Err(Error::LengthMismatch);
        }

        let stride = (n as u32 - 1) | (1 << DIM_STRIDE_SHIFT);
        let base = [(a.as_ptr(), 0u8), (b.as_ptr(), 1), (out.as_mut_ptr(), 2)];
        for (ptr, which) in base {
            let addr = ptr as u32;
            // Each array controller is its own register type, so the three arms perform the writes
            // rather than returning the registers.
            match which {
                0 => {
                    self.reg.array0addrcfg().write(|w| unsafe { w.bits(addr) });
                    self.reg
                        .array0dim0cfg()
                        .write(|w| unsafe { w.bits(BASETYPE_BINARY16 << DIM_BASETYPE_SHIFT) });
                    self.reg.array0dim1cfg().write(|w| unsafe { w.bits(0) });
                    self.reg
                        .array0dim2cfg()
                        .write(|w| unsafe { w.bits(stride) });
                }
                1 => {
                    self.reg.array1addrcfg().write(|w| unsafe { w.bits(addr) });
                    self.reg
                        .array1dim0cfg()
                        .write(|w| unsafe { w.bits(BASETYPE_BINARY16 << DIM_BASETYPE_SHIFT) });
                    self.reg.array1dim1cfg().write(|w| unsafe { w.bits(0) });
                    self.reg
                        .array1dim2cfg()
                        .write(|w| unsafe { w.bits(stride) });
                }
                _ => {
                    self.reg.array2addrcfg().write(|w| unsafe { w.bits(addr) });
                    self.reg
                        .array2dim0cfg()
                        .write(|w| unsafe { w.bits(BASETYPE_BINARY16 << DIM_BASETYPE_SHIFT) });
                    self.reg.array2dim1cfg().write(|w| unsafe { w.bits(0) });
                    self.reg
                        .array2dim2cfg()
                        .write(|w| unsafe { w.bits(stride) });
                }
            }
        }

        // One loop of `n` iterations, NOINCR: the streams advance the arrays.
        self.reg
            .loop0cfg()
            .write(|w| unsafe { w.bits((n as u32 - 1) << LOOP_NUMITERS_SHIFT) });
        self.reg.loop0rst().write(|w| unsafe { w.bits(0) });

        self.reg
            .instr0cfg0()
            .write(|w| unsafe { w.bits(INSTR0CFG0) });
        self.reg
            .instr0cfg1()
            .write(|w| unsafe { w.bits(INSTR0CFG1) });
        self.reg.instr0cfg2().write(|w| unsafe {
            w.bits(
                (ALUOP_ADDC << CFG2_ALUOP_SHIFT)
                    | (1 << 0)   // loop 0 begins here
                    | (1 << 1)   // loop 0 ends here
                    | CFG2_ENDPROG,
            )
        });

        // INIT in its own access: INIT+START together leaves the sequencer idle and the output untouched.
        self.reg.cmd().write(|w| w.init().set_bit());
        self.reg.cmd().write(|w| w.start().set_bit());

        let mut spin = 0u32;
        while !self.reg.status().read().idle().bit_is_set() {
            spin += 1;
            if spin > 20_000_000 {
                return Err(Error::NeverIdle);
            }
        }
        Ok(())
    }
}
