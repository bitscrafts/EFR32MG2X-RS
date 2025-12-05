#[doc = "Register `LOOP3RST` reader"]
pub type R = crate::R<Loop3rstSpec>;
#[doc = "Register `LOOP3RST` writer"]
pub type W = crate::W<Loop3rstSpec>;
#[doc = "Field `ARRAY0RESETDIM0` reader - Reset Dimension 0"]
pub type Array0resetdim0R = crate::BitReader;
#[doc = "Field `ARRAY0RESETDIM0` writer - Reset Dimension 0"]
pub type Array0resetdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY0RESETDIM1` reader - Reset Dimension 1"]
pub type Array0resetdim1R = crate::BitReader;
#[doc = "Field `ARRAY0RESETDIM1` writer - Reset Dimension 1"]
pub type Array0resetdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY0RESETDIM2` reader - Reset Dimension 2"]
pub type Array0resetdim2R = crate::BitReader;
#[doc = "Field `ARRAY0RESETDIM2` writer - Reset Dimension 2"]
pub type Array0resetdim2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY1RESETDIM0` reader - Reset Dimension 0"]
pub type Array1resetdim0R = crate::BitReader;
#[doc = "Field `ARRAY1RESETDIM0` writer - Reset Dimension 0"]
pub type Array1resetdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY1RESETDIM1` reader - Reset Dimension 1"]
pub type Array1resetdim1R = crate::BitReader;
#[doc = "Field `ARRAY1RESETDIM1` writer - Reset Dimension 1"]
pub type Array1resetdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY1RESETDIM2` reader - Reset Dimension 2"]
pub type Array1resetdim2R = crate::BitReader;
#[doc = "Field `ARRAY1RESETDIM2` writer - Reset Dimension 2"]
pub type Array1resetdim2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY2RESETDIM0` reader - Reset Dimension 0"]
pub type Array2resetdim0R = crate::BitReader;
#[doc = "Field `ARRAY2RESETDIM0` writer - Reset Dimension 0"]
pub type Array2resetdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY2RESETDIM1` reader - Reset Dimension 1"]
pub type Array2resetdim1R = crate::BitReader;
#[doc = "Field `ARRAY2RESETDIM1` writer - Reset Dimension 1"]
pub type Array2resetdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY2RESETDIM2` reader - Reset Dimension 2"]
pub type Array2resetdim2R = crate::BitReader;
#[doc = "Field `ARRAY2RESETDIM2` writer - Reset Dimension 2"]
pub type Array2resetdim2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY3RESETDIM0` reader - Reset Dimension 0"]
pub type Array3resetdim0R = crate::BitReader;
#[doc = "Field `ARRAY3RESETDIM0` writer - Reset Dimension 0"]
pub type Array3resetdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY3RESETDIM1` reader - Reset Dimension 1"]
pub type Array3resetdim1R = crate::BitReader;
#[doc = "Field `ARRAY3RESETDIM1` writer - Reset Dimension 1"]
pub type Array3resetdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY3RESETDIM2` reader - Reset Dimension 2"]
pub type Array3resetdim2R = crate::BitReader;
#[doc = "Field `ARRAY3RESETDIM2` writer - Reset Dimension 2"]
pub type Array3resetdim2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY4RESETDIM0` reader - Reset Dimension 0"]
pub type Array4resetdim0R = crate::BitReader;
#[doc = "Field `ARRAY4RESETDIM0` writer - Reset Dimension 0"]
pub type Array4resetdim0W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY4RESETDIM1` reader - Reset Dimension 1"]
pub type Array4resetdim1R = crate::BitReader;
#[doc = "Field `ARRAY4RESETDIM1` writer - Reset Dimension 1"]
pub type Array4resetdim1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ARRAY4RESETDIM2` reader - Reset Dimension 2"]
pub type Array4resetdim2R = crate::BitReader;
#[doc = "Field `ARRAY4RESETDIM2` writer - Reset Dimension 2"]
pub type Array4resetdim2W<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 12 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array0resetdim0(&self) -> Array0resetdim0R {
        Array0resetdim0R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array0resetdim1(&self) -> Array0resetdim1R {
        Array0resetdim1R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array0resetdim2(&self) -> Array0resetdim2R {
        Array0resetdim2R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 16 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array1resetdim0(&self) -> Array1resetdim0R {
        Array1resetdim0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array1resetdim1(&self) -> Array1resetdim1R {
        Array1resetdim1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array1resetdim2(&self) -> Array1resetdim2R {
        Array1resetdim2R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 20 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array2resetdim0(&self) -> Array2resetdim0R {
        Array2resetdim0R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array2resetdim1(&self) -> Array2resetdim1R {
        Array2resetdim1R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array2resetdim2(&self) -> Array2resetdim2R {
        Array2resetdim2R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 24 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array3resetdim0(&self) -> Array3resetdim0R {
        Array3resetdim0R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array3resetdim1(&self) -> Array3resetdim1R {
        Array3resetdim1R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array3resetdim2(&self) -> Array3resetdim2R {
        Array3resetdim2R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 28 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array4resetdim0(&self) -> Array4resetdim0R {
        Array4resetdim0R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array4resetdim1(&self) -> Array4resetdim1R {
        Array4resetdim1R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array4resetdim2(&self) -> Array4resetdim2R {
        Array4resetdim2R::new(((self.bits >> 30) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 12 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array0resetdim0(&mut self) -> Array0resetdim0W<'_, Loop3rstSpec> {
        Array0resetdim0W::new(self, 12)
    }
    #[doc = "Bit 13 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array0resetdim1(&mut self) -> Array0resetdim1W<'_, Loop3rstSpec> {
        Array0resetdim1W::new(self, 13)
    }
    #[doc = "Bit 14 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array0resetdim2(&mut self) -> Array0resetdim2W<'_, Loop3rstSpec> {
        Array0resetdim2W::new(self, 14)
    }
    #[doc = "Bit 16 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array1resetdim0(&mut self) -> Array1resetdim0W<'_, Loop3rstSpec> {
        Array1resetdim0W::new(self, 16)
    }
    #[doc = "Bit 17 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array1resetdim1(&mut self) -> Array1resetdim1W<'_, Loop3rstSpec> {
        Array1resetdim1W::new(self, 17)
    }
    #[doc = "Bit 18 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array1resetdim2(&mut self) -> Array1resetdim2W<'_, Loop3rstSpec> {
        Array1resetdim2W::new(self, 18)
    }
    #[doc = "Bit 20 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array2resetdim0(&mut self) -> Array2resetdim0W<'_, Loop3rstSpec> {
        Array2resetdim0W::new(self, 20)
    }
    #[doc = "Bit 21 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array2resetdim1(&mut self) -> Array2resetdim1W<'_, Loop3rstSpec> {
        Array2resetdim1W::new(self, 21)
    }
    #[doc = "Bit 22 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array2resetdim2(&mut self) -> Array2resetdim2W<'_, Loop3rstSpec> {
        Array2resetdim2W::new(self, 22)
    }
    #[doc = "Bit 24 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array3resetdim0(&mut self) -> Array3resetdim0W<'_, Loop3rstSpec> {
        Array3resetdim0W::new(self, 24)
    }
    #[doc = "Bit 25 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array3resetdim1(&mut self) -> Array3resetdim1W<'_, Loop3rstSpec> {
        Array3resetdim1W::new(self, 25)
    }
    #[doc = "Bit 26 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array3resetdim2(&mut self) -> Array3resetdim2W<'_, Loop3rstSpec> {
        Array3resetdim2W::new(self, 26)
    }
    #[doc = "Bit 28 - Reset Dimension 0"]
    #[inline(always)]
    pub fn array4resetdim0(&mut self) -> Array4resetdim0W<'_, Loop3rstSpec> {
        Array4resetdim0W::new(self, 28)
    }
    #[doc = "Bit 29 - Reset Dimension 1"]
    #[inline(always)]
    pub fn array4resetdim1(&mut self) -> Array4resetdim1W<'_, Loop3rstSpec> {
        Array4resetdim1W::new(self, 29)
    }
    #[doc = "Bit 30 - Reset Dimension 2"]
    #[inline(always)]
    pub fn array4resetdim2(&mut self) -> Array4resetdim2W<'_, Loop3rstSpec> {
        Array4resetdim2W::new(self, 30)
    }
}
#[doc = "Loop N Reset Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`loop3rst::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`loop3rst::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Loop3rstSpec;
impl crate::RegisterSpec for Loop3rstSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`loop3rst::R`](R) reader structure"]
impl crate::Readable for Loop3rstSpec {}
#[doc = "`write(|w| ..)` method takes [`loop3rst::W`](W) writer structure"]
impl crate::Writable for Loop3rstSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LOOP3RST to value 0"]
impl crate::Resettable for Loop3rstSpec {}
