#[doc = "Register `FRCRAMECCADDR` reader"]
pub type R = crate::R<FrcrameccaddrSpec>;
#[doc = "Field `FRCRAMECCADDR` reader - FRCRAM ECC Error Address"]
pub type FrcrameccaddrR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - FRCRAM ECC Error Address"]
    #[inline(always)]
    pub fn frcrameccaddr(&self) -> FrcrameccaddrR {
        FrcrameccaddrR::new(self.bits)
    }
}
#[doc = "Read to get status of the FRCRAM ECC error address.\n\nYou can [`read`](crate::Reg::read) this register and get [`frcrameccaddr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrcrameccaddrSpec;
impl crate::RegisterSpec for FrcrameccaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`frcrameccaddr::R`](R) reader structure"]
impl crate::Readable for FrcrameccaddrSpec {}
#[doc = "`reset()` method sets FRCRAMECCADDR to value 0"]
impl crate::Resettable for FrcrameccaddrSpec {}
