#[doc = "Register `IF` reader"]
pub type R = crate::R<IfSpec>;
#[doc = "Register `IF` writer"]
pub type W = crate::W<IfSpec>;
#[doc = "Field `AHB0ERR1B` reader - AHB0 1-bit ECC Error Interrupt Flag"]
pub type Ahb0err1bR = crate::BitReader;
#[doc = "Field `AHB0ERR1B` writer - AHB0 1-bit ECC Error Interrupt Flag"]
pub type Ahb0err1bW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHB1ERR1B` reader - AHB1 1-bit ECC Error Interrupt Flag"]
pub type Ahb1err1bR = crate::BitReader;
#[doc = "Field `AHB1ERR1B` writer - AHB1 1-bit ECC Error Interrupt Flag"]
pub type Ahb1err1bW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHB2ERR1B` reader - AHB2 1-bit ECC Error Interrupt Flag"]
pub type Ahb2err1bR = crate::BitReader;
#[doc = "Field `AHB2ERR1B` writer - AHB2 1-bit ECC Error Interrupt Flag"]
pub type Ahb2err1bW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHB3ERR1B` reader - AHB3 1-bit ECC Error Interrupt Flag"]
pub type Ahb3err1bR = crate::BitReader;
#[doc = "Field `AHB3ERR1B` writer - AHB3 1-bit ECC Error Interrupt Flag"]
pub type Ahb3err1bW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHB0ERR2B` reader - AHB0 2-bit ECC Error Interrupt Flag"]
pub type Ahb0err2bR = crate::BitReader;
#[doc = "Field `AHB0ERR2B` writer - AHB0 2-bit ECC Error Interrupt Flag"]
pub type Ahb0err2bW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHB1ERR2B` reader - AHB1 2-bit ECC Error Interrupt Flag"]
pub type Ahb1err2bR = crate::BitReader;
#[doc = "Field `AHB1ERR2B` writer - AHB1 2-bit ECC Error Interrupt Flag"]
pub type Ahb1err2bW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHB2ERR2B` reader - AHB2 2-bit ECC Error Interrupt Flag"]
pub type Ahb2err2bR = crate::BitReader;
#[doc = "Field `AHB2ERR2B` writer - AHB2 2-bit ECC Error Interrupt Flag"]
pub type Ahb2err2bW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHB3ERR2B` reader - AHB3 2-bit ECC Error Interrupt Flag"]
pub type Ahb3err2bR = crate::BitReader;
#[doc = "Field `AHB3ERR2B` writer - AHB3 2-bit ECC Error Interrupt Flag"]
pub type Ahb3err2bW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - AHB0 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb0err1b(&self) -> Ahb0err1bR {
        Ahb0err1bR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - AHB1 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb1err1b(&self) -> Ahb1err1bR {
        Ahb1err1bR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - AHB2 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb2err1b(&self) -> Ahb2err1bR {
        Ahb2err1bR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - AHB3 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb3err1b(&self) -> Ahb3err1bR {
        Ahb3err1bR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - AHB0 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb0err2b(&self) -> Ahb0err2bR {
        Ahb0err2bR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - AHB1 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb1err2b(&self) -> Ahb1err2bR {
        Ahb1err2bR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - AHB2 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb2err2b(&self) -> Ahb2err2bR {
        Ahb2err2bR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - AHB3 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb3err2b(&self) -> Ahb3err2bR {
        Ahb3err2bR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - AHB0 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb0err1b(&mut self) -> Ahb0err1bW<'_, IfSpec> {
        Ahb0err1bW::new(self, 0)
    }
    #[doc = "Bit 1 - AHB1 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb1err1b(&mut self) -> Ahb1err1bW<'_, IfSpec> {
        Ahb1err1bW::new(self, 1)
    }
    #[doc = "Bit 2 - AHB2 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb2err1b(&mut self) -> Ahb2err1bW<'_, IfSpec> {
        Ahb2err1bW::new(self, 2)
    }
    #[doc = "Bit 3 - AHB3 1-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb3err1b(&mut self) -> Ahb3err1bW<'_, IfSpec> {
        Ahb3err1bW::new(self, 3)
    }
    #[doc = "Bit 4 - AHB0 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb0err2b(&mut self) -> Ahb0err2bW<'_, IfSpec> {
        Ahb0err2bW::new(self, 4)
    }
    #[doc = "Bit 5 - AHB1 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb1err2b(&mut self) -> Ahb1err2bW<'_, IfSpec> {
        Ahb1err2bW::new(self, 5)
    }
    #[doc = "Bit 6 - AHB2 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb2err2b(&mut self) -> Ahb2err2bW<'_, IfSpec> {
        Ahb2err2bW::new(self, 6)
    }
    #[doc = "Bit 7 - AHB3 2-bit ECC Error Interrupt Flag"]
    #[inline(always)]
    pub fn ahb3err2b(&mut self) -> Ahb3err2bW<'_, IfSpec> {
        Ahb3err2bW::new(self, 7)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`if_::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
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
