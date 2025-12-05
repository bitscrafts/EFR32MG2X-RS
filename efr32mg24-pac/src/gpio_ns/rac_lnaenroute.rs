#[doc = "Register `RAC_LNAENROUTE` reader"]
pub type R = crate::R<RacLnaenrouteSpec>;
#[doc = "Register `RAC_LNAENROUTE` writer"]
pub type W = crate::W<RacLnaenrouteSpec>;
#[doc = "Field `PORT` reader - LNAEN port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - LNAEN port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - LNAEN pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - LNAEN pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - LNAEN port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - LNAEN pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - LNAEN port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, RacLnaenrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - LNAEN pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, RacLnaenrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "LNAEN port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`rac_lnaenroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rac_lnaenroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RacLnaenrouteSpec;
impl crate::RegisterSpec for RacLnaenrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rac_lnaenroute::R`](R) reader structure"]
impl crate::Readable for RacLnaenrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`rac_lnaenroute::W`](W) writer structure"]
impl crate::Writable for RacLnaenrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RAC_LNAENROUTE to value 0"]
impl crate::Resettable for RacLnaenrouteSpec {}
