#[doc = "Register `MODULEINFO` reader"]
pub type R = crate::R<ModuleinfoSpec>;
#[doc = "Field `HWREV` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type HwrevR = crate::FieldReader;
#[doc = "Module Information\n\nValue on reset: 7"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Antenna {
    #[doc = "0: BUILTIN"]
    Builtin = 0,
    #[doc = "1: CONNECTOR"]
    Connector = 1,
    #[doc = "2: RFPAD"]
    Rfpad = 2,
    #[doc = "3: INVERTEDF"]
    Invertedf = 3,
}
impl From<Antenna> for u8 {
    #[inline(always)]
    fn from(variant: Antenna) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Antenna {
    type Ux = u8;
}
impl crate::IsEnum for Antenna {}
#[doc = "Field `ANTENNA` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type AntennaR = crate::FieldReader<Antenna>;
impl AntennaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Antenna> {
        match self.bits {
            0 => Some(Antenna::Builtin),
            1 => Some(Antenna::Connector),
            2 => Some(Antenna::Rfpad),
            3 => Some(Antenna::Invertedf),
            _ => None,
        }
    }
    #[doc = "BUILTIN"]
    #[inline(always)]
    pub fn is_builtin(&self) -> bool {
        *self == Antenna::Builtin
    }
    #[doc = "CONNECTOR"]
    #[inline(always)]
    pub fn is_connector(&self) -> bool {
        *self == Antenna::Connector
    }
    #[doc = "RFPAD"]
    #[inline(always)]
    pub fn is_rfpad(&self) -> bool {
        *self == Antenna::Rfpad
    }
    #[doc = "INVERTEDF"]
    #[inline(always)]
    pub fn is_invertedf(&self) -> bool {
        *self == Antenna::Invertedf
    }
}
#[doc = "Field `MODNUMBER` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type ModnumberR = crate::FieldReader;
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    #[doc = "0: PCB"]
    Pcb = 0,
    #[doc = "1: SIP"]
    Sip = 1,
}
impl From<Type> for bool {
    #[inline(always)]
    fn from(variant: Type) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TYPE` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type TypeR = crate::BitReader<Type>;
impl TypeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Type {
        match self.bits {
            false => Type::Pcb,
            true => Type::Sip,
        }
    }
    #[doc = "PCB"]
    #[inline(always)]
    pub fn is_pcb(&self) -> bool {
        *self == Type::Pcb
    }
    #[doc = "SIP"]
    #[inline(always)]
    pub fn is_sip(&self) -> bool {
        *self == Type::Sip
    }
}
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lfxo {
    #[doc = "0: NONE"]
    None = 0,
    #[doc = "1: PRESENT"]
    Present = 1,
}
impl From<Lfxo> for bool {
    #[inline(always)]
    fn from(variant: Lfxo) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LFXO` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type LfxoR = crate::BitReader<Lfxo>;
impl LfxoR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lfxo {
        match self.bits {
            false => Lfxo::None,
            true => Lfxo::Present,
        }
    }
    #[doc = "NONE"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Lfxo::None
    }
    #[doc = "PRESENT"]
    #[inline(always)]
    pub fn is_present(&self) -> bool {
        *self == Lfxo::Present
    }
}
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Express {
    #[doc = "0: SUPPORTED"]
    Supported = 0,
    #[doc = "1: NONE"]
    None = 1,
}
impl From<Express> for bool {
    #[inline(always)]
    fn from(variant: Express) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EXPRESS` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type ExpressR = crate::BitReader<Express>;
impl ExpressR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Express {
        match self.bits {
            false => Express::Supported,
            true => Express::None,
        }
    }
    #[doc = "SUPPORTED"]
    #[inline(always)]
    pub fn is_supported(&self) -> bool {
        *self == Express::Supported
    }
    #[doc = "NONE"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Express::None
    }
}
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lfxocalval {
    #[doc = "0: VALID"]
    Valid = 0,
    #[doc = "1: NOTVALID"]
    Notvalid = 1,
}
impl From<Lfxocalval> for bool {
    #[inline(always)]
    fn from(variant: Lfxocalval) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LFXOCALVAL` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type LfxocalvalR = crate::BitReader<Lfxocalval>;
impl LfxocalvalR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lfxocalval {
        match self.bits {
            false => Lfxocalval::Valid,
            true => Lfxocalval::Notvalid,
        }
    }
    #[doc = "VALID"]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Lfxocalval::Valid
    }
    #[doc = "NOTVALID"]
    #[inline(always)]
    pub fn is_notvalid(&self) -> bool {
        *self == Lfxocalval::Notvalid
    }
}
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hfxocalval {
    #[doc = "0: VALID"]
    Valid = 0,
    #[doc = "1: NOTVALID"]
    Notvalid = 1,
}
impl From<Hfxocalval> for bool {
    #[inline(always)]
    fn from(variant: Hfxocalval) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HFXOCALVAL` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type HfxocalvalR = crate::BitReader<Hfxocalval>;
impl HfxocalvalR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hfxocalval {
        match self.bits {
            false => Hfxocalval::Valid,
            true => Hfxocalval::Notvalid,
        }
    }
    #[doc = "VALID"]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Hfxocalval::Valid
    }
    #[doc = "NOTVALID"]
    #[inline(always)]
    pub fn is_notvalid(&self) -> bool {
        *self == Hfxocalval::Notvalid
    }
}
#[doc = "Field `MODNUMBERMSB` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type ModnumbermsbR = crate::FieldReader<u16>;
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Padcdc {
    #[doc = "0: VDCDC"]
    Vdcdc = 0,
    #[doc = "1: OTHER"]
    Other = 1,
}
impl From<Padcdc> for bool {
    #[inline(always)]
    fn from(variant: Padcdc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PADCDC` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type PadcdcR = crate::BitReader<Padcdc>;
impl PadcdcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Padcdc {
        match self.bits {
            false => Padcdc::Vdcdc,
            true => Padcdc::Other,
        }
    }
    #[doc = "VDCDC"]
    #[inline(always)]
    pub fn is_vdcdc(&self) -> bool {
        *self == Padcdc::Vdcdc
    }
    #[doc = "OTHER"]
    #[inline(always)]
    pub fn is_other(&self) -> bool {
        *self == Padcdc::Other
    }
}
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phylimited {
    #[doc = "0: LIMITED"]
    Limited = 0,
    #[doc = "1: UNLIMITED"]
    Unlimited = 1,
}
impl From<Phylimited> for bool {
    #[inline(always)]
    fn from(variant: Phylimited) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PHYLIMITED` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type PhylimitedR = crate::BitReader<Phylimited>;
impl PhylimitedR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Phylimited {
        match self.bits {
            false => Phylimited::Limited,
            true => Phylimited::Unlimited,
        }
    }
    #[doc = "LIMITED"]
    #[inline(always)]
    pub fn is_limited(&self) -> bool {
        *self == Phylimited::Limited
    }
    #[doc = "UNLIMITED"]
    #[inline(always)]
    pub fn is_unlimited(&self) -> bool {
        *self == Phylimited::Unlimited
    }
}
#[doc = "DEVINFO MODULEINFO (offset 0x14C, 32 bits). Fields: HWREV[4:0], ANTENNA[7:5], MODNUMBER[14:8], TYPE[15], LFXO[16], EXPRESS[17], LFXOCALVAL[18], HFXOCALVAL[19], MODNUMBERMSB[28:20], PADCDC[29], PHYLIMITED[30], EXTVALID[31]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, DEVINFO chapter.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extvalid {
    #[doc = "0: EXTUSED"]
    Extused = 0,
    #[doc = "1: EXTUNUSED"]
    Extunused = 1,
}
impl From<Extvalid> for bool {
    #[inline(always)]
    fn from(variant: Extvalid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EXTVALID` reader - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
pub type ExtvalidR = crate::BitReader<Extvalid>;
impl ExtvalidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Extvalid {
        match self.bits {
            false => Extvalid::Extused,
            true => Extvalid::Extunused,
        }
    }
    #[doc = "EXTUSED"]
    #[inline(always)]
    pub fn is_extused(&self) -> bool {
        *self == Extvalid::Extused
    }
    #[doc = "EXTUNUSED"]
    #[inline(always)]
    pub fn is_extunused(&self) -> bool {
        *self == Extvalid::Extunused
    }
}
impl R {
    #[doc = "Bits 0:4 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn hwrev(&self) -> HwrevR {
        HwrevR::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 5:7 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn antenna(&self) -> AntennaR {
        AntennaR::new(((self.bits >> 5) & 7) as u8)
    }
    #[doc = "Bits 8:14 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn modnumber(&self) -> ModnumberR {
        ModnumberR::new(((self.bits >> 8) & 0x7f) as u8)
    }
    #[doc = "Bit 15 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn type_(&self) -> TypeR {
        TypeR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn lfxo(&self) -> LfxoR {
        LfxoR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn express(&self) -> ExpressR {
        ExpressR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn lfxocalval(&self) -> LfxocalvalR {
        LfxocalvalR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn hfxocalval(&self) -> HfxocalvalR {
        HfxocalvalR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bits 20:28 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn modnumbermsb(&self) -> ModnumbermsbR {
        ModnumbermsbR::new(((self.bits >> 20) & 0x01ff) as u16)
    }
    #[doc = "Bit 29 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn padcdc(&self) -> PadcdcR {
        PadcdcR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn phylimited(&self) -> PhylimitedR {
        PhylimitedR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - no vendor description; see the EFR32xG24 Reference Manual, DEVINFO chapter"]
    #[inline(always)]
    pub fn extvalid(&self) -> ExtvalidR {
        ExtvalidR::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Module Information\n\nYou can [`read`](crate::Reg::read) this register and get [`moduleinfo::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ModuleinfoSpec;
impl crate::RegisterSpec for ModuleinfoSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`moduleinfo::R`](R) reader structure"]
impl crate::Readable for ModuleinfoSpec {}
#[doc = "`reset()` method sets MODULEINFO to value 0xffff_ffff"]
impl crate::Resettable for ModuleinfoSpec {
    const RESET_VALUE: u32 = 0xffff_ffff;
}
