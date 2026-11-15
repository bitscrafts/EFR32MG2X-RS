#[doc = "Register `PUSHLEN` reader"]
pub type R = crate::R<PushlenSpec>;
#[doc = "Register `PUSHLEN` writer"]
pub type W = crate::W<PushlenSpec>;
#[doc = "Field `LENGTH` reader - Start address of data block"]
pub type LengthR = crate::FieldReader<u32>;
#[doc = "Field `LENGTH` writer - Start address of data block"]
pub type LengthW<'a, REG> = crate::FieldWriter<'a, REG, 28, u32>;
#[doc = "Field `CONSTADDR` reader - Constant address"]
pub type ConstaddrR = crate::BitReader;
#[doc = "Field `CONSTADDR` writer - Constant address"]
pub type ConstaddrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `REALIGN` reader - Realign length"]
pub type RealignR = crate::BitReader;
#[doc = "Field `REALIGN` writer - Realign length"]
pub type RealignW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DISCARD` reader - Discard data"]
pub type DiscardR = crate::BitReader;
#[doc = "Field `DISCARD` writer - Discard data"]
pub type DiscardW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bits 0:27 - Start address of data block"]
    #[inline(always)]
    pub fn length(&self) -> LengthR {
        LengthR::new(self.bits & 0x0fff_ffff)
    }
    #[doc = "Bit 28 - Constant address"]
    #[inline(always)]
    pub fn constaddr(&self) -> ConstaddrR {
        ConstaddrR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Realign length"]
    #[inline(always)]
    pub fn realign(&self) -> RealignR {
        RealignR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Discard data"]
    #[inline(always)]
    pub fn discard(&self) -> DiscardR {
        DiscardR::new(((self.bits >> 30) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:27 - Start address of data block"]
    #[inline(always)]
    pub fn length(&mut self) -> LengthW<'_, PushlenSpec> {
        LengthW::new(self, 0)
    }
    #[doc = "Bit 28 - Constant address"]
    #[inline(always)]
    pub fn constaddr(&mut self) -> ConstaddrW<'_, PushlenSpec> {
        ConstaddrW::new(self, 28)
    }
    #[doc = "Bit 29 - Realign length"]
    #[inline(always)]
    pub fn realign(&mut self) -> RealignW<'_, PushlenSpec> {
        RealignW::new(self, 29)
    }
    #[doc = "Bit 30 - Discard data"]
    #[inline(always)]
    pub fn discard(&mut self) -> DiscardW<'_, PushlenSpec> {
        DiscardW::new(self, 30)
    }
}
#[doc = "Pusher: Length of data block. In direct mode, this register is written by the software. In scatter-gather mode, this register is not used.\n\nYou can [`read`](crate::Reg::read) this register and get [`pushlen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pushlen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PushlenSpec;
impl crate::RegisterSpec for PushlenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pushlen::R`](R) reader structure"]
impl crate::Readable for PushlenSpec {}
#[doc = "`write(|w| ..)` method takes [`pushlen::W`](W) writer structure"]
impl crate::Writable for PushlenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PUSHLEN to value 0"]
impl crate::Resettable for PushlenSpec {}
