#[doc = "Register `CHIPREV` reader"]
pub type R = crate::R<ChiprevSpec>;
#[doc = "Register `CHIPREV` writer"]
pub type W = crate::W<ChiprevSpec>;
#[doc = "Field `MAJOR` reader - Chip Revision Major value"]
pub type MajorR = crate::FieldReader;
#[doc = "Field `MAJOR` writer - Chip Revision Major value"]
pub type MajorW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Chip Family value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Family {
    #[doc = "60: Product is in MG24 family"]
    Mg24 = 60,
    #[doc = "61: Product is in BG24 family"]
    Bg24 = 61,
}
impl From<Family> for u8 {
    #[inline(always)]
    fn from(variant: Family) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Family {
    type Ux = u8;
}
impl crate::IsEnum for Family {}
#[doc = "Field `FAMILY` reader - Chip Family value"]
pub type FamilyR = crate::FieldReader<Family>;
impl FamilyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Family> {
        match self.bits {
            60 => Some(Family::Mg24),
            61 => Some(Family::Bg24),
            _ => None,
        }
    }
    #[doc = "Product is in MG24 family"]
    #[inline(always)]
    pub fn is_mg24(&self) -> bool {
        *self == Family::Mg24
    }
    #[doc = "Product is in BG24 family"]
    #[inline(always)]
    pub fn is_bg24(&self) -> bool {
        *self == Family::Bg24
    }
}
#[doc = "Field `FAMILY` writer - Chip Family value"]
pub type FamilyW<'a, REG> = crate::FieldWriter<'a, REG, 6, Family>;
impl<'a, REG> FamilyW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Product is in MG24 family"]
    #[inline(always)]
    pub fn mg24(self) -> &'a mut crate::W<REG> {
        self.variant(Family::Mg24)
    }
    #[doc = "Product is in BG24 family"]
    #[inline(always)]
    pub fn bg24(self) -> &'a mut crate::W<REG> {
        self.variant(Family::Bg24)
    }
}
#[doc = "Field `MINOR` reader - Chip Revision Minor value"]
pub type MinorR = crate::FieldReader;
#[doc = "Field `MINOR` writer - Chip Revision Minor value"]
pub type MinorW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:5 - Chip Revision Major value"]
    #[inline(always)]
    pub fn major(&self) -> MajorR {
        MajorR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 6:11 - Chip Family value"]
    #[inline(always)]
    pub fn family(&self) -> FamilyR {
        FamilyR::new(((self.bits >> 6) & 0x3f) as u8)
    }
    #[doc = "Bits 12:19 - Chip Revision Minor value"]
    #[inline(always)]
    pub fn minor(&self) -> MinorR {
        MinorR::new(((self.bits >> 12) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - Chip Revision Major value"]
    #[inline(always)]
    pub fn major(&mut self) -> MajorW<'_, ChiprevSpec> {
        MajorW::new(self, 0)
    }
    #[doc = "Bits 6:11 - Chip Family value"]
    #[inline(always)]
    pub fn family(&mut self) -> FamilyW<'_, ChiprevSpec> {
        FamilyW::new(self, 6)
    }
    #[doc = "Bits 12:19 - Chip Revision Minor value"]
    #[inline(always)]
    pub fn minor(&mut self) -> MinorW<'_, ChiprevSpec> {
        MinorW::new(self, 12)
    }
}
#[doc = "Read to get the chip revision programmed by feature configuration.\n\nYou can [`read`](crate::Reg::read) this register and get [`chiprev::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`chiprev::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChiprevSpec;
impl crate::RegisterSpec for ChiprevSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`chiprev::R`](R) reader structure"]
impl crate::Readable for ChiprevSpec {}
#[doc = "`write(|w| ..)` method takes [`chiprev::W`](W) writer structure"]
impl crate::Writable for ChiprevSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CHIPREV to value 0"]
impl crate::Resettable for ChiprevSpec {}
