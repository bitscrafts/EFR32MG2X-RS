#[doc = "Register `STATUS` reader"]
pub type R = crate::R<StatusSpec>;
#[doc = "Field `RUNNING` reader - Running Status"]
pub type RunningR = crate::BitReader;
#[doc = "Field `PAUSED` reader - Paused Status"]
pub type PausedR = crate::BitReader;
#[doc = "Field `IDLE` reader - Idle Status"]
pub type IdleR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - Running Status"]
    #[inline(always)]
    pub fn running(&self) -> RunningR {
        RunningR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Paused Status"]
    #[inline(always)]
    pub fn paused(&self) -> PausedR {
        PausedR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Idle Status"]
    #[inline(always)]
    pub fn idle(&self) -> IdleR {
        IdleR::new(((self.bits >> 2) & 1) != 0)
    }
}
#[doc = "Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatusSpec;
impl crate::RegisterSpec for StatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`status::R`](R) reader structure"]
impl crate::Readable for StatusSpec {}
#[doc = "`reset()` method sets STATUS to value 0x04"]
impl crate::Resettable for StatusSpec {
    const RESET_VALUE: u32 = 0x04;
}
