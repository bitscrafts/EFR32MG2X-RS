#[doc = "Register `INSTR5CFG1` reader"]
pub type R = crate::R<Instr5cfg1Spec>;
#[doc = "Register `INSTR5CFG1` writer"]
pub type W = crate::W<Instr5cfg1Spec>;
#[doc = "Field `ISTREAM0REGID` reader - Register ID"]
pub type Istream0regidR = crate::FieldReader;
#[doc = "Field `ISTREAM0REGID` writer - Register ID"]
pub type Istream0regidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `ISTREAM0LOAD` reader - Load register"]
pub type Istream0loadR = crate::BitReader;
#[doc = "Field `ISTREAM0LOAD` writer - Load register"]
pub type Istream0loadW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ISTREAM0ARRAYID` reader - Array ID"]
pub type Istream0arrayidR = crate::FieldReader;
#[doc = "Field `ISTREAM0ARRAYID` writer - Array ID"]
pub type Istream0arrayidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `ISTREAM0ARRAYINCRDIM0` reader - Increment Array Dimension 0"]
pub type Istream0arrayincrdim0R = crate::BitReader;
#[doc = "Field `ISTREAM0ARRAYINCRDIM0` writer - Increment Array Dimension 0"]
pub type Istream0arrayincrdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ISTREAM0ARRAYINCRDIM1` reader - Increment Array Dimension 1"]
pub type Istream0arrayincrdim1R = crate::BitReader;
#[doc = "Field `ISTREAM0ARRAYINCRDIM1` writer - Increment Array Dimension 1"]
pub type Istream0arrayincrdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ISTREAM0ARRAYINCRDIM2` reader - Increment Array Dimension 2"]
pub type Istream0arrayincrdim2R = crate::BitReader;
#[doc = "Field `ISTREAM0ARRAYINCRDIM2` writer - Increment Array Dimension 2"]
pub type Istream0arrayincrdim2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ISTREAM1REGID` reader - Register ID"]
pub type Istream1regidR = crate::FieldReader;
#[doc = "Field `ISTREAM1REGID` writer - Register ID"]
pub type Istream1regidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `ISTREAM1LOAD` reader - Load register"]
pub type Istream1loadR = crate::BitReader;
#[doc = "Field `ISTREAM1LOAD` writer - Load register"]
pub type Istream1loadW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ISTREAM1ARRAYID` reader - Array ID"]
pub type Istream1arrayidR = crate::FieldReader;
#[doc = "Field `ISTREAM1ARRAYID` writer - Array ID"]
pub type Istream1arrayidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `ISTREAM1ARRAYINCRDIM0` reader - Increment Array Dimension 0"]
pub type Istream1arrayincrdim0R = crate::BitReader;
#[doc = "Field `ISTREAM1ARRAYINCRDIM0` writer - Increment Array Dimension 0"]
pub type Istream1arrayincrdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ISTREAM1ARRAYINCRDIM1` reader - Increment Array Dimension 1"]
pub type Istream1arrayincrdim1R = crate::BitReader;
#[doc = "Field `ISTREAM1ARRAYINCRDIM1` writer - Increment Array Dimension 1"]
pub type Istream1arrayincrdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ISTREAM1ARRAYINCRDIM2` reader - Increment Array Dimension 2"]
pub type Istream1arrayincrdim2R = crate::BitReader;
#[doc = "Field `ISTREAM1ARRAYINCRDIM2` writer - Increment Array Dimension 2"]
pub type Istream1arrayincrdim2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `OSTREAMREGID` reader - Register ID"]
pub type OstreamregidR = crate::FieldReader;
#[doc = "Field `OSTREAMREGID` writer - Register ID"]
pub type OstreamregidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `OSTREAMSTORE` reader - Store to Register"]
pub type OstreamstoreR = crate::BitReader;
#[doc = "Field `OSTREAMSTORE` writer - Store to Register"]
pub type OstreamstoreW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `OSTREAMARRAYID` reader - Array ID"]
pub type OstreamarrayidR = crate::FieldReader;
#[doc = "Field `OSTREAMARRAYID` writer - Array ID"]
pub type OstreamarrayidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `OSTREAMARRAYINCRDIM0` reader - Increment Array Dimension 0"]
pub type Ostreamarrayincrdim0R = crate::BitReader;
#[doc = "Field `OSTREAMARRAYINCRDIM0` writer - Increment Array Dimension 0"]
pub type Ostreamarrayincrdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `OSTREAMARRAYINCRDIM1` reader - Increment Array Dimension 1"]
pub type Ostreamarrayincrdim1R = crate::BitReader;
#[doc = "Field `OSTREAMARRAYINCRDIM1` writer - Increment Array Dimension 1"]
pub type Ostreamarrayincrdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `OSTREAMARRAYINCRDIM2` reader - Increment Array Dimension 2"]
pub type Ostreamarrayincrdim2R = crate::BitReader;
#[doc = "Field `OSTREAMARRAYINCRDIM2` writer - Increment Array Dimension 2"]
pub type Ostreamarrayincrdim2W<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bits 0:2 - Register ID"]
    #[inline(always)]
    pub fn istream0regid(&self) -> Istream0regidR {
        Istream0regidR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 3 - Load register"]
    #[inline(always)]
    pub fn istream0load(&self) -> Istream0loadR {
        Istream0loadR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:6 - Array ID"]
    #[inline(always)]
    pub fn istream0arrayid(&self) -> Istream0arrayidR {
        Istream0arrayidR::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bit 7 - Increment Array Dimension 0"]
    #[inline(always)]
    pub fn istream0arrayincrdim0(&self) -> Istream0arrayincrdim0R {
        Istream0arrayincrdim0R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Increment Array Dimension 1"]
    #[inline(always)]
    pub fn istream0arrayincrdim1(&self) -> Istream0arrayincrdim1R {
        Istream0arrayincrdim1R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Increment Array Dimension 2"]
    #[inline(always)]
    pub fn istream0arrayincrdim2(&self) -> Istream0arrayincrdim2R {
        Istream0arrayincrdim2R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 10:12 - Register ID"]
    #[inline(always)]
    pub fn istream1regid(&self) -> Istream1regidR {
        Istream1regidR::new(((self.bits >> 10) & 7) as u8)
    }
    #[doc = "Bit 13 - Load register"]
    #[inline(always)]
    pub fn istream1load(&self) -> Istream1loadR {
        Istream1loadR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bits 14:16 - Array ID"]
    #[inline(always)]
    pub fn istream1arrayid(&self) -> Istream1arrayidR {
        Istream1arrayidR::new(((self.bits >> 14) & 7) as u8)
    }
    #[doc = "Bit 17 - Increment Array Dimension 0"]
    #[inline(always)]
    pub fn istream1arrayincrdim0(&self) -> Istream1arrayincrdim0R {
        Istream1arrayincrdim0R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Increment Array Dimension 1"]
    #[inline(always)]
    pub fn istream1arrayincrdim1(&self) -> Istream1arrayincrdim1R {
        Istream1arrayincrdim1R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Increment Array Dimension 2"]
    #[inline(always)]
    pub fn istream1arrayincrdim2(&self) -> Istream1arrayincrdim2R {
        Istream1arrayincrdim2R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bits 20:22 - Register ID"]
    #[inline(always)]
    pub fn ostreamregid(&self) -> OstreamregidR {
        OstreamregidR::new(((self.bits >> 20) & 7) as u8)
    }
    #[doc = "Bit 23 - Store to Register"]
    #[inline(always)]
    pub fn ostreamstore(&self) -> OstreamstoreR {
        OstreamstoreR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:26 - Array ID"]
    #[inline(always)]
    pub fn ostreamarrayid(&self) -> OstreamarrayidR {
        OstreamarrayidR::new(((self.bits >> 24) & 7) as u8)
    }
    #[doc = "Bit 27 - Increment Array Dimension 0"]
    #[inline(always)]
    pub fn ostreamarrayincrdim0(&self) -> Ostreamarrayincrdim0R {
        Ostreamarrayincrdim0R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Increment Array Dimension 1"]
    #[inline(always)]
    pub fn ostreamarrayincrdim1(&self) -> Ostreamarrayincrdim1R {
        Ostreamarrayincrdim1R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Increment Array Dimension 2"]
    #[inline(always)]
    pub fn ostreamarrayincrdim2(&self) -> Ostreamarrayincrdim2R {
        Ostreamarrayincrdim2R::new(((self.bits >> 29) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:2 - Register ID"]
    #[inline(always)]
    pub fn istream0regid(&mut self) -> Istream0regidW<'_, Instr5cfg1Spec> {
        Istream0regidW::new(self, 0)
    }
    #[doc = "Bit 3 - Load register"]
    #[inline(always)]
    pub fn istream0load(&mut self) -> Istream0loadW<'_, Instr5cfg1Spec> {
        Istream0loadW::new(self, 3)
    }
    #[doc = "Bits 4:6 - Array ID"]
    #[inline(always)]
    pub fn istream0arrayid(&mut self) -> Istream0arrayidW<'_, Instr5cfg1Spec> {
        Istream0arrayidW::new(self, 4)
    }
    #[doc = "Bit 7 - Increment Array Dimension 0"]
    #[inline(always)]
    pub fn istream0arrayincrdim0(&mut self) -> Istream0arrayincrdim0W<'_, Instr5cfg1Spec> {
        Istream0arrayincrdim0W::new(self, 7)
    }
    #[doc = "Bit 8 - Increment Array Dimension 1"]
    #[inline(always)]
    pub fn istream0arrayincrdim1(&mut self) -> Istream0arrayincrdim1W<'_, Instr5cfg1Spec> {
        Istream0arrayincrdim1W::new(self, 8)
    }
    #[doc = "Bit 9 - Increment Array Dimension 2"]
    #[inline(always)]
    pub fn istream0arrayincrdim2(&mut self) -> Istream0arrayincrdim2W<'_, Instr5cfg1Spec> {
        Istream0arrayincrdim2W::new(self, 9)
    }
    #[doc = "Bits 10:12 - Register ID"]
    #[inline(always)]
    pub fn istream1regid(&mut self) -> Istream1regidW<'_, Instr5cfg1Spec> {
        Istream1regidW::new(self, 10)
    }
    #[doc = "Bit 13 - Load register"]
    #[inline(always)]
    pub fn istream1load(&mut self) -> Istream1loadW<'_, Instr5cfg1Spec> {
        Istream1loadW::new(self, 13)
    }
    #[doc = "Bits 14:16 - Array ID"]
    #[inline(always)]
    pub fn istream1arrayid(&mut self) -> Istream1arrayidW<'_, Instr5cfg1Spec> {
        Istream1arrayidW::new(self, 14)
    }
    #[doc = "Bit 17 - Increment Array Dimension 0"]
    #[inline(always)]
    pub fn istream1arrayincrdim0(&mut self) -> Istream1arrayincrdim0W<'_, Instr5cfg1Spec> {
        Istream1arrayincrdim0W::new(self, 17)
    }
    #[doc = "Bit 18 - Increment Array Dimension 1"]
    #[inline(always)]
    pub fn istream1arrayincrdim1(&mut self) -> Istream1arrayincrdim1W<'_, Instr5cfg1Spec> {
        Istream1arrayincrdim1W::new(self, 18)
    }
    #[doc = "Bit 19 - Increment Array Dimension 2"]
    #[inline(always)]
    pub fn istream1arrayincrdim2(&mut self) -> Istream1arrayincrdim2W<'_, Instr5cfg1Spec> {
        Istream1arrayincrdim2W::new(self, 19)
    }
    #[doc = "Bits 20:22 - Register ID"]
    #[inline(always)]
    pub fn ostreamregid(&mut self) -> OstreamregidW<'_, Instr5cfg1Spec> {
        OstreamregidW::new(self, 20)
    }
    #[doc = "Bit 23 - Store to Register"]
    #[inline(always)]
    pub fn ostreamstore(&mut self) -> OstreamstoreW<'_, Instr5cfg1Spec> {
        OstreamstoreW::new(self, 23)
    }
    #[doc = "Bits 24:26 - Array ID"]
    #[inline(always)]
    pub fn ostreamarrayid(&mut self) -> OstreamarrayidW<'_, Instr5cfg1Spec> {
        OstreamarrayidW::new(self, 24)
    }
    #[doc = "Bit 27 - Increment Array Dimension 0"]
    #[inline(always)]
    pub fn ostreamarrayincrdim0(&mut self) -> Ostreamarrayincrdim0W<'_, Instr5cfg1Spec> {
        Ostreamarrayincrdim0W::new(self, 27)
    }
    #[doc = "Bit 28 - Increment Array Dimension 1"]
    #[inline(always)]
    pub fn ostreamarrayincrdim1(&mut self) -> Ostreamarrayincrdim1W<'_, Instr5cfg1Spec> {
        Ostreamarrayincrdim1W::new(self, 28)
    }
    #[doc = "Bit 29 - Increment Array Dimension 2"]
    #[inline(always)]
    pub fn ostreamarrayincrdim2(&mut self) -> Ostreamarrayincrdim2W<'_, Instr5cfg1Spec> {
        Ostreamarrayincrdim2W::new(self, 29)
    }
}
#[doc = "Instruction N word 1\n\nYou can [`read`](crate::Reg::read) this register and get [`instr5cfg1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr5cfg1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Instr5cfg1Spec;
impl crate::RegisterSpec for Instr5cfg1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`instr5cfg1::R`](R) reader structure"]
impl crate::Readable for Instr5cfg1Spec {}
#[doc = "`write(|w| ..)` method takes [`instr5cfg1::W`](W) writer structure"]
impl crate::Writable for Instr5cfg1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets INSTR5CFG1 to value 0"]
impl crate::Resettable for Instr5cfg1Spec {}
