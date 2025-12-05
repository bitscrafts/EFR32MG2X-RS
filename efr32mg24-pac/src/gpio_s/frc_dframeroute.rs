#[doc = "Register `FRC_DFRAMEROUTE` reader"]
pub type R = crate::R<FrcDframerouteSpec>;
#[doc = "Register `FRC_DFRAMEROUTE` writer"]
pub type W = crate::W<FrcDframerouteSpec>;
#[doc = "Field `PORT` reader - DFRAME port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - DFRAME port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - DFRAME pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - DFRAME pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - DFRAME port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - DFRAME pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - DFRAME port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, FrcDframerouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - DFRAME pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, FrcDframerouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "DFRAME port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`frc_dframeroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frc_dframeroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrcDframerouteSpec;
impl crate::RegisterSpec for FrcDframerouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`frc_dframeroute::R`](R) reader structure"]
impl crate::Readable for FrcDframerouteSpec {}
#[doc = "`write(|w| ..)` method takes [`frc_dframeroute::W`](W) writer structure"]
impl crate::Writable for FrcDframerouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FRC_DFRAMEROUTE to value 0"]
impl crate::Resettable for FrcDframerouteSpec {}
