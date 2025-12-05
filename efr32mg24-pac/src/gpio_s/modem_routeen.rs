#[doc = "Register `MODEM_ROUTEEN` reader"]
pub type R = crate::R<ModemRouteenSpec>;
#[doc = "Register `MODEM_ROUTEEN` writer"]
pub type W = crate::W<ModemRouteenSpec>;
#[doc = "Field `ANT0PEN` reader - ANT0 pin enable control bit"]
pub type Ant0penR = crate::BitReader;
#[doc = "Field `ANT0PEN` writer - ANT0 pin enable control bit"]
pub type Ant0penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANT1PEN` reader - ANT1 pin enable control bit"]
pub type Ant1penR = crate::BitReader;
#[doc = "Field `ANT1PEN` writer - ANT1 pin enable control bit"]
pub type Ant1penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTROLLOVERPEN` reader - ANTROLLOVER pin enable control bit"]
pub type AntrolloverpenR = crate::BitReader;
#[doc = "Field `ANTROLLOVERPEN` writer - ANTROLLOVER pin enable control bit"]
pub type AntrolloverpenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTRR0PEN` reader - ANTRR0 pin enable control bit"]
pub type Antrr0penR = crate::BitReader;
#[doc = "Field `ANTRR0PEN` writer - ANTRR0 pin enable control bit"]
pub type Antrr0penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTRR1PEN` reader - ANTRR1 pin enable control bit"]
pub type Antrr1penR = crate::BitReader;
#[doc = "Field `ANTRR1PEN` writer - ANTRR1 pin enable control bit"]
pub type Antrr1penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTRR2PEN` reader - ANTRR2 pin enable control bit"]
pub type Antrr2penR = crate::BitReader;
#[doc = "Field `ANTRR2PEN` writer - ANTRR2 pin enable control bit"]
pub type Antrr2penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTRR3PEN` reader - ANTRR3 pin enable control bit"]
pub type Antrr3penR = crate::BitReader;
#[doc = "Field `ANTRR3PEN` writer - ANTRR3 pin enable control bit"]
pub type Antrr3penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTRR4PEN` reader - ANTRR4 pin enable control bit"]
pub type Antrr4penR = crate::BitReader;
#[doc = "Field `ANTRR4PEN` writer - ANTRR4 pin enable control bit"]
pub type Antrr4penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTRR5PEN` reader - ANTRR5 pin enable control bit"]
pub type Antrr5penR = crate::BitReader;
#[doc = "Field `ANTRR5PEN` writer - ANTRR5 pin enable control bit"]
pub type Antrr5penW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTSWENPEN` reader - ANTSWEN pin enable control bit"]
pub type AntswenpenR = crate::BitReader;
#[doc = "Field `ANTSWENPEN` writer - ANTSWEN pin enable control bit"]
pub type AntswenpenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTSWUSPEN` reader - ANTSWUS pin enable control bit"]
pub type AntswuspenR = crate::BitReader;
#[doc = "Field `ANTSWUSPEN` writer - ANTSWUS pin enable control bit"]
pub type AntswuspenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTTRIGPEN` reader - ANTTRIG pin enable control bit"]
pub type AnttrigpenR = crate::BitReader;
#[doc = "Field `ANTTRIGPEN` writer - ANTTRIG pin enable control bit"]
pub type AnttrigpenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ANTTRIGSTOPPEN` reader - ANTTRIGSTOP pin enable control bit"]
pub type AnttrigstoppenR = crate::BitReader;
#[doc = "Field `ANTTRIGSTOPPEN` writer - ANTTRIGSTOP pin enable control bit"]
pub type AnttrigstoppenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DCLKPEN` reader - DCLK pin enable control bit"]
pub type DclkpenR = crate::BitReader;
#[doc = "Field `DCLKPEN` writer - DCLK pin enable control bit"]
pub type DclkpenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DOUTPEN` reader - DOUT pin enable control bit"]
pub type DoutpenR = crate::BitReader;
#[doc = "Field `DOUTPEN` writer - DOUT pin enable control bit"]
pub type DoutpenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - ANT0 pin enable control bit"]
    #[inline(always)]
    pub fn ant0pen(&self) -> Ant0penR {
        Ant0penR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - ANT1 pin enable control bit"]
    #[inline(always)]
    pub fn ant1pen(&self) -> Ant1penR {
        Ant1penR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - ANTROLLOVER pin enable control bit"]
    #[inline(always)]
    pub fn antrolloverpen(&self) -> AntrolloverpenR {
        AntrolloverpenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - ANTRR0 pin enable control bit"]
    #[inline(always)]
    pub fn antrr0pen(&self) -> Antrr0penR {
        Antrr0penR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - ANTRR1 pin enable control bit"]
    #[inline(always)]
    pub fn antrr1pen(&self) -> Antrr1penR {
        Antrr1penR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - ANTRR2 pin enable control bit"]
    #[inline(always)]
    pub fn antrr2pen(&self) -> Antrr2penR {
        Antrr2penR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - ANTRR3 pin enable control bit"]
    #[inline(always)]
    pub fn antrr3pen(&self) -> Antrr3penR {
        Antrr3penR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - ANTRR4 pin enable control bit"]
    #[inline(always)]
    pub fn antrr4pen(&self) -> Antrr4penR {
        Antrr4penR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - ANTRR5 pin enable control bit"]
    #[inline(always)]
    pub fn antrr5pen(&self) -> Antrr5penR {
        Antrr5penR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - ANTSWEN pin enable control bit"]
    #[inline(always)]
    pub fn antswenpen(&self) -> AntswenpenR {
        AntswenpenR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - ANTSWUS pin enable control bit"]
    #[inline(always)]
    pub fn antswuspen(&self) -> AntswuspenR {
        AntswuspenR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - ANTTRIG pin enable control bit"]
    #[inline(always)]
    pub fn anttrigpen(&self) -> AnttrigpenR {
        AnttrigpenR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - ANTTRIGSTOP pin enable control bit"]
    #[inline(always)]
    pub fn anttrigstoppen(&self) -> AnttrigstoppenR {
        AnttrigstoppenR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - DCLK pin enable control bit"]
    #[inline(always)]
    pub fn dclkpen(&self) -> DclkpenR {
        DclkpenR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - DOUT pin enable control bit"]
    #[inline(always)]
    pub fn doutpen(&self) -> DoutpenR {
        DoutpenR::new(((self.bits >> 14) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - ANT0 pin enable control bit"]
    #[inline(always)]
    pub fn ant0pen(&mut self) -> Ant0penW<'_, ModemRouteenSpec> {
        Ant0penW::new(self, 0)
    }
    #[doc = "Bit 1 - ANT1 pin enable control bit"]
    #[inline(always)]
    pub fn ant1pen(&mut self) -> Ant1penW<'_, ModemRouteenSpec> {
        Ant1penW::new(self, 1)
    }
    #[doc = "Bit 2 - ANTROLLOVER pin enable control bit"]
    #[inline(always)]
    pub fn antrolloverpen(&mut self) -> AntrolloverpenW<'_, ModemRouteenSpec> {
        AntrolloverpenW::new(self, 2)
    }
    #[doc = "Bit 3 - ANTRR0 pin enable control bit"]
    #[inline(always)]
    pub fn antrr0pen(&mut self) -> Antrr0penW<'_, ModemRouteenSpec> {
        Antrr0penW::new(self, 3)
    }
    #[doc = "Bit 4 - ANTRR1 pin enable control bit"]
    #[inline(always)]
    pub fn antrr1pen(&mut self) -> Antrr1penW<'_, ModemRouteenSpec> {
        Antrr1penW::new(self, 4)
    }
    #[doc = "Bit 5 - ANTRR2 pin enable control bit"]
    #[inline(always)]
    pub fn antrr2pen(&mut self) -> Antrr2penW<'_, ModemRouteenSpec> {
        Antrr2penW::new(self, 5)
    }
    #[doc = "Bit 6 - ANTRR3 pin enable control bit"]
    #[inline(always)]
    pub fn antrr3pen(&mut self) -> Antrr3penW<'_, ModemRouteenSpec> {
        Antrr3penW::new(self, 6)
    }
    #[doc = "Bit 7 - ANTRR4 pin enable control bit"]
    #[inline(always)]
    pub fn antrr4pen(&mut self) -> Antrr4penW<'_, ModemRouteenSpec> {
        Antrr4penW::new(self, 7)
    }
    #[doc = "Bit 8 - ANTRR5 pin enable control bit"]
    #[inline(always)]
    pub fn antrr5pen(&mut self) -> Antrr5penW<'_, ModemRouteenSpec> {
        Antrr5penW::new(self, 8)
    }
    #[doc = "Bit 9 - ANTSWEN pin enable control bit"]
    #[inline(always)]
    pub fn antswenpen(&mut self) -> AntswenpenW<'_, ModemRouteenSpec> {
        AntswenpenW::new(self, 9)
    }
    #[doc = "Bit 10 - ANTSWUS pin enable control bit"]
    #[inline(always)]
    pub fn antswuspen(&mut self) -> AntswuspenW<'_, ModemRouteenSpec> {
        AntswuspenW::new(self, 10)
    }
    #[doc = "Bit 11 - ANTTRIG pin enable control bit"]
    #[inline(always)]
    pub fn anttrigpen(&mut self) -> AnttrigpenW<'_, ModemRouteenSpec> {
        AnttrigpenW::new(self, 11)
    }
    #[doc = "Bit 12 - ANTTRIGSTOP pin enable control bit"]
    #[inline(always)]
    pub fn anttrigstoppen(&mut self) -> AnttrigstoppenW<'_, ModemRouteenSpec> {
        AnttrigstoppenW::new(self, 12)
    }
    #[doc = "Bit 13 - DCLK pin enable control bit"]
    #[inline(always)]
    pub fn dclkpen(&mut self) -> DclkpenW<'_, ModemRouteenSpec> {
        DclkpenW::new(self, 13)
    }
    #[doc = "Bit 14 - DOUT pin enable control bit"]
    #[inline(always)]
    pub fn doutpen(&mut self) -> DoutpenW<'_, ModemRouteenSpec> {
        DoutpenW::new(self, 14)
    }
}
#[doc = "MODEM pin enable\n\nYou can [`read`](crate::Reg::read) this register and get [`modem_routeen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`modem_routeen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModemRouteenSpec;
impl crate::RegisterSpec for ModemRouteenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`modem_routeen::R`](R) reader structure"]
impl crate::Readable for ModemRouteenSpec {}
#[doc = "`write(|w| ..)` method takes [`modem_routeen::W`](W) writer structure"]
impl crate::Writable for ModemRouteenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MODEM_ROUTEEN to value 0"]
impl crate::Resettable for ModemRouteenSpec {}
