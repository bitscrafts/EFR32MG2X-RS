#[doc = "Register `CMD` writer"]
pub type W = crate::W<CmdSpec>;
#[doc = "Field `START` writer - Start Command"]
pub type StartW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `HALT` writer - Halt Command"]
pub type HaltW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `STEP` writer - Step Command"]
pub type StepW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `INIT` writer - Initialization Command/Qualifier"]
pub type InitW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Start Command"]
    #[inline(always)]
    pub fn start(&mut self) -> StartW<'_, CmdSpec> {
        StartW::new(self, 0)
    }
    #[doc = "Bit 1 - Halt Command"]
    #[inline(always)]
    pub fn halt(&mut self) -> HaltW<'_, CmdSpec> {
        HaltW::new(self, 1)
    }
    #[doc = "Bit 2 - Step Command"]
    #[inline(always)]
    pub fn step(&mut self) -> StepW<'_, CmdSpec> {
        StepW::new(self, 2)
    }
    #[doc = "Bit 3 - Initialization Command/Qualifier"]
    #[inline(always)]
    pub fn init(&mut self) -> InitW<'_, CmdSpec> {
        InitW::new(self, 3)
    }
}
#[doc = "Command Register\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cmd::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CmdSpec;
impl crate::RegisterSpec for CmdSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`cmd::W`](W) writer structure"]
impl crate::Writable for CmdSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CMD to value 0"]
impl crate::Resettable for CmdSpec {}
