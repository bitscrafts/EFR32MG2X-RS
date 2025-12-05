#[doc = "Register `FRC_ROUTEEN` reader"]
pub type R = crate::R<FrcRouteenSpec>;
#[doc = "Register `FRC_ROUTEEN` writer"]
pub type W = crate::W<FrcRouteenSpec>;
#[doc = "Field `DCLKPEN` reader - DCLK pin enable control bit"]
pub type DclkpenR = crate::BitReader;
#[doc = "Field `DCLKPEN` writer - DCLK pin enable control bit"]
pub type DclkpenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DFRAMEPEN` reader - DFRAME pin enable control bit"]
pub type DframepenR = crate::BitReader;
#[doc = "Field `DFRAMEPEN` writer - DFRAME pin enable control bit"]
pub type DframepenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DOUTPEN` reader - DOUT pin enable control bit"]
pub type DoutpenR = crate::BitReader;
#[doc = "Field `DOUTPEN` writer - DOUT pin enable control bit"]
pub type DoutpenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - DCLK pin enable control bit"]
    #[inline(always)]
    pub fn dclkpen(&self) -> DclkpenR {
        DclkpenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - DFRAME pin enable control bit"]
    #[inline(always)]
    pub fn dframepen(&self) -> DframepenR {
        DframepenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - DOUT pin enable control bit"]
    #[inline(always)]
    pub fn doutpen(&self) -> DoutpenR {
        DoutpenR::new(((self.bits >> 2) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - DCLK pin enable control bit"]
    #[inline(always)]
    pub fn dclkpen(&mut self) -> DclkpenW<'_, FrcRouteenSpec> {
        DclkpenW::new(self, 0)
    }
    #[doc = "Bit 1 - DFRAME pin enable control bit"]
    #[inline(always)]
    pub fn dframepen(&mut self) -> DframepenW<'_, FrcRouteenSpec> {
        DframepenW::new(self, 1)
    }
    #[doc = "Bit 2 - DOUT pin enable control bit"]
    #[inline(always)]
    pub fn doutpen(&mut self) -> DoutpenW<'_, FrcRouteenSpec> {
        DoutpenW::new(self, 2)
    }
}
#[doc = "FRC pin enable\n\nYou can [`read`](crate::Reg::read) this register and get [`frc_routeen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frc_routeen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrcRouteenSpec;
impl crate::RegisterSpec for FrcRouteenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`frc_routeen::R`](R) reader structure"]
impl crate::Readable for FrcRouteenSpec {}
#[doc = "`write(|w| ..)` method takes [`frc_routeen::W`](W) writer structure"]
impl crate::Writable for FrcRouteenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FRC_ROUTEEN to value 0"]
impl crate::Resettable for FrcRouteenSpec {}
