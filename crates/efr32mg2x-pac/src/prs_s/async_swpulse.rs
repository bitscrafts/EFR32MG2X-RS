#[doc = "Register `ASYNC_SWPULSE` writer"]
pub type W = crate::W<AsyncSwpulseSpec>;
#[doc = "Field `CH0PULSE` writer - Channel pulse"]
pub type Ch0pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH1PULSE` writer - Channel pulse"]
pub type Ch1pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH2PULSE` writer - Channel pulse"]
pub type Ch2pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH3PULSE` writer - Channel pulse"]
pub type Ch3pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH4PULSE` writer - Channel pulse"]
pub type Ch4pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH5PULSE` writer - Channel pulse"]
pub type Ch5pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH6PULSE` writer - Channel pulse"]
pub type Ch6pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH7PULSE` writer - Channel pulse"]
pub type Ch7pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH8PULSE` writer - Channel pulse"]
pub type Ch8pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH9PULSE` writer - Channel pulse"]
pub type Ch9pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH10PULSE` writer - Channel pulse"]
pub type Ch10pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH11PULSE` writer - Channel pulse"]
pub type Ch11pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH12PULSE` writer - Channel pulse"]
pub type Ch12pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH13PULSE` writer - Channel pulse"]
pub type Ch13pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH14PULSE` writer - Channel pulse"]
pub type Ch14pulseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CH15PULSE` writer - Channel pulse"]
pub type Ch15pulseW<'a, REG> = crate::BitWriter<'a, REG>;
impl W {
    #[doc = "Bit 0 - Channel pulse"]
    #[inline(always)]
    pub fn ch0pulse(&mut self) -> Ch0pulseW<'_, AsyncSwpulseSpec> {
        Ch0pulseW::new(self, 0)
    }
    #[doc = "Bit 1 - Channel pulse"]
    #[inline(always)]
    pub fn ch1pulse(&mut self) -> Ch1pulseW<'_, AsyncSwpulseSpec> {
        Ch1pulseW::new(self, 1)
    }
    #[doc = "Bit 2 - Channel pulse"]
    #[inline(always)]
    pub fn ch2pulse(&mut self) -> Ch2pulseW<'_, AsyncSwpulseSpec> {
        Ch2pulseW::new(self, 2)
    }
    #[doc = "Bit 3 - Channel pulse"]
    #[inline(always)]
    pub fn ch3pulse(&mut self) -> Ch3pulseW<'_, AsyncSwpulseSpec> {
        Ch3pulseW::new(self, 3)
    }
    #[doc = "Bit 4 - Channel pulse"]
    #[inline(always)]
    pub fn ch4pulse(&mut self) -> Ch4pulseW<'_, AsyncSwpulseSpec> {
        Ch4pulseW::new(self, 4)
    }
    #[doc = "Bit 5 - Channel pulse"]
    #[inline(always)]
    pub fn ch5pulse(&mut self) -> Ch5pulseW<'_, AsyncSwpulseSpec> {
        Ch5pulseW::new(self, 5)
    }
    #[doc = "Bit 6 - Channel pulse"]
    #[inline(always)]
    pub fn ch6pulse(&mut self) -> Ch6pulseW<'_, AsyncSwpulseSpec> {
        Ch6pulseW::new(self, 6)
    }
    #[doc = "Bit 7 - Channel pulse"]
    #[inline(always)]
    pub fn ch7pulse(&mut self) -> Ch7pulseW<'_, AsyncSwpulseSpec> {
        Ch7pulseW::new(self, 7)
    }
    #[doc = "Bit 8 - Channel pulse"]
    #[inline(always)]
    pub fn ch8pulse(&mut self) -> Ch8pulseW<'_, AsyncSwpulseSpec> {
        Ch8pulseW::new(self, 8)
    }
    #[doc = "Bit 9 - Channel pulse"]
    #[inline(always)]
    pub fn ch9pulse(&mut self) -> Ch9pulseW<'_, AsyncSwpulseSpec> {
        Ch9pulseW::new(self, 9)
    }
    #[doc = "Bit 10 - Channel pulse"]
    #[inline(always)]
    pub fn ch10pulse(&mut self) -> Ch10pulseW<'_, AsyncSwpulseSpec> {
        Ch10pulseW::new(self, 10)
    }
    #[doc = "Bit 11 - Channel pulse"]
    #[inline(always)]
    pub fn ch11pulse(&mut self) -> Ch11pulseW<'_, AsyncSwpulseSpec> {
        Ch11pulseW::new(self, 11)
    }
    #[doc = "Bit 12 - Channel pulse"]
    #[inline(always)]
    pub fn ch12pulse(&mut self) -> Ch12pulseW<'_, AsyncSwpulseSpec> {
        Ch12pulseW::new(self, 12)
    }
    #[doc = "Bit 13 - Channel pulse"]
    #[inline(always)]
    pub fn ch13pulse(&mut self) -> Ch13pulseW<'_, AsyncSwpulseSpec> {
        Ch13pulseW::new(self, 13)
    }
    #[doc = "Bit 14 - Channel pulse"]
    #[inline(always)]
    pub fn ch14pulse(&mut self) -> Ch14pulseW<'_, AsyncSwpulseSpec> {
        Ch14pulseW::new(self, 14)
    }
    #[doc = "Bit 15 - Channel pulse"]
    #[inline(always)]
    pub fn ch15pulse(&mut self) -> Ch15pulseW<'_, AsyncSwpulseSpec> {
        Ch15pulseW::new(self, 15)
    }
}
#[doc = "No Description\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`async_swpulse::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct AsyncSwpulseSpec;
impl crate::RegisterSpec for AsyncSwpulseSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`async_swpulse::W`](W) writer structure"]
impl crate::Writable for AsyncSwpulseSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ASYNC_SWPULSE to value 0"]
impl crate::Resettable for AsyncSwpulseSpec {}
