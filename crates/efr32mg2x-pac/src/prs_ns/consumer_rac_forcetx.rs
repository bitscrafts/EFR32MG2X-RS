#[doc = "Register `CONSUMER_RAC_FORCETX` reader"]
pub type R = crate::R<ConsumerRacForcetxSpec>;
#[doc = "Register `CONSUMER_RAC_FORCETX` writer"]
pub type W = crate::W<ConsumerRacForcetxSpec>;
#[doc = "Field `PRSSEL` reader - FORCETX async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - FORCETX async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - FORCETX async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - FORCETX async channel select"]
    #[inline(always)]
    pub fn prssel(&mut self) -> PrsselW<'_, ConsumerRacForcetxSpec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "FORCETX Consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_rac_forcetx::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_rac_forcetx::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerRacForcetxSpec;
impl crate::RegisterSpec for ConsumerRacForcetxSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_rac_forcetx::R`](R) reader structure"]
impl crate::Readable for ConsumerRacForcetxSpec {}
#[doc = "`write(|w| ..)` method takes [`consumer_rac_forcetx::W`](W) writer structure"]
impl crate::Writable for ConsumerRacForcetxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONSUMER_RAC_FORCETX to value 0"]
impl crate::Resettable for ConsumerRacForcetxSpec {}
