#[doc = "Register `IEN` reader"]
pub type R = crate::R<IenSpec>;
#[doc = "Register `IEN` writer"]
pub type W = crate::W<IenSpec>;
#[doc = "Field `RDY` reader - Ready Enable"]
pub type RdyR = crate::BitReader;
#[doc = "Field `RDY` writer - Ready Enable"]
pub type RdyW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `POSEDGE` reader - Rising Edge Enable"]
pub type PosedgeR = crate::BitReader;
#[doc = "Field `POSEDGE` writer - Rising Edge Enable"]
pub type PosedgeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `NEGEDGE` reader - Falling Edge Enable"]
pub type NegedgeR = crate::BitReader;
#[doc = "Field `NEGEDGE` writer - Falling Edge Enable"]
pub type NegedgeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TCDONE` reader - Temperature Check Done Enable"]
pub type TcdoneR = crate::BitReader;
#[doc = "Field `TCDONE` writer - Temperature Check Done Enable"]
pub type TcdoneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CALDONE` reader - Calibration Done Enable"]
pub type CaldoneR = crate::BitReader;
#[doc = "Field `CALDONE` writer - Calibration Done Enable"]
pub type CaldoneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TEMPCHANGE` reader - Temperature Change Enable"]
pub type TempchangeR = crate::BitReader;
#[doc = "Field `TEMPCHANGE` writer - Temperature Change Enable"]
pub type TempchangeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SCHEDERR` reader - Scheduling Error Enable"]
pub type SchederrR = crate::BitReader;
#[doc = "Field `SCHEDERR` writer - Scheduling Error Enable"]
pub type SchederrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TCOOR` reader - Temperature Check Out Of Range Enable"]
pub type TcoorR = crate::BitReader;
#[doc = "Field `TCOOR` writer - Temperature Check Out Of Range Enable"]
pub type TcoorW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CALOOR` reader - Calibration Out Of Range Enable"]
pub type CaloorR = crate::BitReader;
#[doc = "Field `CALOOR` writer - Calibration Out Of Range Enable"]
pub type CaloorW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Ready Enable"]
    #[inline(always)]
    pub fn rdy(&self) -> RdyR {
        RdyR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Rising Edge Enable"]
    #[inline(always)]
    pub fn posedge(&self) -> PosedgeR {
        PosedgeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Falling Edge Enable"]
    #[inline(always)]
    pub fn negedge(&self) -> NegedgeR {
        NegedgeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 8 - Temperature Check Done Enable"]
    #[inline(always)]
    pub fn tcdone(&self) -> TcdoneR {
        TcdoneR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Calibration Done Enable"]
    #[inline(always)]
    pub fn caldone(&self) -> CaldoneR {
        CaldoneR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Temperature Change Enable"]
    #[inline(always)]
    pub fn tempchange(&self) -> TempchangeR {
        TempchangeR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 16 - Scheduling Error Enable"]
    #[inline(always)]
    pub fn schederr(&self) -> SchederrR {
        SchederrR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Temperature Check Out Of Range Enable"]
    #[inline(always)]
    pub fn tcoor(&self) -> TcoorR {
        TcoorR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Calibration Out Of Range Enable"]
    #[inline(always)]
    pub fn caloor(&self) -> CaloorR {
        CaloorR::new(((self.bits >> 18) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Ready Enable"]
    #[inline(always)]
    pub fn rdy(&mut self) -> RdyW<'_, IenSpec> {
        RdyW::new(self, 0)
    }
    #[doc = "Bit 1 - Rising Edge Enable"]
    #[inline(always)]
    pub fn posedge(&mut self) -> PosedgeW<'_, IenSpec> {
        PosedgeW::new(self, 1)
    }
    #[doc = "Bit 2 - Falling Edge Enable"]
    #[inline(always)]
    pub fn negedge(&mut self) -> NegedgeW<'_, IenSpec> {
        NegedgeW::new(self, 2)
    }
    #[doc = "Bit 8 - Temperature Check Done Enable"]
    #[inline(always)]
    pub fn tcdone(&mut self) -> TcdoneW<'_, IenSpec> {
        TcdoneW::new(self, 8)
    }
    #[doc = "Bit 9 - Calibration Done Enable"]
    #[inline(always)]
    pub fn caldone(&mut self) -> CaldoneW<'_, IenSpec> {
        CaldoneW::new(self, 9)
    }
    #[doc = "Bit 10 - Temperature Change Enable"]
    #[inline(always)]
    pub fn tempchange(&mut self) -> TempchangeW<'_, IenSpec> {
        TempchangeW::new(self, 10)
    }
    #[doc = "Bit 16 - Scheduling Error Enable"]
    #[inline(always)]
    pub fn schederr(&mut self) -> SchederrW<'_, IenSpec> {
        SchederrW::new(self, 16)
    }
    #[doc = "Bit 17 - Temperature Check Out Of Range Enable"]
    #[inline(always)]
    pub fn tcoor(&mut self) -> TcoorW<'_, IenSpec> {
        TcoorW::new(self, 17)
    }
    #[doc = "Bit 18 - Calibration Out Of Range Enable"]
    #[inline(always)]
    pub fn caloor(&mut self) -> CaloorW<'_, IenSpec> {
        CaloorW::new(self, 18)
    }
}
#[doc = "Interrupt enable register\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IenSpec;
impl crate::RegisterSpec for IenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ien::R`](R) reader structure"]
impl crate::Readable for IenSpec {}
#[doc = "`write(|w| ..)` method takes [`ien::W`](W) writer structure"]
impl crate::Writable for IenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IEN to value 0"]
impl crate::Resettable for IenSpec {}
