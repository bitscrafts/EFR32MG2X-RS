#[doc = "Register `RADIORAMRETNCTRL` reader"]
pub type R = crate::R<RadioramretnctrlSpec>;
#[doc = "Register `RADIORAMRETNCTRL` writer"]
pub type W = crate::W<RadioramretnctrlSpec>;
#[doc = "SEQRAM Retention Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Seqramretnctrl {
    #[doc = "0: SEQRAM not powered down"]
    Allon = 0,
    #[doc = "1: Power down SEQRAM block 0"]
    Blk0 = 1,
    #[doc = "2: Power down SEQRAM block 1"]
    Blk1 = 2,
    #[doc = "3: Power down all SEQRAM blocks"]
    Alloff = 3,
}
impl From<Seqramretnctrl> for u8 {
    #[inline(always)]
    fn from(variant: Seqramretnctrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Seqramretnctrl {
    type Ux = u8;
}
impl crate::IsEnum for Seqramretnctrl {}
#[doc = "Field `SEQRAMRETNCTRL` reader - SEQRAM Retention Control"]
pub type SeqramretnctrlR = crate::FieldReader<Seqramretnctrl>;
impl SeqramretnctrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Seqramretnctrl {
        match self.bits {
            0 => Seqramretnctrl::Allon,
            1 => Seqramretnctrl::Blk0,
            2 => Seqramretnctrl::Blk1,
            3 => Seqramretnctrl::Alloff,
            _ => unreachable!(),
        }
    }
    #[doc = "SEQRAM not powered down"]
    #[inline(always)]
    pub fn is_allon(&self) -> bool {
        *self == Seqramretnctrl::Allon
    }
    #[doc = "Power down SEQRAM block 0"]
    #[inline(always)]
    pub fn is_blk0(&self) -> bool {
        *self == Seqramretnctrl::Blk0
    }
    #[doc = "Power down SEQRAM block 1"]
    #[inline(always)]
    pub fn is_blk1(&self) -> bool {
        *self == Seqramretnctrl::Blk1
    }
    #[doc = "Power down all SEQRAM blocks"]
    #[inline(always)]
    pub fn is_alloff(&self) -> bool {
        *self == Seqramretnctrl::Alloff
    }
}
#[doc = "Field `SEQRAMRETNCTRL` writer - SEQRAM Retention Control"]
pub type SeqramretnctrlW<'a, REG> = crate::FieldWriter<'a, REG, 2, Seqramretnctrl, crate::Safe>;
impl<'a, REG> SeqramretnctrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "SEQRAM not powered down"]
    #[inline(always)]
    pub fn allon(self) -> &'a mut crate::W<REG> {
        self.variant(Seqramretnctrl::Allon)
    }
    #[doc = "Power down SEQRAM block 0"]
    #[inline(always)]
    pub fn blk0(self) -> &'a mut crate::W<REG> {
        self.variant(Seqramretnctrl::Blk0)
    }
    #[doc = "Power down SEQRAM block 1"]
    #[inline(always)]
    pub fn blk1(self) -> &'a mut crate::W<REG> {
        self.variant(Seqramretnctrl::Blk1)
    }
    #[doc = "Power down all SEQRAM blocks"]
    #[inline(always)]
    pub fn alloff(self) -> &'a mut crate::W<REG> {
        self.variant(Seqramretnctrl::Alloff)
    }
}
#[doc = "FRCRAM Retention Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frcramretnctrl {
    #[doc = "0: FRCRAM not powered down"]
    Allon = 0,
    #[doc = "1: Power down FRCRAM"]
    Alloff = 1,
}
impl From<Frcramretnctrl> for bool {
    #[inline(always)]
    fn from(variant: Frcramretnctrl) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRCRAMRETNCTRL` reader - FRCRAM Retention Control"]
pub type FrcramretnctrlR = crate::BitReader<Frcramretnctrl>;
impl FrcramretnctrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Frcramretnctrl {
        match self.bits {
            false => Frcramretnctrl::Allon,
            true => Frcramretnctrl::Alloff,
        }
    }
    #[doc = "FRCRAM not powered down"]
    #[inline(always)]
    pub fn is_allon(&self) -> bool {
        *self == Frcramretnctrl::Allon
    }
    #[doc = "Power down FRCRAM"]
    #[inline(always)]
    pub fn is_alloff(&self) -> bool {
        *self == Frcramretnctrl::Alloff
    }
}
#[doc = "Field `FRCRAMRETNCTRL` writer - FRCRAM Retention Control"]
pub type FrcramretnctrlW<'a, REG> = crate::BitWriter<'a, REG, Frcramretnctrl>;
impl<'a, REG> FrcramretnctrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FRCRAM not powered down"]
    #[inline(always)]
    pub fn allon(self) -> &'a mut crate::W<REG> {
        self.variant(Frcramretnctrl::Allon)
    }
    #[doc = "Power down FRCRAM"]
    #[inline(always)]
    pub fn alloff(self) -> &'a mut crate::W<REG> {
        self.variant(Frcramretnctrl::Alloff)
    }
}
impl R {
    #[doc = "Bits 0:1 - SEQRAM Retention Control"]
    #[inline(always)]
    pub fn seqramretnctrl(&self) -> SeqramretnctrlR {
        SeqramretnctrlR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 8 - FRCRAM Retention Control"]
    #[inline(always)]
    pub fn frcramretnctrl(&self) -> FrcramretnctrlR {
        FrcramretnctrlR::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - SEQRAM Retention Control"]
    #[inline(always)]
    pub fn seqramretnctrl(&mut self) -> SeqramretnctrlW<'_, RadioramretnctrlSpec> {
        SeqramretnctrlW::new(self, 0)
    }
    #[doc = "Bit 8 - FRCRAM Retention Control"]
    #[inline(always)]
    pub fn frcramretnctrl(&mut self) -> FrcramretnctrlW<'_, RadioramretnctrlSpec> {
        FrcramretnctrlW::new(self, 8)
    }
}
#[doc = "Configure SEQRAM Retention controls.\n\nYou can [`read`](crate::Reg::read) this register and get [`radioramretnctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`radioramretnctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RadioramretnctrlSpec;
impl crate::RegisterSpec for RadioramretnctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`radioramretnctrl::R`](R) reader structure"]
impl crate::Readable for RadioramretnctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`radioramretnctrl::W`](W) writer structure"]
impl crate::Writable for RadioramretnctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RADIORAMRETNCTRL to value 0"]
impl crate::Resettable for RadioramretnctrlSpec {}
