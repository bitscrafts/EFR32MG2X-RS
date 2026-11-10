#[doc = "Register `SIGFRAMECFG` reader"]
pub type R = crate::R<SigframecfgSpec>;
#[doc = "Register `SIGFRAMECFG` writer"]
pub type W = crate::W<SigframecfgSpec>;
#[doc = "Field `SIGFRAME` reader - Signal Frame Value"]
pub type SigframeR = crate::FieldReader<u32>;
#[doc = "Field `SIGFRAME` writer - Signal Frame Value"]
pub type SigframeW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Signal Frame Value"]
    #[inline(always)]
    pub fn sigframe(&self) -> SigframeR {
        SigframeR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Signal Frame Value"]
    #[inline(always)]
    pub fn sigframe(&mut self) -> SigframeW<'_, SigframecfgSpec> {
        SigframeW::new(self, 0)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`sigframecfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sigframecfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SigframecfgSpec;
impl crate::RegisterSpec for SigframecfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sigframecfg::R`](R) reader structure"]
impl crate::Readable for SigframecfgSpec {}
#[doc = "`write(|w| ..)` method takes [`sigframecfg::W`](W) writer structure"]
impl crate::Writable for SigframecfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SIGFRAMECFG to value 0"]
impl crate::Resettable for SigframecfgSpec {}
