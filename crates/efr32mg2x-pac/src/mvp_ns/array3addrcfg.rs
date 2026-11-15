#[doc = "Register `ARRAY3ADDRCFG` reader"]
pub type R = crate::R<Array3addrcfgSpec>;
#[doc = "Register `ARRAY3ADDRCFG` writer"]
pub type W = crate::W<Array3addrcfgSpec>;
#[doc = "Field `BASE` reader - Array Base Address"]
pub type BaseR = crate::FieldReader<u32>;
#[doc = "Field `BASE` writer - Array Base Address"]
pub type BaseW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Array Base Address"]
    #[inline(always)]
    pub fn base(&self) -> BaseR {
        BaseR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Array Base Address"]
    #[inline(always)]
    pub fn base(&mut self) -> BaseW<'_, Array3addrcfgSpec> {
        BaseW::new(self, 0)
    }
}
#[doc = "Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array3addrcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3addrcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Array3addrcfgSpec;
impl crate::RegisterSpec for Array3addrcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`array3addrcfg::R`](R) reader structure"]
impl crate::Readable for Array3addrcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`array3addrcfg::W`](W) writer structure"]
impl crate::Writable for Array3addrcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ARRAY3ADDRCFG to value 0"]
impl crate::Resettable for Array3addrcfgSpec {}
