#[doc = "Register `LOOP2STATE` reader"]
pub type R = crate::R<Loop2stateSpec>;
#[doc = "Register `LOOP2STATE` writer"]
pub type W = crate::W<Loop2stateSpec>;
#[doc = "Field `CNT` reader - Loop Counter"]
pub type CntR = crate::FieldReader<u16>;
#[doc = "Field `CNT` writer - Loop Counter"]
pub type CntW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `ACTIVE` reader - Loop Active"]
pub type ActiveR = crate::BitReader;
#[doc = "Field `ACTIVE` writer - Loop Active"]
pub type ActiveW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PCBEGIN` reader - Loop Start"]
pub type PcbeginR = crate::FieldReader;
#[doc = "Field `PCBEGIN` writer - Loop Start"]
pub type PcbeginW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:9 - Loop Counter"]
    #[inline(always)]
    pub fn cnt(&self) -> CntR {
        CntR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bit 12 - Loop Active"]
    #[inline(always)]
    pub fn active(&self) -> ActiveR {
        ActiveR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bits 16:18 - Loop Start"]
    #[inline(always)]
    pub fn pcbegin(&self) -> PcbeginR {
        PcbeginR::new(((self.bits >> 16) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:9 - Loop Counter"]
    #[inline(always)]
    pub fn cnt(&mut self) -> CntW<'_, Loop2stateSpec> {
        CntW::new(self, 0)
    }
    #[doc = "Bit 12 - Loop Active"]
    #[inline(always)]
    pub fn active(&mut self) -> ActiveW<'_, Loop2stateSpec> {
        ActiveW::new(self, 12)
    }
    #[doc = "Bits 16:18 - Loop Start"]
    #[inline(always)]
    pub fn pcbegin(&mut self) -> PcbeginW<'_, Loop2stateSpec> {
        PcbeginW::new(self, 16)
    }
}
#[doc = "Loop N State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop2state::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop2state::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Loop2stateSpec;
impl crate::RegisterSpec for Loop2stateSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`loop2state::R`](R) reader structure"]
impl crate::Readable for Loop2stateSpec {}
#[doc = "`write(|w| ..)` method takes [`loop2state::W`](W) writer structure"]
impl crate::Writable for Loop2stateSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LOOP2STATE to value 0"]
impl crate::Resettable for Loop2stateSpec {}
