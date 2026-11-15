#[doc = "Register `PPUSATD1` reader"]
pub type R = crate::R<Ppusatd1Spec>;
#[doc = "Register `PPUSATD1` writer"]
pub type W = crate::W<Ppusatd1Spec>;
#[doc = "Field `KEYSCAN` reader - KEYSCAN Secure Access"]
pub type KeyscanR = crate::BitReader;
#[doc = "Field `KEYSCAN` writer - KEYSCAN Secure Access"]
pub type KeyscanW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DMEM` reader - DMEM Secure Access"]
pub type DmemR = crate::BitReader;
#[doc = "Field `DMEM` writer - DMEM Secure Access"]
pub type DmemW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RADIOAES` reader - RADIOAES Secure Access"]
pub type RadioaesR = crate::BitReader;
#[doc = "Field `RADIOAES` writer - RADIOAES Secure Access"]
pub type RadioaesW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SMU` reader - SMU Secure Access"]
pub type SmuR = crate::BitReader;
#[doc = "Field `SMU` writer - SMU Secure Access"]
pub type SmuW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SMUCFGNS` reader - SMUCFGNS Secure Access"]
pub type SmucfgnsR = crate::BitReader;
#[doc = "Field `SMUCFGNS` writer - SMUCFGNS Secure Access"]
pub type SmucfgnsW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LETIMER0` reader - LETIMER0 Secure Access"]
pub type Letimer0R = crate::BitReader;
#[doc = "Field `LETIMER0` writer - LETIMER0 Secure Access"]
pub type Letimer0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `IADC0` reader - IADC0 Secure Access"]
pub type Iadc0R = crate::BitReader;
#[doc = "Field `IADC0` writer - IADC0 Secure Access"]
pub type Iadc0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ACMP0` reader - ACMP0 Secure Access"]
pub type Acmp0R = crate::BitReader;
#[doc = "Field `ACMP0` writer - ACMP0 Secure Access"]
pub type Acmp0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ACMP1` reader - ACMP1 Secure Access"]
pub type Acmp1R = crate::BitReader;
#[doc = "Field `ACMP1` writer - ACMP1 Secure Access"]
pub type Acmp1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AMUXCP0` reader - AMUXCP0 Secure Access"]
pub type Amuxcp0R = crate::BitReader;
#[doc = "Field `AMUXCP0` writer - AMUXCP0 Secure Access"]
pub type Amuxcp0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `VDAC0` reader - VDAC0 Secure Access"]
pub type Vdac0R = crate::BitReader;
#[doc = "Field `VDAC0` writer - VDAC0 Secure Access"]
pub type Vdac0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `VDAC1` reader - VDAC1 Secure Access"]
pub type Vdac1R = crate::BitReader;
#[doc = "Field `VDAC1` writer - VDAC1 Secure Access"]
pub type Vdac1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PCNT` reader - PCNT Secure Access"]
pub type PcntR = crate::BitReader;
#[doc = "Field `PCNT` writer - PCNT Secure Access"]
pub type PcntW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `HFRCO1` reader - HFRCO1 Secure Access"]
pub type Hfrco1R = crate::BitReader;
#[doc = "Field `HFRCO1` writer - HFRCO1 Secure Access"]
pub type Hfrco1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `HFXO0` reader - HFXO0 Secure Access"]
pub type Hfxo0R = crate::BitReader;
#[doc = "Field `HFXO0` writer - HFXO0 Secure Access"]
pub type Hfxo0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `I2C0` reader - I2C0 Secure Access"]
pub type I2c0R = crate::BitReader;
#[doc = "Field `I2C0` writer - I2C0 Secure Access"]
pub type I2c0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `WDOG0` reader - WDOG0 Secure Access"]
pub type Wdog0R = crate::BitReader;
#[doc = "Field `WDOG0` writer - WDOG0 Secure Access"]
pub type Wdog0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `WDOG1` reader - WDOG1 Secure Access"]
pub type Wdog1R = crate::BitReader;
#[doc = "Field `WDOG1` writer - WDOG1 Secure Access"]
pub type Wdog1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EUSART0` reader - EUSART0 Secure Access"]
pub type Eusart0R = crate::BitReader;
#[doc = "Field `EUSART0` writer - EUSART0 Secure Access"]
pub type Eusart0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SEMAILBOX` reader - SEMAILBOX Secure Access"]
pub type SemailboxR = crate::BitReader;
#[doc = "Field `SEMAILBOX` writer - SEMAILBOX Secure Access"]
pub type SemailboxW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `MVP` reader - MVP Secure Access"]
pub type MvpR = crate::BitReader;
#[doc = "Field `MVP` writer - MVP Secure Access"]
pub type MvpW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `AHBRADIO` reader - AHBRADIO Secure Access"]
pub type AhbradioR = crate::BitReader;
#[doc = "Field `AHBRADIO` writer - AHBRADIO Secure Access"]
pub type AhbradioW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - KEYSCAN Secure Access"]
    #[inline(always)]
    pub fn keyscan(&self) -> KeyscanR {
        KeyscanR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - DMEM Secure Access"]
    #[inline(always)]
    pub fn dmem(&self) -> DmemR {
        DmemR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - RADIOAES Secure Access"]
    #[inline(always)]
    pub fn radioaes(&self) -> RadioaesR {
        RadioaesR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - SMU Secure Access"]
    #[inline(always)]
    pub fn smu(&self) -> SmuR {
        SmuR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - SMUCFGNS Secure Access"]
    #[inline(always)]
    pub fn smucfgns(&self) -> SmucfgnsR {
        SmucfgnsR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - LETIMER0 Secure Access"]
    #[inline(always)]
    pub fn letimer0(&self) -> Letimer0R {
        Letimer0R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - IADC0 Secure Access"]
    #[inline(always)]
    pub fn iadc0(&self) -> Iadc0R {
        Iadc0R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - ACMP0 Secure Access"]
    #[inline(always)]
    pub fn acmp0(&self) -> Acmp0R {
        Acmp0R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - ACMP1 Secure Access"]
    #[inline(always)]
    pub fn acmp1(&self) -> Acmp1R {
        Acmp1R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - AMUXCP0 Secure Access"]
    #[inline(always)]
    pub fn amuxcp0(&self) -> Amuxcp0R {
        Amuxcp0R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - VDAC0 Secure Access"]
    #[inline(always)]
    pub fn vdac0(&self) -> Vdac0R {
        Vdac0R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - VDAC1 Secure Access"]
    #[inline(always)]
    pub fn vdac1(&self) -> Vdac1R {
        Vdac1R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - PCNT Secure Access"]
    #[inline(always)]
    pub fn pcnt(&self) -> PcntR {
        PcntR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - HFRCO1 Secure Access"]
    #[inline(always)]
    pub fn hfrco1(&self) -> Hfrco1R {
        Hfrco1R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - HFXO0 Secure Access"]
    #[inline(always)]
    pub fn hfxo0(&self) -> Hfxo0R {
        Hfxo0R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - I2C0 Secure Access"]
    #[inline(always)]
    pub fn i2c0(&self) -> I2c0R {
        I2c0R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - WDOG0 Secure Access"]
    #[inline(always)]
    pub fn wdog0(&self) -> Wdog0R {
        Wdog0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - WDOG1 Secure Access"]
    #[inline(always)]
    pub fn wdog1(&self) -> Wdog1R {
        Wdog1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - EUSART0 Secure Access"]
    #[inline(always)]
    pub fn eusart0(&self) -> Eusart0R {
        Eusart0R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - SEMAILBOX Secure Access"]
    #[inline(always)]
    pub fn semailbox(&self) -> SemailboxR {
        SemailboxR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - MVP Secure Access"]
    #[inline(always)]
    pub fn mvp(&self) -> MvpR {
        MvpR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - AHBRADIO Secure Access"]
    #[inline(always)]
    pub fn ahbradio(&self) -> AhbradioR {
        AhbradioR::new(((self.bits >> 21) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - KEYSCAN Secure Access"]
    #[inline(always)]
    pub fn keyscan(&mut self) -> KeyscanW<'_, Ppusatd1Spec> {
        KeyscanW::new(self, 0)
    }
    #[doc = "Bit 1 - DMEM Secure Access"]
    #[inline(always)]
    pub fn dmem(&mut self) -> DmemW<'_, Ppusatd1Spec> {
        DmemW::new(self, 1)
    }
    #[doc = "Bit 2 - RADIOAES Secure Access"]
    #[inline(always)]
    pub fn radioaes(&mut self) -> RadioaesW<'_, Ppusatd1Spec> {
        RadioaesW::new(self, 2)
    }
    #[doc = "Bit 3 - SMU Secure Access"]
    #[inline(always)]
    pub fn smu(&mut self) -> SmuW<'_, Ppusatd1Spec> {
        SmuW::new(self, 3)
    }
    #[doc = "Bit 4 - SMUCFGNS Secure Access"]
    #[inline(always)]
    pub fn smucfgns(&mut self) -> SmucfgnsW<'_, Ppusatd1Spec> {
        SmucfgnsW::new(self, 4)
    }
    #[doc = "Bit 5 - LETIMER0 Secure Access"]
    #[inline(always)]
    pub fn letimer0(&mut self) -> Letimer0W<'_, Ppusatd1Spec> {
        Letimer0W::new(self, 5)
    }
    #[doc = "Bit 6 - IADC0 Secure Access"]
    #[inline(always)]
    pub fn iadc0(&mut self) -> Iadc0W<'_, Ppusatd1Spec> {
        Iadc0W::new(self, 6)
    }
    #[doc = "Bit 7 - ACMP0 Secure Access"]
    #[inline(always)]
    pub fn acmp0(&mut self) -> Acmp0W<'_, Ppusatd1Spec> {
        Acmp0W::new(self, 7)
    }
    #[doc = "Bit 8 - ACMP1 Secure Access"]
    #[inline(always)]
    pub fn acmp1(&mut self) -> Acmp1W<'_, Ppusatd1Spec> {
        Acmp1W::new(self, 8)
    }
    #[doc = "Bit 9 - AMUXCP0 Secure Access"]
    #[inline(always)]
    pub fn amuxcp0(&mut self) -> Amuxcp0W<'_, Ppusatd1Spec> {
        Amuxcp0W::new(self, 9)
    }
    #[doc = "Bit 10 - VDAC0 Secure Access"]
    #[inline(always)]
    pub fn vdac0(&mut self) -> Vdac0W<'_, Ppusatd1Spec> {
        Vdac0W::new(self, 10)
    }
    #[doc = "Bit 11 - VDAC1 Secure Access"]
    #[inline(always)]
    pub fn vdac1(&mut self) -> Vdac1W<'_, Ppusatd1Spec> {
        Vdac1W::new(self, 11)
    }
    #[doc = "Bit 12 - PCNT Secure Access"]
    #[inline(always)]
    pub fn pcnt(&mut self) -> PcntW<'_, Ppusatd1Spec> {
        PcntW::new(self, 12)
    }
    #[doc = "Bit 13 - HFRCO1 Secure Access"]
    #[inline(always)]
    pub fn hfrco1(&mut self) -> Hfrco1W<'_, Ppusatd1Spec> {
        Hfrco1W::new(self, 13)
    }
    #[doc = "Bit 14 - HFXO0 Secure Access"]
    #[inline(always)]
    pub fn hfxo0(&mut self) -> Hfxo0W<'_, Ppusatd1Spec> {
        Hfxo0W::new(self, 14)
    }
    #[doc = "Bit 15 - I2C0 Secure Access"]
    #[inline(always)]
    pub fn i2c0(&mut self) -> I2c0W<'_, Ppusatd1Spec> {
        I2c0W::new(self, 15)
    }
    #[doc = "Bit 16 - WDOG0 Secure Access"]
    #[inline(always)]
    pub fn wdog0(&mut self) -> Wdog0W<'_, Ppusatd1Spec> {
        Wdog0W::new(self, 16)
    }
    #[doc = "Bit 17 - WDOG1 Secure Access"]
    #[inline(always)]
    pub fn wdog1(&mut self) -> Wdog1W<'_, Ppusatd1Spec> {
        Wdog1W::new(self, 17)
    }
    #[doc = "Bit 18 - EUSART0 Secure Access"]
    #[inline(always)]
    pub fn eusart0(&mut self) -> Eusart0W<'_, Ppusatd1Spec> {
        Eusart0W::new(self, 18)
    }
    #[doc = "Bit 19 - SEMAILBOX Secure Access"]
    #[inline(always)]
    pub fn semailbox(&mut self) -> SemailboxW<'_, Ppusatd1Spec> {
        SemailboxW::new(self, 19)
    }
    #[doc = "Bit 20 - MVP Secure Access"]
    #[inline(always)]
    pub fn mvp(&mut self) -> MvpW<'_, Ppusatd1Spec> {
        MvpW::new(self, 20)
    }
    #[doc = "Bit 21 - AHBRADIO Secure Access"]
    #[inline(always)]
    pub fn ahbradio(&mut self) -> AhbradioW<'_, Ppusatd1Spec> {
        AhbradioW::new(self, 21)
    }
}
#[doc = "Set peripheral bits to 1 to mark as secure access only\n\nYou can [`read`](crate::Reg::read) this register and get [`ppusatd1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ppusatd1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ppusatd1Spec;
impl crate::RegisterSpec for Ppusatd1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ppusatd1::R`](R) reader structure"]
impl crate::Readable for Ppusatd1Spec {}
#[doc = "`write(|w| ..)` method takes [`ppusatd1::W`](W) writer structure"]
impl crate::Writable for Ppusatd1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PPUSATD1 to value 0x003f_ffff"]
impl crate::Resettable for Ppusatd1Spec {
    const RESET_VALUE: u32 = 0x003f_ffff;
}
