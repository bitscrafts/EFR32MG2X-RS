#[doc = "Register `PUSHADDR` reader"]
pub type R = crate::R<PushaddrSpec>;
#[doc = "Register `PUSHADDR` writer"]
pub type W = crate::W<PushaddrSpec>;
#[doc = "Field `ADDR` reader - Start address of data block"]
pub type AddrR = crate::FieldReader<u32>;
#[doc = "Field `ADDR` writer - Start address of data block"]
pub type AddrW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Start address of data block"]
    #[inline(always)]
    pub fn addr(&self) -> AddrR {
        AddrR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Start address of data block"]
    #[inline(always)]
    pub fn addr(&mut self) -> AddrW<'_, PushaddrSpec> {
        AddrW::new(self, 0)
    }
}
#[doc = "Pusher: Start address of data block (LSB). In direct mode, this register is written by the software. In scatter-gather mode, this register is updated after each processed descriptor.\n\nYou can [`read`](crate::Reg::read) this register and get [`pushaddr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pushaddr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PushaddrSpec;
impl crate::RegisterSpec for PushaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pushaddr::R`](R) reader structure"]
impl crate::Readable for PushaddrSpec {}
#[doc = "`write(|w| ..)` method takes [`pushaddr::W`](W) writer structure"]
impl crate::Writable for PushaddrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PUSHADDR to value 0"]
impl crate::Resettable for PushaddrSpec {}
