#[doc = "Register `FETCHLEN` reader"]
pub type R = crate::R<FetchlenSpec>;
#[doc = "Register `FETCHLEN` writer"]
pub type W = crate::W<FetchlenSpec>;
#[doc = "Field `LENGTH` reader - Length of data block"]
pub type LengthR = crate::FieldReader<u32>;
#[doc = "Field `LENGTH` writer - Length of data block"]
pub type LengthW<'a, REG> = crate::FieldWriter<'a, REG, 28, u32>;
#[doc = "Field `CONSTADDR` reader - Constant address"]
pub type ConstaddrR = crate::BitReader;
#[doc = "Field `CONSTADDR` writer - Constant address"]
pub type ConstaddrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `REALIGN` reader - Realign lengh"]
pub type RealignR = crate::BitReader;
#[doc = "Field `REALIGN` writer - Realign lengh"]
pub type RealignW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bits 0:27 - Length of data block"]
    #[inline(always)]
    pub fn length(&self) -> LengthR {
        LengthR::new(self.bits & 0x0fff_ffff)
    }
    #[doc = "Bit 28 - Constant address"]
    #[inline(always)]
    pub fn constaddr(&self) -> ConstaddrR {
        ConstaddrR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Realign lengh"]
    #[inline(always)]
    pub fn realign(&self) -> RealignR {
        RealignR::new(((self.bits >> 29) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:27 - Length of data block"]
    #[inline(always)]
    pub fn length(&mut self) -> LengthW<'_, FetchlenSpec> {
        LengthW::new(self, 0)
    }
    #[doc = "Bit 28 - Constant address"]
    #[inline(always)]
    pub fn constaddr(&mut self) -> ConstaddrW<'_, FetchlenSpec> {
        ConstaddrW::new(self, 28)
    }
    #[doc = "Bit 29 - Realign lengh"]
    #[inline(always)]
    pub fn realign(&mut self) -> RealignW<'_, FetchlenSpec> {
        RealignW::new(self, 29)
    }
}
#[doc = "Fetcher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used.\n\nYou can [`read`](crate::Reg::read) this register and get [`fetchlen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fetchlen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FetchlenSpec;
impl crate::RegisterSpec for FetchlenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fetchlen::R`](R) reader structure"]
impl crate::Readable for FetchlenSpec {}
#[doc = "`write(|w| ..)` method takes [`fetchlen::W`](W) writer structure"]
impl crate::Writable for FetchlenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FETCHLEN to value 0"]
impl crate::Resettable for FetchlenSpec {}
