#[doc = "Register `DATAREV` reader"]
pub type R = crate::R<DatarevSpec>;
#[doc = "Field `DATAREV` reader - Data Reverse Value"]
pub type DatarevR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Data Reverse Value"]
    #[inline(always)]
    pub fn datarev(&self) -> DatarevR {
        DatarevR::new(self.bits)
    }
}
#[doc = "GPCRC_S DATAREV (offset 0x028, 32 bits). Fields: DATAREV[31:0]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, GPCRC chapter.\n\nYou can [`read`](crate::Reg::read) this register and get [`datarev::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DatarevSpec;
impl crate::RegisterSpec for DatarevSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`datarev::R`](R) reader structure"]
impl crate::Readable for DatarevSpec {}
#[doc = "`reset()` method sets DATAREV to value 0"]
impl crate::Resettable for DatarevSpec {}
