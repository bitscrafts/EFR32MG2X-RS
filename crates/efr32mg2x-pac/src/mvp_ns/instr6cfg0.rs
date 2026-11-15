#[doc = "Register `INSTR6CFG0` reader"]
pub type R = crate::R<Instr6cfg0Spec>;
#[doc = "Register `INSTR6CFG0` writer"]
pub type W = crate::W<Instr6cfg0Spec>;
#[doc = "Field `ALUIN0REGID` reader - Register ID"]
pub type Aluin0regidR = crate::FieldReader;
#[doc = "Field `ALUIN0REGID` writer - Register ID"]
pub type Aluin0regidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `ALUIN0REALZERO` reader - Real Zero"]
pub type Aluin0realzeroR = crate::BitReader;
#[doc = "Field `ALUIN0REALZERO` writer - Real Zero"]
pub type Aluin0realzeroW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN0REALNEGATE` reader - Real Negate"]
pub type Aluin0realnegateR = crate::BitReader;
#[doc = "Field `ALUIN0REALNEGATE` writer - Real Negate"]
pub type Aluin0realnegateW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN0IMAGZERO` reader - Imaginary Not Zero"]
pub type Aluin0imagzeroR = crate::BitReader;
#[doc = "Field `ALUIN0IMAGZERO` writer - Imaginary Not Zero"]
pub type Aluin0imagzeroW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN0IMAGNEGATE` reader - Imaginary Negate"]
pub type Aluin0imagnegateR = crate::BitReader;
#[doc = "Field `ALUIN0IMAGNEGATE` writer - Imaginary Negate"]
pub type Aluin0imagnegateW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN1REGID` reader - Register ID"]
pub type Aluin1regidR = crate::FieldReader;
#[doc = "Field `ALUIN1REGID` writer - Register ID"]
pub type Aluin1regidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `ALUIN1REALZERO` reader - Real Zero"]
pub type Aluin1realzeroR = crate::BitReader;
#[doc = "Field `ALUIN1REALZERO` writer - Real Zero"]
pub type Aluin1realzeroW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN1REALNEGATE` reader - Real Negate"]
pub type Aluin1realnegateR = crate::BitReader;
#[doc = "Field `ALUIN1REALNEGATE` writer - Real Negate"]
pub type Aluin1realnegateW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN1IMAGZERO` reader - Imaginary Not Zero"]
pub type Aluin1imagzeroR = crate::BitReader;
#[doc = "Field `ALUIN1IMAGZERO` writer - Imaginary Not Zero"]
pub type Aluin1imagzeroW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN1IMAGNEGATE` reader - Imaginary Negate"]
pub type Aluin1imagnegateR = crate::BitReader;
#[doc = "Field `ALUIN1IMAGNEGATE` writer - Imaginary Negate"]
pub type Aluin1imagnegateW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN2REGID` reader - Register ID"]
pub type Aluin2regidR = crate::FieldReader;
#[doc = "Field `ALUIN2REGID` writer - Register ID"]
pub type Aluin2regidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `ALUIN2REALZERO` reader - Real Zero"]
pub type Aluin2realzeroR = crate::BitReader;
#[doc = "Field `ALUIN2REALZERO` writer - Real Zero"]
pub type Aluin2realzeroW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN2REALNEGATE` reader - Real Negate"]
pub type Aluin2realnegateR = crate::BitReader;
#[doc = "Field `ALUIN2REALNEGATE` writer - Real Negate"]
pub type Aluin2realnegateW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN2IMAGZERO` reader - Imaginary Not Zero"]
pub type Aluin2imagzeroR = crate::BitReader;
#[doc = "Field `ALUIN2IMAGZERO` writer - Imaginary Not Zero"]
pub type Aluin2imagzeroW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUIN2IMAGNEGATE` reader - Imaginary Negate"]
pub type Aluin2imagnegateR = crate::BitReader;
#[doc = "Field `ALUIN2IMAGNEGATE` writer - Imaginary Negate"]
pub type Aluin2imagnegateW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUOUTREGID` reader - Register ID"]
pub type AluoutregidR = crate::FieldReader;
#[doc = "Field `ALUOUTREGID` writer - Register ID"]
pub type AluoutregidW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:2 - Register ID"]
    #[inline(always)]
    pub fn aluin0regid(&self) -> Aluin0regidR {
        Aluin0regidR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 4 - Real Zero"]
    #[inline(always)]
    pub fn aluin0realzero(&self) -> Aluin0realzeroR {
        Aluin0realzeroR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Real Negate"]
    #[inline(always)]
    pub fn aluin0realnegate(&self) -> Aluin0realnegateR {
        Aluin0realnegateR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Imaginary Not Zero"]
    #[inline(always)]
    pub fn aluin0imagzero(&self) -> Aluin0imagzeroR {
        Aluin0imagzeroR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Imaginary Negate"]
    #[inline(always)]
    pub fn aluin0imagnegate(&self) -> Aluin0imagnegateR {
        Aluin0imagnegateR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:10 - Register ID"]
    #[inline(always)]
    pub fn aluin1regid(&self) -> Aluin1regidR {
        Aluin1regidR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bit 12 - Real Zero"]
    #[inline(always)]
    pub fn aluin1realzero(&self) -> Aluin1realzeroR {
        Aluin1realzeroR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Real Negate"]
    #[inline(always)]
    pub fn aluin1realnegate(&self) -> Aluin1realnegateR {
        Aluin1realnegateR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Imaginary Not Zero"]
    #[inline(always)]
    pub fn aluin1imagzero(&self) -> Aluin1imagzeroR {
        Aluin1imagzeroR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Imaginary Negate"]
    #[inline(always)]
    pub fn aluin1imagnegate(&self) -> Aluin1imagnegateR {
        Aluin1imagnegateR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:18 - Register ID"]
    #[inline(always)]
    pub fn aluin2regid(&self) -> Aluin2regidR {
        Aluin2regidR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bit 20 - Real Zero"]
    #[inline(always)]
    pub fn aluin2realzero(&self) -> Aluin2realzeroR {
        Aluin2realzeroR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Real Negate"]
    #[inline(always)]
    pub fn aluin2realnegate(&self) -> Aluin2realnegateR {
        Aluin2realnegateR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Imaginary Not Zero"]
    #[inline(always)]
    pub fn aluin2imagzero(&self) -> Aluin2imagzeroR {
        Aluin2imagzeroR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Imaginary Negate"]
    #[inline(always)]
    pub fn aluin2imagnegate(&self) -> Aluin2imagnegateR {
        Aluin2imagnegateR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 28:30 - Register ID"]
    #[inline(always)]
    pub fn aluoutregid(&self) -> AluoutregidR {
        AluoutregidR::new(((self.bits >> 28) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Register ID"]
    #[inline(always)]
    pub fn aluin0regid(&mut self) -> Aluin0regidW<'_, Instr6cfg0Spec> {
        Aluin0regidW::new(self, 0)
    }
    #[doc = "Bit 4 - Real Zero"]
    #[inline(always)]
    pub fn aluin0realzero(&mut self) -> Aluin0realzeroW<'_, Instr6cfg0Spec> {
        Aluin0realzeroW::new(self, 4)
    }
    #[doc = "Bit 5 - Real Negate"]
    #[inline(always)]
    pub fn aluin0realnegate(&mut self) -> Aluin0realnegateW<'_, Instr6cfg0Spec> {
        Aluin0realnegateW::new(self, 5)
    }
    #[doc = "Bit 6 - Imaginary Not Zero"]
    #[inline(always)]
    pub fn aluin0imagzero(&mut self) -> Aluin0imagzeroW<'_, Instr6cfg0Spec> {
        Aluin0imagzeroW::new(self, 6)
    }
    #[doc = "Bit 7 - Imaginary Negate"]
    #[inline(always)]
    pub fn aluin0imagnegate(&mut self) -> Aluin0imagnegateW<'_, Instr6cfg0Spec> {
        Aluin0imagnegateW::new(self, 7)
    }
    #[doc = "Bits 8:10 - Register ID"]
    #[inline(always)]
    pub fn aluin1regid(&mut self) -> Aluin1regidW<'_, Instr6cfg0Spec> {
        Aluin1regidW::new(self, 8)
    }
    #[doc = "Bit 12 - Real Zero"]
    #[inline(always)]
    pub fn aluin1realzero(&mut self) -> Aluin1realzeroW<'_, Instr6cfg0Spec> {
        Aluin1realzeroW::new(self, 12)
    }
    #[doc = "Bit 13 - Real Negate"]
    #[inline(always)]
    pub fn aluin1realnegate(&mut self) -> Aluin1realnegateW<'_, Instr6cfg0Spec> {
        Aluin1realnegateW::new(self, 13)
    }
    #[doc = "Bit 14 - Imaginary Not Zero"]
    #[inline(always)]
    pub fn aluin1imagzero(&mut self) -> Aluin1imagzeroW<'_, Instr6cfg0Spec> {
        Aluin1imagzeroW::new(self, 14)
    }
    #[doc = "Bit 15 - Imaginary Negate"]
    #[inline(always)]
    pub fn aluin1imagnegate(&mut self) -> Aluin1imagnegateW<'_, Instr6cfg0Spec> {
        Aluin1imagnegateW::new(self, 15)
    }
    #[doc = "Bits 16:18 - Register ID"]
    #[inline(always)]
    pub fn aluin2regid(&mut self) -> Aluin2regidW<'_, Instr6cfg0Spec> {
        Aluin2regidW::new(self, 16)
    }
    #[doc = "Bit 20 - Real Zero"]
    #[inline(always)]
    pub fn aluin2realzero(&mut self) -> Aluin2realzeroW<'_, Instr6cfg0Spec> {
        Aluin2realzeroW::new(self, 20)
    }
    #[doc = "Bit 21 - Real Negate"]
    #[inline(always)]
    pub fn aluin2realnegate(&mut self) -> Aluin2realnegateW<'_, Instr6cfg0Spec> {
        Aluin2realnegateW::new(self, 21)
    }
    #[doc = "Bit 22 - Imaginary Not Zero"]
    #[inline(always)]
    pub fn aluin2imagzero(&mut self) -> Aluin2imagzeroW<'_, Instr6cfg0Spec> {
        Aluin2imagzeroW::new(self, 22)
    }
    #[doc = "Bit 23 - Imaginary Negate"]
    #[inline(always)]
    pub fn aluin2imagnegate(&mut self) -> Aluin2imagnegateW<'_, Instr6cfg0Spec> {
        Aluin2imagnegateW::new(self, 23)
    }
    #[doc = "Bits 28:30 - Register ID"]
    #[inline(always)]
    pub fn aluoutregid(&mut self) -> AluoutregidW<'_, Instr6cfg0Spec> {
        AluoutregidW::new(self, 28)
    }
}
#[doc = "Instruction N Word 0\n\nYou can [`read`](crate::Reg::read) this register and get [`instr6cfg0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`instr6cfg0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Instr6cfg0Spec;
impl crate::RegisterSpec for Instr6cfg0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`instr6cfg0::R`](R) reader structure"]
impl crate::Readable for Instr6cfg0Spec {}
#[doc = "`write(|w| ..)` method takes [`instr6cfg0::W`](W) writer structure"]
impl crate::Writable for Instr6cfg0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets INSTR6CFG0 to value 0"]
impl crate::Resettable for Instr6cfg0Spec {}
