#[doc = "Register `IF` reader"]
pub type R = crate::R<IfSpec>;
#[doc = "Register `IF` writer"]
pub type W = crate::W<IfSpec>;
#[doc = "Field `RDY` reader - Ready Flag"]
pub type RdyR = crate::BitReader;
#[doc = "Field `RDY` writer - Ready Flag"]
pub type RdyW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `POSEDGE` reader - Rising Edge Flag"]
pub type PosedgeR = crate::BitReader;
#[doc = "Field `POSEDGE` writer - Rising Edge Flag"]
pub type PosedgeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `NEGEDGE` reader - Falling Edge Flag"]
pub type NegedgeR = crate::BitReader;
#[doc = "Field `NEGEDGE` writer - Falling Edge Flag"]
pub type NegedgeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TCDONE` reader - Temperature Check Done Flag"]
pub type TcdoneR = crate::BitReader;
#[doc = "Field `TCDONE` writer - Temperature Check Done Flag"]
pub type TcdoneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CALDONE` reader - Calibration Done Flag"]
pub type CaldoneR = crate::BitReader;
#[doc = "Field `CALDONE` writer - Calibration Done Flag"]
pub type CaldoneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TEMPCHANGE` reader - Temperature Change Flag"]
pub type TempchangeR = crate::BitReader;
#[doc = "Field `TEMPCHANGE` writer - Temperature Change Flag"]
pub type TempchangeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SCHEDERR` reader - Scheduling Error Flag"]
pub type SchederrR = crate::BitReader;
#[doc = "Field `SCHEDERR` writer - Scheduling Error Flag"]
pub type SchederrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TCOOR` reader - Temperature Check Out Of Range Flag"]
pub type TcoorR = crate::BitReader;
#[doc = "Field `TCOOR` writer - Temperature Check Out Of Range Flag"]
pub type TcoorW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CALOOR` reader - Calibration Out Of Range Flag"]
pub type CaloorR = crate::BitReader;
#[doc = "Field `CALOOR` writer - Calibration Out Of Range Flag"]
pub type CaloorW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Ready Flag"]
    #[inline(always)]
    pub fn rdy(&self) -> RdyR {
        RdyR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Rising Edge Flag"]
    #[inline(always)]
    pub fn posedge(&self) -> PosedgeR {
        PosedgeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Falling Edge Flag"]
    #[inline(always)]
    pub fn negedge(&self) -> NegedgeR {
        NegedgeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 8 - Temperature Check Done Flag"]
    #[inline(always)]
    pub fn tcdone(&self) -> TcdoneR {
        TcdoneR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Calibration Done Flag"]
    #[inline(always)]
    pub fn caldone(&self) -> CaldoneR {
        CaldoneR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Temperature Change Flag"]
    #[inline(always)]
    pub fn tempchange(&self) -> TempchangeR {
        TempchangeR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 16 - Scheduling Error Flag"]
    #[inline(always)]
    pub fn schederr(&self) -> SchederrR {
        SchederrR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Temperature Check Out Of Range Flag"]
    #[inline(always)]
    pub fn tcoor(&self) -> TcoorR {
        TcoorR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Calibration Out Of Range Flag"]
    #[inline(always)]
    pub fn caloor(&self) -> CaloorR {
        CaloorR::new(((self.bits >> 18) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Ready Flag"]
    #[inline(always)]
    pub fn rdy(&mut self) -> RdyW<'_, IfSpec> {
        RdyW::new(self, 0)
    }
    #[doc = "Bit 1 - Rising Edge Flag"]
    #[inline(always)]
    pub fn posedge(&mut self) -> PosedgeW<'_, IfSpec> {
        PosedgeW::new(self, 1)
    }
    #[doc = "Bit 2 - Falling Edge Flag"]
    #[inline(always)]
    pub fn negedge(&mut self) -> NegedgeW<'_, IfSpec> {
        NegedgeW::new(self, 2)
    }
    #[doc = "Bit 8 - Temperature Check Done Flag"]
    #[inline(always)]
    pub fn tcdone(&mut self) -> TcdoneW<'_, IfSpec> {
        TcdoneW::new(self, 8)
    }
    #[doc = "Bit 9 - Calibration Done Flag"]
    #[inline(always)]
    pub fn caldone(&mut self) -> CaldoneW<'_, IfSpec> {
        CaldoneW::new(self, 9)
    }
    #[doc = "Bit 10 - Temperature Change Flag"]
    #[inline(always)]
    pub fn tempchange(&mut self) -> TempchangeW<'_, IfSpec> {
        TempchangeW::new(self, 10)
    }
    #[doc = "Bit 16 - Scheduling Error Flag"]
    #[inline(always)]
    pub fn schederr(&mut self) -> SchederrW<'_, IfSpec> {
        SchederrW::new(self, 16)
    }
    #[doc = "Bit 17 - Temperature Check Out Of Range Flag"]
    #[inline(always)]
    pub fn tcoor(&mut self) -> TcoorW<'_, IfSpec> {
        TcoorW::new(self, 17)
    }
    #[doc = "Bit 18 - Calibration Out Of Range Flag"]
    #[inline(always)]
    pub fn caloor(&mut self) -> CaloorW<'_, IfSpec> {
        CaloorW::new(self, 18)
    }
}
#[doc = "Interrupt flag register\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IfSpec;
impl crate::RegisterSpec for IfSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`if_::R`](R) reader structure"]
impl crate::Readable for IfSpec {}
#[doc = "`write(|w| ..)` method takes [`if_::W`](W) writer structure"]
impl crate::Writable for IfSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IF to value 0"]
impl crate::Resettable for IfSpec {}
