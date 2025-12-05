#[doc = "Register `RADIOCLKCTRL` reader"]
pub type R = crate::R<RadioclkctrlSpec>;
#[doc = "Register `RADIOCLKCTRL` writer"]
pub type W = crate::W<RadioclkctrlSpec>;
#[doc = "Field `EN` reader - Enable"]
pub type EnR = crate::BitReader;
#[doc = "Field `EN` writer - Enable"]
pub type EnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DBGCLK` reader - Enable Clock for Debugger"]
pub type DbgclkR = crate::BitReader;
#[doc = "Field `DBGCLK` writer - Enable Clock for Debugger"]
pub type DbgclkW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Enable"]
    #[inline(always)]
    pub fn en(&self) -> EnR {
        EnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 31 - Enable Clock for Debugger"]
    #[inline(always)]
    pub fn dbgclk(&self) -> DbgclkR {
        DbgclkR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable"]
    #[inline(always)]
    pub fn en(&mut self) -> EnW<'_, RadioclkctrlSpec> {
        EnW::new(self, 0)
    }
    #[doc = "Bit 31 - Enable Clock for Debugger"]
    #[inline(always)]
    pub fn dbgclk(&mut self) -> DbgclkW<'_, RadioclkctrlSpec> {
        DbgclkW::new(self, 31)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`radioclkctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`radioclkctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RadioclkctrlSpec;
impl crate::RegisterSpec for RadioclkctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`radioclkctrl::R`](R) reader structure"]
impl crate::Readable for RadioclkctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`radioclkctrl::W`](W) writer structure"]
impl crate::Writable for RadioclkctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RADIOCLKCTRL to value 0"]
impl crate::Resettable for RadioclkctrlSpec {}
