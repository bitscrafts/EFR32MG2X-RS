#[doc = "Register `BA419_HW_CFG` reader"]
pub type R = crate::R<Ba419HwCfgSpec>;
#[doc = "Field `g_SM4ModesPoss` reader - Generic g_SM4ModesPoss value"]
pub type GSm4modesPossR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:6 - Generic g_SM4ModesPoss value"]
    #[inline(always)]
    pub fn g_sm4modes_poss(&self) -> GSm4modesPossR {
        GSm4modesPossR::new((self.bits & 0x7f) as u8)
    }
}
#[doc = "RADIOAES_S BA419_HW_CFG (offset 0x414, 32 bits). Fields: g_SM4ModesPoss[6:0]. The vendor SVD gives no description; see the EFR32xG24 Reference Manual, RADIOAES chapter.\n\nYou can [`read`](crate::Reg::read) this register and get [`ba419_hw_cfg::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ba419HwCfgSpec;
impl crate::RegisterSpec for Ba419HwCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ba419_hw_cfg::R`](R) reader structure"]
impl crate::Readable for Ba419HwCfgSpec {}
#[doc = "`reset()` method sets BA419_HW_CFG to value 0"]
impl crate::Resettable for Ba419HwCfgSpec {}
