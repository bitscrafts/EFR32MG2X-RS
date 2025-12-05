#[doc = "Register `BA413_HW_CFG` reader"]
pub type R = crate::R<Ba413HwCfgSpec>;
#[doc = "Field `g_HashMaskFunc` reader - Generic g_HashMaskFunc value"]
pub type GHashMaskFuncR = crate::FieldReader;
#[doc = "Field `g_HashPadding` reader - Generic g_HashPadding value"]
pub type GHashPaddingR = crate::BitReader;
#[doc = "Field `g_HMAC_enabled` reader - Generic g_HMAC_enabled value"]
pub type GHmacEnabledR = crate::BitReader;
#[doc = "Field `g_HashVerifyDigest` reader - Generic g_HashVerifyDigest value"]
pub type GHashVerifyDigestR = crate::BitReader;
impl R {
    #[doc = "Bits 0:6 - Generic g_HashMaskFunc value"]
    #[inline(always)]
    pub fn g_hash_mask_func(&self) -> GHashMaskFuncR {
        GHashMaskFuncR::new((self.bits & 0x7f) as u8)
    }
    #[doc = "Bit 16 - Generic g_HashPadding value"]
    #[inline(always)]
    pub fn g_hash_padding(&self) -> GHashPaddingR {
        GHashPaddingR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Generic g_HMAC_enabled value"]
    #[inline(always)]
    pub fn g_hmac_enabled(&self) -> GHmacEnabledR {
        GHmacEnabledR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Generic g_HashVerifyDigest value"]
    #[inline(always)]
    pub fn g_hash_verify_digest(&self) -> GHashVerifyDigestR {
        GHashVerifyDigestR::new(((self.bits >> 18) & 1) != 0)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba413_hw_cfg::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ba413HwCfgSpec;
impl crate::RegisterSpec for Ba413HwCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ba413_hw_cfg::R`](R) reader structure"]
impl crate::Readable for Ba413HwCfgSpec {}
#[doc = "`reset()` method sets BA413_HW_CFG to value 0"]
impl crate::Resettable for Ba413HwCfgSpec {}
