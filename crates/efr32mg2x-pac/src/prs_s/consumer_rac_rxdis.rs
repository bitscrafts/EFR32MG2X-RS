#[doc = "Register `CONSUMER_RAC_RXDIS` reader"]
pub type R = crate::R<ConsumerRacRxdisSpec>;
#[doc = "Register `CONSUMER_RAC_RXDIS` writer"]
pub type W = crate::W<ConsumerRacRxdisSpec>;
#[doc = "Field `PRSSEL` reader - RXDIS async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - RXDIS async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - RXDIS async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - RXDIS async channel select"]
    #[inline(always)]
    pub fn prssel(&mut self) -> PrsselW<'_, ConsumerRacRxdisSpec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "RXDIS Consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_rac_rxdis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_rac_rxdis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerRacRxdisSpec;
impl crate::RegisterSpec for ConsumerRacRxdisSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_rac_rxdis::R`](R) reader structure"]
impl crate::Readable for ConsumerRacRxdisSpec {}
#[doc = "`write(|w| ..)` method takes [`consumer_rac_rxdis::W`](W) writer structure"]
impl crate::Writable for ConsumerRacRxdisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONSUMER_RAC_RXDIS to value 0"]
impl crate::Resettable for ConsumerRacRxdisSpec {}
