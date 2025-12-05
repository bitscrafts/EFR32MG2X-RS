#[doc = "Register `CONSUMER_RAC_RXEN` reader"]
pub type R = crate::R<ConsumerRacRxenSpec>;
#[doc = "Register `CONSUMER_RAC_RXEN` writer"]
pub type W = crate::W<ConsumerRacRxenSpec>;
#[doc = "Field `PRSSEL` reader - RXEN async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - RXEN async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - RXEN async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - RXEN async channel select"]
    #[inline(always)]
    pub fn prssel(&mut self) -> PrsselW<'_, ConsumerRacRxenSpec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "RXEN Consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_rac_rxen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_rac_rxen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerRacRxenSpec;
impl crate::RegisterSpec for ConsumerRacRxenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_rac_rxen::R`](R) reader structure"]
impl crate::Readable for ConsumerRacRxenSpec {}
#[doc = "`write(|w| ..)` method takes [`consumer_rac_rxen::W`](W) writer structure"]
impl crate::Writable for ConsumerRacRxenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONSUMER_RAC_RXEN to value 0"]
impl crate::Resettable for ConsumerRacRxenSpec {}
