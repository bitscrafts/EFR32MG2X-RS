#[doc = "Register `MODEM_DINROUTE` reader"]
pub type R = crate::R<ModemDinrouteSpec>;
#[doc = "Register `MODEM_DINROUTE` writer"]
pub type W = crate::W<ModemDinrouteSpec>;
#[doc = "Field `PORT` reader - DIN port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - DIN port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - DIN pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - DIN pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - DIN port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - DIN pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - DIN port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemDinrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - DIN pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemDinrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "DIN port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_dinroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_dinroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemDinrouteSpec;
impl crate::RegisterSpec for ModemDinrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_dinroute::R`](R) reader structure"]
impl crate::Readable for ModemDinrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_dinroute::W`](W) writer structure"]
impl crate::Writable for ModemDinrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_DINROUTE to value 0"]
impl crate::Resettable for ModemDinrouteSpec {}
