#[doc = "Register `NOMCAL` reader"]
pub type R = crate::R<NomcalSpec>;
#[doc = "Register `NOMCAL` writer"]
pub type W = crate::W<NomcalSpec>;
#[doc = "Field `NOMCALCNT` reader - Nominal Calibration Count"]
pub type NomcalcntR = crate::FieldReader<u32>;
#[doc = "Field `NOMCALCNT` writer - Nominal Calibration Count"]
pub type NomcalcntW<'a, REG> = crate::FieldWriter<'a, REG, 21, u32>;
impl R {
    #[doc = "Bits 0:20 - Nominal Calibration Count"]
    #[inline(always)]
    pub fn nomcalcnt(&self) -> NomcalcntR {
        NomcalcntR::new(self.bits & 0x001f_ffff)
    }
}
impl W {
    #[doc = "Bits 0:20 - Nominal Calibration Count"]
    #[inline(always)]
    pub fn nomcalcnt(&mut self) -> NomcalcntW<'_, NomcalSpec> {
        NomcalcntW::new(self, 0)
    }
}
#[doc = "Nominal calibration register\n\nYou can [`read`](crate::Reg::read) this register and get [`nomcal::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`nomcal::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct NomcalSpec;
impl crate::RegisterSpec for NomcalSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`nomcal::R`](R) reader structure"]
impl crate::Readable for NomcalSpec {}
#[doc = "`write(|w| ..)` method takes [`nomcal::W`](W) writer structure"]
impl crate::Writable for NomcalSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets NOMCAL to value 0x0005_b8d8"]
impl crate::Resettable for NomcalSpec {
    const RESET_VALUE: u32 = 0x0005_b8d8;
}
