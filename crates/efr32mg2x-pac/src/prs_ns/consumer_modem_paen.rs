#[doc = "Register `CONSUMER_MODEM_PAEN` reader"]
pub type R = crate::R<ConsumerModemPaenSpec>;
#[doc = "Register `CONSUMER_MODEM_PAEN` writer"]
pub type W = crate::W<ConsumerModemPaenSpec>;
#[doc = "Field `PRSSEL` reader - PAEN async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - PAEN async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - PAEN async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - PAEN async channel select"]
    #[inline(always)]
    pub fn prssel(&mut self) -> PrsselW<'_, ConsumerModemPaenSpec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "PAEN Consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_modem_paen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_modem_paen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerModemPaenSpec;
impl crate::RegisterSpec for ConsumerModemPaenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_modem_paen::R`](R) reader structure"]
impl crate::Readable for ConsumerModemPaenSpec {}
#[doc = "`write(|w| ..)` method takes [`consumer_modem_paen::W`](W) writer structure"]
impl crate::Writable for ConsumerModemPaenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONSUMER_MODEM_PAEN to value 0"]
impl crate::Resettable for ConsumerModemPaenSpec {}
