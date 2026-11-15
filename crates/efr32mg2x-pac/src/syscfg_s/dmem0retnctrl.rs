#[doc = "Register `DMEM0RETNCTRL` reader"]
pub type R = crate::R<Dmem0retnctrlSpec>;
#[doc = "Register `DMEM0RETNCTRL` writer"]
pub type W = crate::W<Dmem0retnctrlSpec>;
#[doc = "DMEM0 blockset retention control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Ramretnctrl {
    #[doc = "0: None of the RAM blocks powered down"]
    Allon = 0,
    #[doc = "16384: Power down RAM block 15 (address range 0x2003C000-0x20040000)"]
    Blk15 = 16384,
    #[doc = "24576: Power down RAM blocks 14 and above (address range 0x20038000-0x20040000)"]
    Blk14to15 = 24576,
    #[doc = "28672: Power down RAM blocks 13 and above (address range 0x20034000-0x20040000)"]
    Blk13to15 = 28672,
    #[doc = "30720: Power down RAM blocks 12 and above (address range 0x20030000-0x20040000)"]
    Blk12to15 = 30720,
    #[doc = "31744: Power down RAM blocks 11 and above (address range 0x2002C000-0x20040000)"]
    Blk11to15 = 31744,
    #[doc = "32256: Power down RAM blocks 10 and above (address range 0x20028000-0x20040000)"]
    Blk10to15 = 32256,
    #[doc = "32512: Power down RAM blocks 9 and above (address range 0x20024000-0x20040000)"]
    Blk9to15 = 32512,
    #[doc = "32640: Power down RAM blocks 8 and above (address range 0x20020000-0x20040000)"]
    Blk8to15 = 32640,
    #[doc = "32704: Power down RAM blocks 7 and above (address range 0x2001C000-0x20040000)"]
    Blk7to15 = 32704,
    #[doc = "32736: Power down RAM blocks 6 and above (address range 0x20018000-0x20040000)"]
    Blk6to15 = 32736,
    #[doc = "32752: Power down RAM blocks 5 and above (address range 0x20014000-0x20040000)"]
    Blk5to15 = 32752,
    #[doc = "32760: Power down RAM blocks 4 and above (address range 0x20010000-0x20040000)"]
    Blk4to15 = 32760,
    #[doc = "32764: Power down RAM blocks 3 and above (address range 0x2000C000-0x20040000)"]
    Blk3to15 = 32764,
    #[doc = "32766: Power down RAM blocks 2 and above (address range 0x20008000-0x20040000)"]
    Blk2to15 = 32766,
    #[doc = "32767: Power down RAM blocks 1 and above (address range 0x20004000-0x20040000)"]
    Blk1to15 = 32767,
}
impl From<Ramretnctrl> for u16 {
    #[inline(always)]
    fn from(variant: Ramretnctrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ramretnctrl {
    type Ux = u16;
}
impl crate::IsEnum for Ramretnctrl {}
#[doc = "Field `RAMRETNCTRL` reader - DMEM0 blockset retention control"]
pub type RamretnctrlR = crate::FieldReader<Ramretnctrl>;
impl RamretnctrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Ramretnctrl> {
        match self.bits {
            0 => Some(Ramretnctrl::Allon),
            16384 => Some(Ramretnctrl::Blk15),
            24576 => Some(Ramretnctrl::Blk14to15),
            28672 => Some(Ramretnctrl::Blk13to15),
            30720 => Some(Ramretnctrl::Blk12to15),
            31744 => Some(Ramretnctrl::Blk11to15),
            32256 => Some(Ramretnctrl::Blk10to15),
            32512 => Some(Ramretnctrl::Blk9to15),
            32640 => Some(Ramretnctrl::Blk8to15),
            32704 => Some(Ramretnctrl::Blk7to15),
            32736 => Some(Ramretnctrl::Blk6to15),
            32752 => Some(Ramretnctrl::Blk5to15),
            32760 => Some(Ramretnctrl::Blk4to15),
            32764 => Some(Ramretnctrl::Blk3to15),
            32766 => Some(Ramretnctrl::Blk2to15),
            32767 => Some(Ramretnctrl::Blk1to15),
            _ => None,
        }
    }
    #[doc = "None of the RAM blocks powered down"]
    #[inline(always)]
    pub fn is_allon(&self) -> bool {
        *self == Ramretnctrl::Allon
    }
    #[doc = "Power down RAM block 15 (address range 0x2003C000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk15(&self) -> bool {
        *self == Ramretnctrl::Blk15
    }
    #[doc = "Power down RAM blocks 14 and above (address range 0x20038000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk14to15(&self) -> bool {
        *self == Ramretnctrl::Blk14to15
    }
    #[doc = "Power down RAM blocks 13 and above (address range 0x20034000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk13to15(&self) -> bool {
        *self == Ramretnctrl::Blk13to15
    }
    #[doc = "Power down RAM blocks 12 and above (address range 0x20030000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk12to15(&self) -> bool {
        *self == Ramretnctrl::Blk12to15
    }
    #[doc = "Power down RAM blocks 11 and above (address range 0x2002C000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk11to15(&self) -> bool {
        *self == Ramretnctrl::Blk11to15
    }
    #[doc = "Power down RAM blocks 10 and above (address range 0x20028000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk10to15(&self) -> bool {
        *self == Ramretnctrl::Blk10to15
    }
    #[doc = "Power down RAM blocks 9 and above (address range 0x20024000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk9to15(&self) -> bool {
        *self == Ramretnctrl::Blk9to15
    }
    #[doc = "Power down RAM blocks 8 and above (address range 0x20020000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk8to15(&self) -> bool {
        *self == Ramretnctrl::Blk8to15
    }
    #[doc = "Power down RAM blocks 7 and above (address range 0x2001C000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk7to15(&self) -> bool {
        *self == Ramretnctrl::Blk7to15
    }
    #[doc = "Power down RAM blocks 6 and above (address range 0x20018000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk6to15(&self) -> bool {
        *self == Ramretnctrl::Blk6to15
    }
    #[doc = "Power down RAM blocks 5 and above (address range 0x20014000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk5to15(&self) -> bool {
        *self == Ramretnctrl::Blk5to15
    }
    #[doc = "Power down RAM blocks 4 and above (address range 0x20010000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk4to15(&self) -> bool {
        *self == Ramretnctrl::Blk4to15
    }
    #[doc = "Power down RAM blocks 3 and above (address range 0x2000C000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk3to15(&self) -> bool {
        *self == Ramretnctrl::Blk3to15
    }
    #[doc = "Power down RAM blocks 2 and above (address range 0x20008000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk2to15(&self) -> bool {
        *self == Ramretnctrl::Blk2to15
    }
    #[doc = "Power down RAM blocks 1 and above (address range 0x20004000-0x20040000)"]
    #[inline(always)]
    pub fn is_blk1to15(&self) -> bool {
        *self == Ramretnctrl::Blk1to15
    }
}
#[doc = "Field `RAMRETNCTRL` writer - DMEM0 blockset retention control"]
pub type RamretnctrlW<'a, REG> = crate::FieldWriter<'a, REG, 15, Ramretnctrl>;
impl<'a, REG> RamretnctrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u16>,
{
    #[doc = "None of the RAM blocks powered down"]
    #[inline(always)]
    pub fn allon(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Allon)
    }
    #[doc = "Power down RAM block 15 (address range 0x2003C000-0x20040000)"]
    #[inline(always)]
    pub fn blk15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk15)
    }
    #[doc = "Power down RAM blocks 14 and above (address range 0x20038000-0x20040000)"]
    #[inline(always)]
    pub fn blk14to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk14to15)
    }
    #[doc = "Power down RAM blocks 13 and above (address range 0x20034000-0x20040000)"]
    #[inline(always)]
    pub fn blk13to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk13to15)
    }
    #[doc = "Power down RAM blocks 12 and above (address range 0x20030000-0x20040000)"]
    #[inline(always)]
    pub fn blk12to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk12to15)
    }
    #[doc = "Power down RAM blocks 11 and above (address range 0x2002C000-0x20040000)"]
    #[inline(always)]
    pub fn blk11to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk11to15)
    }
    #[doc = "Power down RAM blocks 10 and above (address range 0x20028000-0x20040000)"]
    #[inline(always)]
    pub fn blk10to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk10to15)
    }
    #[doc = "Power down RAM blocks 9 and above (address range 0x20024000-0x20040000)"]
    #[inline(always)]
    pub fn blk9to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk9to15)
    }
    #[doc = "Power down RAM blocks 8 and above (address range 0x20020000-0x20040000)"]
    #[inline(always)]
    pub fn blk8to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk8to15)
    }
    #[doc = "Power down RAM blocks 7 and above (address range 0x2001C000-0x20040000)"]
    #[inline(always)]
    pub fn blk7to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk7to15)
    }
    #[doc = "Power down RAM blocks 6 and above (address range 0x20018000-0x20040000)"]
    #[inline(always)]
    pub fn blk6to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk6to15)
    }
    #[doc = "Power down RAM blocks 5 and above (address range 0x20014000-0x20040000)"]
    #[inline(always)]
    pub fn blk5to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk5to15)
    }
    #[doc = "Power down RAM blocks 4 and above (address range 0x20010000-0x20040000)"]
    #[inline(always)]
    pub fn blk4to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk4to15)
    }
    #[doc = "Power down RAM blocks 3 and above (address range 0x2000C000-0x20040000)"]
    #[inline(always)]
    pub fn blk3to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk3to15)
    }
    #[doc = "Power down RAM blocks 2 and above (address range 0x20008000-0x20040000)"]
    #[inline(always)]
    pub fn blk2to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk2to15)
    }
    #[doc = "Power down RAM blocks 1 and above (address range 0x20004000-0x20040000)"]
    #[inline(always)]
    pub fn blk1to15(self) -> &'a mut crate::W<REG> {
        self.variant(Ramretnctrl::Blk1to15)
    }
}
impl R {
    #[doc = "Bits 0:14 - DMEM0 blockset retention control"]
    #[inline(always)]
    pub fn ramretnctrl(&self) -> RamretnctrlR {
        RamretnctrlR::new((self.bits & 0x7fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:14 - DMEM0 blockset retention control"]
    #[inline(always)]
    pub fn ramretnctrl(&mut self) -> RamretnctrlW<'_, Dmem0retnctrlSpec> {
        RamretnctrlW::new(self, 0)
    }
}
#[doc = "Configure to provide general RAM retention configuration.\n\nYou can [`read`](crate::Reg::read) this register and get [`dmem0retnctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dmem0retnctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Dmem0retnctrlSpec;
impl crate::RegisterSpec for Dmem0retnctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dmem0retnctrl::R`](R) reader structure"]
impl crate::Readable for Dmem0retnctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`dmem0retnctrl::W`](W) writer structure"]
impl crate::Writable for Dmem0retnctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DMEM0RETNCTRL to value 0"]
impl crate::Resettable for Dmem0retnctrlSpec {}
