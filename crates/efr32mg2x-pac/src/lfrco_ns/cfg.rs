#[doc = "Register `CFG` reader"]
pub type R = crate::R<CfgSpec>;
#[doc = "Register `CFG` writer"]
pub type W = crate::W<CfgSpec>;
#[doc = "Field `HIGHPRECEN` reader - High Precision Enable"]
pub type HighprecenR = crate::BitReader;
#[doc = "Field `HIGHPRECEN` writer - High Precision Enable"]
pub type HighprecenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - High Precision Enable"]
    #[inline(always)]
    pub fn highprecen(&self) -> HighprecenR {
        HighprecenR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - High Precision Enable"]
    #[inline(always)]
    pub fn highprecen(&mut self) -> HighprecenW<'_, CfgSpec> {
        HighprecenW::new(self, 0)
    }
}
#[doc = "Configuration register\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CfgSpec;
impl crate::RegisterSpec for CfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cfg::R`](R) reader structure"]
impl crate::Readable for CfgSpec {}
#[doc = "`write(|w| ..)` method takes [`cfg::W`](W) writer structure"]
impl crate::Writable for CfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CFG to value 0"]
impl crate::Resettable for CfgSpec {}
