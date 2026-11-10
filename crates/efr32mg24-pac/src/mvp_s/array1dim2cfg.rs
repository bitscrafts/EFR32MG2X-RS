#[doc = "Register `ARRAY1DIM2CFG` reader"]
pub type R = crate::R<Array1dim2cfgSpec>;
#[doc = "Register `ARRAY1DIM2CFG` writer"]
pub type W = crate::W<Array1dim2cfgSpec>;
#[doc = "Field `SIZE` reader - Array Dimension Size"]
pub type SizeR = crate::FieldReader<u16>;
#[doc = "Field `SIZE` writer - Array Dimension Size"]
pub type SizeW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `STRIDE` reader - Dimension Stride Step"]
pub type StrideR = crate::FieldReader<u16>;
#[doc = "Field `STRIDE` writer - Dimension Stride Step"]
pub type StrideW<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;
impl R {
    #[doc = "Bits 0:9 - Array Dimension Size"]
    #[inline(always)]
    pub fn size(&self) -> SizeR {
        SizeR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 16:27 - Dimension Stride Step"]
    #[inline(always)]
    pub fn stride(&self) -> StrideR {
        StrideR::new(((self.bits >> 16) & 0x0fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Array Dimension Size"]
    #[inline(always)]
    pub fn size(&mut self) -> SizeW<'_, Array1dim2cfgSpec> {
        SizeW::new(self, 0)
    }
    #[doc = "Bits 16:27 - Dimension Stride Step"]
    #[inline(always)]
    pub fn stride(&mut self) -> StrideW<'_, Array1dim2cfgSpec> {
        StrideW::new(self, 16)
    }
}
#[doc = "Array N Dimenion 2 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array1dim2cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array1dim2cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Array1dim2cfgSpec;
impl crate::RegisterSpec for Array1dim2cfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`array1dim2cfg::R`](R) reader structure"]
impl crate::Readable for Array1dim2cfgSpec {}
#[doc = "`write(|w| ..)` method takes [`array1dim2cfg::W`](W) writer structure"]
impl crate::Writable for Array1dim2cfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ARRAY1DIM2CFG to value 0"]
impl crate::Resettable for Array1dim2cfgSpec {}
