#[doc = "Register `ALU6REGSTATE` reader"]
pub type R = crate::R<Alu6regstateSpec>;
#[doc = "Register `ALU6REGSTATE` writer"]
pub type W = crate::W<Alu6regstateSpec>;
#[doc = "Field `FREAL` reader - Float Real Value"]
pub type FrealR = crate::FieldReader<u16>;
#[doc = "Field `FREAL` writer - Float Real Value"]
pub type FrealW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `FIMAG` reader - Float Imaginary Value"]
pub type FimagR = crate::FieldReader<u16>;
#[doc = "Field `FIMAG` writer - Float Imaginary Value"]
pub type FimagW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Float Real Value"]
    #[inline(always)]
    pub fn freal(&self) -> FrealR {
        FrealR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Float Imaginary Value"]
    #[inline(always)]
    pub fn fimag(&self) -> FimagR {
        FimagR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Float Real Value"]
    #[inline(always)]
    pub fn freal(&mut self) -> FrealW<'_, Alu6regstateSpec> {
        FrealW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Float Imaginary Value"]
    #[inline(always)]
    pub fn fimag(&mut self) -> FimagW<'_, Alu6regstateSpec> {
        FimagW::new(self, 16)
    }
}
#[doc = "ALU Register\n\nYou can [`read`](crate::Reg::read) this register and get [`alu6regstate::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`alu6regstate::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Alu6regstateSpec;
impl crate::RegisterSpec for Alu6regstateSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`alu6regstate::R`](R) reader structure"]
impl crate::Readable for Alu6regstateSpec {}
#[doc = "`write(|w| ..)` method takes [`alu6regstate::W`](W) writer structure"]
impl crate::Writable for Alu6regstateSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ALU6REGSTATE to value 0"]
impl crate::Resettable for Alu6regstateSpec {}
