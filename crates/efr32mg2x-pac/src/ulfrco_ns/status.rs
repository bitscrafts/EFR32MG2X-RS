#[doc = "Register `STATUS` reader"]
pub type R = crate::R<StatusSpec>;
#[doc = "Field `RDY` reader - Ready Status"]
pub type RdyR = crate::BitReader;
#[doc = "Field `ENS` reader - Enable Status"]
pub type EnsR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - Ready Status"]
    #[inline(always)]
    pub fn rdy(&self) -> RdyR {
        RdyR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 16 - Enable Status"]
    #[inline(always)]
    pub fn ens(&self) -> EnsR {
        EnsR::new(((self.bits >> 16) & 1) != 0)
    }
}
#[doc = "ULFRCO_NS STATUS (offset 0x008, 32 bits). Fields: RDY[0], ENS[16]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, ULFRCO chapter.\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatusSpec;
impl crate::RegisterSpec for StatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`status::R`](R) reader structure"]
impl crate::Readable for StatusSpec {}
#[doc = "`reset()` method sets STATUS to value 0"]
impl crate::Resettable for StatusSpec {}
