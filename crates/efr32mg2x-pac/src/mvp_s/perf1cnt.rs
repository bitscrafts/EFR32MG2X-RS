#[doc = "Register `PERF1CNT` reader"]
pub type R = crate::R<Perf1cntSpec>;
#[doc = "Field `COUNT` reader - Performance Counter"]
pub type CountR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:23 - Performance Counter"]
    #[inline(always)]
    pub fn count(&self) -> CountR {
        CountR::new(self.bits & 0x00ff_ffff)
    }
}
#[doc = "Run Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`perf1cnt::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Perf1cntSpec;
impl crate::RegisterSpec for Perf1cntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`perf1cnt::R`](R) reader structure"]
impl crate::Readable for Perf1cntSpec {}
#[doc = "`reset()` method sets PERF1CNT to value 0"]
impl crate::Resettable for Perf1cntSpec {}
