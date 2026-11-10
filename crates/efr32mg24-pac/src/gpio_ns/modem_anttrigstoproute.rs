#[doc = "Register `MODEM_ANTTRIGSTOPROUTE` reader"]
pub type R = crate::R<ModemAnttrigstoprouteSpec>;
#[doc = "Register `MODEM_ANTTRIGSTOPROUTE` writer"]
pub type W = crate::W<ModemAnttrigstoprouteSpec>;
#[doc = "Field `PORT` reader - ANTTRIGSTOP port select register"]
pub type PortR = crate::FieldReader;
#[doc = "Field `PORT` writer - ANTTRIGSTOP port select register"]
pub type PortW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `PIN` reader - ANTTRIGSTOP pin select register"]
pub type PinR = crate::FieldReader;
#[doc = "Field `PIN` writer - ANTTRIGSTOP pin select register"]
pub type PinW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - ANTTRIGSTOP port select register"]
    #[inline(always)]
    pub fn port(&self) -> PortR {
        PortR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:19 - ANTTRIGSTOP pin select register"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - ANTTRIGSTOP port select register"]
    #[inline(always)]
    pub fn port(&mut self) -> PortW<'_, ModemAnttrigstoprouteSpec> {
        PortW::new(self, 0)
    }
    #[doc = "Bits 16:19 - ANTTRIGSTOP pin select register"]
    #[inline(always)]
    pub fn pin(&mut self) -> PinW<'_, ModemAnttrigstoprouteSpec> {
        PinW::new(self, 16)
    }
}
#[doc = "ANTTRIGSTOP port/pin select\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_anttrigstoproute::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_anttrigstoproute::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemAnttrigstoprouteSpec;
impl crate::RegisterSpec for ModemAnttrigstoprouteSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_anttrigstoproute::R`](R) reader structure"]
impl crate::Readable for ModemAnttrigstoprouteSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_anttrigstoproute::W`](W) writer structure"]
impl crate::Writable for ModemAnttrigstoprouteSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_ANTTRIGSTOPROUTE to value 0"]
impl crate::Resettable for ModemAnttrigstoprouteSpec {}
