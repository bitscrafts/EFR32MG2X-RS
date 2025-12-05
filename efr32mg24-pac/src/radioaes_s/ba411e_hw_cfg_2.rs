#[doc = "Register `BA411E_HW_CFG_2` reader"]
pub type R = crate::R<Ba411eHwCfg2Spec>;
#[doc = "Field `g_CtrSize` reader - Generic g_CtrSize value"]
pub type GCtrSizeR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Generic g_CtrSize value"]
    #[inline(always)]
    pub fn g_ctr_size(&self) -> GCtrSizeR {
        GCtrSizeR::new((self.bits & 0xffff) as u16)
    }
}
#[doc = "No Description\n\nYou can [`read`](crate::Reg::read) this register and get [`ba411e_hw_cfg_2::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ba411eHwCfg2Spec;
impl crate::RegisterSpec for Ba411eHwCfg2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ba411e_hw_cfg_2::R`](R) reader structure"]
impl crate::Readable for Ba411eHwCfg2Spec {}
#[doc = "`reset()` method sets BA411E_HW_CFG_2 to value 0x80"]
impl crate::Resettable for Ba411eHwCfg2Spec {
    const RESET_VALUE: u32 = 0x80;
}
