#[doc = "Register `CFG` reader"]
pub type R = crate::R<CfgSpec>;
#[doc = "Register `CFG` writer"]
pub type W = crate::W<CfgSpec>;
#[doc = "Field `PERFCNTEN` reader - Performance Counter Enable"]
pub type PerfcntenR = crate::BitReader;
#[doc = "Field `PERFCNTEN` writer - Performance Counter Enable"]
pub type PerfcntenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `OUTCOMPRESSDIS` reader - ALU Output Stream Compression Disable"]
pub type OutcompressdisR = crate::BitReader;
#[doc = "Field `OUTCOMPRESSDIS` writer - ALU Output Stream Compression Disable"]
pub type OutcompressdisW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `INCACHEDIS` reader - ALU Input Word Cache Disable"]
pub type IncachedisR = crate::BitReader;
#[doc = "Field `INCACHEDIS` writer - ALU Input Word Cache Disable"]
pub type IncachedisW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOPERRHALTDIS` reader - Loop Error Halt Disable"]
pub type LooperrhaltdisR = crate::BitReader;
#[doc = "Field `LOOPERRHALTDIS` writer - Loop Error Halt Disable"]
pub type LooperrhaltdisW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Performance Counter Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Perf0cntsel {
    #[doc = "0: Total run count"]
    Run = 0,
    #[doc = "1: Total Commands Issued"]
    Cmd = 1,
    #[doc = "2: Total stall count (at sequencer issue)"]
    Stall = 2,
    #[doc = "3: NOOP ALU Op counter"]
    Noop = 3,
    #[doc = "4: Count cycles that ALU is active (not stalled), excluding NOOPs"]
    Aluactive = 4,
    #[doc = "5: Stalls caused by register and resource conflicts within the ALU."]
    Pipestall = 5,
    #[doc = "6: Count stall cycles caused by memory hazards"]
    Iofencestall = 6,
    #[doc = "7: Count stall cycles when accessing memory from load stream 0"]
    Load0stall = 7,
    #[doc = "8: Count stall cycles when accessing memory from load stream 1"]
    Load1stall = 8,
    #[doc = "9: Count stall cycles when writing memory from store stream"]
    Storestall = 9,
    #[doc = "10: Count cycles where any of previous 3 events is occurring"]
    Busstall = 10,
    #[doc = "11: All stall cycles on load bus 0 AHB interface"]
    Load0ahbstall = 11,
    #[doc = "12: All stall cycles on load bus 1 AHB interface"]
    Load1ahbstall = 12,
    #[doc = "13: LOAD0 Fence Stall cycles"]
    Load0fencestall = 13,
    #[doc = "14: LOAD1 Fence Stall cycles"]
    Load1fencestall = 14,
}
impl From<Perf0cntsel> for u8 {
    #[inline(always)]
    fn from(variant: Perf0cntsel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Perf0cntsel {
    type Ux = u8;
}
impl crate::IsEnum for Perf0cntsel {}
#[doc = "Field `PERF0CNTSEL` reader - Performance Counter Select"]
pub type Perf0cntselR = crate::FieldReader<Perf0cntsel>;
impl Perf0cntselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Perf0cntsel> {
        match self.bits {
            0 => Some(Perf0cntsel::Run),
            1 => Some(Perf0cntsel::Cmd),
            2 => Some(Perf0cntsel::Stall),
            3 => Some(Perf0cntsel::Noop),
            4 => Some(Perf0cntsel::Aluactive),
            5 => Some(Perf0cntsel::Pipestall),
            6 => Some(Perf0cntsel::Iofencestall),
            7 => Some(Perf0cntsel::Load0stall),
            8 => Some(Perf0cntsel::Load1stall),
            9 => Some(Perf0cntsel::Storestall),
            10 => Some(Perf0cntsel::Busstall),
            11 => Some(Perf0cntsel::Load0ahbstall),
            12 => Some(Perf0cntsel::Load1ahbstall),
            13 => Some(Perf0cntsel::Load0fencestall),
            14 => Some(Perf0cntsel::Load1fencestall),
            _ => None,
        }
    }
    #[doc = "Total run count"]
    #[inline(always)]
    pub fn is_run(&self) -> bool {
        *self == Perf0cntsel::Run
    }
    #[doc = "Total Commands Issued"]
    #[inline(always)]
    pub fn is_cmd(&self) -> bool {
        *self == Perf0cntsel::Cmd
    }
    #[doc = "Total stall count (at sequencer issue)"]
    #[inline(always)]
    pub fn is_stall(&self) -> bool {
        *self == Perf0cntsel::Stall
    }
    #[doc = "NOOP ALU Op counter"]
    #[inline(always)]
    pub fn is_noop(&self) -> bool {
        *self == Perf0cntsel::Noop
    }
    #[doc = "Count cycles that ALU is active (not stalled), excluding NOOPs"]
    #[inline(always)]
    pub fn is_aluactive(&self) -> bool {
        *self == Perf0cntsel::Aluactive
    }
    #[doc = "Stalls caused by register and resource conflicts within the ALU."]
    #[inline(always)]
    pub fn is_pipestall(&self) -> bool {
        *self == Perf0cntsel::Pipestall
    }
    #[doc = "Count stall cycles caused by memory hazards"]
    #[inline(always)]
    pub fn is_iofencestall(&self) -> bool {
        *self == Perf0cntsel::Iofencestall
    }
    #[doc = "Count stall cycles when accessing memory from load stream 0"]
    #[inline(always)]
    pub fn is_load0stall(&self) -> bool {
        *self == Perf0cntsel::Load0stall
    }
    #[doc = "Count stall cycles when accessing memory from load stream 1"]
    #[inline(always)]
    pub fn is_load1stall(&self) -> bool {
        *self == Perf0cntsel::Load1stall
    }
    #[doc = "Count stall cycles when writing memory from store stream"]
    #[inline(always)]
    pub fn is_storestall(&self) -> bool {
        *self == Perf0cntsel::Storestall
    }
    #[doc = "Count cycles where any of previous 3 events is occurring"]
    #[inline(always)]
    pub fn is_busstall(&self) -> bool {
        *self == Perf0cntsel::Busstall
    }
    #[doc = "All stall cycles on load bus 0 AHB interface"]
    #[inline(always)]
    pub fn is_load0ahbstall(&self) -> bool {
        *self == Perf0cntsel::Load0ahbstall
    }
    #[doc = "All stall cycles on load bus 1 AHB interface"]
    #[inline(always)]
    pub fn is_load1ahbstall(&self) -> bool {
        *self == Perf0cntsel::Load1ahbstall
    }
    #[doc = "LOAD0 Fence Stall cycles"]
    #[inline(always)]
    pub fn is_load0fencestall(&self) -> bool {
        *self == Perf0cntsel::Load0fencestall
    }
    #[doc = "LOAD1 Fence Stall cycles"]
    #[inline(always)]
    pub fn is_load1fencestall(&self) -> bool {
        *self == Perf0cntsel::Load1fencestall
    }
}
#[doc = "Field `PERF0CNTSEL` writer - Performance Counter Select"]
pub type Perf0cntselW<'a, REG> = crate::FieldWriter<'a, REG, 4, Perf0cntsel>;
impl<'a, REG> Perf0cntselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Total run count"]
    #[inline(always)]
    pub fn run(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Run)
    }
    #[doc = "Total Commands Issued"]
    #[inline(always)]
    pub fn cmd(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Cmd)
    }
    #[doc = "Total stall count (at sequencer issue)"]
    #[inline(always)]
    pub fn stall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Stall)
    }
    #[doc = "NOOP ALU Op counter"]
    #[inline(always)]
    pub fn noop(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Noop)
    }
    #[doc = "Count cycles that ALU is active (not stalled), excluding NOOPs"]
    #[inline(always)]
    pub fn aluactive(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Aluactive)
    }
    #[doc = "Stalls caused by register and resource conflicts within the ALU."]
    #[inline(always)]
    pub fn pipestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Pipestall)
    }
    #[doc = "Count stall cycles caused by memory hazards"]
    #[inline(always)]
    pub fn iofencestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Iofencestall)
    }
    #[doc = "Count stall cycles when accessing memory from load stream 0"]
    #[inline(always)]
    pub fn load0stall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Load0stall)
    }
    #[doc = "Count stall cycles when accessing memory from load stream 1"]
    #[inline(always)]
    pub fn load1stall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Load1stall)
    }
    #[doc = "Count stall cycles when writing memory from store stream"]
    #[inline(always)]
    pub fn storestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Storestall)
    }
    #[doc = "Count cycles where any of previous 3 events is occurring"]
    #[inline(always)]
    pub fn busstall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Busstall)
    }
    #[doc = "All stall cycles on load bus 0 AHB interface"]
    #[inline(always)]
    pub fn load0ahbstall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Load0ahbstall)
    }
    #[doc = "All stall cycles on load bus 1 AHB interface"]
    #[inline(always)]
    pub fn load1ahbstall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Load1ahbstall)
    }
    #[doc = "LOAD0 Fence Stall cycles"]
    #[inline(always)]
    pub fn load0fencestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Load0fencestall)
    }
    #[doc = "LOAD1 Fence Stall cycles"]
    #[inline(always)]
    pub fn load1fencestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf0cntsel::Load1fencestall)
    }
}
#[doc = "Performance Counter Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Perf1cntsel {
    #[doc = "0: Total run count"]
    Run = 0,
    #[doc = "1: Total Commands Issued"]
    Cmd = 1,
    #[doc = "2: Total stall count (at sequencer issue)"]
    Stall = 2,
    #[doc = "3: NOOP ALU Op counter"]
    Noop = 3,
    #[doc = "4: Count cycles that ALU is active (not stalled), excluding NOOPs"]
    Aluactive = 4,
    #[doc = "5: Stalls caused by register and resource conflicts within the ALU."]
    Pipestall = 5,
    #[doc = "6: Count stall cycles caused by memory hazards"]
    Iofencestall = 6,
    #[doc = "7: Count stall cycles when accessing memory from load stream 0"]
    Load0stall = 7,
    #[doc = "8: Count stall cycles when accessing memory from load stream 1"]
    Load1stall = 8,
    #[doc = "9: Count stall cycles when writing memory from store stream"]
    Storestall = 9,
    #[doc = "10: Count cycles where any of previous 3 events is occurring"]
    Busstall = 10,
    #[doc = "11: All stall cycles on load bus 0 AHB interface"]
    Load0ahbstall = 11,
    #[doc = "12: All stall cycles on load bus 1 AHB interface"]
    Load1ahbstall = 12,
    #[doc = "13: LOAD0 Fence Stall cycles"]
    Load0fencestall = 13,
    #[doc = "14: LOAD1 Fence Stall cycles"]
    Load1fencestall = 14,
}
impl From<Perf1cntsel> for u8 {
    #[inline(always)]
    fn from(variant: Perf1cntsel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Perf1cntsel {
    type Ux = u8;
}
impl crate::IsEnum for Perf1cntsel {}
#[doc = "Field `PERF1CNTSEL` reader - Performance Counter Select"]
pub type Perf1cntselR = crate::FieldReader<Perf1cntsel>;
impl Perf1cntselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Perf1cntsel> {
        match self.bits {
            0 => Some(Perf1cntsel::Run),
            1 => Some(Perf1cntsel::Cmd),
            2 => Some(Perf1cntsel::Stall),
            3 => Some(Perf1cntsel::Noop),
            4 => Some(Perf1cntsel::Aluactive),
            5 => Some(Perf1cntsel::Pipestall),
            6 => Some(Perf1cntsel::Iofencestall),
            7 => Some(Perf1cntsel::Load0stall),
            8 => Some(Perf1cntsel::Load1stall),
            9 => Some(Perf1cntsel::Storestall),
            10 => Some(Perf1cntsel::Busstall),
            11 => Some(Perf1cntsel::Load0ahbstall),
            12 => Some(Perf1cntsel::Load1ahbstall),
            13 => Some(Perf1cntsel::Load0fencestall),
            14 => Some(Perf1cntsel::Load1fencestall),
            _ => None,
        }
    }
    #[doc = "Total run count"]
    #[inline(always)]
    pub fn is_run(&self) -> bool {
        *self == Perf1cntsel::Run
    }
    #[doc = "Total Commands Issued"]
    #[inline(always)]
    pub fn is_cmd(&self) -> bool {
        *self == Perf1cntsel::Cmd
    }
    #[doc = "Total stall count (at sequencer issue)"]
    #[inline(always)]
    pub fn is_stall(&self) -> bool {
        *self == Perf1cntsel::Stall
    }
    #[doc = "NOOP ALU Op counter"]
    #[inline(always)]
    pub fn is_noop(&self) -> bool {
        *self == Perf1cntsel::Noop
    }
    #[doc = "Count cycles that ALU is active (not stalled), excluding NOOPs"]
    #[inline(always)]
    pub fn is_aluactive(&self) -> bool {
        *self == Perf1cntsel::Aluactive
    }
    #[doc = "Stalls caused by register and resource conflicts within the ALU."]
    #[inline(always)]
    pub fn is_pipestall(&self) -> bool {
        *self == Perf1cntsel::Pipestall
    }
    #[doc = "Count stall cycles caused by memory hazards"]
    #[inline(always)]
    pub fn is_iofencestall(&self) -> bool {
        *self == Perf1cntsel::Iofencestall
    }
    #[doc = "Count stall cycles when accessing memory from load stream 0"]
    #[inline(always)]
    pub fn is_load0stall(&self) -> bool {
        *self == Perf1cntsel::Load0stall
    }
    #[doc = "Count stall cycles when accessing memory from load stream 1"]
    #[inline(always)]
    pub fn is_load1stall(&self) -> bool {
        *self == Perf1cntsel::Load1stall
    }
    #[doc = "Count stall cycles when writing memory from store stream"]
    #[inline(always)]
    pub fn is_storestall(&self) -> bool {
        *self == Perf1cntsel::Storestall
    }
    #[doc = "Count cycles where any of previous 3 events is occurring"]
    #[inline(always)]
    pub fn is_busstall(&self) -> bool {
        *self == Perf1cntsel::Busstall
    }
    #[doc = "All stall cycles on load bus 0 AHB interface"]
    #[inline(always)]
    pub fn is_load0ahbstall(&self) -> bool {
        *self == Perf1cntsel::Load0ahbstall
    }
    #[doc = "All stall cycles on load bus 1 AHB interface"]
    #[inline(always)]
    pub fn is_load1ahbstall(&self) -> bool {
        *self == Perf1cntsel::Load1ahbstall
    }
    #[doc = "LOAD0 Fence Stall cycles"]
    #[inline(always)]
    pub fn is_load0fencestall(&self) -> bool {
        *self == Perf1cntsel::Load0fencestall
    }
    #[doc = "LOAD1 Fence Stall cycles"]
    #[inline(always)]
    pub fn is_load1fencestall(&self) -> bool {
        *self == Perf1cntsel::Load1fencestall
    }
}
#[doc = "Field `PERF1CNTSEL` writer - Performance Counter Select"]
pub type Perf1cntselW<'a, REG> = crate::FieldWriter<'a, REG, 4, Perf1cntsel>;
impl<'a, REG> Perf1cntselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Total run count"]
    #[inline(always)]
    pub fn run(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Run)
    }
    #[doc = "Total Commands Issued"]
    #[inline(always)]
    pub fn cmd(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Cmd)
    }
    #[doc = "Total stall count (at sequencer issue)"]
    #[inline(always)]
    pub fn stall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Stall)
    }
    #[doc = "NOOP ALU Op counter"]
    #[inline(always)]
    pub fn noop(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Noop)
    }
    #[doc = "Count cycles that ALU is active (not stalled), excluding NOOPs"]
    #[inline(always)]
    pub fn aluactive(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Aluactive)
    }
    #[doc = "Stalls caused by register and resource conflicts within the ALU."]
    #[inline(always)]
    pub fn pipestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Pipestall)
    }
    #[doc = "Count stall cycles caused by memory hazards"]
    #[inline(always)]
    pub fn iofencestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Iofencestall)
    }
    #[doc = "Count stall cycles when accessing memory from load stream 0"]
    #[inline(always)]
    pub fn load0stall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Load0stall)
    }
    #[doc = "Count stall cycles when accessing memory from load stream 1"]
    #[inline(always)]
    pub fn load1stall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Load1stall)
    }
    #[doc = "Count stall cycles when writing memory from store stream"]
    #[inline(always)]
    pub fn storestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Storestall)
    }
    #[doc = "Count cycles where any of previous 3 events is occurring"]
    #[inline(always)]
    pub fn busstall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Busstall)
    }
    #[doc = "All stall cycles on load bus 0 AHB interface"]
    #[inline(always)]
    pub fn load0ahbstall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Load0ahbstall)
    }
    #[doc = "All stall cycles on load bus 1 AHB interface"]
    #[inline(always)]
    pub fn load1ahbstall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Load1ahbstall)
    }
    #[doc = "LOAD0 Fence Stall cycles"]
    #[inline(always)]
    pub fn load0fencestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Load0fencestall)
    }
    #[doc = "LOAD1 Fence Stall cycles"]
    #[inline(always)]
    pub fn load1fencestall(self) -> &'a mut crate::W<REG> {
        self.variant(Perf1cntsel::Load1fencestall)
    }
}
impl R {
    #[doc = "Bit 0 - Performance Counter Enable"]
    #[inline(always)]
    pub fn perfcnten(&self) -> PerfcntenR {
        PerfcntenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - ALU Output Stream Compression Disable"]
    #[inline(always)]
    pub fn outcompressdis(&self) -> OutcompressdisR {
        OutcompressdisR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - ALU Input Word Cache Disable"]
    #[inline(always)]
    pub fn incachedis(&self) -> IncachedisR {
        IncachedisR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Loop Error Halt Disable"]
    #[inline(always)]
    pub fn looperrhaltdis(&self) -> LooperrhaltdisR {
        LooperrhaltdisR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 16:19 - Performance Counter Select"]
    #[inline(always)]
    pub fn perf0cntsel(&self) -> Perf0cntselR {
        Perf0cntselR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 20:23 - Performance Counter Select"]
    #[inline(always)]
    pub fn perf1cntsel(&self) -> Perf1cntselR {
        Perf1cntselR::new(((self.bits >> 20) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Performance Counter Enable"]
    #[inline(always)]
    pub fn perfcnten(&mut self) -> PerfcntenW<'_, CfgSpec> {
        PerfcntenW::new(self, 0)
    }
    #[doc = "Bit 1 - ALU Output Stream Compression Disable"]
    #[inline(always)]
    pub fn outcompressdis(&mut self) -> OutcompressdisW<'_, CfgSpec> {
        OutcompressdisW::new(self, 1)
    }
    #[doc = "Bit 2 - ALU Input Word Cache Disable"]
    #[inline(always)]
    pub fn incachedis(&mut self) -> IncachedisW<'_, CfgSpec> {
        IncachedisW::new(self, 2)
    }
    #[doc = "Bit 3 - Loop Error Halt Disable"]
    #[inline(always)]
    pub fn looperrhaltdis(&mut self) -> LooperrhaltdisW<'_, CfgSpec> {
        LooperrhaltdisW::new(self, 3)
    }
    #[doc = "Bits 16:19 - Performance Counter Select"]
    #[inline(always)]
    pub fn perf0cntsel(&mut self) -> Perf0cntselW<'_, CfgSpec> {
        Perf0cntselW::new(self, 16)
    }
    #[doc = "Bits 20:23 - Performance Counter Select"]
    #[inline(always)]
    pub fn perf1cntsel(&mut self) -> Perf1cntselW<'_, CfgSpec> {
        Perf1cntselW::new(self, 20)
    }
}
#[doc = "Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CfgSpec;
impl crate::RegisterSpec for CfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cfg::R`](R) reader structure"]
impl crate::Readable for CfgSpec {}
#[doc = "`write(|w| ..)` method takes [`cfg::W`](W) writer structure"]
impl crate::Writable for CfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CFG to value 0"]
impl crate::Resettable for CfgSpec {}
