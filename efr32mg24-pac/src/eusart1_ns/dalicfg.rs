#[doc = "Register `DALICFG` reader"]
pub type R = crate::R<DalicfgSpec>;
#[doc = "Register `DALICFG` writer"]
pub type W = crate::W<DalicfgSpec>;
#[doc = "Field `DALIEN` reader - DALI Enable Bit"]
pub type DalienR = crate::BitReader;
#[doc = "Field `DALIEN` writer - DALI Enable Bit"]
pub type DalienW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "DALI TX Databits\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dalitxdatabits {
    #[doc = "0: Each frame contains 8 data bits"]
    Eight = 0,
    #[doc = "1: Each frame contains 9 data bits"]
    Nine = 1,
    #[doc = "2: Each frame contains 10 data bits"]
    Ten = 2,
    #[doc = "3: Each frame contains 11 data bits"]
    Eleven = 3,
    #[doc = "4: Each frame contains 12 data bits"]
    Twelve = 4,
    #[doc = "5: Each frame contains 13 data bits"]
    Thirteen = 5,
    #[doc = "6: Each frame contains 14 data bits"]
    Fourteen = 6,
    #[doc = "7: Each frame contains 15 data bits"]
    Fifteen = 7,
    #[doc = "8: Each frame contains 16 data bits"]
    Sixteen = 8,
    #[doc = "9: Each frame contains 17 data bits"]
    Seventeen = 9,
    #[doc = "10: Each frame contains 18 data bits"]
    Eighteen = 10,
    #[doc = "11: Each frame contains 19 data bits"]
    Nineteen = 11,
    #[doc = "12: Each frame contains 20 data bits"]
    Twenty = 12,
    #[doc = "13: Each frame contains 21 data bits"]
    Twentyone = 13,
    #[doc = "14: Each frame contains 22 data bits"]
    Twentytwo = 14,
    #[doc = "15: Each frame contains 23 data bits"]
    Twentythree = 15,
    #[doc = "16: Each frame contains 24 data bits"]
    Twentyfour = 16,
    #[doc = "17: Each frame contains 25 data bits"]
    Twentyfive = 17,
    #[doc = "18: Each frame contains 26 data bits"]
    Twentysix = 18,
    #[doc = "19: Each frame contains 27 data bits"]
    Twentyseven = 19,
    #[doc = "20: Each frame contains 28 data bits"]
    Twentyeight = 20,
    #[doc = "21: Each frame contains 29 data bits"]
    Twentynine = 21,
    #[doc = "22: Each frame contains 30 data bits"]
    Thirty = 22,
    #[doc = "23: Each frame contains 31 data bits"]
    Thirtyone = 23,
    #[doc = "24: Each frame contains 32 data bits"]
    Thirtytwo = 24,
}
impl From<Dalitxdatabits> for u8 {
    #[inline(always)]
    fn from(variant: Dalitxdatabits) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Dalitxdatabits {
    type Ux = u8;
}
impl crate::IsEnum for Dalitxdatabits {}
#[doc = "Field `DALITXDATABITS` reader - DALI TX Databits"]
pub type DalitxdatabitsR = crate::FieldReader<Dalitxdatabits>;
impl DalitxdatabitsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Dalitxdatabits> {
        match self.bits {
            0 => Some(Dalitxdatabits::Eight),
            1 => Some(Dalitxdatabits::Nine),
            2 => Some(Dalitxdatabits::Ten),
            3 => Some(Dalitxdatabits::Eleven),
            4 => Some(Dalitxdatabits::Twelve),
            5 => Some(Dalitxdatabits::Thirteen),
            6 => Some(Dalitxdatabits::Fourteen),
            7 => Some(Dalitxdatabits::Fifteen),
            8 => Some(Dalitxdatabits::Sixteen),
            9 => Some(Dalitxdatabits::Seventeen),
            10 => Some(Dalitxdatabits::Eighteen),
            11 => Some(Dalitxdatabits::Nineteen),
            12 => Some(Dalitxdatabits::Twenty),
            13 => Some(Dalitxdatabits::Twentyone),
            14 => Some(Dalitxdatabits::Twentytwo),
            15 => Some(Dalitxdatabits::Twentythree),
            16 => Some(Dalitxdatabits::Twentyfour),
            17 => Some(Dalitxdatabits::Twentyfive),
            18 => Some(Dalitxdatabits::Twentysix),
            19 => Some(Dalitxdatabits::Twentyseven),
            20 => Some(Dalitxdatabits::Twentyeight),
            21 => Some(Dalitxdatabits::Twentynine),
            22 => Some(Dalitxdatabits::Thirty),
            23 => Some(Dalitxdatabits::Thirtyone),
            24 => Some(Dalitxdatabits::Thirtytwo),
            _ => None,
        }
    }
    #[doc = "Each frame contains 8 data bits"]
    #[inline(always)]
    pub fn is_eight(&self) -> bool {
        *self == Dalitxdatabits::Eight
    }
    #[doc = "Each frame contains 9 data bits"]
    #[inline(always)]
    pub fn is_nine(&self) -> bool {
        *self == Dalitxdatabits::Nine
    }
    #[doc = "Each frame contains 10 data bits"]
    #[inline(always)]
    pub fn is_ten(&self) -> bool {
        *self == Dalitxdatabits::Ten
    }
    #[doc = "Each frame contains 11 data bits"]
    #[inline(always)]
    pub fn is_eleven(&self) -> bool {
        *self == Dalitxdatabits::Eleven
    }
    #[doc = "Each frame contains 12 data bits"]
    #[inline(always)]
    pub fn is_twelve(&self) -> bool {
        *self == Dalitxdatabits::Twelve
    }
    #[doc = "Each frame contains 13 data bits"]
    #[inline(always)]
    pub fn is_thirteen(&self) -> bool {
        *self == Dalitxdatabits::Thirteen
    }
    #[doc = "Each frame contains 14 data bits"]
    #[inline(always)]
    pub fn is_fourteen(&self) -> bool {
        *self == Dalitxdatabits::Fourteen
    }
    #[doc = "Each frame contains 15 data bits"]
    #[inline(always)]
    pub fn is_fifteen(&self) -> bool {
        *self == Dalitxdatabits::Fifteen
    }
    #[doc = "Each frame contains 16 data bits"]
    #[inline(always)]
    pub fn is_sixteen(&self) -> bool {
        *self == Dalitxdatabits::Sixteen
    }
    #[doc = "Each frame contains 17 data bits"]
    #[inline(always)]
    pub fn is_seventeen(&self) -> bool {
        *self == Dalitxdatabits::Seventeen
    }
    #[doc = "Each frame contains 18 data bits"]
    #[inline(always)]
    pub fn is_eighteen(&self) -> bool {
        *self == Dalitxdatabits::Eighteen
    }
    #[doc = "Each frame contains 19 data bits"]
    #[inline(always)]
    pub fn is_nineteen(&self) -> bool {
        *self == Dalitxdatabits::Nineteen
    }
    #[doc = "Each frame contains 20 data bits"]
    #[inline(always)]
    pub fn is_twenty(&self) -> bool {
        *self == Dalitxdatabits::Twenty
    }
    #[doc = "Each frame contains 21 data bits"]
    #[inline(always)]
    pub fn is_twentyone(&self) -> bool {
        *self == Dalitxdatabits::Twentyone
    }
    #[doc = "Each frame contains 22 data bits"]
    #[inline(always)]
    pub fn is_twentytwo(&self) -> bool {
        *self == Dalitxdatabits::Twentytwo
    }
    #[doc = "Each frame contains 23 data bits"]
    #[inline(always)]
    pub fn is_twentythree(&self) -> bool {
        *self == Dalitxdatabits::Twentythree
    }
    #[doc = "Each frame contains 24 data bits"]
    #[inline(always)]
    pub fn is_twentyfour(&self) -> bool {
        *self == Dalitxdatabits::Twentyfour
    }
    #[doc = "Each frame contains 25 data bits"]
    #[inline(always)]
    pub fn is_twentyfive(&self) -> bool {
        *self == Dalitxdatabits::Twentyfive
    }
    #[doc = "Each frame contains 26 data bits"]
    #[inline(always)]
    pub fn is_twentysix(&self) -> bool {
        *self == Dalitxdatabits::Twentysix
    }
    #[doc = "Each frame contains 27 data bits"]
    #[inline(always)]
    pub fn is_twentyseven(&self) -> bool {
        *self == Dalitxdatabits::Twentyseven
    }
    #[doc = "Each frame contains 28 data bits"]
    #[inline(always)]
    pub fn is_twentyeight(&self) -> bool {
        *self == Dalitxdatabits::Twentyeight
    }
    #[doc = "Each frame contains 29 data bits"]
    #[inline(always)]
    pub fn is_twentynine(&self) -> bool {
        *self == Dalitxdatabits::Twentynine
    }
    #[doc = "Each frame contains 30 data bits"]
    #[inline(always)]
    pub fn is_thirty(&self) -> bool {
        *self == Dalitxdatabits::Thirty
    }
    #[doc = "Each frame contains 31 data bits"]
    #[inline(always)]
    pub fn is_thirtyone(&self) -> bool {
        *self == Dalitxdatabits::Thirtyone
    }
    #[doc = "Each frame contains 32 data bits"]
    #[inline(always)]
    pub fn is_thirtytwo(&self) -> bool {
        *self == Dalitxdatabits::Thirtytwo
    }
}
#[doc = "Field `DALITXDATABITS` writer - DALI TX Databits"]
pub type DalitxdatabitsW<'a, REG> = crate::FieldWriter<'a, REG, 5, Dalitxdatabits>;
impl<'a, REG> DalitxdatabitsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Each frame contains 8 data bits"]
    #[inline(always)]
    pub fn eight(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Eight)
    }
    #[doc = "Each frame contains 9 data bits"]
    #[inline(always)]
    pub fn nine(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Nine)
    }
    #[doc = "Each frame contains 10 data bits"]
    #[inline(always)]
    pub fn ten(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Ten)
    }
    #[doc = "Each frame contains 11 data bits"]
    #[inline(always)]
    pub fn eleven(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Eleven)
    }
    #[doc = "Each frame contains 12 data bits"]
    #[inline(always)]
    pub fn twelve(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twelve)
    }
    #[doc = "Each frame contains 13 data bits"]
    #[inline(always)]
    pub fn thirteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Thirteen)
    }
    #[doc = "Each frame contains 14 data bits"]
    #[inline(always)]
    pub fn fourteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Fourteen)
    }
    #[doc = "Each frame contains 15 data bits"]
    #[inline(always)]
    pub fn fifteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Fifteen)
    }
    #[doc = "Each frame contains 16 data bits"]
    #[inline(always)]
    pub fn sixteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Sixteen)
    }
    #[doc = "Each frame contains 17 data bits"]
    #[inline(always)]
    pub fn seventeen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Seventeen)
    }
    #[doc = "Each frame contains 18 data bits"]
    #[inline(always)]
    pub fn eighteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Eighteen)
    }
    #[doc = "Each frame contains 19 data bits"]
    #[inline(always)]
    pub fn nineteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Nineteen)
    }
    #[doc = "Each frame contains 20 data bits"]
    #[inline(always)]
    pub fn twenty(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twenty)
    }
    #[doc = "Each frame contains 21 data bits"]
    #[inline(always)]
    pub fn twentyone(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentyone)
    }
    #[doc = "Each frame contains 22 data bits"]
    #[inline(always)]
    pub fn twentytwo(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentytwo)
    }
    #[doc = "Each frame contains 23 data bits"]
    #[inline(always)]
    pub fn twentythree(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentythree)
    }
    #[doc = "Each frame contains 24 data bits"]
    #[inline(always)]
    pub fn twentyfour(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentyfour)
    }
    #[doc = "Each frame contains 25 data bits"]
    #[inline(always)]
    pub fn twentyfive(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentyfive)
    }
    #[doc = "Each frame contains 26 data bits"]
    #[inline(always)]
    pub fn twentysix(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentysix)
    }
    #[doc = "Each frame contains 27 data bits"]
    #[inline(always)]
    pub fn twentyseven(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentyseven)
    }
    #[doc = "Each frame contains 28 data bits"]
    #[inline(always)]
    pub fn twentyeight(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentyeight)
    }
    #[doc = "Each frame contains 29 data bits"]
    #[inline(always)]
    pub fn twentynine(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Twentynine)
    }
    #[doc = "Each frame contains 30 data bits"]
    #[inline(always)]
    pub fn thirty(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Thirty)
    }
    #[doc = "Each frame contains 31 data bits"]
    #[inline(always)]
    pub fn thirtyone(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Thirtyone)
    }
    #[doc = "Each frame contains 32 data bits"]
    #[inline(always)]
    pub fn thirtytwo(self) -> &'a mut crate::W<REG> {
        self.variant(Dalitxdatabits::Thirtytwo)
    }
}
#[doc = "DALI RX Databits\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dalirxdatabits {
    #[doc = "0: Each frame contains 8 data bits"]
    Eight = 0,
    #[doc = "1: Each frame contains 9 data bits"]
    Nine = 1,
    #[doc = "2: Each frame contains 10 data bits"]
    Ten = 2,
    #[doc = "3: Each frame contains 11 data bits"]
    Eleven = 3,
    #[doc = "4: Each frame contains 12 data bits"]
    Twelve = 4,
    #[doc = "5: Each frame contains 13 data bits"]
    Thirteen = 5,
    #[doc = "6: Each frame contains 14 data bits"]
    Fourteen = 6,
    #[doc = "7: Each frame contains 15 data bits"]
    Fifteen = 7,
    #[doc = "8: Each frame contains 16 data bits"]
    Sixteen = 8,
    #[doc = "9: Each frame contains 17 data bits"]
    Seventeen = 9,
    #[doc = "10: Each frame contains 18 data bits"]
    Eighteen = 10,
    #[doc = "11: Each frame contains 19 data bits"]
    Nineteen = 11,
    #[doc = "12: Each frame contains 20 data bits"]
    Twenty = 12,
    #[doc = "13: Each frame contains 21 data bits"]
    Twentyone = 13,
    #[doc = "14: Each frame contains 22 data bits"]
    Twentytwo = 14,
    #[doc = "15: Each frame contains 23 data bits"]
    Twentythree = 15,
    #[doc = "16: Each frame contains 24 data bits"]
    Twentyfour = 16,
    #[doc = "17: Each frame contains 25 data bits"]
    Twentyfive = 17,
    #[doc = "18: Each frame contains 26 data bits"]
    Twentysix = 18,
    #[doc = "19: Each frame contains 27 data bits"]
    Twentyseven = 19,
    #[doc = "20: Each frame contains 28 data bits"]
    Twentyeight = 20,
    #[doc = "21: Each frame contains 29 data bits"]
    Twentynine = 21,
    #[doc = "22: Each frame contains 30 data bits"]
    Thirty = 22,
    #[doc = "23: Each frame contains 31 data bits"]
    Thirtyone = 23,
    #[doc = "24: Each frame contains 32 data bits"]
    Thirtytwo = 24,
}
impl From<Dalirxdatabits> for u8 {
    #[inline(always)]
    fn from(variant: Dalirxdatabits) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Dalirxdatabits {
    type Ux = u8;
}
impl crate::IsEnum for Dalirxdatabits {}
#[doc = "Field `DALIRXDATABITS` reader - DALI RX Databits"]
pub type DalirxdatabitsR = crate::FieldReader<Dalirxdatabits>;
impl DalirxdatabitsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Dalirxdatabits> {
        match self.bits {
            0 => Some(Dalirxdatabits::Eight),
            1 => Some(Dalirxdatabits::Nine),
            2 => Some(Dalirxdatabits::Ten),
            3 => Some(Dalirxdatabits::Eleven),
            4 => Some(Dalirxdatabits::Twelve),
            5 => Some(Dalirxdatabits::Thirteen),
            6 => Some(Dalirxdatabits::Fourteen),
            7 => Some(Dalirxdatabits::Fifteen),
            8 => Some(Dalirxdatabits::Sixteen),
            9 => Some(Dalirxdatabits::Seventeen),
            10 => Some(Dalirxdatabits::Eighteen),
            11 => Some(Dalirxdatabits::Nineteen),
            12 => Some(Dalirxdatabits::Twenty),
            13 => Some(Dalirxdatabits::Twentyone),
            14 => Some(Dalirxdatabits::Twentytwo),
            15 => Some(Dalirxdatabits::Twentythree),
            16 => Some(Dalirxdatabits::Twentyfour),
            17 => Some(Dalirxdatabits::Twentyfive),
            18 => Some(Dalirxdatabits::Twentysix),
            19 => Some(Dalirxdatabits::Twentyseven),
            20 => Some(Dalirxdatabits::Twentyeight),
            21 => Some(Dalirxdatabits::Twentynine),
            22 => Some(Dalirxdatabits::Thirty),
            23 => Some(Dalirxdatabits::Thirtyone),
            24 => Some(Dalirxdatabits::Thirtytwo),
            _ => None,
        }
    }
    #[doc = "Each frame contains 8 data bits"]
    #[inline(always)]
    pub fn is_eight(&self) -> bool {
        *self == Dalirxdatabits::Eight
    }
    #[doc = "Each frame contains 9 data bits"]
    #[inline(always)]
    pub fn is_nine(&self) -> bool {
        *self == Dalirxdatabits::Nine
    }
    #[doc = "Each frame contains 10 data bits"]
    #[inline(always)]
    pub fn is_ten(&self) -> bool {
        *self == Dalirxdatabits::Ten
    }
    #[doc = "Each frame contains 11 data bits"]
    #[inline(always)]
    pub fn is_eleven(&self) -> bool {
        *self == Dalirxdatabits::Eleven
    }
    #[doc = "Each frame contains 12 data bits"]
    #[inline(always)]
    pub fn is_twelve(&self) -> bool {
        *self == Dalirxdatabits::Twelve
    }
    #[doc = "Each frame contains 13 data bits"]
    #[inline(always)]
    pub fn is_thirteen(&self) -> bool {
        *self == Dalirxdatabits::Thirteen
    }
    #[doc = "Each frame contains 14 data bits"]
    #[inline(always)]
    pub fn is_fourteen(&self) -> bool {
        *self == Dalirxdatabits::Fourteen
    }
    #[doc = "Each frame contains 15 data bits"]
    #[inline(always)]
    pub fn is_fifteen(&self) -> bool {
        *self == Dalirxdatabits::Fifteen
    }
    #[doc = "Each frame contains 16 data bits"]
    #[inline(always)]
    pub fn is_sixteen(&self) -> bool {
        *self == Dalirxdatabits::Sixteen
    }
    #[doc = "Each frame contains 17 data bits"]
    #[inline(always)]
    pub fn is_seventeen(&self) -> bool {
        *self == Dalirxdatabits::Seventeen
    }
    #[doc = "Each frame contains 18 data bits"]
    #[inline(always)]
    pub fn is_eighteen(&self) -> bool {
        *self == Dalirxdatabits::Eighteen
    }
    #[doc = "Each frame contains 19 data bits"]
    #[inline(always)]
    pub fn is_nineteen(&self) -> bool {
        *self == Dalirxdatabits::Nineteen
    }
    #[doc = "Each frame contains 20 data bits"]
    #[inline(always)]
    pub fn is_twenty(&self) -> bool {
        *self == Dalirxdatabits::Twenty
    }
    #[doc = "Each frame contains 21 data bits"]
    #[inline(always)]
    pub fn is_twentyone(&self) -> bool {
        *self == Dalirxdatabits::Twentyone
    }
    #[doc = "Each frame contains 22 data bits"]
    #[inline(always)]
    pub fn is_twentytwo(&self) -> bool {
        *self == Dalirxdatabits::Twentytwo
    }
    #[doc = "Each frame contains 23 data bits"]
    #[inline(always)]
    pub fn is_twentythree(&self) -> bool {
        *self == Dalirxdatabits::Twentythree
    }
    #[doc = "Each frame contains 24 data bits"]
    #[inline(always)]
    pub fn is_twentyfour(&self) -> bool {
        *self == Dalirxdatabits::Twentyfour
    }
    #[doc = "Each frame contains 25 data bits"]
    #[inline(always)]
    pub fn is_twentyfive(&self) -> bool {
        *self == Dalirxdatabits::Twentyfive
    }
    #[doc = "Each frame contains 26 data bits"]
    #[inline(always)]
    pub fn is_twentysix(&self) -> bool {
        *self == Dalirxdatabits::Twentysix
    }
    #[doc = "Each frame contains 27 data bits"]
    #[inline(always)]
    pub fn is_twentyseven(&self) -> bool {
        *self == Dalirxdatabits::Twentyseven
    }
    #[doc = "Each frame contains 28 data bits"]
    #[inline(always)]
    pub fn is_twentyeight(&self) -> bool {
        *self == Dalirxdatabits::Twentyeight
    }
    #[doc = "Each frame contains 29 data bits"]
    #[inline(always)]
    pub fn is_twentynine(&self) -> bool {
        *self == Dalirxdatabits::Twentynine
    }
    #[doc = "Each frame contains 30 data bits"]
    #[inline(always)]
    pub fn is_thirty(&self) -> bool {
        *self == Dalirxdatabits::Thirty
    }
    #[doc = "Each frame contains 31 data bits"]
    #[inline(always)]
    pub fn is_thirtyone(&self) -> bool {
        *self == Dalirxdatabits::Thirtyone
    }
    #[doc = "Each frame contains 32 data bits"]
    #[inline(always)]
    pub fn is_thirtytwo(&self) -> bool {
        *self == Dalirxdatabits::Thirtytwo
    }
}
#[doc = "Field `DALIRXDATABITS` writer - DALI RX Databits"]
pub type DalirxdatabitsW<'a, REG> = crate::FieldWriter<'a, REG, 5, Dalirxdatabits>;
impl<'a, REG> DalirxdatabitsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Each frame contains 8 data bits"]
    #[inline(always)]
    pub fn eight(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Eight)
    }
    #[doc = "Each frame contains 9 data bits"]
    #[inline(always)]
    pub fn nine(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Nine)
    }
    #[doc = "Each frame contains 10 data bits"]
    #[inline(always)]
    pub fn ten(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Ten)
    }
    #[doc = "Each frame contains 11 data bits"]
    #[inline(always)]
    pub fn eleven(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Eleven)
    }
    #[doc = "Each frame contains 12 data bits"]
    #[inline(always)]
    pub fn twelve(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twelve)
    }
    #[doc = "Each frame contains 13 data bits"]
    #[inline(always)]
    pub fn thirteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Thirteen)
    }
    #[doc = "Each frame contains 14 data bits"]
    #[inline(always)]
    pub fn fourteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Fourteen)
    }
    #[doc = "Each frame contains 15 data bits"]
    #[inline(always)]
    pub fn fifteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Fifteen)
    }
    #[doc = "Each frame contains 16 data bits"]
    #[inline(always)]
    pub fn sixteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Sixteen)
    }
    #[doc = "Each frame contains 17 data bits"]
    #[inline(always)]
    pub fn seventeen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Seventeen)
    }
    #[doc = "Each frame contains 18 data bits"]
    #[inline(always)]
    pub fn eighteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Eighteen)
    }
    #[doc = "Each frame contains 19 data bits"]
    #[inline(always)]
    pub fn nineteen(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Nineteen)
    }
    #[doc = "Each frame contains 20 data bits"]
    #[inline(always)]
    pub fn twenty(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twenty)
    }
    #[doc = "Each frame contains 21 data bits"]
    #[inline(always)]
    pub fn twentyone(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentyone)
    }
    #[doc = "Each frame contains 22 data bits"]
    #[inline(always)]
    pub fn twentytwo(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentytwo)
    }
    #[doc = "Each frame contains 23 data bits"]
    #[inline(always)]
    pub fn twentythree(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentythree)
    }
    #[doc = "Each frame contains 24 data bits"]
    #[inline(always)]
    pub fn twentyfour(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentyfour)
    }
    #[doc = "Each frame contains 25 data bits"]
    #[inline(always)]
    pub fn twentyfive(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentyfive)
    }
    #[doc = "Each frame contains 26 data bits"]
    #[inline(always)]
    pub fn twentysix(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentysix)
    }
    #[doc = "Each frame contains 27 data bits"]
    #[inline(always)]
    pub fn twentyseven(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentyseven)
    }
    #[doc = "Each frame contains 28 data bits"]
    #[inline(always)]
    pub fn twentyeight(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentyeight)
    }
    #[doc = "Each frame contains 29 data bits"]
    #[inline(always)]
    pub fn twentynine(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Twentynine)
    }
    #[doc = "Each frame contains 30 data bits"]
    #[inline(always)]
    pub fn thirty(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Thirty)
    }
    #[doc = "Each frame contains 31 data bits"]
    #[inline(always)]
    pub fn thirtyone(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Thirtyone)
    }
    #[doc = "Each frame contains 32 data bits"]
    #[inline(always)]
    pub fn thirtytwo(self) -> &'a mut crate::W<REG> {
        self.variant(Dalirxdatabits::Thirtytwo)
    }
}
#[doc = "Field `DALIRXENDT` reader - DALI RX Enabled During Transmission"]
pub type DalirxendtR = crate::BitReader;
#[doc = "Field `DALIRXENDT` writer - DALI RX Enabled During Transmission"]
pub type DalirxendtW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - DALI Enable Bit"]
    #[inline(always)]
    pub fn dalien(&self) -> DalienR {
        DalienR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 1:5 - DALI TX Databits"]
    #[inline(always)]
    pub fn dalitxdatabits(&self) -> DalitxdatabitsR {
        DalitxdatabitsR::new(((self.bits >> 1) & 0x1f) as u8)
    }
    #[doc = "Bits 8:12 - DALI RX Databits"]
    #[inline(always)]
    pub fn dalirxdatabits(&self) -> DalirxdatabitsR {
        DalirxdatabitsR::new(((self.bits >> 8) & 0x1f) as u8)
    }
    #[doc = "Bit 15 - DALI RX Enabled During Transmission"]
    #[inline(always)]
    pub fn dalirxendt(&self) -> DalirxendtR {
        DalirxendtR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - DALI Enable Bit"]
    #[inline(always)]
    pub fn dalien(&mut self) -> DalienW<'_, DalicfgSpec> {
        DalienW::new(self, 0)
    }
    #[doc = "Bits 1:5 - DALI TX Databits"]
    #[inline(always)]
    pub fn dalitxdatabits(&mut self) -> DalitxdatabitsW<'_, DalicfgSpec> {
        DalitxdatabitsW::new(self, 1)
    }
    #[doc = "Bits 8:12 - DALI RX Databits"]
    #[inline(always)]
    pub fn dalirxdatabits(&mut self) -> DalirxdatabitsW<'_, DalicfgSpec> {
        DalirxdatabitsW::new(self, 8)
    }
    #[doc = "Bit 15 - DALI RX Enabled During Transmission"]
    #[inline(always)]
    pub fn dalirxendt(&mut self) -> DalirxendtW<'_, DalicfgSpec> {
        DalirxendtW::new(self, 15)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`dalicfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dalicfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DalicfgSpec;
impl crate::RegisterSpec for DalicfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dalicfg::R`](R) reader structure"]
impl crate::Readable for DalicfgSpec {}
#[doc = "`write(|w| ..)` method takes [`dalicfg::W`](W) writer structure"]
impl crate::Writable for DalicfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DALICFG to value 0"]
impl crate::Resettable for DalicfgSpec {}
