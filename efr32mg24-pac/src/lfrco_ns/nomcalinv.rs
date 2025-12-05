#[doc = "Register `NOMCALINV` reader"]
pub type R = crate::R<NomcalinvSpec>;
#[doc = "Register `NOMCALINV` writer"]
pub type W = crate::W<NomcalinvSpec>;
#[doc = "Field `NOMCALCNTINV` reader - Nominal Calibration Count Inverted"]
pub type NomcalcntinvR = crate::FieldReader<u32>;
#[doc = "Field `NOMCALCNTINV` writer - Nominal Calibration Count Inverted"]
pub type NomcalcntinvW<'a, REG> = crate::FieldWriter<'a, REG, 17, u32>;
impl R {
    #[doc = "Bits 0:16 - Nominal Calibration Count Inverted"]
    #[inline(always)]
    pub fn nomcalcntinv(&self) -> NomcalcntinvR {
        NomcalcntinvR::new(self.bits & 0x0001_ffff)
    }
}
impl W {
    #[doc = "Bits 0:16 - Nominal Calibration Count Inverted"]
    #[inline(always)]
    pub fn nomcalcntinv(&mut self) -> NomcalcntinvW<'_, NomcalinvSpec> {
        NomcalcntinvW::new(self, 0)
    }
}
#[doc = "Nominal calibration inverted register\n\nYou can [`read`](crate::Reg::read) this register and get [`nomcalinv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`nomcalinv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct NomcalinvSpec;
impl crate::RegisterSpec for NomcalinvSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`nomcalinv::R`](R) reader structure"]
impl crate::Readable for NomcalinvSpec {}
#[doc = "`write(|w| ..)` method takes [`nomcalinv::W`](W) writer structure"]
impl crate::Writable for NomcalinvSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets NOMCALINV to value 0x597a"]
impl crate::Resettable for NomcalinvSpec {
    const RESET_VALUE: u32 = 0x597a;
}
