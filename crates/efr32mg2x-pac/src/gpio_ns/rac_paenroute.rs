#[doc = "Register `RAC_PAENROUTE` reader"]
pub type R = crate::R<RacPaenrouteSpec>;
#[doc = "Register `RAC_PAENROUTE` writer"]
pub type W = crate::W<RacPaenrouteSpec>;
#[doc = "Field `PORT` reader - PAEN port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - PAEN port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - PAEN pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - PAEN pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - PAEN port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - PAEN pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - PAEN port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, RacPaenrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - PAEN pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, RacPaenrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "PAEN port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`rac_paenroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rac_paenroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RacPaenrouteSpec;
impl crate::RegisterSpec for RacPaenrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rac_paenroute::R`](R) reader structure"]
impl crate::Readable for RacPaenrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`rac_paenroute::W`](W) writer structure"]
impl crate::Writable for RacPaenrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RAC_PAENROUTE to value 0"]
impl crate::Resettable for RacPaenrouteSpec {}
