#[doc = "Register `DEBUGEN` reader"]
pub type R = crate::R<DebugenSpec>;
#[doc = "Register `DEBUGEN` writer"]
pub type W = crate::W<DebugenSpec>;
#[doc = "Field `BKPTLOOP0DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop0doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP0DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop0doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTLOOP1DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop1doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP1DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop1doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTLOOP2DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop2doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP2DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop2doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTLOOP3DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop3doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP3DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop3doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTLOOP4DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop4doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP4DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop4doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTLOOP5DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop5doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP5DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop5doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTLOOP6DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop6doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP6DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop6doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTLOOP7DONE` reader - Enable Breakpoint on Loop Done"]
pub type Bkptloop7doneR = crate::BitReader;
#[doc = "Field `BKPTLOOP7DONE` writer - Enable Breakpoint on Loop Done"]
pub type Bkptloop7doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTALUNAN` reader - Enable Breakpoint on ALUNAN"]
pub type BkptalunanR = crate::BitReader;
#[doc = "Field `BKPTALUNAN` writer - Enable Breakpoint on ALUNAN"]
pub type BkptalunanW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTR0POSREAL` reader - Enable Breakpoint on R0POSREAL"]
pub type Bkptr0posrealR = crate::BitReader;
#[doc = "Field `BKPTR0POSREAL` writer - Enable Breakpoint on R0POSREAL"]
pub type Bkptr0posrealW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTALUOF` reader - Enable Breakpoint on ALUOF"]
pub type BkptaluofR = crate::BitReader;
#[doc = "Field `BKPTALUOF` writer - Enable Breakpoint on ALUOF"]
pub type BkptaluofW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTALUUF` reader - Enable Breakpoint on ALUUF"]
pub type BkptaluufR = crate::BitReader;
#[doc = "Field `BKPTALUUF` writer - Enable Breakpoint on ALUUF"]
pub type BkptaluufW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTSTORECONVERTOF` reader - Enable Breakpoint on STORECONVERTOF"]
pub type BkptstoreconvertofR = crate::BitReader;
#[doc = "Field `BKPTSTORECONVERTOF` writer - Enable Breakpoint on STORECONVERTOF"]
pub type BkptstoreconvertofW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTSTORECONVERTUF` reader - Enable Breakpoint on STORECONVERTUF"]
pub type BkptstoreconvertufR = crate::BitReader;
#[doc = "Field `BKPTSTORECONVERTUF` writer - Enable Breakpoint on STORECONVERTUF"]
pub type BkptstoreconvertufW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTSTORECONVERTINF` reader - Enable Breakpoint on STORECONVERTINF"]
pub type BkptstoreconvertinfR = crate::BitReader;
#[doc = "Field `BKPTSTORECONVERTINF` writer - Enable Breakpoint on STORECONVERTINF"]
pub type BkptstoreconvertinfW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BKPTSTORECONVERTNAN` reader - Enable Breakpoint on STORECONVERTNAN"]
pub type BkptstoreconvertnanR = crate::BitReader;
#[doc = "Field `BKPTSTORECONVERTNAN` writer - Enable Breakpoint on STORECONVERTNAN"]
pub type BkptstoreconvertnanW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DEBUGSTEPCNTEN` reader - Debug Step Count Enable"]
pub type DebugstepcntenR = crate::BitReader;
#[doc = "Field `DEBUGSTEPCNTEN` writer - Debug Step Count Enable"]
pub type DebugstepcntenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DEBUGBKPTALLEN` reader - Trigger Breakpoint when ALL conditions match"]
pub type DebugbkptallenR = crate::BitReader;
#[doc = "Field `DEBUGBKPTALLEN` writer - Trigger Breakpoint when ALL conditions match"]
pub type DebugbkptallenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `DEBUGBKPTANYEN` reader - Enable Breakpoint when ANY conditions match"]
pub type DebugbkptanyenR = crate::BitReader;
#[doc = "Field `DEBUGBKPTANYEN` writer - Enable Breakpoint when ANY conditions match"]
pub type DebugbkptanyenW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 1 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop0done(&self) -> Bkptloop0doneR {
        Bkptloop0doneR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop1done(&self) -> Bkptloop1doneR {
        Bkptloop1doneR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop2done(&self) -> Bkptloop2doneR {
        Bkptloop2doneR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop3done(&self) -> Bkptloop3doneR {
        Bkptloop3doneR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop4done(&self) -> Bkptloop4doneR {
        Bkptloop4doneR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop5done(&self) -> Bkptloop5doneR {
        Bkptloop5doneR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop6done(&self) -> Bkptloop6doneR {
        Bkptloop6doneR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop7done(&self) -> Bkptloop7doneR {
        Bkptloop7doneR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 10 - Enable Breakpoint on ALUNAN"]
    #[inline(always)]
    pub fn bkptalunan(&self) -> BkptalunanR {
        BkptalunanR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Enable Breakpoint on R0POSREAL"]
    #[inline(always)]
    pub fn bkptr0posreal(&self) -> Bkptr0posrealR {
        Bkptr0posrealR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Enable Breakpoint on ALUOF"]
    #[inline(always)]
    pub fn bkptaluof(&self) -> BkptaluofR {
        BkptaluofR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Enable Breakpoint on ALUUF"]
    #[inline(always)]
    pub fn bkptaluuf(&self) -> BkptaluufR {
        BkptaluufR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Enable Breakpoint on STORECONVERTOF"]
    #[inline(always)]
    pub fn bkptstoreconvertof(&self) -> BkptstoreconvertofR {
        BkptstoreconvertofR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Enable Breakpoint on STORECONVERTUF"]
    #[inline(always)]
    pub fn bkptstoreconvertuf(&self) -> BkptstoreconvertufR {
        BkptstoreconvertufR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Enable Breakpoint on STORECONVERTINF"]
    #[inline(always)]
    pub fn bkptstoreconvertinf(&self) -> BkptstoreconvertinfR {
        BkptstoreconvertinfR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Enable Breakpoint on STORECONVERTNAN"]
    #[inline(always)]
    pub fn bkptstoreconvertnan(&self) -> BkptstoreconvertnanR {
        BkptstoreconvertnanR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 28 - Debug Step Count Enable"]
    #[inline(always)]
    pub fn debugstepcnten(&self) -> DebugstepcntenR {
        DebugstepcntenR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Trigger Breakpoint when ALL conditions match"]
    #[inline(always)]
    pub fn debugbkptallen(&self) -> DebugbkptallenR {
        DebugbkptallenR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Enable Breakpoint when ANY conditions match"]
    #[inline(always)]
    pub fn debugbkptanyen(&self) -> DebugbkptanyenR {
        DebugbkptanyenR::new(((self.bits >> 30) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop0done(&mut self) -> Bkptloop0doneW<'_, DebugenSpec> {
        Bkptloop0doneW::new(self, 1)
    }
    #[doc = "Bit 2 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop1done(&mut self) -> Bkptloop1doneW<'_, DebugenSpec> {
        Bkptloop1doneW::new(self, 2)
    }
    #[doc = "Bit 3 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop2done(&mut self) -> Bkptloop2doneW<'_, DebugenSpec> {
        Bkptloop2doneW::new(self, 3)
    }
    #[doc = "Bit 4 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop3done(&mut self) -> Bkptloop3doneW<'_, DebugenSpec> {
        Bkptloop3doneW::new(self, 4)
    }
    #[doc = "Bit 5 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop4done(&mut self) -> Bkptloop4doneW<'_, DebugenSpec> {
        Bkptloop4doneW::new(self, 5)
    }
    #[doc = "Bit 6 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop5done(&mut self) -> Bkptloop5doneW<'_, DebugenSpec> {
        Bkptloop5doneW::new(self, 6)
    }
    #[doc = "Bit 7 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop6done(&mut self) -> Bkptloop6doneW<'_, DebugenSpec> {
        Bkptloop6doneW::new(self, 7)
    }
    #[doc = "Bit 8 - Enable Breakpoint on Loop Done"]
    #[inline(always)]
    pub fn bkptloop7done(&mut self) -> Bkptloop7doneW<'_, DebugenSpec> {
        Bkptloop7doneW::new(self, 8)
    }
    #[doc = "Bit 10 - Enable Breakpoint on ALUNAN"]
    #[inline(always)]
    pub fn bkptalunan(&mut self) -> BkptalunanW<'_, DebugenSpec> {
        BkptalunanW::new(self, 10)
    }
    #[doc = "Bit 11 - Enable Breakpoint on R0POSREAL"]
    #[inline(always)]
    pub fn bkptr0posreal(&mut self) -> Bkptr0posrealW<'_, DebugenSpec> {
        Bkptr0posrealW::new(self, 11)
    }
    #[doc = "Bit 12 - Enable Breakpoint on ALUOF"]
    #[inline(always)]
    pub fn bkptaluof(&mut self) -> BkptaluofW<'_, DebugenSpec> {
        BkptaluofW::new(self, 12)
    }
    #[doc = "Bit 13 - Enable Breakpoint on ALUUF"]
    #[inline(always)]
    pub fn bkptaluuf(&mut self) -> BkptaluufW<'_, DebugenSpec> {
        BkptaluufW::new(self, 13)
    }
    #[doc = "Bit 14 - Enable Breakpoint on STORECONVERTOF"]
    #[inline(always)]
    pub fn bkptstoreconvertof(&mut self) -> BkptstoreconvertofW<'_, DebugenSpec> {
        BkptstoreconvertofW::new(self, 14)
    }
    #[doc = "Bit 15 - Enable Breakpoint on STORECONVERTUF"]
    #[inline(always)]
    pub fn bkptstoreconvertuf(&mut self) -> BkptstoreconvertufW<'_, DebugenSpec> {
        BkptstoreconvertufW::new(self, 15)
    }
    #[doc = "Bit 16 - Enable Breakpoint on STORECONVERTINF"]
    #[inline(always)]
    pub fn bkptstoreconvertinf(&mut self) -> BkptstoreconvertinfW<'_, DebugenSpec> {
        BkptstoreconvertinfW::new(self, 16)
    }
    #[doc = "Bit 17 - Enable Breakpoint on STORECONVERTNAN"]
    #[inline(always)]
    pub fn bkptstoreconvertnan(&mut self) -> BkptstoreconvertnanW<'_, DebugenSpec> {
        BkptstoreconvertnanW::new(self, 17)
    }
    #[doc = "Bit 28 - Debug Step Count Enable"]
    #[inline(always)]
    pub fn debugstepcnten(&mut self) -> DebugstepcntenW<'_, DebugenSpec> {
        DebugstepcntenW::new(self, 28)
    }
    #[doc = "Bit 29 - Trigger Breakpoint when ALL conditions match"]
    #[inline(always)]
    pub fn debugbkptallen(&mut self) -> DebugbkptallenW<'_, DebugenSpec> {
        DebugbkptallenW::new(self, 29)
    }
    #[doc = "Bit 30 - Enable Breakpoint when ANY conditions match"]
    #[inline(always)]
    pub fn debugbkptanyen(&mut self) -> DebugbkptanyenW<'_, DebugenSpec> {
        DebugbkptanyenW::new(self, 30)
    }
}
#[doc = "Debug Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`debugen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`debugen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DebugenSpec;
impl crate::RegisterSpec for DebugenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`debugen::R`](R) reader structure"]
impl crate::Readable for DebugenSpec {}
#[doc = "`write(|w| ..)` method takes [`debugen::W`](W) writer structure"]
impl crate::Writable for DebugenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DEBUGEN to value 0"]
impl crate::Resettable for DebugenSpec {}
