#[doc = "Register `MODEM_DOUTROUTE` reader"]
pub type R = crate::R<ModemDoutrouteSpec>;
#[doc = "Register `MODEM_DOUTROUTE` writer"]
pub type W = crate::W<ModemDoutrouteSpec>;
#[doc = "Field `PORT` reader - DOUT port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - DOUT port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - DOUT pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - DOUT pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - DOUT port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - DOUT pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - DOUT port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemDoutrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - DOUT pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemDoutrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "DOUT port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_doutroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_doutroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemDoutrouteSpec;
impl crate::RegisterSpec for ModemDoutrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_doutroute::R`](R) reader structure"]
impl crate::Readable for ModemDoutrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_doutroute::W`](W) writer structure"]
impl crate::Writable for ModemDoutrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_DOUTROUTE to value 0"]
impl crate::Resettable for ModemDoutrouteSpec {}
