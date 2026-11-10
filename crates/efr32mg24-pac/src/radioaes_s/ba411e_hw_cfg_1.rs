#[doc = "Register `BA411E_HW_CFG_1` reader"]
pub type R = crate::R<Ba411eHwCfg1Spec>;
#[doc = "Field `g_AesModesPoss` reader - AES Modes Supported"]
pub type GAesModesPossR = crate::FieldReader<u16>;
#[doc = "Field `g_CS` reader - Generic g_CS value"]
pub type GCsR = crate::BitReader;
#[doc = "Field `g_UseMasking` reader - Generic g_UseMasking value"]
pub type GUseMaskingR = crate::BitReader;
#[doc = "Field `g_Keysize` reader - Generic g_Keysize value"]
pub type GKeysizeR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:8 - AES Modes Supported"]
    #[inline(always)]
    pub fn g_aes_modes_poss(&self) -> GAesModesPossR {
        GAesModesPossR::new((self.bits & 0x01ff) as u16)
    }
    #[doc = "Bit 16 - Generic g_CS value"]
    #[inline(always)]
    pub fn g_cs(&self) -> GCsR {
        GCsR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Generic g_UseMasking value"]
    #[inline(always)]
    pub fn g_use_masking(&self) -> GUseMaskingR {
        GUseMaskingR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bits 24:26 - Generic g_Keysize value"]
    #[inline(always)]
    pub fn g_keysize(&self) -> GKeysizeR {
        GKeysizeR::new(((self.bits >> 24) & 7) as u8)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba411e_hw_cfg_1::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ba411eHwCfg1Spec;
impl crate::RegisterSpec for Ba411eHwCfg1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ba411e_hw_cfg_1::R`](R) reader structure"]
impl crate::Readable for Ba411eHwCfg1Spec {}
#[doc = "`reset()` method sets BA411E_HW_CFG_1 to value 0x0501_0127"]
impl crate::Resettable for Ba411eHwCfg1Spec {
    const RESET_VALUE: u32 = 0x0501_0127;
}
