#[doc = "Register `RADIOECCCTRL` reader"]
pub type R = crate::R<RadioeccctrlSpec>;
#[doc = "Register `RADIOECCCTRL` writer"]
pub type W = crate::W<RadioeccctrlSpec>;
#[doc = "Field `SEQRAMECCEN` reader - SEQRAM ECC Enable"]
pub type SeqrameccenR = crate::BitReader;
#[doc = "Field `SEQRAMECCEN` writer - SEQRAM ECC Enable"]
pub type SeqrameccenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `SEQRAMECCEWEN` reader - SEQRAM ECC Error Writeback Enable"]
pub type SeqrameccewenR = crate::BitReader;
#[doc = "Field `SEQRAMECCEWEN` writer - SEQRAM ECC Error Writeback Enable"]
pub type SeqrameccewenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FRCRAMECCEN` reader - FRCRAM ECC Enable"]
pub type FrcrameccenR = crate::BitReader;
#[doc = "Field `FRCRAMECCEN` writer - FRCRAM ECC Enable"]
pub type FrcrameccenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FRCRAMECCEWEN` reader - FRCRAM ECC Error Writeback Enable"]
pub type FrcrameccewenR = crate::BitReader;
#[doc = "Field `FRCRAMECCEWEN` writer - FRCRAM ECC Error Writeback Enable"]
pub type FrcrameccewenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - SEQRAM ECC Enable"]
    #[inline(always)]
    pub fn seqrameccen(&self) -> SeqrameccenR {
        SeqrameccenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SEQRAM ECC Error Writeback Enable"]
    #[inline(always)]
    pub fn seqrameccewen(&self) -> SeqrameccewenR {
        SeqrameccewenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 8 - FRCRAM ECC Enable"]
    #[inline(always)]
    pub fn frcrameccen(&self) -> FrcrameccenR {
        FrcrameccenR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - FRCRAM ECC Error Writeback Enable"]
    #[inline(always)]
    pub fn frcrameccewen(&self) -> FrcrameccewenR {
        FrcrameccewenR::new(((self.bits >> 9) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - SEQRAM ECC Enable"]
    #[inline(always)]
    pub fn seqrameccen(&mut self) -> SeqrameccenW<'_, RadioeccctrlSpec> {
        SeqrameccenW::new(self, 0)
    }
    #[doc = "Bit 1 - SEQRAM ECC Error Writeback Enable"]
    #[inline(always)]
    pub fn seqrameccewen(&mut self) -> SeqrameccewenW<'_, RadioeccctrlSpec> {
        SeqrameccewenW::new(self, 1)
    }
    #[doc = "Bit 8 - FRCRAM ECC Enable"]
    #[inline(always)]
    pub fn frcrameccen(&mut self) -> FrcrameccenW<'_, RadioeccctrlSpec> {
        FrcrameccenW::new(self, 8)
    }
    #[doc = "Bit 9 - FRCRAM ECC Error Writeback Enable"]
    #[inline(always)]
    pub fn frcrameccewen(&mut self) -> FrcrameccewenW<'_, RadioeccctrlSpec> {
        FrcrameccewenW::new(self, 9)
    }
}
#[doc = "Configure to set RAM ECC control.\n\nYou can [`read`](crate::Reg::read) this register and get [`radioeccctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`radioeccctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RadioeccctrlSpec;
impl crate::RegisterSpec for RadioeccctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`radioeccctrl::R`](R) reader structure"]
impl crate::Readable for RadioeccctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`radioeccctrl::W`](W) writer structure"]
impl crate::Writable for RadioeccctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RADIOECCCTRL to value 0"]
impl crate::Resettable for RadioeccctrlSpec {}
