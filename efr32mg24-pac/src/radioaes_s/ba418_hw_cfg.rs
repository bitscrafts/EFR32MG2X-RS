#[doc = "Register `BA418_HW_CFG` reader"]
pub type R = crate::R<Ba418HwCfgSpec>;
#[doc = "Field `g_Sha3CtxtEn` reader - Generic g_Sha3CtxtEn value"]
pub type GSha3ctxtEnR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - Generic g_Sha3CtxtEn value"]
    #[inline(always)]
    pub fn g_sha3ctxt_en(&self) -> GSha3ctxtEnR {
        GSha3ctxtEnR::new((self.bits & 1) != 0)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba418_hw_cfg::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ba418HwCfgSpec;
impl crate::RegisterSpec for Ba418HwCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ba418_hw_cfg::R`](R) reader structure"]
impl crate::Readable for Ba418HwCfgSpec {}
#[doc = "`reset()` method sets BA418_HW_CFG to value 0x01"]
impl crate::Resettable for Ba418HwCfgSpec {
    const RESET_VALUE: u32 = 0x01;
}
