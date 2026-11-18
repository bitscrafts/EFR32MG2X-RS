#[doc = "Register `CC1_ICOF` reader"]
pub type R = crate::R<Cc1IcofSpec>;
#[doc = "Field `ICOF` reader - Input Capture FIFO Overflow"]
pub type IcofR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Input Capture FIFO Overflow"]
    #[inline(always)]
    pub fn icof(&self) -> IcofR {
        IcofR::new((self.bits & 0xffff) as u16)
    }
}
#[doc = "TIMER2_S CC1_ICOF (offset 0x098, 32 bits). Fields: ICOF[15:0]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, TIMER chapter.\n\nYou can [`read`](crate::Reg::read) this register and get [`cc1_icof::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cc1IcofSpec;
impl crate::RegisterSpec for Cc1IcofSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cc1_icof::R`](R) reader structure"]
impl crate::Readable for Cc1IcofSpec {}
#[doc = "`reset()` method sets CC1_ICOF to value 0"]
impl crate::Resettable for Cc1IcofSpec {}
