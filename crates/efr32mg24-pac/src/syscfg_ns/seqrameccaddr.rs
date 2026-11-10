#[doc = "Register `SEQRAMECCADDR` reader"]
pub type R = crate::R<SeqrameccaddrSpec>;
#[doc = "Field `SEQRAMECCADDR` reader - SEQRAM ECC Address"]
pub type SeqrameccaddrR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - SEQRAM ECC Address"]
    #[inline(always)]
    pub fn seqrameccaddr(&self) -> SeqrameccaddrR {
        SeqrameccaddrR::new(self.bits)
    }
}
#[doc = "Read to get status of the SEQRAM ECC error address.\n\nYou can [`read`](crate::Reg::read) this register and get [`seqrameccaddr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SeqrameccaddrSpec;
impl crate::RegisterSpec for SeqrameccaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`seqrameccaddr::R`](R) reader structure"]
impl crate::Readable for SeqrameccaddrSpec {}
#[doc = "`reset()` method sets SEQRAMECCADDR to value 0"]
impl crate::Resettable for SeqrameccaddrSpec {}
