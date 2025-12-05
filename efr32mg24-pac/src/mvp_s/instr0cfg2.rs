#[doc = "Register `INSTR0CFG2` reader"]
pub type R = crate::R<Instr0cfg2Spec>;
#[doc = "Register `INSTR0CFG2` writer"]
pub type W = crate::W<Instr0cfg2Spec>;
#[doc = "Field `LOOP0BEGIN` reader - Loop Begin"]
pub type Loop0beginR = crate::BitReader;
#[doc = "Field `LOOP0BEGIN` writer - Loop Begin"]
pub type Loop0beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP0END` reader - Loop End"]
pub type Loop0endR = crate::BitReader;
#[doc = "Field `LOOP0END` writer - Loop End"]
pub type Loop0endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP1BEGIN` reader - Loop Begin"]
pub type Loop1beginR = crate::BitReader;
#[doc = "Field `LOOP1BEGIN` writer - Loop Begin"]
pub type Loop1beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP1END` reader - Loop End"]
pub type Loop1endR = crate::BitReader;
#[doc = "Field `LOOP1END` writer - Loop End"]
pub type Loop1endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP2BEGIN` reader - Loop Begin"]
pub type Loop2beginR = crate::BitReader;
#[doc = "Field `LOOP2BEGIN` writer - Loop Begin"]
pub type Loop2beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP2END` reader - Loop End"]
pub type Loop2endR = crate::BitReader;
#[doc = "Field `LOOP2END` writer - Loop End"]
pub type Loop2endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP3BEGIN` reader - Loop Begin"]
pub type Loop3beginR = crate::BitReader;
#[doc = "Field `LOOP3BEGIN` writer - Loop Begin"]
pub type Loop3beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP3END` reader - Loop End"]
pub type Loop3endR = crate::BitReader;
#[doc = "Field `LOOP3END` writer - Loop End"]
pub type Loop3endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP4BEGIN` reader - Loop Begin"]
pub type Loop4beginR = crate::BitReader;
#[doc = "Field `LOOP4BEGIN` writer - Loop Begin"]
pub type Loop4beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP4END` reader - Loop End"]
pub type Loop4endR = crate::BitReader;
#[doc = "Field `LOOP4END` writer - Loop End"]
pub type Loop4endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP5BEGIN` reader - Loop Begin"]
pub type Loop5beginR = crate::BitReader;
#[doc = "Field `LOOP5BEGIN` writer - Loop Begin"]
pub type Loop5beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP5END` reader - Loop End"]
pub type Loop5endR = crate::BitReader;
#[doc = "Field `LOOP5END` writer - Loop End"]
pub type Loop5endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP6BEGIN` reader - Loop Begin"]
pub type Loop6beginR = crate::BitReader;
#[doc = "Field `LOOP6BEGIN` writer - Loop Begin"]
pub type Loop6beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP6END` reader - Loop End"]
pub type Loop6endR = crate::BitReader;
#[doc = "Field `LOOP6END` writer - Loop End"]
pub type Loop6endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP7BEGIN` reader - Loop Begin"]
pub type Loop7beginR = crate::BitReader;
#[doc = "Field `LOOP7BEGIN` writer - Loop Begin"]
pub type Loop7beginW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP7END` reader - Loop End"]
pub type Loop7endR = crate::BitReader;
#[doc = "Field `LOOP7END` writer - Loop End"]
pub type Loop7endW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "ALU opcode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Aluop {
    #[doc = "0: No Operation"]
    Noop = 0,
    #[doc = "1: Clear register (set to +0)"]
    Clear = 1,
    #[doc = "65: Copy operation"]
    Copy = 65,
    #[doc = "66: Swap operation"]
    Swap = 66,
    #[doc = "67: Double operation (multiply by 2)"]
    Dbl = 67,
    #[doc = "68: Load real and imag (form A)"]
    Fana = 68,
    #[doc = "69: Load real and imag (form B)"]
    Fanb = 69,
    #[doc = "70: ReLU of real (max of real and +0)"]
    Relu2 = 70,
    #[doc = "71: Min of real and -0"]
    Nrelu2 = 71,
    #[doc = "72: Increment by 1.0"]
    Inc2 = 72,
    #[doc = "73: Decrement by 1.0"]
    Dec2 = 73,
    #[doc = "74: Addition of 2 reals"]
    Addr = 74,
    #[doc = "75: Maximum of 2 reals"]
    Max = 75,
    #[doc = "76: Minimum of 2 reals"]
    Min = 76,
    #[doc = "292: Square of real (form B)"]
    Rsqr2b = 292,
    #[doc = "334: Add Complex"]
    Addc = 334,
    #[doc = "339: Max of reals (form A)"]
    Max2a = 339,
    #[doc = "340: Min of reals (form A)"]
    Min2a = 340,
    #[doc = "350: Extract real from complex"]
    Xrealc2 = 350,
    #[doc = "351: Extract imag from complex"]
    Ximagc2 = 351,
    #[doc = "353: Add reals (form B)"]
    Addr2b = 353,
    #[doc = "354: Max of reals (form B)"]
    Max2b = 354,
    #[doc = "355: Min of reals (form B)"]
    Min2b = 355,
    #[doc = "397: Multiply Complex"]
    Mulc = 397,
    #[doc = "407: Multiply reals (form A)"]
    Mulr2a = 407,
    #[doc = "408: Multiply reals (form B)"]
    Mulr2b = 408,
    #[doc = "410: Add 4 reals"]
    Addr4 = 410,
    #[doc = "411: Max of 4 reals"]
    Max4 = 411,
    #[doc = "412: Min of 4 reals"]
    Min4 = 412,
    #[doc = "413: Squared magnitude Complex"]
    Sqrmagc2 = 413,
    #[doc = "416: Parametric ReLU (form B)"]
    Prelu2b = 416,
    #[doc = "461: Multiply Accumulate Complex"]
    Macc = 461,
    #[doc = "462: Add Accumulate Complex"]
    Aacc = 462,
    #[doc = "463: part of ELU activation (form A)"]
    Elu2a = 463,
    #[doc = "464: part of ELU activation (form B)"]
    Elu2b = 464,
    #[doc = "465: If A then X else Y (form A)"]
    Ifr2a = 465,
    #[doc = "466: If A then X else Y (form B)"]
    Ifr2b = 466,
    #[doc = "467: Max of reals and accumulator"]
    Maxac2 = 467,
    #[doc = "468: Min of reals and accumulators"]
    Minac2 = 468,
    #[doc = "469: Clipping activation (form A)"]
    Clip2a = 469,
    #[doc = "470: Clipping activation (form B)"]
    Clip2b = 470,
    #[doc = "471: Multiply accumulate reals (form A)"]
    Macr2a = 471,
    #[doc = "472: Multiply accumulate reals (form B)"]
    Macr2b = 472,
    #[doc = "473: If A then X else Y (complex)"]
    Ifc = 473,
}
impl From<Aluop> for u16 {
    #[inline(always)]
    fn from(variant: Aluop) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Aluop {
    type Ux = u16;
}
impl crate::IsEnum for Aluop {}
#[doc = "Field `ALUOP` reader - ALU opcode"]
pub type AluopR = crate::FieldReader<Aluop>;
impl AluopR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Aluop> {
        match self.bits {
            0 => Some(Aluop::Noop),
            1 => Some(Aluop::Clear),
            65 => Some(Aluop::Copy),
            66 => Some(Aluop::Swap),
            67 => Some(Aluop::Dbl),
            68 => Some(Aluop::Fana),
            69 => Some(Aluop::Fanb),
            70 => Some(Aluop::Relu2),
            71 => Some(Aluop::Nrelu2),
            72 => Some(Aluop::Inc2),
            73 => Some(Aluop::Dec2),
            74 => Some(Aluop::Addr),
            75 => Some(Aluop::Max),
            76 => Some(Aluop::Min),
            292 => Some(Aluop::Rsqr2b),
            334 => Some(Aluop::Addc),
            339 => Some(Aluop::Max2a),
            340 => Some(Aluop::Min2a),
            350 => Some(Aluop::Xrealc2),
            351 => Some(Aluop::Ximagc2),
            353 => Some(Aluop::Addr2b),
            354 => Some(Aluop::Max2b),
            355 => Some(Aluop::Min2b),
            397 => Some(Aluop::Mulc),
            407 => Some(Aluop::Mulr2a),
            408 => Some(Aluop::Mulr2b),
            410 => Some(Aluop::Addr4),
            411 => Some(Aluop::Max4),
            412 => Some(Aluop::Min4),
            413 => Some(Aluop::Sqrmagc2),
            416 => Some(Aluop::Prelu2b),
            461 => Some(Aluop::Macc),
            462 => Some(Aluop::Aacc),
            463 => Some(Aluop::Elu2a),
            464 => Some(Aluop::Elu2b),
            465 => Some(Aluop::Ifr2a),
            466 => Some(Aluop::Ifr2b),
            467 => Some(Aluop::Maxac2),
            468 => Some(Aluop::Minac2),
            469 => Some(Aluop::Clip2a),
            470 => Some(Aluop::Clip2b),
            471 => Some(Aluop::Macr2a),
            472 => Some(Aluop::Macr2b),
            473 => Some(Aluop::Ifc),
            _ => None,
        }
    }
    #[doc = "No Operation"]
    #[inline(always)]
    pub fn is_noop(&self) -> bool {
        *self == Aluop::Noop
    }
    #[doc = "Clear register (set to +0)"]
    #[inline(always)]
    pub fn is_clear(&self) -> bool {
        *self == Aluop::Clear
    }
    #[doc = "Copy operation"]
    #[inline(always)]
    pub fn is_copy(&self) -> bool {
        *self == Aluop::Copy
    }
    #[doc = "Swap operation"]
    #[inline(always)]
    pub fn is_swap(&self) -> bool {
        *self == Aluop::Swap
    }
    #[doc = "Double operation (multiply by 2)"]
    #[inline(always)]
    pub fn is_dbl(&self) -> bool {
        *self == Aluop::Dbl
    }
    #[doc = "Load real and imag (form A)"]
    #[inline(always)]
    pub fn is_fana(&self) -> bool {
        *self == Aluop::Fana
    }
    #[doc = "Load real and imag (form B)"]
    #[inline(always)]
    pub fn is_fanb(&self) -> bool {
        *self == Aluop::Fanb
    }
    #[doc = "ReLU of real (max of real and +0)"]
    #[inline(always)]
    pub fn is_relu2(&self) -> bool {
        *self == Aluop::Relu2
    }
    #[doc = "Min of real and -0"]
    #[inline(always)]
    pub fn is_nrelu2(&self) -> bool {
        *self == Aluop::Nrelu2
    }
    #[doc = "Increment by 1.0"]
    #[inline(always)]
    pub fn is_inc2(&self) -> bool {
        *self == Aluop::Inc2
    }
    #[doc = "Decrement by 1.0"]
    #[inline(always)]
    pub fn is_dec2(&self) -> bool {
        *self == Aluop::Dec2
    }
    #[doc = "Addition of 2 reals"]
    #[inline(always)]
    pub fn is_addr(&self) -> bool {
        *self == Aluop::Addr
    }
    #[doc = "Maximum of 2 reals"]
    #[inline(always)]
    pub fn is_max(&self) -> bool {
        *self == Aluop::Max
    }
    #[doc = "Minimum of 2 reals"]
    #[inline(always)]
    pub fn is_min(&self) -> bool {
        *self == Aluop::Min
    }
    #[doc = "Square of real (form B)"]
    #[inline(always)]
    pub fn is_rsqr2b(&self) -> bool {
        *self == Aluop::Rsqr2b
    }
    #[doc = "Add Complex"]
    #[inline(always)]
    pub fn is_addc(&self) -> bool {
        *self == Aluop::Addc
    }
    #[doc = "Max of reals (form A)"]
    #[inline(always)]
    pub fn is_max2a(&self) -> bool {
        *self == Aluop::Max2a
    }
    #[doc = "Min of reals (form A)"]
    #[inline(always)]
    pub fn is_min2a(&self) -> bool {
        *self == Aluop::Min2a
    }
    #[doc = "Extract real from complex"]
    #[inline(always)]
    pub fn is_xrealc2(&self) -> bool {
        *self == Aluop::Xrealc2
    }
    #[doc = "Extract imag from complex"]
    #[inline(always)]
    pub fn is_ximagc2(&self) -> bool {
        *self == Aluop::Ximagc2
    }
    #[doc = "Add reals (form B)"]
    #[inline(always)]
    pub fn is_addr2b(&self) -> bool {
        *self == Aluop::Addr2b
    }
    #[doc = "Max of reals (form B)"]
    #[inline(always)]
    pub fn is_max2b(&self) -> bool {
        *self == Aluop::Max2b
    }
    #[doc = "Min of reals (form B)"]
    #[inline(always)]
    pub fn is_min2b(&self) -> bool {
        *self == Aluop::Min2b
    }
    #[doc = "Multiply Complex"]
    #[inline(always)]
    pub fn is_mulc(&self) -> bool {
        *self == Aluop::Mulc
    }
    #[doc = "Multiply reals (form A)"]
    #[inline(always)]
    pub fn is_mulr2a(&self) -> bool {
        *self == Aluop::Mulr2a
    }
    #[doc = "Multiply reals (form B)"]
    #[inline(always)]
    pub fn is_mulr2b(&self) -> bool {
        *self == Aluop::Mulr2b
    }
    #[doc = "Add 4 reals"]
    #[inline(always)]
    pub fn is_addr4(&self) -> bool {
        *self == Aluop::Addr4
    }
    #[doc = "Max of 4 reals"]
    #[inline(always)]
    pub fn is_max4(&self) -> bool {
        *self == Aluop::Max4
    }
    #[doc = "Min of 4 reals"]
    #[inline(always)]
    pub fn is_min4(&self) -> bool {
        *self == Aluop::Min4
    }
    #[doc = "Squared magnitude Complex"]
    #[inline(always)]
    pub fn is_sqrmagc2(&self) -> bool {
        *self == Aluop::Sqrmagc2
    }
    #[doc = "Parametric ReLU (form B)"]
    #[inline(always)]
    pub fn is_prelu2b(&self) -> bool {
        *self == Aluop::Prelu2b
    }
    #[doc = "Multiply Accumulate Complex"]
    #[inline(always)]
    pub fn is_macc(&self) -> bool {
        *self == Aluop::Macc
    }
    #[doc = "Add Accumulate Complex"]
    #[inline(always)]
    pub fn is_aacc(&self) -> bool {
        *self == Aluop::Aacc
    }
    #[doc = "part of ELU activation (form A)"]
    #[inline(always)]
    pub fn is_elu2a(&self) -> bool {
        *self == Aluop::Elu2a
    }
    #[doc = "part of ELU activation (form B)"]
    #[inline(always)]
    pub fn is_elu2b(&self) -> bool {
        *self == Aluop::Elu2b
    }
    #[doc = "If A then X else Y (form A)"]
    #[inline(always)]
    pub fn is_ifr2a(&self) -> bool {
        *self == Aluop::Ifr2a
    }
    #[doc = "If A then X else Y (form B)"]
    #[inline(always)]
    pub fn is_ifr2b(&self) -> bool {
        *self == Aluop::Ifr2b
    }
    #[doc = "Max of reals and accumulator"]
    #[inline(always)]
    pub fn is_maxac2(&self) -> bool {
        *self == Aluop::Maxac2
    }
    #[doc = "Min of reals and accumulators"]
    #[inline(always)]
    pub fn is_minac2(&self) -> bool {
        *self == Aluop::Minac2
    }
    #[doc = "Clipping activation (form A)"]
    #[inline(always)]
    pub fn is_clip2a(&self) -> bool {
        *self == Aluop::Clip2a
    }
    #[doc = "Clipping activation (form B)"]
    #[inline(always)]
    pub fn is_clip2b(&self) -> bool {
        *self == Aluop::Clip2b
    }
    #[doc = "Multiply accumulate reals (form A)"]
    #[inline(always)]
    pub fn is_macr2a(&self) -> bool {
        *self == Aluop::Macr2a
    }
    #[doc = "Multiply accumulate reals (form B)"]
    #[inline(always)]
    pub fn is_macr2b(&self) -> bool {
        *self == Aluop::Macr2b
    }
    #[doc = "If A then X else Y (complex)"]
    #[inline(always)]
    pub fn is_ifc(&self) -> bool {
        *self == Aluop::Ifc
    }
}
#[doc = "Field `ALUOP` writer - ALU opcode"]
pub type AluopW<'a, REG> = crate::FieldWriter<'a, REG, 9, Aluop>;
impl<'a, REG> AluopW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u16>,
{
    #[doc = "No Operation"]
    #[inline(always)]
    pub fn noop(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Noop)
    }
    #[doc = "Clear register (set to +0)"]
    #[inline(always)]
    pub fn clear(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Clear)
    }
    #[doc = "Copy operation"]
    #[inline(always)]
    pub fn copy(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Copy)
    }
    #[doc = "Swap operation"]
    #[inline(always)]
    pub fn swap(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Swap)
    }
    #[doc = "Double operation (multiply by 2)"]
    #[inline(always)]
    pub fn dbl(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Dbl)
    }
    #[doc = "Load real and imag (form A)"]
    #[inline(always)]
    pub fn fana(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Fana)
    }
    #[doc = "Load real and imag (form B)"]
    #[inline(always)]
    pub fn fanb(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Fanb)
    }
    #[doc = "ReLU of real (max of real and +0)"]
    #[inline(always)]
    pub fn relu2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Relu2)
    }
    #[doc = "Min of real and -0"]
    #[inline(always)]
    pub fn nrelu2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Nrelu2)
    }
    #[doc = "Increment by 1.0"]
    #[inline(always)]
    pub fn inc2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Inc2)
    }
    #[doc = "Decrement by 1.0"]
    #[inline(always)]
    pub fn dec2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Dec2)
    }
    #[doc = "Addition of 2 reals"]
    #[inline(always)]
    pub fn addr(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Addr)
    }
    #[doc = "Maximum of 2 reals"]
    #[inline(always)]
    pub fn max(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Max)
    }
    #[doc = "Minimum of 2 reals"]
    #[inline(always)]
    pub fn min(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Min)
    }
    #[doc = "Square of real (form B)"]
    #[inline(always)]
    pub fn rsqr2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Rsqr2b)
    }
    #[doc = "Add Complex"]
    #[inline(always)]
    pub fn addc(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Addc)
    }
    #[doc = "Max of reals (form A)"]
    #[inline(always)]
    pub fn max2a(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Max2a)
    }
    #[doc = "Min of reals (form A)"]
    #[inline(always)]
    pub fn min2a(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Min2a)
    }
    #[doc = "Extract real from complex"]
    #[inline(always)]
    pub fn xrealc2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Xrealc2)
    }
    #[doc = "Extract imag from complex"]
    #[inline(always)]
    pub fn ximagc2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Ximagc2)
    }
    #[doc = "Add reals (form B)"]
    #[inline(always)]
    pub fn addr2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Addr2b)
    }
    #[doc = "Max of reals (form B)"]
    #[inline(always)]
    pub fn max2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Max2b)
    }
    #[doc = "Min of reals (form B)"]
    #[inline(always)]
    pub fn min2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Min2b)
    }
    #[doc = "Multiply Complex"]
    #[inline(always)]
    pub fn mulc(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Mulc)
    }
    #[doc = "Multiply reals (form A)"]
    #[inline(always)]
    pub fn mulr2a(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Mulr2a)
    }
    #[doc = "Multiply reals (form B)"]
    #[inline(always)]
    pub fn mulr2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Mulr2b)
    }
    #[doc = "Add 4 reals"]
    #[inline(always)]
    pub fn addr4(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Addr4)
    }
    #[doc = "Max of 4 reals"]
    #[inline(always)]
    pub fn max4(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Max4)
    }
    #[doc = "Min of 4 reals"]
    #[inline(always)]
    pub fn min4(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Min4)
    }
    #[doc = "Squared magnitude Complex"]
    #[inline(always)]
    pub fn sqrmagc2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Sqrmagc2)
    }
    #[doc = "Parametric ReLU (form B)"]
    #[inline(always)]
    pub fn prelu2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Prelu2b)
    }
    #[doc = "Multiply Accumulate Complex"]
    #[inline(always)]
    pub fn macc(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Macc)
    }
    #[doc = "Add Accumulate Complex"]
    #[inline(always)]
    pub fn aacc(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Aacc)
    }
    #[doc = "part of ELU activation (form A)"]
    #[inline(always)]
    pub fn elu2a(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Elu2a)
    }
    #[doc = "part of ELU activation (form B)"]
    #[inline(always)]
    pub fn elu2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Elu2b)
    }
    #[doc = "If A then X else Y (form A)"]
    #[inline(always)]
    pub fn ifr2a(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Ifr2a)
    }
    #[doc = "If A then X else Y (form B)"]
    #[inline(always)]
    pub fn ifr2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Ifr2b)
    }
    #[doc = "Max of reals and accumulator"]
    #[inline(always)]
    pub fn maxac2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Maxac2)
    }
    #[doc = "Min of reals and accumulators"]
    #[inline(always)]
    pub fn minac2(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Minac2)
    }
    #[doc = "Clipping activation (form A)"]
    #[inline(always)]
    pub fn clip2a(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Clip2a)
    }
    #[doc = "Clipping activation (form B)"]
    #[inline(always)]
    pub fn clip2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Clip2b)
    }
    #[doc = "Multiply accumulate reals (form A)"]
    #[inline(always)]
    pub fn macr2a(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Macr2a)
    }
    #[doc = "Multiply accumulate reals (form B)"]
    #[inline(always)]
    pub fn macr2b(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Macr2b)
    }
    #[doc = "If A then X else Y (complex)"]
    #[inline(always)]
    pub fn ifc(self) -> &'a mut crate::W<REG> {
        self.variant(Aluop::Ifc)
    }
}
#[doc = "Field `ENDPROG` reader - End of Program"]
pub type EndprogR = crate::BitReader;
#[doc = "Field `ENDPROG` writer - End of Program"]
pub type EndprogW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Loop Begin"]
    #[inline(always)]
    pub fn loop0begin(&self) -> Loop0beginR {
        Loop0beginR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Loop End"]
    #[inline(always)]
    pub fn loop0end(&self) -> Loop0endR {
        Loop0endR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Loop Begin"]
    #[inline(always)]
    pub fn loop1begin(&self) -> Loop1beginR {
        Loop1beginR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Loop End"]
    #[inline(always)]
    pub fn loop1end(&self) -> Loop1endR {
        Loop1endR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Loop Begin"]
    #[inline(always)]
    pub fn loop2begin(&self) -> Loop2beginR {
        Loop2beginR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Loop End"]
    #[inline(always)]
    pub fn loop2end(&self) -> Loop2endR {
        Loop2endR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Loop Begin"]
    #[inline(always)]
    pub fn loop3begin(&self) -> Loop3beginR {
        Loop3beginR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Loop End"]
    #[inline(always)]
    pub fn loop3end(&self) -> Loop3endR {
        Loop3endR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Loop Begin"]
    #[inline(always)]
    pub fn loop4begin(&self) -> Loop4beginR {
        Loop4beginR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Loop End"]
    #[inline(always)]
    pub fn loop4end(&self) -> Loop4endR {
        Loop4endR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Loop Begin"]
    #[inline(always)]
    pub fn loop5begin(&self) -> Loop5beginR {
        Loop5beginR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Loop End"]
    #[inline(always)]
    pub fn loop5end(&self) -> Loop5endR {
        Loop5endR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Loop Begin"]
    #[inline(always)]
    pub fn loop6begin(&self) -> Loop6beginR {
        Loop6beginR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Loop End"]
    #[inline(always)]
    pub fn loop6end(&self) -> Loop6endR {
        Loop6endR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Loop Begin"]
    #[inline(always)]
    pub fn loop7begin(&self) -> Loop7beginR {
        Loop7beginR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Loop End"]
    #[inline(always)]
    pub fn loop7end(&self) -> Loop7endR {
        Loop7endR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 20:28 - ALU opcode"]
    #[inline(always)]
    pub fn aluop(&self) -> AluopR {
        AluopR::new(((self.bits >> 20) & 0x01ff) as u16)
    }
    #[doc = "Bit 31 - End of Program"]
    #[inline(always)]
    pub fn endprog(&self) -> EndprogR {
        EndprogR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Loop Begin"]
    #[inline(always)]
    pub fn loop0begin(&mut self) -> Loop0beginW<'_, Instr0cfg2Spec> {
        Loop0beginW::new(self, 0)
    }
    #[doc = "Bit 1 - Loop End"]
    #[inline(always)]
    pub fn loop0end(&mut self) -> Loop0endW<'_, Instr0cfg2Spec> {
        Loop0endW::new(self, 1)
    }
    #[doc = "Bit 2 - Loop Begin"]
    #[inline(always)]
    pub fn loop1begin(&mut self) -> Loop1beginW<'_, Instr0cfg2Spec> {
        Loop1beginW::new(self, 2)
    }
    #[doc = "Bit 3 - Loop End"]
    #[inline(always)]
    pub fn loop1end(&mut self) -> Loop1endW<'_, Instr0cfg2Spec> {
        Loop1endW::new(self, 3)
    }
    #[doc = "Bit 4 - Loop Begin"]
    #[inline(always)]
    pub fn loop2begin(&mut self) -> Loop2beginW<'_, Instr0cfg2Spec> {
        Loop2beginW::new(self, 4)
    }
    #[doc = "Bit 5 - Loop End"]
    #[inline(always)]
    pub fn loop2end(&mut self) -> Loop2endW<'_, Instr0cfg2Spec> {
        Loop2endW::new(self, 5)
    }
    #[doc = "Bit 6 - Loop Begin"]
    #[inline(always)]
    pub fn loop3begin(&mut self) -> Loop3beginW<'_, Instr0cfg2Spec> {
        Loop3beginW::new(self, 6)
    }
    #[doc = "Bit 7 - Loop End"]
    #[inline(always)]
    pub fn loop3end(&mut self) -> Loop3endW<'_, Instr0cfg2Spec> {
        Loop3endW::new(self, 7)
    }
    #[doc = "Bit 8 - Loop Begin"]
    #[inline(always)]
    pub fn loop4begin(&mut self) -> Loop4beginW<'_, Instr0cfg2Spec> {
        Loop4beginW::new(self, 8)
    }
    #[doc = "Bit 9 - Loop End"]
    #[inline(always)]
    pub fn loop4end(&mut self) -> Loop4endW<'_, Instr0cfg2Spec> {
        Loop4endW::new(self, 9)
    }
    #[doc = "Bit 10 - Loop Begin"]
    #[inline(always)]
    pub fn loop5begin(&mut self) -> Loop5beginW<'_, Instr0cfg2Spec> {
        Loop5beginW::new(self, 10)
    }
    #[doc = "Bit 11 - Loop End"]
    #[inline(always)]
    pub fn loop5end(&mut self) -> Loop5endW<'_, Instr0cfg2Spec> {
        Loop5endW::new(self, 11)
    }
    #[doc = "Bit 12 - Loop Begin"]
    #[inline(always)]
    pub fn loop6begin(&mut self) -> Loop6beginW<'_, Instr0cfg2Spec> {
        Loop6beginW::new(self, 12)
    }
    #[doc = "Bit 13 - Loop End"]
    #[inline(always)]
    pub fn loop6end(&mut self) -> Loop6endW<'_, Instr0cfg2Spec> {
        Loop6endW::new(self, 13)
    }
    #[doc = "Bit 14 - Loop Begin"]
    #[inline(always)]
    pub fn loop7begin(&mut self) -> Loop7beginW<'_, Instr0cfg2Spec> {
        Loop7beginW::new(self, 14)
    }
    #[doc = "Bit 15 - Loop End"]
    #[inline(always)]
    pub fn loop7end(&mut self) -> Loop7endW<'_, Instr0cfg2Spec> {
        Loop7endW::new(self, 15)
    }
    #[doc = "Bits 20:28 - ALU opcode"]
    #[inline(always)]
    pub fn aluop(&mut self) -> AluopW<'_, Instr0cfg2Spec> {
        AluopW::new(self, 20)
    }
    #[doc = "Bit 31 - End of Program"]
    #[inline(always)]
    pub fn endprog(&mut self) -> EndprogW<'_, Instr0cfg2Spec> {
        EndprogW::new(self, 31)
    }
}
#[doc = "Instruction N word 2\n\nYou can [`read`](crate::Reg::read) this register and get [`instr0cfg2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr0cfg2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Instr0cfg2Spec;
impl crate::RegisterSpec for Instr0cfg2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`instr0cfg2::R`](R) reader structure"]
impl crate::Readable for Instr0cfg2Spec {}
#[doc = "`write(|w| ..)` method takes [`instr0cfg2::W`](W) writer structure"]
impl crate::Writable for Instr0cfg2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets INSTR0CFG2 to value 0"]
impl crate::Resettable for Instr0cfg2Spec {}
