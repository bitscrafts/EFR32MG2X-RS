#[doc = "Register `IEN` reader"]
pub type R = crate::R<IenSpec>;
#[doc = "Register `IEN` writer"]
pub type W = crate::W<IenSpec>;
#[doc = "Field `FETCHERENDOFBLOCK` reader - End of block interrupt enable"]
pub type FetcherendofblockR = crate::BitReader;
#[doc = "Field `FETCHERENDOFBLOCK` writer - End of block interrupt enable"]
pub type FetcherendofblockW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FETCHERSTOPPED` reader - Stopped interrupt enable"]
pub type FetcherstoppedR = crate::BitReader;
#[doc = "Field `FETCHERSTOPPED` writer - Stopped interrupt enable"]
pub type FetcherstoppedW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FETCHERERROR` reader - Error interrupt enable"]
pub type FetchererrorR = crate::BitReader;
#[doc = "Field `FETCHERERROR` writer - Error interrupt enable"]
pub type FetchererrorW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PUSHERENDOFBLOCK` reader - End of block interrupt enable"]
pub type PusherendofblockR = crate::BitReader;
#[doc = "Field `PUSHERENDOFBLOCK` writer - End of block interrupt enable"]
pub type PusherendofblockW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PUSHERSTOPPED` reader - Stopped interrupt enable"]
pub type PusherstoppedR = crate::BitReader;
#[doc = "Field `PUSHERSTOPPED` writer - Stopped interrupt enable"]
pub type PusherstoppedW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PUSHERERROR` reader - Error interrupt enable"]
pub type PushererrorR = crate::BitReader;
#[doc = "Field `PUSHERERROR` writer - Error interrupt enable"]
pub type PushererrorW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - End of block interrupt enable"]
    #[inline(always)]
    pub fn fetcherendofblock(&self) -> FetcherendofblockR {
        FetcherendofblockR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Stopped interrupt enable"]
    #[inline(always)]
    pub fn fetcherstopped(&self) -> FetcherstoppedR {
        FetcherstoppedR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Error interrupt enable"]
    #[inline(always)]
    pub fn fetchererror(&self) -> FetchererrorR {
        FetchererrorR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - End of block interrupt enable"]
    #[inline(always)]
    pub fn pusherendofblock(&self) -> PusherendofblockR {
        PusherendofblockR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Stopped interrupt enable"]
    #[inline(always)]
    pub fn pusherstopped(&self) -> PusherstoppedR {
        PusherstoppedR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Error interrupt enable"]
    #[inline(always)]
    pub fn pushererror(&self) -> PushererrorR {
        PushererrorR::new(((self.bits >> 5) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - End of block interrupt enable"]
    #[inline(always)]
    pub fn fetcherendofblock(&mut self) -> FetcherendofblockW<'_, IenSpec> {
        FetcherendofblockW::new(self, 0)
    }
    #[doc = "Bit 1 - Stopped interrupt enable"]
    #[inline(always)]
    pub fn fetcherstopped(&mut self) -> FetcherstoppedW<'_, IenSpec> {
        FetcherstoppedW::new(self, 1)
    }
    #[doc = "Bit 2 - Error interrupt enable"]
    #[inline(always)]
    pub fn fetchererror(&mut self) -> FetchererrorW<'_, IenSpec> {
        FetchererrorW::new(self, 2)
    }
    #[doc = "Bit 3 - End of block interrupt enable"]
    #[inline(always)]
    pub fn pusherendofblock(&mut self) -> PusherendofblockW<'_, IenSpec> {
        PusherendofblockW::new(self, 3)
    }
    #[doc = "Bit 4 - Stopped interrupt enable"]
    #[inline(always)]
    pub fn pusherstopped(&mut self) -> PusherstoppedW<'_, IenSpec> {
        PusherstoppedW::new(self, 4)
    }
    #[doc = "Bit 5 - Error interrupt enable"]
    #[inline(always)]
    pub fn pushererror(&mut self) -> PushererrorW<'_, IenSpec> {
        PushererrorW::new(self, 5)
    }
}
#[doc = "Interrupt enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IenSpec;
impl crate::RegisterSpec for IenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ien::R`](R) reader structure"]
impl crate::Readable for IenSpec {}
#[doc = "`write(|w| ..)` method takes [`ien::W`](W) writer structure"]
impl crate::Writable for IenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IEN to value 0"]
impl crate::Resettable for IenSpec {}
