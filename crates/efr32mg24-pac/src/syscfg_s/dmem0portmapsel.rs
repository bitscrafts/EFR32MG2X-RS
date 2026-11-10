#[doc = "Register `DMEM0PORTMAPSEL` reader"]
pub type R = crate::R<Dmem0portmapselSpec>;
#[doc = "Register `DMEM0PORTMAPSEL` writer"]
pub type W = crate::W<Dmem0portmapselSpec>;
#[doc = "Field `LDMAPORTSEL` reader - LDMA portmap selection"]
pub type LdmaportselR = crate::FieldReader;
#[doc = "Field `LDMAPORTSEL` writer - LDMA portmap selection"]
pub type LdmaportselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `SRWAESPORTSEL` reader - SRWAES portmap selection"]
pub type SrwaesportselR = crate::FieldReader;
#[doc = "Field `SRWAESPORTSEL` writer - SRWAES portmap selection"]
pub type SrwaesportselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `AHBSRWPORTSEL` reader - AHBSRW portmap selection"]
pub type AhbsrwportselR = crate::FieldReader;
#[doc = "Field `AHBSRWPORTSEL` writer - AHBSRW portmap selection"]
pub type AhbsrwportselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `SRWECA0PORTSEL` reader - SRWECA0 portmap selection"]
pub type Srweca0portselR = crate::FieldReader;
#[doc = "Field `SRWECA0PORTSEL` writer - SRWECA0 portmap selection"]
pub type Srweca0portselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `SRWECA1PORTSEL` reader - SRWECA1 portmap selection"]
pub type Srweca1portselR = crate::FieldReader;
#[doc = "Field `SRWECA1PORTSEL` writer - SRWECA1 portmap selection"]
pub type Srweca1portselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `MVPAHBDATA0PORTSEL` reader - MVPAHBDATA0 portmap selection"]
pub type Mvpahbdata0portselR = crate::FieldReader;
#[doc = "Field `MVPAHBDATA0PORTSEL` writer - MVPAHBDATA0 portmap selection"]
pub type Mvpahbdata0portselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `MVPAHBDATA1PORTSEL` reader - MVPAHBDATA1 portmap selection"]
pub type Mvpahbdata1portselR = crate::FieldReader;
#[doc = "Field `MVPAHBDATA1PORTSEL` writer - MVPAHBDATA1 portmap selection"]
pub type Mvpahbdata1portselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `MVPAHBDATA2PORTSEL` reader - MVPAHBDATA2 portmap selection"]
pub type Mvpahbdata2portselR = crate::FieldReader;
#[doc = "Field `MVPAHBDATA2PORTSEL` writer - MVPAHBDATA2 portmap selection"]
pub type Mvpahbdata2portselW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
impl R {
    #[doc = "Bits 0:1 - LDMA portmap selection"]
    #[inline(always)]
    pub fn ldmaportsel(&self) -> LdmaportselR {
        LdmaportselR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - SRWAES portmap selection"]
    #[inline(always)]
    pub fn srwaesportsel(&self) -> SrwaesportselR {
        SrwaesportselR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - AHBSRW portmap selection"]
    #[inline(always)]
    pub fn ahbsrwportsel(&self) -> AhbsrwportselR {
        AhbsrwportselR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - SRWECA0 portmap selection"]
    #[inline(always)]
    pub fn srweca0portsel(&self) -> Srweca0portselR {
        Srweca0portselR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - SRWECA1 portmap selection"]
    #[inline(always)]
    pub fn srweca1portsel(&self) -> Srweca1portselR {
        Srweca1portselR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - MVPAHBDATA0 portmap selection"]
    #[inline(always)]
    pub fn mvpahbdata0portsel(&self) -> Mvpahbdata0portselR {
        Mvpahbdata0portselR::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - MVPAHBDATA1 portmap selection"]
    #[inline(always)]
    pub fn mvpahbdata1portsel(&self) -> Mvpahbdata1portselR {
        Mvpahbdata1portselR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - MVPAHBDATA2 portmap selection"]
    #[inline(always)]
    pub fn mvpahbdata2portsel(&self) -> Mvpahbdata2portselR {
        Mvpahbdata2portselR::new(((self.bits >> 14) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - LDMA portmap selection"]
    #[inline(always)]
    pub fn ldmaportsel(&mut self) -> LdmaportselW<'_, Dmem0portmapselSpec> {
        LdmaportselW::new(self, 0)
    }
    #[doc = "Bits 2:3 - SRWAES portmap selection"]
    #[inline(always)]
    pub fn srwaesportsel(&mut self) -> SrwaesportselW<'_, Dmem0portmapselSpec> {
        SrwaesportselW::new(self, 2)
    }
    #[doc = "Bits 4:5 - AHBSRW portmap selection"]
    #[inline(always)]
    pub fn ahbsrwportsel(&mut self) -> AhbsrwportselW<'_, Dmem0portmapselSpec> {
        AhbsrwportselW::new(self, 4)
    }
    #[doc = "Bits 6:7 - SRWECA0 portmap selection"]
    #[inline(always)]
    pub fn srweca0portsel(&mut self) -> Srweca0portselW<'_, Dmem0portmapselSpec> {
        Srweca0portselW::new(self, 6)
    }
    #[doc = "Bits 8:9 - SRWECA1 portmap selection"]
    #[inline(always)]
    pub fn srweca1portsel(&mut self) -> Srweca1portselW<'_, Dmem0portmapselSpec> {
        Srweca1portselW::new(self, 8)
    }
    #[doc = "Bits 10:11 - MVPAHBDATA0 portmap selection"]
    #[inline(always)]
    pub fn mvpahbdata0portsel(&mut self) -> Mvpahbdata0portselW<'_, Dmem0portmapselSpec> {
        Mvpahbdata0portselW::new(self, 10)
    }
    #[doc = "Bits 12:13 - MVPAHBDATA1 portmap selection"]
    #[inline(always)]
    pub fn mvpahbdata1portsel(&mut self) -> Mvpahbdata1portselW<'_, Dmem0portmapselSpec> {
        Mvpahbdata1portselW::new(self, 12)
    }
    #[doc = "Bits 14:15 - MVPAHBDATA2 portmap selection"]
    #[inline(always)]
    pub fn mvpahbdata2portsel(&mut self) -> Mvpahbdata2portselW<'_, Dmem0portmapselSpec> {
        Mvpahbdata2portselW::new(self, 14)
    }
}
#[doc = "Configure DMEM0 port remap selection.\n\nYou can [`read`](crate::Reg::read) this register and get [`dmem0portmapsel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dmem0portmapsel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Dmem0portmapselSpec;
impl crate::RegisterSpec for Dmem0portmapselSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dmem0portmapsel::R`](R) reader structure"]
impl crate::Readable for Dmem0portmapselSpec {}
#[doc = "`write(|w| ..)` method takes [`dmem0portmapsel::W`](W) writer structure"]
impl crate::Writable for Dmem0portmapselSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DMEM0PORTMAPSEL to value 0x7905"]
impl crate::Resettable for Dmem0portmapselSpec {
    const RESET_VALUE: u32 = 0x7905;
}
