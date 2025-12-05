#[doc = "Register `FETCHTAG` reader"]
pub type R = crate::R<FetchtagSpec>;
#[doc = "Register `FETCHTAG` writer"]
pub type W = crate::W<FetchtagSpec>;
#[doc = "Field `TAG` reader - User tag"]
pub type TagR = crate::FieldReader<u32>;
#[doc = "Field `TAG` writer - User tag"]
pub type TagW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - User tag"]
    #[inline(always)]
    pub fn tag(&self) -> TagR {
        TagR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - User tag"]
    #[inline(always)]
    pub fn tag(&mut self) -> TagW<'_, FetchtagSpec> {
        TagW::new(self, 0)
    }
}
#[doc = "Fetcher: User tag. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used.\n\nYou can [`read`](crate::Reg::read) this register and get [`fetchtag::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fetchtag::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FetchtagSpec;
impl crate::RegisterSpec for FetchtagSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fetchtag::R`](R) reader structure"]
impl crate::Readable for FetchtagSpec {}
#[doc = "`write(|w| ..)` method takes [`fetchtag::W`](W) writer structure"]
impl crate::Writable for FetchtagSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FETCHTAG to value 0"]
impl crate::Resettable for FetchtagSpec {}
