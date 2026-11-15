#[doc = "Register `INCL_IPS_HW_CFG` reader"]
pub type R = crate::R<InclIpsHwCfgSpec>;
#[doc = "Field `g_IncludeAES` reader - Generic g_IncludeAES value"]
pub type GIncludeAesR = crate::BitReader;
#[doc = "Field `g_IncludeAESGCM` reader - Generic g_IncludeAESGCM value"]
pub type GIncludeAesgcmR = crate::BitReader;
#[doc = "Field `g_IncludeAESXTS` reader - Generic g_IncludeAESXTS value"]
pub type GIncludeAesxtsR = crate::BitReader;
#[doc = "Field `g_IncludeDES` reader - Generic g_IncludeDES value"]
pub type GIncludeDesR = crate::BitReader;
#[doc = "Field `g_IncludeHASH` reader - Generic g_IncludeHASH value"]
pub type GIncludeHashR = crate::BitReader;
#[doc = "Field `g_IncludeChachaPoly` reader - Generic g_IncludeChachaPoly value"]
pub type GIncludeChachaPolyR = crate::BitReader;
#[doc = "Field `g_IncludeSHA3` reader - Generic g_IncludeSHA3 value"]
pub type GIncludeSha3R = crate::BitReader;
#[doc = "Field `g_IncludeZUC` reader - Generic g_IncludeZUC value"]
pub type GIncludeZucR = crate::BitReader;
#[doc = "Field `g_IncludeSM4` reader - Generic g_IncludeSM4 value"]
pub type GIncludeSm4R = crate::BitReader;
#[doc = "Field `g_IncludePKE` reader - Generic g_IncludePKE value"]
pub type GIncludePkeR = crate::BitReader;
#[doc = "Field `g_IncludeNDRNG` reader - Generic g_IncludeNDRNG value"]
pub type GIncludeNdrngR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - Generic g_IncludeAES value"]
    #[inline(always)]
    pub fn g_include_aes(&self) -> GIncludeAesR {
        GIncludeAesR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Generic g_IncludeAESGCM value"]
    #[inline(always)]
    pub fn g_include_aesgcm(&self) -> GIncludeAesgcmR {
        GIncludeAesgcmR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Generic g_IncludeAESXTS value"]
    #[inline(always)]
    pub fn g_include_aesxts(&self) -> GIncludeAesxtsR {
        GIncludeAesxtsR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Generic g_IncludeDES value"]
    #[inline(always)]
    pub fn g_include_des(&self) -> GIncludeDesR {
        GIncludeDesR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Generic g_IncludeHASH value"]
    #[inline(always)]
    pub fn g_include_hash(&self) -> GIncludeHashR {
        GIncludeHashR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Generic g_IncludeChachaPoly value"]
    #[inline(always)]
    pub fn g_include_chacha_poly(&self) -> GIncludeChachaPolyR {
        GIncludeChachaPolyR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Generic g_IncludeSHA3 value"]
    #[inline(always)]
    pub fn g_include_sha3(&self) -> GIncludeSha3R {
        GIncludeSha3R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Generic g_IncludeZUC value"]
    #[inline(always)]
    pub fn g_include_zuc(&self) -> GIncludeZucR {
        GIncludeZucR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Generic g_IncludeSM4 value"]
    #[inline(always)]
    pub fn g_include_sm4(&self) -> GIncludeSm4R {
        GIncludeSm4R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Generic g_IncludePKE value"]
    #[inline(always)]
    pub fn g_include_pke(&self) -> GIncludePkeR {
        GIncludePkeR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Generic g_IncludeNDRNG value"]
    #[inline(always)]
    pub fn g_include_ndrng(&self) -> GIncludeNdrngR {
        GIncludeNdrngR::new(((self.bits >> 10) & 1) != 0)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`incl_ips_hw_cfg::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct InclIpsHwCfgSpec;
impl crate::RegisterSpec for InclIpsHwCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`incl_ips_hw_cfg::R`](R) reader structure"]
impl crate::Readable for InclIpsHwCfgSpec {}
#[doc = "`reset()` method sets INCL_IPS_HW_CFG to value 0x01"]
impl crate::Resettable for InclIpsHwCfgSpec {
    const RESET_VALUE: u32 = 0x01;
}
