#[doc = "Register `ARRAY4ADDRCFG` reader"]
pub type R = crate::R<Array4addrcfgSpec>;
#[doc = "Register `ARRAY4ADDRCFG` writer"]
pub type W = crate::W<Array4addrcfgSpec>;
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
    pub fn base(&mut self) -> BaseW<'_, Array4addrcfgSpec> {
        BaseW::new(self, 0)
    }
}
#[doc = "Array N Base Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array4addrcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array4addrcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Array4addrcfgSpec;
impl crate::RegisterSpec for Array4addrcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`array4addrcfg::R`](R) reader structure"]
impl crate::Readable for Array4addrcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`array4addrcfg::W`](W) writer structure"]
impl crate::Writable for Array4addrcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ARRAY4ADDRCFG to value 0"]
impl crate::Resettable for Array4addrcfgSpec {}
