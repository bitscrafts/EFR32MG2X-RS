#[doc = "Register `IADC0OFFSETCAL0` reader"]
pub type R = crate::R<Iadc0offsetcal0Spec>;
#[doc = "Field `OFFSETANABASE` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type OffsetanabaseR = crate::FieldReader<u16>;
#[doc = "Field `OFFSETANA1HIACC` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type Offsetana1hiaccR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn offsetanabase(&self) -> OffsetanabaseR {
        OffsetanabaseR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn offsetana1hiacc(&self) -> Offsetana1hiaccR {
        Offsetana1hiaccR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "IADC0 Offset Calibration Info\n\nYou can [`read`](crate::Reg::read) this register and get [`iadc0offsetcal0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Iadc0offsetcal0Spec;
impl crate::RegisterSpec for Iadc0offsetcal0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`iadc0offsetcal0::R`](R) reader structure"]
impl crate::Readable for Iadc0offsetcal0Spec {}
#[doc = "`reset()` method sets IADC0OFFSETCAL0 to value 0"]
impl crate::Resettable for Iadc0offsetcal0Spec {}
