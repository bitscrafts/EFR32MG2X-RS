#[doc = "Register `CONSUMER_MODEM_DIN` reader"]
pub type R = crate::R<ConsumerModemDinSpec>;
#[doc = "Register `CONSUMER_MODEM_DIN` writer"]
pub type W = crate::W<ConsumerModemDinSpec>;
#[doc = "Field `PRSSEL` reader - DIN async channel select"]
pub type PrsselR = crate::FieldReader;
#[doc = "Field `PRSSEL` writer - DIN async channel select"]
pub type PrsselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - DIN async channel select"]
    #[inline(always)]
    pub fn prssel(&self) -> PrsselR {
        PrsselR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - DIN async channel select"]
    #[inline(always)]
    pub fn prssel(&mut self) -> PrsselW<'_, ConsumerModemDinSpec> {
        PrsselW::new(self, 0)
    }
}
#[doc = "DIN consumer register\n\nYou can [`read`](crate::Reg::read) this register and get [`consumer_modem_din::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`consumer_modem_din::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ConsumerModemDinSpec;
impl crate::RegisterSpec for ConsumerModemDinSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`consumer_modem_din::R`](R) reader structure"]
impl crate::Readable for ConsumerModemDinSpec {}
#[doc = "`write(|w| ..)` method takes [`consumer_modem_din::W`](W) writer structure"]
impl crate::Writable for ConsumerModemDinSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CONSUMER_MODEM_DIN to value 0"]
impl crate::Resettable for ConsumerModemDinSpec {}
