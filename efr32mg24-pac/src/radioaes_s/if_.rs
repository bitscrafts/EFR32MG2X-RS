#[doc = "Register `IF` reader"]
pub type R = crate::R<IfSpec>;
#[doc = "Field `FETCHERENDOFBLOCK` reader - End of block interrupt flag"]
pub type FetcherendofblockR = crate::BitReader;
#[doc = "Field `FETCHERSTOPPED` reader - Stopped interrupt flag"]
pub type FetcherstoppedR = crate::BitReader;
#[doc = "Field `FETCHERERROR` reader - Error interrupt flag"]
pub type FetchererrorR = crate::BitReader;
#[doc = "Field `PUSHERENDOFBLOCK` reader - End of block interrupt flag"]
pub type PusherendofblockR = crate::BitReader;
#[doc = "Field `PUSHERSTOPPED` reader - Stopped interrupt flag"]
pub type PusherstoppedR = crate::BitReader;
#[doc = "Field `PUSHERERROR` reader - Error interrupt flag"]
pub type PushererrorR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - End of block interrupt flag"]
    #[inline(always)]
    pub fn fetcherendofblock(&self) -> FetcherendofblockR {
        FetcherendofblockR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Stopped interrupt flag"]
    #[inline(always)]
    pub fn fetcherstopped(&self) -> FetcherstoppedR {
        FetcherstoppedR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Error interrupt flag"]
    #[inline(always)]
    pub fn fetchererror(&self) -> FetchererrorR {
        FetchererrorR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - End of block interrupt flag"]
    #[inline(always)]
    pub fn pusherendofblock(&self) -> PusherendofblockR {
        PusherendofblockR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Stopped interrupt flag"]
    #[inline(always)]
    pub fn pusherstopped(&self) -> PusherstoppedR {
        PusherstoppedR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Error interrupt flag"]
    #[inline(always)]
    pub fn pushererror(&self) -> PushererrorR {
        PushererrorR::new(((self.bits >> 5) & 1) != 0)
    }
}
#[doc = "Interrupt flag register\n\nYou can [`read`](crate::Reg::read) this register and get [`if_::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IfSpec;
impl crate::RegisterSpec for IfSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`if_::R`](R) reader structure"]
impl crate::Readable for IfSpec {}
#[doc = "`reset()` method sets IF to value 0"]
impl crate::Resettable for IfSpec {}
