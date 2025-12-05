#[doc = "Register `MODEM_ANTTRIGROUTE` reader"]
pub type R = crate::R<ModemAnttrigrouteSpec>;
#[doc = "Register `MODEM_ANTTRIGROUTE` writer"]
pub type W = crate::W<ModemAnttrigrouteSpec>;
#[doc = "Field `PORT` reader - ANTTRIG port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - ANTTRIG port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - ANTTRIG pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - ANTTRIG pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - ANTTRIG port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - ANTTRIG pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - ANTTRIG port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemAnttrigrouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - ANTTRIG pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemAnttrigrouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "ANTTRIG port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_anttrigroute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_anttrigroute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemAnttrigrouteSpec;
impl crate::RegisterSpec for ModemAnttrigrouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_anttrigroute::R`](R) reader structure"]
impl crate::Readable for ModemAnttrigrouteSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_anttrigroute::W`](W) writer structure"]
impl crate::Writable for ModemAnttrigrouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_ANTTRIGROUTE to value 0"]
impl crate::Resettable for ModemAnttrigrouteSpec {}
