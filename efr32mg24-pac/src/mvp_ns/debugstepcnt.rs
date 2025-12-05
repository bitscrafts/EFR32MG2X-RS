#[doc = "Register `DEBUGSTEPCNT` reader"]
pub type R = crate::R<DebugstepcntSpec>;
#[doc = "Register `DEBUGSTEPCNT` writer"]
pub type W = crate::W<DebugstepcntSpec>;
#[doc = "Field `DEBUGSTEPCNT` reader - Debug Step Counter"]
pub type DebugstepcntR = crate::FieldReader<u32>;
#[doc = "Field `DEBUGSTEPCNT` writer - Debug Step Counter"]
pub type DebugstepcntW<'a, REG> = crate::FieldWriter<'a, REG, 24, u32>;
impl R {
    #[doc = "Bits 0:23 - Debug Step Counter"]
    #[inline(always)]
    pub fn debugstepcnt(&self) -> DebugstepcntR {
        DebugstepcntR::new(self.bits & 0x00ff_ffff)
    }
}
impl W {
    #[doc = "Bits 0:23 - Debug Step Counter"]
    #[inline(always)]
    pub fn debugstepcnt(&mut self) -> DebugstepcntW<'_, DebugstepcntSpec> {
        DebugstepcntW::new(self, 0)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`debugstepcnt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debugstepcnt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DebugstepcntSpec;
impl crate::RegisterSpec for DebugstepcntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`debugstepcnt::R`](R) reader structure"]
impl crate::Readable for DebugstepcntSpec {}
#[doc = "`write(|w| ..)` method takes [`debugstepcnt::W`](W) writer structure"]
impl crate::Writable for DebugstepcntSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DEBUGSTEPCNT to value 0"]
impl crate::Resettable for DebugstepcntSpec {}
