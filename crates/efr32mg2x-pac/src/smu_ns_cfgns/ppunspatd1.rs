#[doc = "Register `PPUNSPATD1` reader"]
pub type R = crate::R<Ppunspatd1Spec>;
#[doc = "Register `PPUNSPATD1` writer"]
pub type W = crate::W<Ppunspatd1Spec>;
#[doc = "Field `KEYSCAN` reader - KEYSCAN Privileged Access"]
pub type KeyscanR = crate::BitReader;
#[doc = "Field `KEYSCAN` writer - KEYSCAN Privileged Access"]
pub type KeyscanW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DMEM` reader - DMEM Privileged Access"]
pub type DmemR = crate::BitReader;
#[doc = "Field `DMEM` writer - DMEM Privileged Access"]
pub type DmemW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RADIOAES` reader - RADIOAES Privileged Access"]
pub type RadioaesR = crate::BitReader;
#[doc = "Field `RADIOAES` writer - RADIOAES Privileged Access"]
pub type RadioaesW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SMU` reader - SMU Privileged Access"]
pub type SmuR = crate::BitReader;
#[doc = "Field `SMU` writer - SMU Privileged Access"]
pub type SmuW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SMUCFGNS` reader - SMUCFGNS Privileged Access"]
pub type SmucfgnsR = crate::BitReader;
#[doc = "Field `SMUCFGNS` writer - SMUCFGNS Privileged Access"]
pub type SmucfgnsW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LETIMER0` reader - LETIMER0 Privileged Access"]
pub type Letimer0R = crate::BitReader;
#[doc = "Field `LETIMER0` writer - LETIMER0 Privileged Access"]
pub type Letimer0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `IADC0` reader - IADC0 Privileged Access"]
pub type Iadc0R = crate::BitReader;
#[doc = "Field `IADC0` writer - IADC0 Privileged Access"]
pub type Iadc0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ACMP0` reader - ACMP0 Privileged Access"]
pub type Acmp0R = crate::BitReader;
#[doc = "Field `ACMP0` writer - ACMP0 Privileged Access"]
pub type Acmp0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ACMP1` reader - ACMP1 Privileged Access"]
pub type Acmp1R = crate::BitReader;
#[doc = "Field `ACMP1` writer - ACMP1 Privileged Access"]
pub type Acmp1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AMUXCP0` reader - AMUXCP0 Privileged Access"]
pub type Amuxcp0R = crate::BitReader;
#[doc = "Field `AMUXCP0` writer - AMUXCP0 Privileged Access"]
pub type Amuxcp0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `VDAC0` reader - VDAC0 Privileged Access"]
pub type Vdac0R = crate::BitReader;
#[doc = "Field `VDAC0` writer - VDAC0 Privileged Access"]
pub type Vdac0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `VDAC1` reader - VDAC1 Privileged Access"]
pub type Vdac1R = crate::BitReader;
#[doc = "Field `VDAC1` writer - VDAC1 Privileged Access"]
pub type Vdac1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PCNT` reader - PCNT Privileged Access"]
pub type PcntR = crate::BitReader;
#[doc = "Field `PCNT` writer - PCNT Privileged Access"]
pub type PcntW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `HFRCO1` reader - HFRCO1 Privileged Access"]
pub type Hfrco1R = crate::BitReader;
#[doc = "Field `HFRCO1` writer - HFRCO1 Privileged Access"]
pub type Hfrco1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `HFXO0` reader - HFXO0 Privileged Access"]
pub type Hfxo0R = crate::BitReader;
#[doc = "Field `HFXO0` writer - HFXO0 Privileged Access"]
pub type Hfxo0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `I2C0` reader - I2C0 Privileged Access"]
pub type I2c0R = crate::BitReader;
#[doc = "Field `I2C0` writer - I2C0 Privileged Access"]
pub type I2c0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `WDOG0` reader - WDOG0 Privileged Access"]
pub type Wdog0R = crate::BitReader;
#[doc = "Field `WDOG0` writer - WDOG0 Privileged Access"]
pub type Wdog0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `WDOG1` reader - WDOG1 Privileged Access"]
pub type Wdog1R = crate::BitReader;
#[doc = "Field `WDOG1` writer - WDOG1 Privileged Access"]
pub type Wdog1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EUSART0` reader - EUSART0 Privileged Access"]
pub type Eusart0R = crate::BitReader;
#[doc = "Field `EUSART0` writer - EUSART0 Privileged Access"]
pub type Eusart0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SEMAILBOX` reader - SEMAILBOX Privileged Access"]
pub type SemailboxR = crate::BitReader;
#[doc = "Field `SEMAILBOX` writer - SEMAILBOX Privileged Access"]
pub type SemailboxW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `MVP` reader - MVP Privileged Access"]
pub type MvpR = crate::BitReader;
#[doc = "Field `MVP` writer - MVP Privileged Access"]
pub type MvpW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHBRADIO` reader - AHBRADIO Privileged Access"]
pub type AhbradioR = crate::BitReader;
#[doc = "Field `AHBRADIO` writer - AHBRADIO Privileged Access"]
pub type AhbradioW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - KEYSCAN Privileged Access"]
    #[inline(always)]
    pub fn keyscan(&self) -> KeyscanR {
        KeyscanR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - DMEM Privileged Access"]
    #[inline(always)]
    pub fn dmem(&self) -> DmemR {
        DmemR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - RADIOAES Privileged Access"]
    #[inline(always)]
    pub fn radioaes(&self) -> RadioaesR {
        RadioaesR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - SMU Privileged Access"]
    #[inline(always)]
    pub fn smu(&self) -> SmuR {
        SmuR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - SMUCFGNS Privileged Access"]
    #[inline(always)]
    pub fn smucfgns(&self) -> SmucfgnsR {
        SmucfgnsR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - LETIMER0 Privileged Access"]
    #[inline(always)]
    pub fn letimer0(&self) -> Letimer0R {
        Letimer0R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - IADC0 Privileged Access"]
    #[inline(always)]
    pub fn iadc0(&self) -> Iadc0R {
        Iadc0R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - ACMP0 Privileged Access"]
    #[inline(always)]
    pub fn acmp0(&self) -> Acmp0R {
        Acmp0R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - ACMP1 Privileged Access"]
    #[inline(always)]
    pub fn acmp1(&self) -> Acmp1R {
        Acmp1R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - AMUXCP0 Privileged Access"]
    #[inline(always)]
    pub fn amuxcp0(&self) -> Amuxcp0R {
        Amuxcp0R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - VDAC0 Privileged Access"]
    #[inline(always)]
    pub fn vdac0(&self) -> Vdac0R {
        Vdac0R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - VDAC1 Privileged Access"]
    #[inline(always)]
    pub fn vdac1(&self) -> Vdac1R {
        Vdac1R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - PCNT Privileged Access"]
    #[inline(always)]
    pub fn pcnt(&self) -> PcntR {
        PcntR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - HFRCO1 Privileged Access"]
    #[inline(always)]
    pub fn hfrco1(&self) -> Hfrco1R {
        Hfrco1R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - HFXO0 Privileged Access"]
    #[inline(always)]
    pub fn hfxo0(&self) -> Hfxo0R {
        Hfxo0R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - I2C0 Privileged Access"]
    #[inline(always)]
    pub fn i2c0(&self) -> I2c0R {
        I2c0R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - WDOG0 Privileged Access"]
    #[inline(always)]
    pub fn wdog0(&self) -> Wdog0R {
        Wdog0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - WDOG1 Privileged Access"]
    #[inline(always)]
    pub fn wdog1(&self) -> Wdog1R {
        Wdog1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - EUSART0 Privileged Access"]
    #[inline(always)]
    pub fn eusart0(&self) -> Eusart0R {
        Eusart0R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - SEMAILBOX Privileged Access"]
    #[inline(always)]
    pub fn semailbox(&self) -> SemailboxR {
        SemailboxR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - MVP Privileged Access"]
    #[inline(always)]
    pub fn mvp(&self) -> MvpR {
        MvpR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - AHBRADIO Privileged Access"]
    #[inline(always)]
    pub fn ahbradio(&self) -> AhbradioR {
        AhbradioR::new(((self.bits >> 21) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - KEYSCAN Privileged Access"]
    #[inline(always)]
    pub fn keyscan(&mut self) -> KeyscanW<'_, Ppunspatd1Spec> {
        KeyscanW::new(self, 0)
    }
    #[doc = "Bit 1 - DMEM Privileged Access"]
    #[inline(always)]
    pub fn dmem(&mut self) -> DmemW<'_, Ppunspatd1Spec> {
        DmemW::new(self, 1)
    }
    #[doc = "Bit 2 - RADIOAES Privileged Access"]
    #[inline(always)]
    pub fn radioaes(&mut self) -> RadioaesW<'_, Ppunspatd1Spec> {
        RadioaesW::new(self, 2)
    }
    #[doc = "Bit 3 - SMU Privileged Access"]
    #[inline(always)]
    pub fn smu(&mut self) -> SmuW<'_, Ppunspatd1Spec> {
        SmuW::new(self, 3)
    }
    #[doc = "Bit 4 - SMUCFGNS Privileged Access"]
    #[inline(always)]
    pub fn smucfgns(&mut self) -> SmucfgnsW<'_, Ppunspatd1Spec> {
        SmucfgnsW::new(self, 4)
    }
    #[doc = "Bit 5 - LETIMER0 Privileged Access"]
    #[inline(always)]
    pub fn letimer0(&mut self) -> Letimer0W<'_, Ppunspatd1Spec> {
        Letimer0W::new(self, 5)
    }
    #[doc = "Bit 6 - IADC0 Privileged Access"]
    #[inline(always)]
    pub fn iadc0(&mut self) -> Iadc0W<'_, Ppunspatd1Spec> {
        Iadc0W::new(self, 6)
    }
    #[doc = "Bit 7 - ACMP0 Privileged Access"]
    #[inline(always)]
    pub fn acmp0(&mut self) -> Acmp0W<'_, Ppunspatd1Spec> {
        Acmp0W::new(self, 7)
    }
    #[doc = "Bit 8 - ACMP1 Privileged Access"]
    #[inline(always)]
    pub fn acmp1(&mut self) -> Acmp1W<'_, Ppunspatd1Spec> {
        Acmp1W::new(self, 8)
    }
    #[doc = "Bit 9 - AMUXCP0 Privileged Access"]
    #[inline(always)]
    pub fn amuxcp0(&mut self) -> Amuxcp0W<'_, Ppunspatd1Spec> {
        Amuxcp0W::new(self, 9)
    }
    #[doc = "Bit 10 - VDAC0 Privileged Access"]
    #[inline(always)]
    pub fn vdac0(&mut self) -> Vdac0W<'_, Ppunspatd1Spec> {
        Vdac0W::new(self, 10)
    }
    #[doc = "Bit 11 - VDAC1 Privileged Access"]
    #[inline(always)]
    pub fn vdac1(&mut self) -> Vdac1W<'_, Ppunspatd1Spec> {
        Vdac1W::new(self, 11)
    }
    #[doc = "Bit 12 - PCNT Privileged Access"]
    #[inline(always)]
    pub fn pcnt(&mut self) -> PcntW<'_, Ppunspatd1Spec> {
        PcntW::new(self, 12)
    }
    #[doc = "Bit 13 - HFRCO1 Privileged Access"]
    #[inline(always)]
    pub fn hfrco1(&mut self) -> Hfrco1W<'_, Ppunspatd1Spec> {
        Hfrco1W::new(self, 13)
    }
    #[doc = "Bit 14 - HFXO0 Privileged Access"]
    #[inline(always)]
    pub fn hfxo0(&mut self) -> Hfxo0W<'_, Ppunspatd1Spec> {
        Hfxo0W::new(self, 14)
    }
    #[doc = "Bit 15 - I2C0 Privileged Access"]
    #[inline(always)]
    pub fn i2c0(&mut self) -> I2c0W<'_, Ppunspatd1Spec> {
        I2c0W::new(self, 15)
    }
    #[doc = "Bit 16 - WDOG0 Privileged Access"]
    #[inline(always)]
    pub fn wdog0(&mut self) -> Wdog0W<'_, Ppunspatd1Spec> {
        Wdog0W::new(self, 16)
    }
    #[doc = "Bit 17 - WDOG1 Privileged Access"]
    #[inline(always)]
    pub fn wdog1(&mut self) -> Wdog1W<'_, Ppunspatd1Spec> {
        Wdog1W::new(self, 17)
    }
    #[doc = "Bit 18 - EUSART0 Privileged Access"]
    #[inline(always)]
    pub fn eusart0(&mut self) -> Eusart0W<'_, Ppunspatd1Spec> {
        Eusart0W::new(self, 18)
    }
    #[doc = "Bit 19 - SEMAILBOX Privileged Access"]
    #[inline(always)]
    pub fn semailbox(&mut self) -> SemailboxW<'_, Ppunspatd1Spec> {
        SemailboxW::new(self, 19)
    }
    #[doc = "Bit 20 - MVP Privileged Access"]
    #[inline(always)]
    pub fn mvp(&mut self) -> MvpW<'_, Ppunspatd1Spec> {
        MvpW::new(self, 20)
    }
    #[doc = "Bit 21 - AHBRADIO Privileged Access"]
    #[inline(always)]
    pub fn ahbradio(&mut self) -> AhbradioW<'_, Ppunspatd1Spec> {
        AhbradioW::new(self, 21)
    }
}
#[doc = "Set peripheral bits to 1 to mark as privileged access only\n\nYou can [`read`](crate::Reg::read) this register and get [`ppunspatd1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ppunspatd1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ppunspatd1Spec;
impl crate::RegisterSpec for Ppunspatd1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ppunspatd1::R`](R) reader structure"]
impl crate::Readable for Ppunspatd1Spec {}
#[doc = "`write(|w| ..)` method takes [`ppunspatd1::W`](W) writer structure"]
impl crate::Writable for Ppunspatd1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PPUNSPATD1 to value 0"]
impl crate::Resettable for Ppunspatd1Spec {}
