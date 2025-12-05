#[doc = "Register `STATUS` reader"]
pub type R = crate::R<StatusSpec>;
#[doc = "Field `FETCHERBSY` reader - Fetcher busy"]
pub type FetcherbsyR = crate::BitReader;
#[doc = "Field `PUSHERBSY` reader - Pusher busy"]
pub type PusherbsyR = crate::BitReader;
#[doc = "Field `NOTEMPTY` reader - Not empty flag from input FIFO (fetcher)"]
pub type NotemptyR = crate::BitReader;
#[doc = "Field `WAITING` reader - Pusher waiting for FIFO"]
pub type WaitingR = crate::BitReader;
#[doc = "Field `SOFTRSTBSY` reader - Software reset busy"]
pub type SoftrstbsyR = crate::BitReader;
#[doc = "Field `FIFODATANUM` reader - Number of data in output FIFO"]
pub type FifodatanumR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bit 0 - Fetcher busy"]
    #[inline(always)]
    pub fn fetcherbsy(&self) -> FetcherbsyR {
        FetcherbsyR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Pusher busy"]
    #[inline(always)]
    pub fn pusherbsy(&self) -> PusherbsyR {
        PusherbsyR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 4 - Not empty flag from input FIFO (fetcher)"]
    #[inline(always)]
    pub fn notempty(&self) -> NotemptyR {
        NotemptyR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Pusher waiting for FIFO"]
    #[inline(always)]
    pub fn waiting(&self) -> WaitingR {
        WaitingR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Software reset busy"]
    #[inline(always)]
    pub fn softrstbsy(&self) -> SoftrstbsyR {
        SoftrstbsyR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bits 16:31 - Number of data in output FIFO"]
    #[inline(always)]
    pub fn fifodatanum(&self) -> FifodatanumR {
        FifodatanumR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Status register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatusSpec;
impl crate::RegisterSpec for StatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`status::R`](R) reader structure"]
impl crate::Readable for StatusSpec {}
#[doc = "`reset()` method sets STATUS to value 0"]
impl crate::Resettable for StatusSpec {}
