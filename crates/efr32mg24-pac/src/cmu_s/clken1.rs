#[doc = "Register `CLKEN1` reader"]
pub type R = crate::R<Clken1Spec>;
#[doc = "Register `CLKEN1` writer"]
pub type W = crate::W<Clken1Spec>;
#[doc = "Field `AGC` reader - Enable Bus Clock"]
pub type AgcR = crate::BitReader;
#[doc = "Field `AGC` writer - Enable Bus Clock"]
pub type AgcW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `MODEM` reader - Enable Bus Clock"]
pub type ModemR = crate::BitReader;
#[doc = "Field `MODEM` writer - Enable Bus Clock"]
pub type ModemW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RFCRC` reader - Enable Bus Clock"]
pub type RfcrcR = crate::BitReader;
#[doc = "Field `RFCRC` writer - Enable Bus Clock"]
pub type RfcrcW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FRC` reader - Enable Bus Clock"]
pub type FrcR = crate::BitReader;
#[doc = "Field `FRC` writer - Enable Bus Clock"]
pub type FrcW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PROTIMER` reader - Enable Bus Clock"]
pub type ProtimerR = crate::BitReader;
#[doc = "Field `PROTIMER` writer - Enable Bus Clock"]
pub type ProtimerW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RAC` reader - Enable Bus Clock"]
pub type RacR = crate::BitReader;
#[doc = "Field `RAC` writer - Enable Bus Clock"]
pub type RacW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SYNTH` reader - Enable Bus Clock"]
pub type SynthR = crate::BitReader;
#[doc = "Field `SYNTH` writer - Enable Bus Clock"]
pub type SynthW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RFSCRATCHPAD` reader - Enable Bus Clock"]
pub type RfscratchpadR = crate::BitReader;
#[doc = "Field `RFSCRATCHPAD` writer - Enable Bus Clock"]
pub type RfscratchpadW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `HOSTMAILBOX` reader - Enable Bus Clock"]
pub type HostmailboxR = crate::BitReader;
#[doc = "Field `HOSTMAILBOX` writer - Enable Bus Clock"]
pub type HostmailboxW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RFMAILBOX` reader - Enable Bus Clock"]
pub type RfmailboxR = crate::BitReader;
#[doc = "Field `RFMAILBOX` writer - Enable Bus Clock"]
pub type RfmailboxW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SEMAILBOXHOST` reader - Enable Bus Clock"]
pub type SemailboxhostR = crate::BitReader;
#[doc = "Field `SEMAILBOXHOST` writer - Enable Bus Clock"]
pub type SemailboxhostW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BUFC` reader - Enable Bus Clock"]
pub type BufcR = crate::BitReader;
#[doc = "Field `BUFC` writer - Enable Bus Clock"]
pub type BufcW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `KEYSCAN` reader - Enable Bus Clock"]
pub type KeyscanR = crate::BitReader;
#[doc = "Field `KEYSCAN` writer - Enable Bus Clock"]
pub type KeyscanW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SMU` reader - Enable Bus Clock"]
pub type SmuR = crate::BitReader;
#[doc = "Field `SMU` writer - Enable Bus Clock"]
pub type SmuW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ICACHE0` reader - Enable Bus Clock"]
pub type Icache0R = crate::BitReader;
#[doc = "Field `ICACHE0` writer - Enable Bus Clock"]
pub type Icache0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `MSC` reader - Enable Bus Clock"]
pub type MscR = crate::BitReader;
#[doc = "Field `MSC` writer - Enable Bus Clock"]
pub type MscW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `WDOG1` reader - Enable Bus Clock"]
pub type Wdog1R = crate::BitReader;
#[doc = "Field `WDOG1` writer - Enable Bus Clock"]
pub type Wdog1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ACMP0` reader - Enable Bus Clock"]
pub type Acmp0R = crate::BitReader;
#[doc = "Field `ACMP0` writer - Enable Bus Clock"]
pub type Acmp0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ACMP1` reader - Enable Bus Clock"]
pub type Acmp1R = crate::BitReader;
#[doc = "Field `ACMP1` writer - Enable Bus Clock"]
pub type Acmp1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `VDAC0` reader - Enable Bus Clock"]
pub type Vdac0R = crate::BitReader;
#[doc = "Field `VDAC0` writer - Enable Bus Clock"]
pub type Vdac0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PCNT0` reader - Enable Bus Clock"]
pub type Pcnt0R = crate::BitReader;
#[doc = "Field `PCNT0` writer - Enable Bus Clock"]
pub type Pcnt0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EUSART0` reader - Enable Bus Clock"]
pub type Eusart0R = crate::BitReader;
#[doc = "Field `EUSART0` writer - Enable Bus Clock"]
pub type Eusart0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EUSART1` reader - Enable Bus Clock"]
pub type Eusart1R = crate::BitReader;
#[doc = "Field `EUSART1` writer - Enable Bus Clock"]
pub type Eusart1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RFECA0` reader - Enable Bus Clock"]
pub type Rfeca0R = crate::BitReader;
#[doc = "Field `RFECA0` writer - Enable Bus Clock"]
pub type Rfeca0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RFECA1` reader - Enable Bus Clock"]
pub type Rfeca1R = crate::BitReader;
#[doc = "Field `RFECA1` writer - Enable Bus Clock"]
pub type Rfeca1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DMEM` reader - Enable Bus Clock"]
pub type DmemR = crate::BitReader;
#[doc = "Field `DMEM` writer - Enable Bus Clock"]
pub type DmemW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ECAIFADC` reader - Enable Bus Clock"]
pub type EcaifadcR = crate::BitReader;
#[doc = "Field `ECAIFADC` writer - Enable Bus Clock"]
pub type EcaifadcW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `VDAC1` reader - Enable Bus Clock"]
pub type Vdac1R = crate::BitReader;
#[doc = "Field `VDAC1` writer - Enable Bus Clock"]
pub type Vdac1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `MVP` reader - Enable Bus Clock"]
pub type MvpR = crate::BitReader;
#[doc = "Field `MVP` writer - Enable Bus Clock"]
pub type MvpW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Enable Bus Clock"]
    #[inline(always)]
    pub fn agc(&self) -> AgcR {
        AgcR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Enable Bus Clock"]
    #[inline(always)]
    pub fn modem(&self) -> ModemR {
        ModemR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfcrc(&self) -> RfcrcR {
        RfcrcR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Enable Bus Clock"]
    #[inline(always)]
    pub fn frc(&self) -> FrcR {
        FrcR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Enable Bus Clock"]
    #[inline(always)]
    pub fn protimer(&self) -> ProtimerR {
        ProtimerR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rac(&self) -> RacR {
        RacR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Enable Bus Clock"]
    #[inline(always)]
    pub fn synth(&self) -> SynthR {
        SynthR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfscratchpad(&self) -> RfscratchpadR {
        RfscratchpadR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Enable Bus Clock"]
    #[inline(always)]
    pub fn hostmailbox(&self) -> HostmailboxR {
        HostmailboxR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfmailbox(&self) -> RfmailboxR {
        RfmailboxR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Enable Bus Clock"]
    #[inline(always)]
    pub fn semailboxhost(&self) -> SemailboxhostR {
        SemailboxhostR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Enable Bus Clock"]
    #[inline(always)]
    pub fn bufc(&self) -> BufcR {
        BufcR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 13 - Enable Bus Clock"]
    #[inline(always)]
    pub fn keyscan(&self) -> KeyscanR {
        KeyscanR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Enable Bus Clock"]
    #[inline(always)]
    pub fn smu(&self) -> SmuR {
        SmuR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Enable Bus Clock"]
    #[inline(always)]
    pub fn icache0(&self) -> Icache0R {
        Icache0R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Enable Bus Clock"]
    #[inline(always)]
    pub fn msc(&self) -> MscR {
        MscR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Enable Bus Clock"]
    #[inline(always)]
    pub fn wdog1(&self) -> Wdog1R {
        Wdog1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Enable Bus Clock"]
    #[inline(always)]
    pub fn acmp0(&self) -> Acmp0R {
        Acmp0R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Enable Bus Clock"]
    #[inline(always)]
    pub fn acmp1(&self) -> Acmp1R {
        Acmp1R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Enable Bus Clock"]
    #[inline(always)]
    pub fn vdac0(&self) -> Vdac0R {
        Vdac0R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Enable Bus Clock"]
    #[inline(always)]
    pub fn pcnt0(&self) -> Pcnt0R {
        Pcnt0R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Enable Bus Clock"]
    #[inline(always)]
    pub fn eusart0(&self) -> Eusart0R {
        Eusart0R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Enable Bus Clock"]
    #[inline(always)]
    pub fn eusart1(&self) -> Eusart1R {
        Eusart1R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 25 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfeca0(&self) -> Rfeca0R {
        Rfeca0R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfeca1(&self) -> Rfeca1R {
        Rfeca1R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Enable Bus Clock"]
    #[inline(always)]
    pub fn dmem(&self) -> DmemR {
        DmemR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Enable Bus Clock"]
    #[inline(always)]
    pub fn ecaifadc(&self) -> EcaifadcR {
        EcaifadcR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Enable Bus Clock"]
    #[inline(always)]
    pub fn vdac1(&self) -> Vdac1R {
        Vdac1R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Enable Bus Clock"]
    #[inline(always)]
    pub fn mvp(&self) -> MvpR {
        MvpR::new(((self.bits >> 30) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Bus Clock"]
    #[inline(always)]
    pub fn agc(&mut self) -> AgcW<'_, Clken1Spec> {
        AgcW::new(self, 0)
    }
    #[doc = "Bit 1 - Enable Bus Clock"]
    #[inline(always)]
    pub fn modem(&mut self) -> ModemW<'_, Clken1Spec> {
        ModemW::new(self, 1)
    }
    #[doc = "Bit 2 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfcrc(&mut self) -> RfcrcW<'_, Clken1Spec> {
        RfcrcW::new(self, 2)
    }
    #[doc = "Bit 3 - Enable Bus Clock"]
    #[inline(always)]
    pub fn frc(&mut self) -> FrcW<'_, Clken1Spec> {
        FrcW::new(self, 3)
    }
    #[doc = "Bit 4 - Enable Bus Clock"]
    #[inline(always)]
    pub fn protimer(&mut self) -> ProtimerW<'_, Clken1Spec> {
        ProtimerW::new(self, 4)
    }
    #[doc = "Bit 5 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rac(&mut self) -> RacW<'_, Clken1Spec> {
        RacW::new(self, 5)
    }
    #[doc = "Bit 6 - Enable Bus Clock"]
    #[inline(always)]
    pub fn synth(&mut self) -> SynthW<'_, Clken1Spec> {
        SynthW::new(self, 6)
    }
    #[doc = "Bit 7 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfscratchpad(&mut self) -> RfscratchpadW<'_, Clken1Spec> {
        RfscratchpadW::new(self, 7)
    }
    #[doc = "Bit 8 - Enable Bus Clock"]
    #[inline(always)]
    pub fn hostmailbox(&mut self) -> HostmailboxW<'_, Clken1Spec> {
        HostmailboxW::new(self, 8)
    }
    #[doc = "Bit 9 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfmailbox(&mut self) -> RfmailboxW<'_, Clken1Spec> {
        RfmailboxW::new(self, 9)
    }
    #[doc = "Bit 10 - Enable Bus Clock"]
    #[inline(always)]
    pub fn semailboxhost(&mut self) -> SemailboxhostW<'_, Clken1Spec> {
        SemailboxhostW::new(self, 10)
    }
    #[doc = "Bit 11 - Enable Bus Clock"]
    #[inline(always)]
    pub fn bufc(&mut self) -> BufcW<'_, Clken1Spec> {
        BufcW::new(self, 11)
    }
    #[doc = "Bit 13 - Enable Bus Clock"]
    #[inline(always)]
    pub fn keyscan(&mut self) -> KeyscanW<'_, Clken1Spec> {
        KeyscanW::new(self, 13)
    }
    #[doc = "Bit 14 - Enable Bus Clock"]
    #[inline(always)]
    pub fn smu(&mut self) -> SmuW<'_, Clken1Spec> {
        SmuW::new(self, 14)
    }
    #[doc = "Bit 15 - Enable Bus Clock"]
    #[inline(always)]
    pub fn icache0(&mut self) -> Icache0W<'_, Clken1Spec> {
        Icache0W::new(self, 15)
    }
    #[doc = "Bit 16 - Enable Bus Clock"]
    #[inline(always)]
    pub fn msc(&mut self) -> MscW<'_, Clken1Spec> {
        MscW::new(self, 16)
    }
    #[doc = "Bit 17 - Enable Bus Clock"]
    #[inline(always)]
    pub fn wdog1(&mut self) -> Wdog1W<'_, Clken1Spec> {
        Wdog1W::new(self, 17)
    }
    #[doc = "Bit 18 - Enable Bus Clock"]
    #[inline(always)]
    pub fn acmp0(&mut self) -> Acmp0W<'_, Clken1Spec> {
        Acmp0W::new(self, 18)
    }
    #[doc = "Bit 19 - Enable Bus Clock"]
    #[inline(always)]
    pub fn acmp1(&mut self) -> Acmp1W<'_, Clken1Spec> {
        Acmp1W::new(self, 19)
    }
    #[doc = "Bit 20 - Enable Bus Clock"]
    #[inline(always)]
    pub fn vdac0(&mut self) -> Vdac0W<'_, Clken1Spec> {
        Vdac0W::new(self, 20)
    }
    #[doc = "Bit 21 - Enable Bus Clock"]
    #[inline(always)]
    pub fn pcnt0(&mut self) -> Pcnt0W<'_, Clken1Spec> {
        Pcnt0W::new(self, 21)
    }
    #[doc = "Bit 22 - Enable Bus Clock"]
    #[inline(always)]
    pub fn eusart0(&mut self) -> Eusart0W<'_, Clken1Spec> {
        Eusart0W::new(self, 22)
    }
    #[doc = "Bit 23 - Enable Bus Clock"]
    #[inline(always)]
    pub fn eusart1(&mut self) -> Eusart1W<'_, Clken1Spec> {
        Eusart1W::new(self, 23)
    }
    #[doc = "Bit 25 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfeca0(&mut self) -> Rfeca0W<'_, Clken1Spec> {
        Rfeca0W::new(self, 25)
    }
    #[doc = "Bit 26 - Enable Bus Clock"]
    #[inline(always)]
    pub fn rfeca1(&mut self) -> Rfeca1W<'_, Clken1Spec> {
        Rfeca1W::new(self, 26)
    }
    #[doc = "Bit 27 - Enable Bus Clock"]
    #[inline(always)]
    pub fn dmem(&mut self) -> DmemW<'_, Clken1Spec> {
        DmemW::new(self, 27)
    }
    #[doc = "Bit 28 - Enable Bus Clock"]
    #[inline(always)]
    pub fn ecaifadc(&mut self) -> EcaifadcW<'_, Clken1Spec> {
        EcaifadcW::new(self, 28)
    }
    #[doc = "Bit 29 - Enable Bus Clock"]
    #[inline(always)]
    pub fn vdac1(&mut self) -> Vdac1W<'_, Clken1Spec> {
        Vdac1W::new(self, 29)
    }
    #[doc = "Bit 30 - Enable Bus Clock"]
    #[inline(always)]
    pub fn mvp(&mut self) -> MvpW<'_, Clken1Spec> {
        MvpW::new(self, 30)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`clken1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clken1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Clken1Spec;
impl crate::RegisterSpec for Clken1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`clken1::R`](R) reader structure"]
impl crate::Readable for Clken1Spec {}
#[doc = "`write(|w| ..)` method takes [`clken1::W`](W) writer structure"]
impl crate::Writable for Clken1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CLKEN1 to value 0"]
impl crate::Resettable for Clken1Spec {}
