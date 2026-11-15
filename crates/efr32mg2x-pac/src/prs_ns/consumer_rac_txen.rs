#[doc = "Register `CONSUMER_RAC_TXEN` reader"]
pub type R = crate::R<ConsumerRacTxenSpec>;
#[doc = "Register `CONSUMER_RAC_TXEN` writer"]
pub type W = crate::W<ConsumerRacTxenSpec>;
#[doc = "Field `PRSSEL` reader - TXEN async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - TXEN async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - TXEN async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - TXEN async channel select"]
    #[inline(always)]
    pub fn prssel(&mut self) -> PrsselW<'_, ConsumerRacTxenSpec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "TXEN Consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_rac_txen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_rac_txen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerRacTxenSpec;
impl crate::RegisterSpec for ConsumerRacTxenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_rac_txen::R`](R) reader structure"]
impl crate::Readable for ConsumerRacTxenSpec {}
#[doc = "`write(|w| ..)` method takes [`consumer_rac_txen::W`](W) writer structure"]
impl crate::Writable for ConsumerRacTxenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONSUMER_RAC_TXEN to value 0"]
impl crate::Resettable for ConsumerRacTxenSpec {}
