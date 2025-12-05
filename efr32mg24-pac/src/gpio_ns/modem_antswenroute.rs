#[doc = "Register `MODEM_ANTSWENROUTE` reader"]
pub type R = crate::R<ModemAntswenrouteSpec>;
#[doc = "Register `MODEM_ANTSWENROUTE` writer"]
pub type W = crate::W<ModemAntswenrouteSpec>;
#[doc = "Field `PORT` reader - ANTSWEN port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - ANTSWEN port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - ANTSWEN pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - ANTSWEN pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - ANTSWEN port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - ANTSWEN pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - ANTSWEN port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemAntswenrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - ANTSWEN pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemAntswenrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "ANTSWEN port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_antswenroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_antswenroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemAntswenrouteSpec;
impl crate::RegisterSpec for ModemAntswenrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_antswenroute::R`](R) reader structure"]
impl crate::Readable for ModemAntswenrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_antswenroute::W`](W) writer structure"]
impl crate::Writable for ModemAntswenrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_ANTSWENROUTE to value 0"]
impl crate::Resettable for ModemAntswenrouteSpec {}
