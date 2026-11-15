#[doc = "Register `TRACECLKCTRL` reader"]
pub type R = crate::R<TraceclkctrlSpec>;
#[doc = "Register `TRACECLKCTRL` writer"]
pub type W = crate::W<TraceclkctrlSpec>;
#[doc = "Clock Select\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Clksel {
    #[doc = "0: TRACE clock is disable"]
    Disable = 0,
    #[doc = "1: SYSCLK is driving TRACE"]
    Sysclk = 1,
    #[doc = "2: HFRCOEM23 is driving TRACE"]
    Hfrcoem23 = 2,
    #[doc = "3: HFRCODPLLRT is driving TRACE"]
    Hfrcodpllrt = 3,
}
impl From<Clksel> for u8 {
    #[inline(always)]
    fn from(variant: Clksel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Clksel {
    type Ux = u8;
}
impl crate::IsEnum for Clksel {}
#[doc = "Field `CLKSEL` reader - Clock Select"]
pub type ClkselR = crate::FieldReader<Clksel>;
impl ClkselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Clksel {
        match self.bits {
            0 => Clksel::Disable,
            1 => Clksel::Sysclk,
            2 => Clksel::Hfrcoem23,
            3 => Clksel::Hfrcodpllrt,
            _ => unreachable!(),
        }
    }
    #[doc = "TRACE clock is disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Clksel::Disable
    }
    #[doc = "SYSCLK is driving TRACE"]
    #[inline(always)]
    pub fn is_sysclk(&self) -> bool {
        *self == Clksel::Sysclk
    }
    #[doc = "HFRCOEM23 is driving TRACE"]
    #[inline(always)]
    pub fn is_hfrcoem23(&self) -> bool {
        *self == Clksel::Hfrcoem23
    }
    #[doc = "HFRCODPLLRT is driving TRACE"]
    #[inline(always)]
    pub fn is_hfrcodpllrt(&self) -> bool {
        *self == Clksel::Hfrcodpllrt
    }
}
#[doc = "Field `CLKSEL` writer - Clock Select"]
pub type ClkselW<'a, REG> = crate::FieldWriter<'a, REG, 2, Clksel, crate::Safe>;
impl<'a, REG> ClkselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "TRACE clock is disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Clksel::Disable)
    }
    #[doc = "SYSCLK is driving TRACE"]
    #[inline(always)]
    pub fn sysclk(self) -> &'a mut crate::W<REG> {
        self.variant(Clksel::Sysclk)
    }
    #[doc = "HFRCOEM23 is driving TRACE"]
    #[inline(always)]
    pub fn hfrcoem23(self) -> &'a mut crate::W<REG> {
        self.variant(Clksel::Hfrcoem23)
    }
    #[doc = "HFRCODPLLRT is driving TRACE"]
    #[inline(always)]
    pub fn hfrcodpllrt(self) -> &'a mut crate::W<REG> {
        self.variant(Clksel::Hfrcodpllrt)
    }
}
#[doc = "TRACECLK Prescaler\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Presc {
    #[doc = "0: TRACECLK is SYSCLK divided by 1"]
    Div1 = 0,
    #[doc = "1: TRACECLK is SYSCLK divided by 2"]
    Div2 = 1,
    #[doc = "2: TRACECLK is SYSCLK divided by 3"]
    Div3 = 2,
    #[doc = "3: TRACECLK is SYSCLK divided by 4"]
    Div4 = 3,
}
impl From<Presc> for u8 {
    #[inline(always)]
    fn from(variant: Presc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Presc {
    type Ux = u8;
}
impl crate::IsEnum for Presc {}
#[doc = "Field `PRESC` reader - TRACECLK Prescaler"]
pub type PrescR = crate::FieldReader<Presc>;
impl PrescR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Presc {
        match self.bits {
            0 => Presc::Div1,
            1 => Presc::Div2,
            2 => Presc::Div3,
            3 => Presc::Div4,
            _ => unreachable!(),
        }
    }
    #[doc = "TRACECLK is SYSCLK divided by 1"]
    #[inline(always)]
    pub fn is_div1(&self) -> bool {
        *self == Presc::Div1
    }
    #[doc = "TRACECLK is SYSCLK divided by 2"]
    #[inline(always)]
    pub fn is_div2(&self) -> bool {
        *self == Presc::Div2
    }
    #[doc = "TRACECLK is SYSCLK divided by 3"]
    #[inline(always)]
    pub fn is_div3(&self) -> bool {
        *self == Presc::Div3
    }
    #[doc = "TRACECLK is SYSCLK divided by 4"]
    #[inline(always)]
    pub fn is_div4(&self) -> bool {
        *self == Presc::Div4
    }
}
#[doc = "Field `PRESC` writer - TRACECLK Prescaler"]
pub type PrescW<'a, REG> = crate::FieldWriter<'a, REG, 2, Presc, crate::Safe>;
impl<'a, REG> PrescW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "TRACECLK is SYSCLK divided by 1"]
    #[inline(always)]
    pub fn div1(self) -> &'a mut crate::W<REG> {
        self.variant(Presc::Div1)
    }
    #[doc = "TRACECLK is SYSCLK divided by 2"]
    #[inline(always)]
    pub fn div2(self) -> &'a mut crate::W<REG> {
        self.variant(Presc::Div2)
    }
    #[doc = "TRACECLK is SYSCLK divided by 3"]
    #[inline(always)]
    pub fn div3(self) -> &'a mut crate::W<REG> {
        self.variant(Presc::Div3)
    }
    #[doc = "TRACECLK is SYSCLK divided by 4"]
    #[inline(always)]
    pub fn div4(self) -> &'a mut crate::W<REG> {
        self.variant(Presc::Div4)
    }
}
impl R {
    #[doc = "Bits 0:1 - Clock Select"]
    #[inline(always)]
    pub fn clksel(&self) -> ClkselR {
        ClkselR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 4:5 - TRACECLK Prescaler"]
    #[inline(always)]
    pub fn presc(&self) -> PrescR {
        PrescR::new(((self.bits >> 4) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Clock Select"]
    #[inline(always)]
    pub fn clksel(&mut self) -> ClkselW<'_, TraceclkctrlSpec> {
        ClkselW::new(self, 0)
    }
    #[doc = "Bits 4:5 - TRACECLK Prescaler"]
    #[inline(always)]
    pub fn presc(&mut self) -> PrescW<'_, TraceclkctrlSpec> {
        PrescW::new(self, 4)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`traceclkctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`traceclkctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TraceclkctrlSpec;
impl crate::RegisterSpec for TraceclkctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`traceclkctrl::R`](R) reader structure"]
impl crate::Readable for TraceclkctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`traceclkctrl::W`](W) writer structure"]
impl crate::Writable for TraceclkctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TRACECLKCTRL to value 0x01"]
impl crate::Resettable for TraceclkctrlSpec {
    const RESET_VALUE: u32 = 0x01;
}
