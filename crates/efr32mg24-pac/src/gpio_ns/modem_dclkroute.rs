#[doc = "Register `MODEM_DCLKROUTE` reader"]
pub type R = crate::R<ModemDclkrouteSpec>;
#[doc = "Register `MODEM_DCLKROUTE` writer"]
pub type W = crate::W<ModemDclkrouteSpec>;
#[doc = "Field `PORT` reader - DCLK port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - DCLK port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - DCLK pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - DCLK pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - DCLK port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - DCLK pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - DCLK port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemDclkrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - DCLK pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemDclkrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "DCLK port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_dclkroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_dclkroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemDclkrouteSpec;
impl crate::RegisterSpec for ModemDclkrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_dclkroute::R`](R) reader structure"]
impl crate::Readable for ModemDclkrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_dclkroute::W`](W) writer structure"]
impl crate::Writable for ModemDclkrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_DCLKROUTE to value 0"]
impl crate::Resettable for ModemDclkrouteSpec {}
