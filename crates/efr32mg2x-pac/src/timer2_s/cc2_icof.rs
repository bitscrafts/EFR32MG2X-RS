#[doc = "Register `CC2_ICOF` reader"]
pub type R = crate::R<Cc2IcofSpec>;
#[doc = "Field `ICOF` reader - Input Capture FIFO Overflow"]
pub type IcofR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Input Capture FIFO Overflow"]
    #[inline(always)]
    pub fn icof(&self) -> IcofR {
        IcofR::new((self.bits & 0xffff) as u16)
    }
}
#[doc = "TIMER2_S CC2_ICOF (offset 0x0B8, 32 bits). Fields: ICOF[15:0]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, TIMER chapter.\n\nYou can [`read`](crate::Reg::read) this register and get [`cc2_icof::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cc2IcofSpec;
impl crate::RegisterSpec for Cc2IcofSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cc2_icof::R`](R) reader structure"]
impl crate::Readable for Cc2IcofSpec {}
#[doc = "`reset()` method sets CC2_ICOF to value 0"]
impl crate::Resettable for Cc2IcofSpec {}
