#[doc = "Register `NSSTATUS` reader"]
pub type R = crate::R<NsstatusSpec>;
#[doc = "SMUNS Lock\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Smunslock {
    #[doc = "0: UNLOCKED"]
    Unlocked = 0,
    #[doc = "1: LOCKED"]
    Locked = 1,
}
impl From<Smunslock> for bool {
    #[inline(always)]
    fn from(variant: Smunslock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SMUNSLOCK` reader - SMUNS Lock"]
pub type SmunslockR = crate::BitReader<Smunslock>;
impl SmunslockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Smunslock {
        match self.bits {
            false => Smunslock::Unlocked,
            true => Smunslock::Locked,
        }
    }
    #[doc = "UNLOCKED"]
    #[inline(always)]
    pub fn is_unlocked(&self) -> bool {
        *self == Smunslock::Unlocked
    }
    #[doc = "LOCKED"]
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        *self == Smunslock::Locked
    }
}
impl R {
    #[doc = "Bit 0 - SMUNS Lock"]
    #[inline(always)]
    pub fn smunslock(&self) -> SmunslockR {
        SmunslockR::new((self.bits & 1) != 0)
    }
}
#[doc = "SMU_NS_CFGNS NSSTATUS (offset 0x004, 32 bits). Fields: SMUNSLOCK[0]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, SMU_NS_CFGNS chapter.\n\nYou can [`read`](crate::Reg::read) this register and get [`nsstatus::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct NsstatusSpec;
impl crate::RegisterSpec for NsstatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`nsstatus::R`](R) reader structure"]
impl crate::Readable for NsstatusSpec {}
#[doc = "`reset()` method sets NSSTATUS to value 0"]
impl crate::Resettable for NsstatusSpec {}
