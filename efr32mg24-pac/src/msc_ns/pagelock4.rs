#[doc = "Register `PAGELOCK4` reader"]
pub type R = crate::R<Pagelock4Spec>;
#[doc = "Register `PAGELOCK4` writer"]
pub type W = crate::W<Pagelock4Spec>;
#[doc = "Field `LOCKBIT` reader - page lock bit"]
pub type LockbitR = crate::FieldReader<u32>;
#[doc = "Field `LOCKBIT` writer - page lock bit"]
pub type LockbitW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - page lock bit"]
    #[inline(always)]
    pub fn lockbit(&self) -> LockbitR {
        LockbitR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - page lock bit"]
    #[inline(always)]
    pub fn lockbit(&mut self) -> LockbitW<'_, Pagelock4Spec> {
        LockbitW::new(self, 0)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`pagelock4::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pagelock4::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pagelock4Spec;
impl crate::RegisterSpec for Pagelock4Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pagelock4::R`](R) reader structure"]
impl crate::Readable for Pagelock4Spec {}
#[doc = "`write(|w| ..)` method takes [`pagelock4::W`](W) writer structure"]
impl crate::Writable for Pagelock4Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PAGELOCK4 to value 0"]
impl crate::Resettable for Pagelock4Spec {}
