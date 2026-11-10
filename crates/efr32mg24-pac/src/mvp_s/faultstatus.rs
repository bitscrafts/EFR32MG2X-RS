#[doc = "Register `FAULTSTATUS` reader"]
pub type R = crate::R<FaultstatusSpec>;
#[doc = "Field `FAULTPC` reader - PC when fault occurred"]
pub type FaultpcR = crate::FieldReader;
#[doc = "Field `FAULTARRAY` reader - Array access that generated a fault"]
pub type FaultarrayR = crate::FieldReader;
#[doc = "Bus where fault occurred\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Faultbus {
    #[doc = "0: NONE"]
    None = 0,
    #[doc = "1: LOAD0STREAM"]
    Load0stream = 1,
    #[doc = "2: LOAD1STREAM"]
    Load1stream = 2,
    #[doc = "3: STORESTREAM"]
    Storestream = 3,
}
impl From<Faultbus> for u8 {
    #[inline(always)]
    fn from(variant: Faultbus) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Faultbus {
    type Ux = u8;
}
impl crate::IsEnum for Faultbus {}
#[doc = "Field `FAULTBUS` reader - Bus where fault occurred"]
pub type FaultbusR = crate::FieldReader<Faultbus>;
impl FaultbusR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Faultbus {
        match self.bits {
            0 => Faultbus::None,
            1 => Faultbus::Load0stream,
            2 => Faultbus::Load1stream,
            3 => Faultbus::Storestream,
            _ => unreachable!(),
        }
    }
    #[doc = "NONE"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Faultbus::None
    }
    #[doc = "LOAD0STREAM"]
    #[inline(always)]
    pub fn is_load0stream(&self) -> bool {
        *self == Faultbus::Load0stream
    }
    #[doc = "LOAD1STREAM"]
    #[inline(always)]
    pub fn is_load1stream(&self) -> bool {
        *self == Faultbus::Load1stream
    }
    #[doc = "STORESTREAM"]
    #[inline(always)]
    pub fn is_storestream(&self) -> bool {
        *self == Faultbus::Storestream
    }
}
#[doc = "Field `FAULTLOOP` reader - Loop Fault Indicator"]
pub type FaultloopR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:2 - PC when fault occurred"]
    #[inline(always)]
    pub fn faultpc(&self) -> FaultpcR {
        FaultpcR::new((self.bits & 7) as u8)
    }
    #[doc = "Bits 8:10 - Array access that generated a fault"]
    #[inline(always)]
    pub fn faultarray(&self) -> FaultarrayR {
        FaultarrayR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bits 12:13 - Bus where fault occurred"]
    #[inline(always)]
    pub fn faultbus(&self) -> FaultbusR {
        FaultbusR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 16:19 - Loop Fault Indicator"]
    #[inline(always)]
    pub fn faultloop(&self) -> FaultloopR {
        FaultloopR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
#[doc = "Fault Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`faultstatus::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FaultstatusSpec;
impl crate::RegisterSpec for FaultstatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`faultstatus::R`](R) reader structure"]
impl crate::Readable for FaultstatusSpec {}
#[doc = "`reset()` method sets FAULTSTATUS to value 0"]
impl crate::Resettable for FaultstatusSpec {}
