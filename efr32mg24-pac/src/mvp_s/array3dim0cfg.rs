#[doc = "Register `ARRAY3DIM0CFG` reader"]
pub type R = crate::R<Array3dim0cfgSpec>;
#[doc = "Register `ARRAY3DIM0CFG` writer"]
pub type W = crate::W<Array3dim0cfgSpec>;
#[doc = "Field `SIZE` reader - Array Dimension Size"]
pub type SizeR = crate::FieldReader<u16>;
#[doc = "Field `SIZE` writer - Array Dimension Size"]
pub type SizeW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Element Type\n\nValue on reset: 2"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Basetype {
    #[doc = "0: Data is unsigned 8-bit integer (can only be used for loads)"]
    Uint8 = 0,
    #[doc = "1: Data is signed 8-bit integer (can only be used for loads)"]
    Int8 = 1,
    #[doc = "2: Data is 16-bit float"]
    Binary16 = 2,
}
impl From<Basetype> for u8 {
    #[inline(always)]
    fn from(variant: Basetype) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Basetype {
    type Ux = u8;
}
impl crate::IsEnum for Basetype {}
#[doc = "Field `BASETYPE` reader - Element Type"]
pub type BasetypeR = crate::FieldReader<Basetype>;
impl BasetypeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Basetype {
        match self.bits {
            0 => Basetype::Uint8,
            1 => Basetype::Int8,
            2 => Basetype::Binary16,
            _ => unreachable!(),
        }
    }
    #[doc = "Data is unsigned 8-bit integer (can only be used for loads)"]
    #[inline(always)]
    pub fn is_uint8(&self) -> bool {
        *self == Basetype::Uint8
    }
    #[doc = "Data is signed 8-bit integer (can only be used for loads)"]
    #[inline(always)]
    pub fn is_int8(&self) -> bool {
        *self == Basetype::Int8
    }
    #[doc = "Data is 16-bit float"]
    #[inline(always)]
    pub fn is_binary16(&self) -> bool {
        *self == Basetype::Binary16
    }
}
#[doc = "Field `BASETYPE` writer - Element Type"]
pub type BasetypeW<'a, REG> = crate::FieldWriter<'a, REG, 2, Basetype>;
impl<'a, REG> BasetypeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Data is unsigned 8-bit integer (can only be used for loads)"]
    #[inline(always)]
    pub fn uint8(self) -> &'a mut crate::W<REG> {
        self.variant(Basetype::Uint8)
    }
    #[doc = "Data is signed 8-bit integer (can only be used for loads)"]
    #[inline(always)]
    pub fn int8(self) -> &'a mut crate::W<REG> {
        self.variant(Basetype::Int8)
    }
    #[doc = "Data is 16-bit float"]
    #[inline(always)]
    pub fn binary16(self) -> &'a mut crate::W<REG> {
        self.variant(Basetype::Binary16)
    }
}
#[doc = "Complex Data Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Complex {
    #[doc = "0: Data represents a scalar number"]
    Scalar = 0,
    #[doc = "1: Data represents a complex pair or packed pair of reals."]
    Complex = 1,
}
impl From<Complex> for bool {
    #[inline(always)]
    fn from(variant: Complex) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COMPLEX` reader - Complex Data Type"]
pub type ComplexR = crate::BitReader<Complex>;
impl ComplexR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Complex {
        match self.bits {
            false => Complex::Scalar,
            true => Complex::Complex,
        }
    }
    #[doc = "Data represents a scalar number"]
    #[inline(always)]
    pub fn is_scalar(&self) -> bool {
        *self == Complex::Scalar
    }
    #[doc = "Data represents a complex pair or packed pair of reals."]
    #[inline(always)]
    pub fn is_complex(&self) -> bool {
        *self == Complex::Complex
    }
}
#[doc = "Field `COMPLEX` writer - Complex Data Type"]
pub type ComplexW<'a, REG> = crate::BitWriter<'a, REG, Complex>;
impl<'a, REG> ComplexW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Data represents a scalar number"]
    #[inline(always)]
    pub fn scalar(self) -> &'a mut crate::W<REG> {
        self.variant(Complex::Scalar)
    }
    #[doc = "Data represents a complex pair or packed pair of reals."]
    #[inline(always)]
    pub fn complex(self) -> &'a mut crate::W<REG> {
        self.variant(Complex::Complex)
    }
}
#[doc = "Field `STRIDE` reader - Dimension Stride Step"]
pub type StrideR = crate::FieldReader<u16>;
#[doc = "Field `STRIDE` writer - Dimension Stride Step"]
pub type StrideW<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;
impl R {
    #[doc = "Bits 0:9 - Array Dimension Size"]
    #[inline(always)]
    pub fn size(&self) -> SizeR {
        SizeR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 12:13 - Element Type"]
    #[inline(always)]
    pub fn basetype(&self) -> BasetypeR {
        BasetypeR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bit 14 - Complex Data Type"]
    #[inline(always)]
    pub fn complex(&self) -> ComplexR {
        ComplexR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bits 16:27 - Dimension Stride Step"]
    #[inline(always)]
    pub fn stride(&self) -> StrideR {
        StrideR::new(((self.bits >> 16) & 0x0fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Array Dimension Size"]
    #[inline(always)]
    pub fn size(&mut self) -> SizeW<'_, Array3dim0cfgSpec> {
        SizeW::new(self, 0)
    }
    #[doc = "Bits 12:13 - Element Type"]
    #[inline(always)]
    pub fn basetype(&mut self) -> BasetypeW<'_, Array3dim0cfgSpec> {
        BasetypeW::new(self, 12)
    }
    #[doc = "Bit 14 - Complex Data Type"]
    #[inline(always)]
    pub fn complex(&mut self) -> ComplexW<'_, Array3dim0cfgSpec> {
        ComplexW::new(self, 14)
    }
    #[doc = "Bits 16:27 - Dimension Stride Step"]
    #[inline(always)]
    pub fn stride(&mut self) -> StrideW<'_, Array3dim0cfgSpec> {
        StrideW::new(self, 16)
    }
}
#[doc = "Array N Dimenion 0 Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`array3dim0cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`array3dim0cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Array3dim0cfgSpec;
impl crate::RegisterSpec for Array3dim0cfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`array3dim0cfg::R`](R) reader structure"]
impl crate::Readable for Array3dim0cfgSpec {}
#[doc = "`write(|w| ..)` method takes [`array3dim0cfg::W`](W) writer structure"]
impl crate::Writable for Array3dim0cfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ARRAY3DIM0CFG to value 0x2000"]
impl crate::Resettable for Array3dim0cfgSpec {
    const RESET_VALUE: u32 = 0x2000;
}
