#[doc = "Register `ARRAY1ADDRCFG` reader"]
pub type R = crate::R<Array1addrcfgSpec>;
#[doc = "Register `ARRAY1ADDRCFG` writer"]
pub type W = crate::W<Array1addrcfgSpec>;
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
    pub fn base(&mut self) -> BaseW<'_, Array1addrcfgSpec> {
        BaseW::new(self, 0)
    }
}
#[doc = "Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array1addrcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array1addrcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Array1addrcfgSpec;
impl crate::RegisterSpec for Array1addrcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`array1addrcfg::R`](R) reader structure"]
impl crate::Readable for Array1addrcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`array1addrcfg::W`](W) writer structure"]
impl crate::Writable for Array1addrcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ARRAY1ADDRCFG to value 0"]
impl crate::Resettable for Array1addrcfgSpec {}
