#[doc = "Register `FENOTCHCAL` reader"]
pub type R = crate::R<FenotchcalSpec>;
#[doc = "Field `FENOTCHCAPCRSE` reader - Cap Coarse"]
pub type FenotchcapcrseR = crate::FieldReader;
#[doc = "Field `FENOTCHCAPFINE` reader - Cap Fine"]
pub type FenotchcapfineR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - Cap Coarse"]
    #[inline(always)]
    pub fn fenotchcapcrse(&self) -> FenotchcapcrseR {
        FenotchcapcrseR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:7 - Cap Fine"]
    #[inline(always)]
    pub fn fenotchcapfine(&self) -> FenotchcapfineR {
        FenotchcapfineR::new(((self.bits >> 4) & 0x0f) as u8)
    }
}
#[doc = "FENOTCH Calibration\n\nYou can [`read`](crate::Reg::read) this register and get [`fenotchcal::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FenotchcalSpec;
impl crate::RegisterSpec for FenotchcalSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fenotchcal::R`](R) reader structure"]
impl crate::Readable for FenotchcalSpec {}
#[doc = "`reset()` method sets FENOTCHCAL to value 0xff"]
impl crate::Resettable for FenotchcalSpec {
    const RESET_VALUE: u32 = 0xff;
}
