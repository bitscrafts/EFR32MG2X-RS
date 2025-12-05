#[doc = "Register `PROGRAMSTATE` reader"]
pub type R = crate::R<ProgramstateSpec>;
#[doc = "Register `PROGRAMSTATE` writer"]
pub type W = crate::W<ProgramstateSpec>;
#[doc = "Field `PC` reader - Program Counter"]
pub type PcR = crate::FieldReader;
#[doc = "Field `PC` writer - Program Counter"]
pub type PcW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:2 - Program Counter"]
    #[inline(always)]
    pub fn pc(&self) -> PcR {
        PcR::new((self.bits & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Program Counter"]
    #[inline(always)]
    pub fn pc(&mut self) -> PcW<'_, ProgramstateSpec> {
        PcW::new(self, 0)
    }
}
#[doc = "Program State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`programstate::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`programstate::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ProgramstateSpec;
impl crate::RegisterSpec for ProgramstateSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`programstate::R`](R) reader structure"]
impl crate::Readable for ProgramstateSpec {}
#[doc = "`write(|w| ..)` method takes [`programstate::W`](W) writer structure"]
impl crate::Writable for ProgramstateSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PROGRAMSTATE to value 0"]
impl crate::Resettable for ProgramstateSpec {}
