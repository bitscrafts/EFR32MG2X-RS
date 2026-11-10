#[doc = "Register `IEN` reader"]
pub type R = crate::R<IenSpec>;
#[doc = "Register `IEN` writer"]
pub type W = crate::W<IenSpec>;
#[doc = "Field `PROGDONE` reader - Program Done Interrupt Enable"]
pub type ProgdoneR = crate::BitReader;
#[doc = "Field `PROGDONE` writer - Program Done Interrupt Enable"]
pub type ProgdoneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP0DONE` reader - Loop Done Interrupt Enable"]
pub type Loop0doneR = crate::BitReader;
#[doc = "Field `LOOP0DONE` writer - Loop Done Interrupt Enable"]
pub type Loop0doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP1DONE` reader - Loop Done Interrupt Enable"]
pub type Loop1doneR = crate::BitReader;
#[doc = "Field `LOOP1DONE` writer - Loop Done Interrupt Enable"]
pub type Loop1doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP2DONE` reader - Loop Done Interrupt Enable"]
pub type Loop2doneR = crate::BitReader;
#[doc = "Field `LOOP2DONE` writer - Loop Done Interrupt Enable"]
pub type Loop2doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP3DONE` reader - Loop Done Interrupt Enable"]
pub type Loop3doneR = crate::BitReader;
#[doc = "Field `LOOP3DONE` writer - Loop Done Interrupt Enable"]
pub type Loop3doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP4DONE` reader - Loop Done Interrupt Enable"]
pub type Loop4doneR = crate::BitReader;
#[doc = "Field `LOOP4DONE` writer - Loop Done Interrupt Enable"]
pub type Loop4doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP5DONE` reader - Loop Done Interrupt Enable"]
pub type Loop5doneR = crate::BitReader;
#[doc = "Field `LOOP5DONE` writer - Loop Done Interrupt Enable"]
pub type Loop5doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP6DONE` reader - Loop Done Interrupt Enable"]
pub type Loop6doneR = crate::BitReader;
#[doc = "Field `LOOP6DONE` writer - Loop Done Interrupt Enable"]
pub type Loop6doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOP7DONE` reader - Loop Done Interrupt Enable"]
pub type Loop7doneR = crate::BitReader;
#[doc = "Field `LOOP7DONE` writer - Loop Done Interrupt Enable"]
pub type Loop7doneW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUNAN` reader - Not-a-Number Interrupt Enable"]
pub type AlunanR = crate::BitReader;
#[doc = "Field `ALUNAN` writer - Not-a-Number Interrupt Enable"]
pub type AlunanW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `R0POSREAL` reader - R0 Non-Zero Interrupt Enable"]
pub type R0posrealR = crate::BitReader;
#[doc = "Field `R0POSREAL` writer - R0 Non-Zero Interrupt Enable"]
pub type R0posrealW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUOF` reader - ALU Overflow Interrupt Enable"]
pub type AluofR = crate::BitReader;
#[doc = "Field `ALUOF` writer - ALU Overflow Interrupt Enable"]
pub type AluofW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUUF` reader - ALU Underflow Interrupt Enable"]
pub type AluufR = crate::BitReader;
#[doc = "Field `ALUUF` writer - ALU Underflow Interrupt Enable"]
pub type AluufW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `STORECONVERTOF` reader - Store conversion Overflow Interrupt Enable"]
pub type StoreconvertofR = crate::BitReader;
#[doc = "Field `STORECONVERTOF` writer - Store conversion Overflow Interrupt Enable"]
pub type StoreconvertofW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `STORECONVERTUF` reader - Store Conversion Underflow Interrupt Enable"]
pub type StoreconvertufR = crate::BitReader;
#[doc = "Field `STORECONVERTUF` writer - Store Conversion Underflow Interrupt Enable"]
pub type StoreconvertufW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `STORECONVERTINF` reader - Store Conversion Infinity Interrupt Enable"]
pub type StoreconvertinfR = crate::BitReader;
#[doc = "Field `STORECONVERTINF` writer - Store Conversion Infinity Interrupt Enable"]
pub type StoreconvertinfW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `STORECONVERTNAN` reader - Store Conversion NaN Interrupt Enable"]
pub type StoreconvertnanR = crate::BitReader;
#[doc = "Field `STORECONVERTNAN` writer - Store Conversion NaN Interrupt Enable"]
pub type StoreconvertnanW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PERFCNT0` reader - Perf Counter 0 Overflow Interrupt Enable"]
pub type Perfcnt0R = crate::BitReader;
#[doc = "Field `PERFCNT0` writer - Perf Counter 0 Overflow Interrupt Enable"]
pub type Perfcnt0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `PERFCNT1` reader - Perf Counter 1 Overflow Interrupt Enable"]
pub type Perfcnt1R = crate::BitReader;
#[doc = "Field `PERFCNT1` writer - Perf Counter 1 Overflow Interrupt Enable"]
pub type Perfcnt1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `LOOPFAULT` reader - Loop Fault Interrupt Enable"]
pub type LoopfaultR = crate::BitReader;
#[doc = "Field `LOOPFAULT` writer - Loop Fault Interrupt Enable"]
pub type LoopfaultW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BUSERRFAULT` reader - Bus Error Fault Interrupt Enable"]
pub type BuserrfaultR = crate::BitReader;
#[doc = "Field `BUSERRFAULT` writer - Bus Error Fault Interrupt Enable"]
pub type BuserrfaultW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `BUSALIGNFAULT` reader - Bus Alignment Fault Interrupt Enable"]
pub type BusalignfaultR = crate::BitReader;
#[doc = "Field `BUSALIGNFAULT` writer - Bus Alignment Fault Interrupt Enable"]
pub type BusalignfaultW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALUFAULT` reader - ALU Input Fault Interrupt Enable"]
pub type AlufaultR = crate::BitReader;
#[doc = "Field `ALUFAULT` writer - ALU Input Fault Interrupt Enable"]
pub type AlufaultW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAYFAULT` reader - Array Fault Interrupt Enable"]
pub type ArrayfaultR = crate::BitReader;
#[doc = "Field `ARRAYFAULT` writer - Array Fault Interrupt Enable"]
pub type ArrayfaultW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Program Done Interrupt Enable"]
    #[inline(always)]
    pub fn progdone(&self) -> ProgdoneR {
        ProgdoneR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop0done(&self) -> Loop0doneR {
        Loop0doneR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop1done(&self) -> Loop1doneR {
        Loop1doneR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop2done(&self) -> Loop2doneR {
        Loop2doneR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop3done(&self) -> Loop3doneR {
        Loop3doneR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop4done(&self) -> Loop4doneR {
        Loop4doneR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop5done(&self) -> Loop5doneR {
        Loop5doneR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop6done(&self) -> Loop6doneR {
        Loop6doneR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop7done(&self) -> Loop7doneR {
        Loop7doneR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 10 - Not-a-Number Interrupt Enable"]
    #[inline(always)]
    pub fn alunan(&self) -> AlunanR {
        AlunanR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - R0 Non-Zero Interrupt Enable"]
    #[inline(always)]
    pub fn r0posreal(&self) -> R0posrealR {
        R0posrealR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - ALU Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn aluof(&self) -> AluofR {
        AluofR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - ALU Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn aluuf(&self) -> AluufR {
        AluufR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Store conversion Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertof(&self) -> StoreconvertofR {
        StoreconvertofR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Store Conversion Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertuf(&self) -> StoreconvertufR {
        StoreconvertufR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Store Conversion Infinity Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertinf(&self) -> StoreconvertinfR {
        StoreconvertinfR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Store Conversion NaN Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertnan(&self) -> StoreconvertnanR {
        StoreconvertnanR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Perf Counter 0 Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn perfcnt0(&self) -> Perfcnt0R {
        Perfcnt0R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Perf Counter 1 Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn perfcnt1(&self) -> Perfcnt1R {
        Perfcnt1R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 24 - Loop Fault Interrupt Enable"]
    #[inline(always)]
    pub fn loopfault(&self) -> LoopfaultR {
        LoopfaultR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Bus Error Fault Interrupt Enable"]
    #[inline(always)]
    pub fn buserrfault(&self) -> BuserrfaultR {
        BuserrfaultR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Bus Alignment Fault Interrupt Enable"]
    #[inline(always)]
    pub fn busalignfault(&self) -> BusalignfaultR {
        BusalignfaultR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - ALU Input Fault Interrupt Enable"]
    #[inline(always)]
    pub fn alufault(&self) -> AlufaultR {
        AlufaultR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Array Fault Interrupt Enable"]
    #[inline(always)]
    pub fn arrayfault(&self) -> ArrayfaultR {
        ArrayfaultR::new(((self.bits >> 28) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Program Done Interrupt Enable"]
    #[inline(always)]
    pub fn progdone(&mut self) -> ProgdoneW<'_, IenSpec> {
        ProgdoneW::new(self, 0)
    }
    #[doc = "Bit 1 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop0done(&mut self) -> Loop0doneW<'_, IenSpec> {
        Loop0doneW::new(self, 1)
    }
    #[doc = "Bit 2 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop1done(&mut self) -> Loop1doneW<'_, IenSpec> {
        Loop1doneW::new(self, 2)
    }
    #[doc = "Bit 3 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop2done(&mut self) -> Loop2doneW<'_, IenSpec> {
        Loop2doneW::new(self, 3)
    }
    #[doc = "Bit 4 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop3done(&mut self) -> Loop3doneW<'_, IenSpec> {
        Loop3doneW::new(self, 4)
    }
    #[doc = "Bit 5 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop4done(&mut self) -> Loop4doneW<'_, IenSpec> {
        Loop4doneW::new(self, 5)
    }
    #[doc = "Bit 6 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop5done(&mut self) -> Loop5doneW<'_, IenSpec> {
        Loop5doneW::new(self, 6)
    }
    #[doc = "Bit 7 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop6done(&mut self) -> Loop6doneW<'_, IenSpec> {
        Loop6doneW::new(self, 7)
    }
    #[doc = "Bit 8 - Loop Done Interrupt Enable"]
    #[inline(always)]
    pub fn loop7done(&mut self) -> Loop7doneW<'_, IenSpec> {
        Loop7doneW::new(self, 8)
    }
    #[doc = "Bit 10 - Not-a-Number Interrupt Enable"]
    #[inline(always)]
    pub fn alunan(&mut self) -> AlunanW<'_, IenSpec> {
        AlunanW::new(self, 10)
    }
    #[doc = "Bit 11 - R0 Non-Zero Interrupt Enable"]
    #[inline(always)]
    pub fn r0posreal(&mut self) -> R0posrealW<'_, IenSpec> {
        R0posrealW::new(self, 11)
    }
    #[doc = "Bit 12 - ALU Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn aluof(&mut self) -> AluofW<'_, IenSpec> {
        AluofW::new(self, 12)
    }
    #[doc = "Bit 13 - ALU Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn aluuf(&mut self) -> AluufW<'_, IenSpec> {
        AluufW::new(self, 13)
    }
    #[doc = "Bit 14 - Store conversion Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertof(&mut self) -> StoreconvertofW<'_, IenSpec> {
        StoreconvertofW::new(self, 14)
    }
    #[doc = "Bit 15 - Store Conversion Underflow Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertuf(&mut self) -> StoreconvertufW<'_, IenSpec> {
        StoreconvertufW::new(self, 15)
    }
    #[doc = "Bit 16 - Store Conversion Infinity Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertinf(&mut self) -> StoreconvertinfW<'_, IenSpec> {
        StoreconvertinfW::new(self, 16)
    }
    #[doc = "Bit 17 - Store Conversion NaN Interrupt Enable"]
    #[inline(always)]
    pub fn storeconvertnan(&mut self) -> StoreconvertnanW<'_, IenSpec> {
        StoreconvertnanW::new(self, 17)
    }
    #[doc = "Bit 18 - Perf Counter 0 Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn perfcnt0(&mut self) -> Perfcnt0W<'_, IenSpec> {
        Perfcnt0W::new(self, 18)
    }
    #[doc = "Bit 19 - Perf Counter 1 Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn perfcnt1(&mut self) -> Perfcnt1W<'_, IenSpec> {
        Perfcnt1W::new(self, 19)
    }
    #[doc = "Bit 24 - Loop Fault Interrupt Enable"]
    #[inline(always)]
    pub fn loopfault(&mut self) -> LoopfaultW<'_, IenSpec> {
        LoopfaultW::new(self, 24)
    }
    #[doc = "Bit 25 - Bus Error Fault Interrupt Enable"]
    #[inline(always)]
    pub fn buserrfault(&mut self) -> BuserrfaultW<'_, IenSpec> {
        BuserrfaultW::new(self, 25)
    }
    #[doc = "Bit 26 - Bus Alignment Fault Interrupt Enable"]
    #[inline(always)]
    pub fn busalignfault(&mut self) -> BusalignfaultW<'_, IenSpec> {
        BusalignfaultW::new(self, 26)
    }
    #[doc = "Bit 27 - ALU Input Fault Interrupt Enable"]
    #[inline(always)]
    pub fn alufault(&mut self) -> AlufaultW<'_, IenSpec> {
        AlufaultW::new(self, 27)
    }
    #[doc = "Bit 28 - Array Fault Interrupt Enable"]
    #[inline(always)]
    pub fn arrayfault(&mut self) -> ArrayfaultW<'_, IenSpec> {
        ArrayfaultW::new(self, 28)
    }
}
#[doc = "Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ien::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ien::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
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
