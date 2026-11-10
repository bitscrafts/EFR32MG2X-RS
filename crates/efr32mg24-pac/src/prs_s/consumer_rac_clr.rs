#[doc = "Register `CONSUMER_RAC_CLR` reader"]
pub type R = crate::R<ConsumerRacClrSpec>;
#[doc = "Register `CONSUMER_RAC_CLR` writer"]
pub type W = crate::W<ConsumerRacClrSpec>;
#[doc = "Field `PRSSEL` reader - CLR async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - CLR async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - CLR async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - CLR async channel select"]
    #[inline(always)]
    pub fn prssel(&mut self) -> PrsselW<'_, ConsumerRacClrSpec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "CLR consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_rac_clr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_rac_clr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerRacClrSpec;
impl crate::RegisterSpec for ConsumerRacClrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_rac_clr::R`](R) reader structure"]
impl crate::Readable for ConsumerRacClrSpec {}
#[doc = "`write(|w| ..)` method takes [`consumer_rac_clr::W`](W) writer structure"]
impl crate::Writable for ConsumerRacClrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONSUMER_RAC_CLR to value 0"]
impl crate::Resettable for ConsumerRacClrSpec {}
