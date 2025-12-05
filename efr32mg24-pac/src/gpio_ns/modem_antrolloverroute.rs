#[doc = "Register `MODEM_ANTROLLOVERROUTE` reader"]
pub type R = crate::R<ModemAntrolloverrouteSpec>;
#[doc = "Register `MODEM_ANTROLLOVERROUTE` writer"]
pub type W = crate::W<ModemAntrolloverrouteSpec>;
#[doc = "Field `PORT` reader - ANTROLLOVER port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - ANTROLLOVER port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - ANTROLLOVER pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - ANTROLLOVER pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - ANTROLLOVER port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - ANTROLLOVER pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - ANTROLLOVER port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemAntrolloverrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - ANTROLLOVER pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemAntrolloverrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "ANTROLLOVER port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_antrolloverroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_antrolloverroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemAntrolloverrouteSpec;
impl crate::RegisterSpec for ModemAntrolloverrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_antrolloverroute::R`](R) reader structure"]
impl crate::Readable for ModemAntrolloverrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_antrolloverroute::W`](W) writer structure"]
impl crate::Writable for ModemAntrolloverrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_ANTROLLOVERROUTE to value 0"]
impl crate::Resettable for ModemAntrolloverrouteSpec {}
