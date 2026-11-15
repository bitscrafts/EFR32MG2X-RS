#[doc = "Register `RAC_ROUTEEN` reader"]
pub type R = crate::R<RacRouteenSpec>;
#[doc = "Register `RAC_ROUTEEN` writer"]
pub type W = crate::W<RacRouteenSpec>;
#[doc = "Field `LNAENPEN` reader - LNAEN pin enable control bit"]
pub type LnaenpenR = crate::BitReader;
#[doc = "Field `LNAENPEN` writer - LNAEN pin enable control bit"]
pub type LnaenpenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PAENPEN` reader - PAEN pin enable control bit"]
pub type PaenpenR = crate::BitReader;
#[doc = "Field `PAENPEN` writer - PAEN pin enable control bit"]
pub type PaenpenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - LNAEN pin enable control bit"]
    #[inline(always)]
    pub fn lnaenpen(&self) -> LnaenpenR {
        LnaenpenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - PAEN pin enable control bit"]
    #[inline(always)]
    pub fn paenpen(&self) -> PaenpenR {
        PaenpenR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LNAEN pin enable control bit"]
    #[inline(always)]
    pub fn lnaenpen(&mut self) -> LnaenpenW<'_, RacRouteenSpec> {
        LnaenpenW::new(self, 0)
    }
    #[doc = "Bit 1 - PAEN pin enable control bit"]
    #[inline(always)]
    pub fn paenpen(&mut self) -> PaenpenW<'_, RacRouteenSpec> {
        PaenpenW::new(self, 1)
    }
}
#[doc = "RAC pin enable\n\nYou can [`read`](crate::Reg::read) this register and get [`rac_routeen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rac_routeen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RacRouteenSpec;
impl crate::RegisterSpec for RacRouteenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rac_routeen::R`](R) reader structure"]
impl crate::Readable for RacRouteenSpec {}
#[doc = "`write(|w| ..)` method takes [`rac_routeen::W`](W) writer structure"]
impl crate::Writable for RacRouteenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RAC_ROUTEEN to value 0"]
impl crate::Resettable for RacRouteenSpec {}
