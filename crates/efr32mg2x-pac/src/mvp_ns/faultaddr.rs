#[doc = "Register `FAULTADDR` reader"]
pub type R = crate::R<FaultaddrSpec>;
#[doc = "Field `FAULTADDR` reader - Bus Fault Address Register"]
pub type FaultaddrR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Bus Fault Address Register"]
    #[inline(always)]
    pub fn faultaddr(&self) -> FaultaddrR {
        FaultaddrR::new(self.bits)
    }
}
#[doc = "Fault Address Register\n\nYou can [`read`](crate::Reg::read) this register and get [`faultaddr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FaultaddrSpec;
impl crate::RegisterSpec for FaultaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`faultaddr::R`](R) reader structure"]
impl crate::Readable for FaultaddrSpec {}
#[doc = "`reset()` method sets FAULTADDR to value 0"]
impl crate::Resettable for FaultaddrSpec {}
