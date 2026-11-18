#[doc = "Register `HFRCODPLLCAL14` reader"]
pub type R = crate::R<Hfrcodpllcal14Spec>;
#[doc = "Field `TUNING` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type TuningR = crate::FieldReader;
#[doc = "Field `FINETUNING` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type FinetuningR = crate::FieldReader;
#[doc = "Field `LDOHP` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type LdohpR = crate::BitReader;
#[doc = "Field `FREQRANGE` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type FreqrangeR = crate::FieldReader;
#[doc = "Field `CMPBIAS` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type CmpbiasR = crate::FieldReader;
#[doc = "Field `CLKDIV` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type ClkdivR = crate::FieldReader;
#[doc = "Field `CMPSEL` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type CmpselR = crate::FieldReader;
#[doc = "Field `IREFTC` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type IreftcR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:6 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn tuning(&self) -> TuningR {
        TuningR::new((self.bits & 0x7f) as u8)
    }
    #[doc = "Bits 8:13 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn finetuning(&self) -> FinetuningR {
        FinetuningR::new(((self.bits >> 8) & 0x3f) as u8)
    }
    #[doc = "Bit 15 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn ldohp(&self) -> LdohpR {
        LdohpR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:20 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn freqrange(&self) -> FreqrangeR {
        FreqrangeR::new(((self.bits >> 16) & 0x1f) as u8)
    }
    #[doc = "Bits 21:23 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn cmpbias(&self) -> CmpbiasR {
        CmpbiasR::new(((self.bits >> 21) & 7) as u8)
    }
    #[doc = "Bits 24:25 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn clkdiv(&self) -> ClkdivR {
        ClkdivR::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bits 26:27 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn cmpsel(&self) -> CmpselR {
        CmpselR::new(((self.bits >> 26) & 3) as u8)
    }
    #[doc = "Bits 28:31 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn ireftc(&self) -> IreftcR {
        IreftcR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
#[doc = "HFRCODPLL Calibration\n\nYou can [`read`](crate::Reg::read) this register and get [`hfrcodpllcal14::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Hfrcodpllcal14Spec;
impl crate::RegisterSpec for Hfrcodpllcal14Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`hfrcodpllcal14::R`](R) reader structure"]
impl crate::Readable for Hfrcodpllcal14Spec {}
#[doc = "`reset()` method sets HFRCODPLLCAL14 to value 0"]
impl crate::Resettable for Hfrcodpllcal14Spec {}
