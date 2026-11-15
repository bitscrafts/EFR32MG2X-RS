#[doc = "Register `ARRAY3INDEXSTATE` reader"]
pub type R = crate::R<Array3indexstateSpec>;
#[doc = "Register `ARRAY3INDEXSTATE` writer"]
pub type W = crate::W<Array3indexstateSpec>;
#[doc = "Field `DIM0INDEX` reader - Current Index"]
pub type Dim0indexR = crate::FieldReader<u16>;
#[doc = "Field `DIM0INDEX` writer - Current Index"]
pub type Dim0indexW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `DIM1INDEX` reader - Current Index"]
pub type Dim1indexR = crate::FieldReader<u16>;
#[doc = "Field `DIM1INDEX` writer - Current Index"]
pub type Dim1indexW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `DIM2INDEX` reader - Current Index"]
pub type Dim2indexR = crate::FieldReader<u16>;
#[doc = "Field `DIM2INDEX` writer - Current Index"]
pub type Dim2indexW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - Current Index"]
    #[inline(always)]
    pub fn dim0index(&self) -> Dim0indexR {
        Dim0indexR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 10:19 - Current Index"]
    #[inline(always)]
    pub fn dim1index(&self) -> Dim1indexR {
        Dim1indexR::new(((self.bits >> 10) & 0x03ff) as u16)
    }
    #[doc = "Bits 20:29 - Current Index"]
    #[inline(always)]
    pub fn dim2index(&self) -> Dim2indexR {
        Dim2indexR::new(((self.bits >> 20) & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Current Index"]
    #[inline(always)]
    pub fn dim0index(&mut self) -> Dim0indexW<'_, Array3indexstateSpec> {
        Dim0indexW::new(self, 0)
    }
    #[doc = "Bits 10:19 - Current Index"]
    #[inline(always)]
    pub fn dim1index(&mut self) -> Dim1indexW<'_, Array3indexstateSpec> {
        Dim1indexW::new(self, 10)
    }
    #[doc = "Bits 20:29 - Current Index"]
    #[inline(always)]
    pub fn dim2index(&mut self) -> Dim2indexW<'_, Array3indexstateSpec> {
        Dim2indexW::new(self, 20)
    }
}
#[doc = "Array N Index State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`array3indexstate::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3indexstate::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Array3indexstateSpec;
impl crate::RegisterSpec for Array3indexstateSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`array3indexstate::R`](R) reader structure"]
impl crate::Readable for Array3indexstateSpec {}
#[doc = "`write(|w| ..)` method takes [`array3indexstate::W`](W) writer structure"]
impl crate::Writable for Array3indexstateSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ARRAY3INDEXSTATE to value 0"]
impl crate::Resettable for Array3indexstateSpec {}
