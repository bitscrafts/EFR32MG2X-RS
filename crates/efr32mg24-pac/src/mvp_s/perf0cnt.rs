#[doc = "Register `PERF0CNT` reader"]
pub type R = crate::R<Perf0cntSpec>;
#[doc = "Field `COUNT` reader - Performance Counter"]
pub type CountR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:23 - Performance Counter"]
    #[inline(always)]
    pub fn count(&self) -> CountR {
        CountR::new(self.bits & 0x00ff_ffff)
    }
}
#[doc = "Run Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`perf0cnt::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Perf0cntSpec;
impl crate::RegisterSpec for Perf0cntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`perf0cnt::R`](R) reader structure"]
impl crate::Readable for Perf0cntSpec {}
#[doc = "`reset()` method sets PERF0CNT to value 0"]
impl crate::Resettable for Perf0cntSpec {}
