#[doc = "Register `MODEM_ANTRR3ROUTE` reader"]
pub type R = crate::R<ModemAntrr3routeSpec>;
#[doc = "Register `MODEM_ANTRR3ROUTE` writer"]
pub type W = crate::W<ModemAntrr3routeSpec>;
#[doc = "Field `PORT` reader - ANTRR3 port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - ANTRR3 port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - ANTRR3 pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - ANTRR3 pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - ANTRR3 port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - ANTRR3 pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - ANTRR3 port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemAntrr3routeSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - ANTRR3 pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemAntrr3routeSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "ANTRR3 port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_antrr3route::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_antrr3route::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemAntrr3routeSpec;
impl crate::RegisterSpec for ModemAntrr3routeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_antrr3route::R`](R) reader structure"]
impl crate::Readable for ModemAntrr3routeSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_antrr3route::W`](W) writer structure"]
impl crate::Writable for ModemAntrr3routeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_ANTRR3ROUTE to value 0"]
impl crate::Resettable for ModemAntrr3routeSpec {}
